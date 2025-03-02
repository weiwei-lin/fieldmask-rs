use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Error)]
#[error(r#"there's no "{field}" in `{type_str}`"#)]
pub struct DeserializeMaskError {
    pub type_str: &'static str,
    pub field: String,
    pub depth: u8,
}

/// A trait for types that have an associated field mask type.
pub trait Maskable: Sized {
    /// A type that can be used to represent a field mask for this particular type.
    /// Different types may have different representations for field masks.
    ///
    /// For example:
    ///  * a `i32` type may use a `bool` to represent a field mask, where `true` means the entire
    ///    value is selected and `false` means the entire value is deselected.
    ///  * a struct type may use a bool tuple to represent a field mask, where each `bool` in the
    ///    tuple controls the selection of a field.
    type Mask;

    /// Make `mask` include the field specified by `field_path``.
    ///
    /// When the function returns Ok, `mask` is modified to include the field specified by
    /// `field_path`. Otherwise, `mask` is unchanged.
    ///
    /// `field_path` is the path to a (nested) field in a type.
    /// For example, given the following struct:
    /// ```rust
    /// struct Parent {
    ///     child: Child,
    /// }
    /// struct Child {
    ///     field: String,
    ///     grandchild: Grandchild,
    /// }
    /// struct Grandchild {
    ///     field_a: String,
    ///     field_b: String,
    /// }
    /// ```
    ///  * `["child", "grandchild", "field_a"]` is the field path to
    ///    `Parent.child.grandchild.field_a`. Only `field_a` in `Grandchild` is included.
    ///  * `["child", "grandchild"]` is the field path to `Parent.child.grandchild`. All the fields
    ///    in `Grandchild` are included.
    ///  * `["child"]` is the field path to `Parent.child`. All the fields in `Child` are included.
    ///  * `[]` is the field path to the entire `Parent` struct. All the fields in `Parent` are
    ///    included.
    fn make_mask_include_field(
        // Take a reference here instead of the ownership. Because:
        // 1. We may want to try performing other operations on `mask` if the current one doesn't
        // work.
        // 2. Therefore, if we take the ownership, we will need to return the ownership even when
        // the method failed, which is cumbersome.
        mask: &mut Self::Mask,
        // Take a slice of segments instead of a full field mask string. Because:
        // 1. It's easier to perform pattern matching on slices.
        // 2. It's easier to distinguish empty field mask (e.g. "") and empty tail (e.g. "parent.").
        field_path: &[&str],
    ) -> Result<(), DeserializeMaskError>;
}

/// A trait for types whose fields can be selected using field masks.
pub trait SelfMaskable: Maskable {
    /// Assign all the fields in `update` that are selected by `mask` to `self`.
    fn apply_mask(&mut self, update: Self, mask: &Self::Mask);
}

/// A trait for types whose content can be selected using field masks when it's wrapped in an
/// `Option`.
///
/// This is useful when the entire type can be dropped when the mask doesn't match any field.
pub trait OptionMaskable: Maskable {
    /// Assign all the fields in `update` that are selected by `mask` to `self`.
    ///
    /// Returns `false` if the entire type should be dropped.
    /// Returns `true` otherwise.
    fn apply_mask(&mut self, update: Self, mask: &Self::Mask) -> bool;
}

/// If a type is `SelfMaskable`, it's also `OptionMaskable`.
impl<T: SelfMaskable> OptionMaskable for T
where
    T: Default,
    T::Mask: PartialEq,
{
    fn apply_mask(&mut self, update: Self, mask: &Self::Mask) -> bool {
        self.apply_mask(update, mask);
        true
    }
}

impl<T: Maskable> Maskable for Option<T>
where
    T: Default,
    T::Mask: PartialEq,
{
    type Mask = T::Mask;

    fn make_mask_include_field(
        mask: &mut Self::Mask,
        field_path: &[&str],
    ) -> Result<(), DeserializeMaskError> {
        T::make_mask_include_field(mask, field_path)
    }
}

impl<T: OptionMaskable> SelfMaskable for Option<T>
where
    T: Default,
    T::Mask: PartialEq + Default,
{
    fn apply_mask(&mut self, update: Self, mask: &Self::Mask) {
        // If the mask is the default value, we don't need to do anything.
        // Default value means the mask matches no field.
        if mask == &Self::Mask::default() {
            return;
        }
        match self {
            Some(t) => match update {
                Some(s) => {
                    if !t.apply_mask(s, mask) {
                        *self = None;
                    }
                }
                None => *self = None,
            },
            None => {
                if let Some(o) = update {
                    let mut new = T::default();
                    if new.apply_mask(o, mask) {
                        *self = Some(new);
                    } else {
                        *self = None;
                    }
                }
            }
        }
    }
}

macro_rules! maskable {
    ($T:path) => {
        impl Maskable for $T {
            type Mask = bool;

            fn make_mask_include_field(
                mask: &mut Self::Mask,
                field_path: &[&str],
            ) -> Result<(), DeserializeMaskError> {
                if field_path.len() == 0 {
                    *mask = true;
                    Ok(())
                } else {
                    Err(DeserializeMaskError {
                        type_str: stringify!($T),
                        field: field_path[0].into(),
                        depth: 0,
                    })
                }
            }
        }

        impl SelfMaskable for $T {
            fn apply_mask(&mut self, other: Self, mask: &Self::Mask) {
                if *mask {
                    *self = other;
                }
            }
        }
    };
}

maskable!(bool);
maskable!(char);

maskable!(f32);
maskable!(f64);

maskable!(i8);
maskable!(u8);
maskable!(i16);
maskable!(u16);
maskable!(i32);
maskable!(u32);
maskable!(i64);
maskable!(u64);
maskable!(i128);
maskable!(u128);
maskable!(isize);
maskable!(usize);

maskable!(String);

#[cfg(feature = "prost")]
maskable!(prost::bytes::Bytes);

impl<T> Maskable for Vec<T> {
    type Mask = bool;

    fn make_mask_include_field(
        mask: &mut Self::Mask,
        field_path: &[&str],
    ) -> Result<(), DeserializeMaskError> {
        if field_path.is_empty() {
            *mask = true;
            Ok(())
        } else {
            Err(DeserializeMaskError {
                type_str: "Vec",
                field: field_path[0].into(),
                depth: 0,
            })
        }
    }
}

impl<T> SelfMaskable for Vec<T> {
    fn apply_mask(&mut self, other: Self, mask: &Self::Mask) {
        if *mask {
            *self = other;
        }
    }
}

impl<K, V> Maskable for HashMap<K, V> {
    type Mask = bool;

    fn make_mask_include_field(
        mask: &mut Self::Mask,
        field_path: &[&str],
    ) -> Result<(), DeserializeMaskError> {
        if field_path.is_empty() {
            *mask = true;
            Ok(())
        } else {
            Err(DeserializeMaskError {
                type_str: "HashMap",
                field: field_path[0].into(),
                depth: 0,
            })
        }
    }
}

impl<K, V> SelfMaskable for HashMap<K, V> {
    fn apply_mask(&mut self, other: Self, mask: &Self::Mask) {
        if *mask {
            *self = other;
        }
    }
}
