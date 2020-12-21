pub trait Maskable: Sized {
    type Mask: Default;

    /// Deserialize a mask and return a FieldMask.
    ///
    /// Call deserialize_mask_impl to compute the mask and wrap the mask in FieldMask.
    ///
    /// This is the only public interface, other than bitwise, default and not operations, from
    /// which a FieldMask can be obtained.
    fn deserialize_mask<'a>(mask: &mut Self::Mask, field_mask: &'a str) -> Result<(), ()>;

    /// Implementation of the application process of a mask.
    fn apply_mask(&mut self, src: Self, mask: Self::Mask);
}

impl<I: Maskable> Maskable for Option<I>
where
    I: Default,
    I::Mask: Default + PartialEq,
{
    type Mask = I::Mask;

    fn deserialize_mask<'a>(mask: &mut Self::Mask, field_mask: &'a str) -> Result<(), ()> {
        I::deserialize_mask(mask, field_mask)
    }

    fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
        if mask == Self::Mask::default() {
            return;
        }
        match self {
            Some(s) => match src {
                Some(o) => s.apply_mask(o, mask),
                None => *self = None,
            },
            None => match src {
                Some(o) => {
                    let mut new = I::default();
                    new.apply_mask(o, mask);
                    *self = Some(new);
                }
                None => {}
            },
        }
    }
}

impl Maskable for u32 {
    type Mask = bool;

    fn deserialize_mask<'a>(_mask: &mut Self::Mask, _field_mask: &'a str) -> Result<(), ()> {
        Err(())
    }

    fn apply_mask(&mut self, other: Self, mask: Self::Mask) {
        if mask {
            *self = other;
        }
    }
}
