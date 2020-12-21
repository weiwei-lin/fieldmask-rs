use std::{
    marker::PhantomData,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

use crate::FieldMask;

pub struct Seal<'a>(PhantomData<&'a ()>);

pub trait Maskable: Sized {
    type Mask: Default + BitAnd + BitAndAssign + BitOr + BitOrAssign + BitXor + BitXorAssign;

    /// Implementation of the deserialization process of a mask.
    fn deserialize_mask_impl<'a, T: Iterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<Self::Mask, &'a str>;

    /// Deserialize a mask and return a FieldMask.
    ///
    /// Call deserialize_mask_impl to compute the mask and wrap the mask in FieldMask.
    ///
    /// This is the only public interface from which a FieldMask can be obtained.
    fn deserialize_mask<'a, T: Iterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<FieldMask<Self>, &'a str> {
        let mask = Self::deserialize_mask_impl(field_mask)?;
        Ok(FieldMask(mask))
    }

    /// Implementation of the application process of a mask.
    ///
    /// _seal is here to ensure that this function can only be called by apply_mask.
    /// You should ignore _seal when implementing this function.
    ///
    /// Because
    /// 1. FieldMask can only be obtained from deserialize_mask.
    /// 2. deserialize_mask can't have a custom implementation.
    /// 3. deserialize_mask wraps the mask  from deserialize_mask_impl into FieldMask.
    /// 4. apply_mask_impl can only be called by apply_mask.
    /// 5. apply_mask can't have a custom implementation.
    /// 6. apply_mask unwraps the mask from FieldMask and passes it to apply_mask_impl.
    ///
    /// mask passed to this function can only be generated from deserialize_mask_impl.
    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask, _seal: Seal);

    /// Update the object according to mask.
    ///
    /// This is the only function that can call apply_mask_impl thanks to Seal.
    ///
    /// It takes the mask value out of FieldMask and passes it to apply_mask_impl.
    fn apply_mask(&mut self, other: Self, mask: FieldMask<Self>) {
        let seal = Seal(PhantomData);
        self.apply_mask_impl(other, mask.0, seal);
    }
}

impl<I: Maskable> Maskable for Option<I>
where
    I: Default,
    I::Mask: Default + PartialEq,
{
    type Mask = I::Mask;

    fn deserialize_mask_impl<'a, T: Iterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<Self::Mask, &'a str> {
        I::deserialize_mask_impl(field_mask)
    }

    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask, _seal: Seal) {
        if mask == Self::Mask::default() {
            return;
        }
        match self {
            Some(s) => match other {
                Some(o) => s.apply_mask_impl(o, mask, _seal),
                None => *self = None,
            },
            None => match other {
                Some(o) => {
                    let mut new = I::default();
                    new.apply_mask_impl(o, mask, _seal);
                    *self = Some(new);
                }
                None => {}
            },
        }
    }
}
