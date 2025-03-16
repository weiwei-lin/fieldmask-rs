use std::{collections::HashMap, mem};

use fieldmask_derive::maskable_atomic;
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

/// Options for updating a message with a field mask.
#[derive(Debug, Default)]
pub struct UpdateOptions {
    /// If true, the repeated field in `self` will be replaced by the repeated field in `source`.
    /// Otherwise, the repeated field in `source` will be appended to the repeated field in `self`.
    ///
    /// Defaults to `false`. The default behavior is consistent with [the official Java
    /// implementation][1].
    ///
    /// [1]: https://protobuf.dev/reference/java/api-docs/com/google/protobuf/FieldMask.html
    pub replace_repeated: bool,

    /// Controls the behavior of updating the value of a message field in `self` with the same value
    /// of the same field in `source` when the message field is specified in the last position of
    /// the field mask.
    ///
    /// If true, the message field in `self` will be replaced by the message field in `source`.
    /// Otherwise, the message field in `source` will be merged into the message field in `self`.
    ///
    /// Defaults to `false`. The default behavior is consistent with [the official Java
    /// implementation][1].
    ///
    /// [1]: https://protobuf.dev/reference/java/api-docs/com/google/protobuf/FieldMask.html
    pub replace_message: bool,
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
    /// `Mask` must also implements the `PartialEq` trait. We need to compare the mask with the
    /// empty mask to determine whether any of the field is selected.
    type Mask: PartialEq;

    /// Returns an empty mask that selects no field.
    ///
    /// For atomic types, the empty mask is the same as the full mask.
    fn empty_mask() -> Self::Mask;

    /// Returns a full mask that selects all fields.
    ///
    /// For atomic types, the empty mask is the same as the full mask.
    fn full_mask() -> Self::Mask;

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
}

/// A trait for types that can be projected or updated according to a field mask.
pub trait SelfMaskable: Maskable {
    /// Project the fields of `self` according to `mask`.
    ///
    /// An empty `mask` is the same as a full `mask`. This is consistent with [the official field
    /// mask protobuf specification][1].
    ///
    /// [1]: https://protobuf.dev/reference/protobuf/google.protobuf/#field-mask.
    fn project(self, mask: &Self::Mask) -> Self;

    /// Update the fields of `self` with the fields of `source` according to `mask`.
    ///
    /// This message is treated as a field of the parent message, which means an empty `mask` is not
    /// treated as a full `mask`.
    fn update_as_field(&mut self, source: Self, mask: &Self::Mask, options: &UpdateOptions);

    /// Merge the fields of `source` into `self`.
    fn merge(&mut self, source: Self, options: &UpdateOptions);
}

/// A trait for types that can be projected or updated according to a field mask when wrapped in an
/// `Option`.
///
/// Note that a field of message type with unspecified value should be treated the same as the field
/// with a default value. This must be true. Otherwise, we cannot clearly define the behavior of the
/// update operation when `source` is `None`. Also it can lead to confusions on whether the a
/// `Some(Default::default())` field should be normalized to `None` or not.
pub trait OptionMaskable: Maskable + Sized {
    /// Similar to `SelfMaskable::project`, but it takes `Option<Self>` instead of `Self`.
    fn option_project(this: Option<Self>, mask: &Self::Mask) -> Option<Self>;

    /// Similar to `SelfMaskable::update_as_field`, but it takes `Option<Self>` instead of `Self`.
    fn option_update_as_field(
        this: &mut Option<Self>,
        source: Option<Self>,
        mask: &Self::Mask,
        options: &UpdateOptions,
    );

    /// Similar to `SelfMaskable::merge`, but it takes `Option<Self>` instead of `Self`.
    fn option_merge(this: &mut Option<Self>, source: Option<Self>, options: &UpdateOptions);
}

// Do not implement this. Otherwise we will not be able to implement `OptionMaskable` for any other
//  foreign types (e.g. `Box<T>`) without specialization.
// impl<T: SelfMaskable + Default> OptionMaskable for T {}

impl<T: Maskable> Maskable for Option<T> {
    type Mask = T::Mask;

    fn empty_mask() -> Self::Mask {
        T::empty_mask()
    }

    fn full_mask() -> Self::Mask {
        T::full_mask()
    }

    fn make_mask_include_field<'a>(
        mask: &mut Self::Mask,
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        T::make_mask_include_field(mask, field_path)
    }
}

impl<T: OptionMaskable> OptionMaskable for Option<T> {
    fn option_project(this: Option<Self>, mask: &Self::Mask) -> Option<Self> {
        this.map(|this| T::option_project(this, mask))
    }

    fn option_update_as_field(
        this: &mut Option<Self>,
        source: Option<Self>,
        mask: &Self::Mask,
        options: &UpdateOptions,
    ) {
        match this {
            Some(this) => T::option_update_as_field(this, source.flatten(), mask, options),
            None => {
                let mut temp = None;
                T::option_update_as_field(&mut temp, source.flatten(), mask, options);
                if temp.is_some() {
                    *this = Some(temp);
                }
            }
        }
    }

    fn option_merge(this: &mut Option<Self>, source: Option<Self>, options: &UpdateOptions) {
        match this {
            Some(this) => T::option_merge(this, source.flatten(), options),
            None => {
                if source.is_some() {
                    *this = source;
                }
            }
        }
    }
}

impl<T: OptionMaskable> SelfMaskable for Option<T> {
    fn project(self, mask: &Self::Mask) -> Self {
        T::option_project(self, mask)
    }

    fn update_as_field(&mut self, source: Self, mask: &Self::Mask, options: &UpdateOptions) {
        T::option_update_as_field(self, source, mask, options)
    }

    fn merge(&mut self, source: Self, options: &UpdateOptions) {
        T::option_merge(self, source, options)
    }
}

impl<T: Maskable> Maskable for Box<T> {
    type Mask = T::Mask;

    fn empty_mask() -> Self::Mask {
        T::empty_mask()
    }

    fn full_mask() -> Self::Mask {
        T::full_mask()
    }

    fn make_mask_include_field<'a>(
        mask: &mut Self::Mask,
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        T::make_mask_include_field(mask, field_path)
    }
}

impl<T: SelfMaskable> SelfMaskable for Box<T> {
    fn project(self, mask: &Self::Mask) -> Self {
        Box::new((*self).project(mask))
    }

    fn update_as_field(&mut self, source: Self, mask: &Self::Mask, options: &UpdateOptions) {
        self.as_mut().update_as_field(*source, mask, options);
    }

    fn merge(&mut self, source: Self, options: &UpdateOptions) {
        self.as_mut().merge(*source, options);
    }
}

impl<T: OptionMaskable> OptionMaskable for Box<T> {
    fn option_project(this: Option<Self>, mask: &Self::Mask) -> Option<Self> {
        match this {
            Some(this) => T::option_project(Some(*this), mask).map(Box::new),
            None => None,
        }
    }

    fn option_update_as_field(
        this: &mut Option<Self>,
        source: Option<Self>,
        mask: &Self::Mask,
        options: &UpdateOptions,
    ) {
        let mut temp = None;
        mem::swap(this, &mut temp);
        let mut temp = temp.map(|temp| *temp);
        temp.update_as_field(source.map(|source| *source), mask, options);
        *this = temp.map(Box::new);
    }

    fn option_merge(this: &mut Option<Self>, source: Option<Self>, options: &UpdateOptions) {
        let mut temp = None;
        mem::swap(this, &mut temp);
        let mut temp = temp.map(|temp| *temp);
        temp.merge(source.map(|source| *source), options);
        *this = temp.map(Box::new);
    }
}

maskable_atomic!(impl bool {});
maskable_atomic!(impl char {});

maskable_atomic!(impl f32 {});
maskable_atomic!(impl f64 {});

maskable_atomic!(impl i8 {});
maskable_atomic!(impl u8 {});
maskable_atomic!(impl i16 {});
maskable_atomic!(impl u16 {});
maskable_atomic!(impl i32 {});
maskable_atomic!(impl u32 {});
maskable_atomic!(impl i64 {});
maskable_atomic!(impl u64 {});
maskable_atomic!(impl i128 {});
maskable_atomic!(impl u128 {});
maskable_atomic!(impl isize {});
maskable_atomic!(impl usize {});

maskable_atomic!(
    impl String {
        fn merge(&mut self, source: Self, _options: &UpdateOptions) {
            if !source.is_empty() {
                *self = source;
            }
        }
    }
);

maskable_atomic!(
    impl<K, V> HashMap<K, V> {
        fn merge(&mut self, source: Self, _options: &UpdateOptions) {
            if !source.is_empty() {
                *self = source;
            }
        }
    }
);

maskable_atomic!(
    impl<T> Vec<T> {
        fn update_as_field(&mut self, source: Self, _mask: &Self::Mask, options: &UpdateOptions) {
            self.merge(source, options);
        }

        fn merge(&mut self, source: Self, options: &UpdateOptions) {
            if options.replace_repeated {
                *self = source;
                return;
            }

            self.extend(source);
        }
    }
);

#[cfg(feature = "prost")]
mod prost_integration {
    use super::*;

    maskable_atomic!(
        impl ::prost::bytes::Bytes {
            fn merge(&mut self, source: Self, _options: &UpdateOptions) {
                if !source.is_empty() {
                    *self = source;
                }
            }
        }
    );
}
