use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use crate::maskable::Maskable;

#[derive(Clone, Copy)]
pub struct FieldMask<T: Maskable>(pub(crate) T::Mask);

impl<T: Maskable> BitAnd for FieldMask<T>
where
    T::Mask: BitAnd<Output = T::Mask>,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        FieldMask(self.0 & rhs.0)
    }
}

impl<T: Maskable> BitAndAssign for FieldMask<T>
where
    T::Mask: BitAndAssign,
{
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl<T: Maskable> BitOr for FieldMask<T>
where
    T::Mask: BitOr<Output = T::Mask>,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        FieldMask(self.0 | rhs.0)
    }
}

impl<T: Maskable> BitOrAssign for FieldMask<T>
where
    T::Mask: BitOrAssign,
{
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl<T: Maskable> BitXor for FieldMask<T>
where
    T::Mask: BitXor<Output = T::Mask>,
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        FieldMask(self.0 ^ rhs.0)
    }
}

impl<T: Maskable> BitXorAssign for FieldMask<T>
where
    T::Mask: BitXorAssign,
{
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
