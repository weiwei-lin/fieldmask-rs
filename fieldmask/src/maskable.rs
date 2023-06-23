use thiserror::Error;

#[derive(Debug, Error)]
#[error(r#"there's no "{field}" in `{type_str}`"#)]
pub struct DeserializeMaskError {
    pub type_str: &'static str,
    pub field: String,
    pub depth: u8,
}

pub trait Maskable: Sized {
    type Mask;

    /// Perform a 'bitor' operation between the `mask` and a fieldmask in string format.
    /// When the function returns Ok, `mask` should be modified to include fields in
    /// `field_mask_segs`.
    fn try_bitor_assign_mask(
        // Take a reference here instead of the ownership. Because:
        // 1. We may want to try performing other operations on `mask` if the current one doesn't
        // work.
        // 2. Therefore, if we take the ownership, we will need to return the ownership even when
        // the method failed, which is cumbersome.
        mask: &mut Self::Mask,
        // Take a slice of segments instead of a full fieldmask string. Because:
        // 1. It's easier to perform pattern matching on slices.
        // 2. It's easier to distinguish empty fieldmask (e.g. "") and empty tail (e.g. "parent.").
        field_mask_segs: &[&str],
    ) -> Result<(), DeserializeMaskError>;
}

pub trait SelfMaskable: Maskable {
    /// Implementation of the application process of a mask.
    fn apply_mask(&mut self, src: Self, mask: Self::Mask);
}

pub trait OptionMaskable: Maskable {
    /// Implementation of the application process of a mask.
    fn apply_mask(&mut self, src: Self, mask: Self::Mask) -> bool;
}

impl<T: SelfMaskable> OptionMaskable for T
where
    T: Default,
    T::Mask: PartialEq,
{
    fn apply_mask(&mut self, src: Self, mask: Self::Mask) -> bool {
        self.apply_mask(src, mask);
        true
    }
}

impl<T: Maskable> Maskable for Option<T>
where
    T: Default,
    T::Mask: PartialEq,
{
    type Mask = T::Mask;

    fn try_bitor_assign_mask(
        mask: &mut Self::Mask,
        field_mask_segs: &[&str],
    ) -> Result<(), DeserializeMaskError> {
        T::try_bitor_assign_mask(mask, field_mask_segs)
    }
}

impl<T: OptionMaskable> SelfMaskable for Option<T>
where
    T: Default,
    T::Mask: PartialEq + Default,
{
    fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
        if mask == Self::Mask::default() {
            return;
        }
        match self {
            Some(s) => match src {
                Some(o) => {
                    if !s.apply_mask(o, mask) {
                        *self = None;
                    }
                }
                None => *self = None,
            },
            None => match src {
                Some(o) => {
                    let mut new = T::default();
                    if new.apply_mask(o, mask) {
                        *self = Some(new);
                    } else {
                        *self = None;
                    }
                }
                None => {}
            },
        }
    }
}

macro_rules! maskable {
    ($T:ident) => {
        impl Maskable for $T {
            type Mask = bool;

            fn try_bitor_assign_mask(
                mask: &mut Self::Mask,
                field_mask_segs: &[&str],
            ) -> Result<(), DeserializeMaskError> {
                if field_mask_segs.len() == 0 {
                    *mask = true;
                    Ok(())
                } else {
                    Err(DeserializeMaskError {
                        type_str: stringify!($T),
                        field: field_mask_segs[0].into(),
                        depth: 0,
                    })
                }
            }
        }

        impl SelfMaskable for $T {
            fn apply_mask(&mut self, other: Self, mask: Self::Mask) {
                if mask {
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

    fn try_bitor_assign_mask(
        mask: &mut Self::Mask,
        field_mask_segs: &[&str],
    ) -> Result<(), DeserializeMaskError> {
        if field_mask_segs.len() == 0 {
            *mask = true;
            Ok(())
        } else {
            Err(DeserializeMaskError {
                type_str: "Vec",
                field: field_mask_segs[0].into(),
                depth: 0,
            })
        }
    }
}

impl<T> SelfMaskable for Vec<T> {
    fn apply_mask(&mut self, other: Self, mask: Self::Mask) {
        if mask {
            *self = other;
        }
    }
}
