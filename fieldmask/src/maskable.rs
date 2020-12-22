use std::{
    iter::Peekable,
    ops::{BitOr, Not},
};

pub trait Maskable: Sized {
    type Mask: Default + Not + BitOr;

    /// Deserialize a mask and return a FieldMask.
    ///
    /// Call deserialize_mask_impl to compute the mask and wrap the mask in FieldMask.
    ///
    /// This is the only public interface, other than bitwise, default and not operations, from
    /// which a FieldMask can be obtained.
    fn deserialize_mask<'a, I: Iterator<Item = &'a str>>(
        mask: &mut Self::Mask,
        field_mask_segs: Peekable<I>,
    ) -> Result<(), ()>;
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

    fn deserialize_mask<'a, I: Iterator<Item = &'a str>>(
        mask: &mut Self::Mask,
        field_mask_segs: Peekable<I>,
    ) -> Result<(), ()> {
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

            fn deserialize_mask<'a, I: Iterator<Item = &'a str>>(
                mask: &mut Self::Mask,
                mut field_mask_segs: Peekable<I>,
            ) -> Result<(), ()> {
                match field_mask_segs.next() {
                    Some(_) => return Err(()),
                    None => *mask = true,
                }
                Ok(())
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
