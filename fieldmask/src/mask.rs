use std::convert::TryFrom;

use derive_more::{Deref, DerefMut};

use crate::{DeserializeMaskError, Maskable, SelfMaskable, UpdateOptions};

/// A convenient wrapper around a mask value.
/// Allows us to
///  * implement traits and methods for it.
///  * name the mask of a `Maskable` type more easily.
#[derive(Deref, DerefMut)]
pub struct Mask<T: Maskable>(T::Mask);

impl<T: Maskable> Mask<T> {
    /// Returns a full mask that selects all fields.
    pub fn full() -> Self {
        Self(T::full_mask())
    }

    /// Includes the field specified by `field_path``.
    ///
    /// When the function returns `Ok`, `self` is modified to include the field specified by
    /// `field_path`. Otherwise, `self` is unchanged.
    ///
    /// `field_path` is a field mask path splitted by '.'.
    pub fn include_field<'a>(
        &mut self,
        field_path: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        T::make_mask_include_field(&mut self.0, field_path)
    }
}

impl<T: SelfMaskable> Mask<T> {
    /// Project the fields of `source` according to the field mask.
    ///
    /// An empty field mask is treated as a full mask.
    pub fn project(&self, source: T) -> T {
        source.project(self)
    }

    /// Update the fields of `target` with the fields of `source` according to the field mask.
    ///
    /// An empty field mask is treated as a full mask.
    pub fn update(&self, target: &mut T, source: T, options: &UpdateOptions) {
        if self == &Self::default() {
            target.update_as_field(source, &Self::full(), options);
            return;
        }
        target.update_as_field(source, self, options);
    }
}

impl<T> std::fmt::Debug for Mask<T>
where
    T: Maskable,
    T::Mask: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Mask").field(&self.0).finish()
    }
}

impl<T: Maskable> Default for Mask<T> {
    fn default() -> Self {
        Self(T::Mask::default())
    }
}

impl<T: Maskable> PartialEq for Mask<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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
