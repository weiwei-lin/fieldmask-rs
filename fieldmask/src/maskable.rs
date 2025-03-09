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
    /// `Mask` must implements the `Default` trait. When constructing a mask, the default mask is
    /// used as the initial mask that selects no fields.
    ///
    /// `Mask` must also implements the `PartialEq` trait. We need to compare the mask with the
    /// default mask to determine whether any of the field is selected.
    ///
    /// An empty `mask` (i.e. the default value) is the same as a full `mask`. This is consistent
    /// with the [official field mask protobuf specification][1].
    ///
    /// [1]: https://protobuf.dev/reference/protobuf/google.protobuf/#field-mask.
    type Mask: Default + PartialEq;

    // Returns a full mask that selects all fields.
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
pub trait OptionMaskable: Maskable + Sized {
    /// Similar to `SelfMaskable::project`, but it takes `Option<Self>` instead of `Self`.
    fn option_project(this: Option<Self>, mask: &Self::Mask) -> Option<Self>;

    /// Similar to `SelfMaskable::update_as_field`, but it takes `Option<Self>` instead of `Self`.
    ///
    /// Note that a field of message type with unspecified value should be treated the same as the
    /// field with a default value. This must be true. Otherwise, we cannot clearly define the
    /// behavior of the update operation when `source` is `None`.
    fn option_update_as_field(
        this: &mut Option<Self>,
        source: Option<Self>,
        mask: &Self::Mask,
        options: &UpdateOptions,
    );

    /// Similar to `SelfMaskable::merge`, but it takes `Option<Self>` instead of `Self`.
    fn option_merge(this: &mut Option<Self>, source: Option<Self>, options: &UpdateOptions);
}

/// If we want a `SelfMaskable` to be `OptionMaskable`, it must implement `Default`. Otherwise, we
/// cannot update the message with a partial mask when the source value is `None`.
impl<T: SelfMaskable + Default> OptionMaskable for T {
    fn option_project(this: Option<Self>, mask: &Self::Mask) -> Option<Self> {
        this.map(|this| this.project(mask))
    }

    fn option_update_as_field(
        this: &mut Option<Self>,
        source: Option<Self>,
        mask: &Self::Mask,
        options: &UpdateOptions,
    ) {
        match (this.as_mut(), source) {
            (Some(this), Some(source)) => {
                this.update_as_field(source, mask, options);
            }
            (Some(this), None) => {
                this.update_as_field(Self::default(), mask, options);
            }
            (None, source) => {
                *this = source.map(|s| s.project(mask));
            }
        }
    }

    fn option_merge(this: &mut Option<Self>, source: Option<Self>, options: &UpdateOptions) {
        match (this.as_mut(), source) {
            (Some(this), Some(source)) => {
                this.merge(source, options);
            }
            (_, None) => {}
            (None, source) => {
                *this = source;
            }
        }
    }
}

impl<T: Maskable> Maskable for Option<T> {
    type Mask = T::Mask;

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

macro_rules! maskable_atomic {
    ($name:ident$(<$($ty_param:ident),*>)? $(where $($where_clause:tt)+)?) => {
        maskable_atomic!(
            $name$(<$($ty_param),*>)? $(where $($where_clause)+)?
            fn merge(&mut self, source: Self, _options: &UpdateOptions) {
                if source != Default::default() {
                    *self = source;
                }
            }
        );
    };
    (
        $name:ident$(<$($ty_param:ident),*>)? $(where $($where_clause:tt)+)?
        fn merge($($fn_params:tt)*) {
            $($fn_impl:tt)*
        }
    ) => {
        impl$(<$($ty_param),*>)? Maskable for $name$(<$($ty_param),*>)?
        $(where $($where_clause)+)?
        {
            type Mask = ();

            fn full_mask() -> Self::Mask {}

            fn make_mask_include_field<'a>(_mask: &mut Self::Mask, field_path: &[&'a str]) -> Result<(), DeserializeMaskError<'a>> {
                if field_path.is_empty() {
                    return Ok(());
                }
                Err(DeserializeMaskError::FieldNotFound {
                    type_name: stringify!($name),
                    field: field_path[0],
                })
            }
        }

        impl$(<$($ty_param),*>)? SelfMaskable for $name$(<$($ty_param),*>)?
        $(where $($where_clause)+)?
        {
            fn project(self, _mask: &Self::Mask) -> Self {
                return self;
            }

            fn update_as_field(&mut self, source: Self, _mask: &Self::Mask, _options: &UpdateOptions) {
                *self = source;
            }

            fn merge($($fn_params)*) {
                $($fn_impl)*
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

maskable_atomic!(
    String

    fn merge(&mut self, source: Self, _options: &UpdateOptions) {
        if !source.is_empty() {
            *self = source;
        }
    }
);
maskable_atomic!(
    HashMap<K, V>

    fn merge(&mut self, source: Self, _options: &UpdateOptions) {
        if !source.is_empty() {
            *self = source;
        }
    }
);

impl<T> Maskable for Vec<T> {
    type Mask = ();

    fn full_mask() -> Self::Mask {}

    fn make_mask_include_field<'a>(
        _mask: &mut Self::Mask,
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        if field_path.is_empty() {
            return Ok(());
        }
        Err(DeserializeMaskError::FieldNotFound {
            type_name: "Vec",
            field: field_path[0],
        })
    }
}

impl<T> SelfMaskable for Vec<T> {
    fn project(self, _mask: &Self::Mask) -> Self {
        self
    }

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

#[cfg(feature = "prost")]
mod prost_integration {
    use ::prost::bytes::Bytes;

    use super::*;

    maskable_atomic!(
        Bytes
        fn merge(&mut self, source: Self, _options: &UpdateOptions) {
            if !source.is_empty() {
                *self = source;
            }
        }
    );
}
