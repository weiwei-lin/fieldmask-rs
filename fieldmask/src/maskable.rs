use std::collections::HashMap;

use textwrap::indent;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializeMaskError<'a> {
    #[error("type `{type_name}` has no field named \"{field}\"")]
    FieldNotFound {
        field: &'a str,
        type_name: &'static str,
    },
    #[error("error in field \"{field}\":\n{indented_err}", indented_err = indent(&err.to_string(), "\t"))]
    InvalidField {
        field: &'static str,
        err: Box<DeserializeMaskError<'a>>,
    },
}

/// A trait for types that have an associated field mask type.
pub trait Maskable {
    /// A type that can be used to represent a field mask for this particular type.
    /// Different types may have different representations for field masks.
    ///
    /// For example:
    ///  * a type with no fields may use a unit type to represent a field mask.
    ///  * a struct with only fields of primitive types may use a bool tuple to represent a
    ///    field mask, where each `bool` in the tuple controls the selection of a primitive field.
    ///  * a struct with only nested structs may use a tuple of `Option<ChildMask>` to represent a
    ///    field mask, where `None` in the tuple means the corresponding field is not selected, and
    ///    `Some(ChildMask)` means the corresponding field is selected with the specified sub-mask.
    ///
    /// `Mask` must implements the `Default` trait. When constructing a mask, the default mask is
    /// used as the initial mask that selects no fields.
    ///
    /// `Mask` must also implements the `PartialEq` trait. We need to compare the mask with the
    /// default mask to determine whether any of the field is selected. When no field is selected,
    /// the entire object is selected during projection, non-default fields are selected during
    /// update. See https://protobuf.dev/reference/java/api-docs/com/google/protobuf/FieldMask.html
    /// for the behavior of empty field masks.
    type Mask: Default + PartialEq;

    /// Make `mask` include the field specified by `field_path``.
    ///
    /// When the function returns `Ok`, `mask` is modified to include the field specified by
    /// `field_path`. Otherwise, `mask` is unchanged.
    ///
    /// `field_path` is a field mask path splitted by '.'.
    fn make_mask_include_field<'a>(
        // Take a reference here instead of the ownership. Because:
        // 1. We may want to try performing other operations on `mask` if the current one doesn't
        // work.
        // 2. Therefore, if we take the ownership, we will need to return the ownership even when
        // the method failed, which is cumbersome.
        mask: &mut Self::Mask,
        // Take a slice of segments instead of a full field mask string. Because:
        // 1. It's easier to perform pattern matching on slices.
        // 2. It's easier to distinguish empty field mask (e.g. "") and empty tail (e.g. "parent.").
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>>;

    /// Project the fields of `self` according to `mask`.
    fn project(self, mask: &Self::Mask) -> Self;
}

// If a type is `Maskable`, then `Option<T>` is also `Maskable`.
impl<T: Maskable> Maskable for Option<T> {
    type Mask = T::Mask;

    fn make_mask_include_field<'a>(
        mask: &mut Self::Mask,
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        T::make_mask_include_field(mask, field_path)
    }

    fn project(self, mask: &Self::Mask) -> Self {
        self.map(|inner| inner.project(mask))
    }
}

macro_rules! maskable_atomic {
    ($name:ident$(<$($ty_param:ident),*>)? $(where $($where_clause:tt)+)?) => {
        impl$(<$($ty_param),*>)? Maskable for $name$(<$($ty_param),*>)?
        $(where $($where_clause)+)?
        {
            type Mask = ();

            fn make_mask_include_field<'a>(_mask: &mut Self::Mask, field_path: &[&'a str]) -> Result<(), DeserializeMaskError<'a>> {
                if field_path.is_empty() {
                    return Ok(());
                }
                Err(DeserializeMaskError::FieldNotFound {
                    type_name: stringify!($name),
                    field: field_path[0],
                })
            }

            fn project(self, _mask: &Self::Mask) -> Self {
                return self;
            }
        }
    };
}

maskable_atomic!(bool);
maskable_atomic!(char);

maskable_atomic!(f32);
maskable_atomic!(f64);

maskable_atomic!(i8);
maskable_atomic!(u8);
maskable_atomic!(i16);
maskable_atomic!(u16);
maskable_atomic!(i32);
maskable_atomic!(u32);
maskable_atomic!(i64);
maskable_atomic!(u64);
maskable_atomic!(i128);
maskable_atomic!(u128);
maskable_atomic!(isize);
maskable_atomic!(usize);

maskable_atomic!(String);
maskable_atomic!(Vec<T>);
maskable_atomic!(HashMap<K, V>);

#[cfg(feature = "prost")]
mod prost_integration {
    use ::prost::bytes::Bytes;

    use super::*;

    maskable_atomic!(Bytes);
}
