use std::convert::TryFrom;

use derive_more::{Deref, DerefMut};

use crate::{DeserializeMaskError, maskable::Maskable};

/// A convenient wrapper around a mask value.
/// Allows us to
///  * implement traits for it.
///  * name the mask of a `Maskable` type more easily.
#[derive(Clone, Copy, Debug, Deref, DerefMut, PartialEq)]
pub struct Mask<T: Maskable>(T::Mask);

impl<T: Maskable> Default for Mask<T> {
    fn default() -> Self {
        Self(T::Mask::default())
    }
}

impl<T: Maskable> Mask<T> {
    pub fn include_field<'a>(
        &mut self,
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        T::make_mask_include_field(&mut self.0, field_path)
    }
}

pub struct MaskInput<T>(pub T);

impl<'a, I, T> TryFrom<MaskInput<I>> for Mask<T>
where
    I: Iterator<Item = &'a str>,
    T: Maskable,
    T::Mask: Default,
{
    type Error = DeserializeMaskError<'a>;

    fn try_from(value: MaskInput<I>) -> Result<Self, Self::Error> {
        let mut mask = Self::default();
        for entry in value.0 {
            mask.include_field(&entry.split('.').collect::<Vec<_>>())?;
        }
        Ok(mask)
    }
}
