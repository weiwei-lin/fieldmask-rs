use std::ops::{BitOr, Not};

pub trait Maskable: Sized {
    type Mask: Default + Not + BitOr;

    /// Deserialize a mask and return a FieldMask.
    ///
    /// Call deserialize_mask_impl to compute the mask and wrap the mask in FieldMask.
    ///
    /// This is the only public interface, other than bitwise, default and not operations, from
    /// which a FieldMask can be obtained.
    fn deserialize_mask<'a>(
        mask: &mut Self::Mask,
        field_mask_segs: &'a [&'a str],
    ) -> Result<(), u8>;
}

pub trait AbsoluteMaskable: Maskable {
    /// Implementation of the application process of a mask.
    fn apply_mask(&mut self, src: Self, mask: Self::Mask);
}

pub trait OptionalMaskable: Maskable {
    /// Implementation of the application process of a mask.
    fn apply_mask(&mut self, src: Self, mask: Self::Mask) -> bool;
}

impl<T: AbsoluteMaskable> OptionalMaskable for T
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

    fn deserialize_mask(mask: &mut Self::Mask, field_mask_segs: &[&str]) -> Result<(), u8> {
        T::deserialize_mask(mask, field_mask_segs)
    }
}

impl<T: OptionalMaskable> AbsoluteMaskable for Option<T>
where
    T: Default,
    T::Mask: PartialEq,
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

            fn deserialize_mask(mask: &mut Self::Mask, field_mask_segs: &[&str]) -> Result<(), u8> {
                if field_mask_segs.len() == 0 {
                    *mask = true;
                    Ok(())
                } else {
                    Err(0)
                }
            }
        }

        impl AbsoluteMaskable for $T {
            fn apply_mask(&mut self, other: Self, mask: Self::Mask) {
                if mask {
                    *self = other;
                }
            }
        }
    };
}

maskable!(u32);
maskable!(String);
