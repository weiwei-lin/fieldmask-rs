use std::marker::PhantomData;

use crate::FieldMask;

pub struct Seal<'a>(PhantomData<&'a ()>);

pub trait Maskable: Sized {
    type Mask;

    /// Implementation of the deserialization process of a mask.
    fn deserialize_mask_impl<S: AsRef<str>, T: IntoIterator<Item = S>>(
        field_mask: T,
    ) -> Result<Self::Mask, S>;

    /// Deserialize a mask and return a FieldMask.
    ///
    /// Call deserialize_mask_impl to compute the mask and wrap the mask in FieldMask.
    ///
    /// This is the only public interface from which a FieldMask can be obtained.
    fn deserialize_mask<S: AsRef<str>, T: IntoIterator<Item = S>>(
        field_mask: T,
    ) -> Result<FieldMask<Self>, S> {
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
