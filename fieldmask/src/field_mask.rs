use core::{
    convert::TryFrom,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

use derive_more::{AsMut, AsRef, Deref, DerefMut, From};
use thiserror::Error;

use crate::maskable::{AbsoluteMaskable, DeserializeMaskError, Maskable};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FieldMask<T: Maskable>(T::Mask);

impl<T: Maskable> FieldMask<T> {
    pub fn try_bitor_assign<'a>(
        &mut self,
        rhs: &[&'a str],
    ) -> Result<(), DeserializeMaskError<'a>> {
        T::try_bitor_assign_mask(&mut self.0, rhs)
    }
}

pub struct FieldMaskInput<T>(pub T);

#[derive(Debug, Error)]
#[error(r#"can not parse fieldmask "{entry}"; {err}"#)]
pub struct DeserializeFieldMaskError<'a> {
    pub entry: &'a str,
    err: DeserializeMaskError<'a>,
}

impl<'a, I, T> TryFrom<FieldMaskInput<I>> for FieldMask<T>
where
    I: Iterator<Item = &'a str>,
    T: Maskable,
    T::Mask: Default,
{
    type Error = DeserializeFieldMaskError<'a>;

    fn try_from(value: FieldMaskInput<I>) -> Result<Self, Self::Error> {
        let mut mask = Self::default();
        for entry in value.0 {
            mask.try_bitor_assign(&entry.split('.').collect::<Vec<_>>())
                .map_err(|err| DeserializeFieldMaskError { entry, err })?;
        }
        Ok(mask)
    }
}

impl<T: AbsoluteMaskable> FieldMask<T> {
    /// Update the object according to mask.
    pub fn apply(self, target: &mut T, src: T) {
        T::apply_mask(target, src, self.0);
    }
}

impl<T> BitAnd for FieldMask<T>
where
    T: Maskable,
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
        self.0 &= rhs.0;
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
        self.0 |= rhs.0;
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
        self.0 ^= rhs.0;
    }
}

impl<T: Maskable> Not for FieldMask<T>
where
    T::Mask: Not<Output = T::Mask>,
{
    type Output = Self;

    fn not(self) -> Self::Output {
        FieldMask(!self.0)
    }
}

impl<T: Maskable> Default for FieldMask<T>
where
    T::Mask: Default,
{
    fn default() -> Self {
        FieldMask(T::Mask::default())
    }
}

#[derive(AsMut, AsRef, Deref, DerefMut, From, Default, PartialEq, Debug)]
#[deref(forward)]
#[deref_mut(forward)]
pub struct BitwiseWrap<T>(pub T);

macro_rules! tuple_impls {
    ($($idx:tt -> $T:ident),+) => {
        impl<$($T: BitAnd<Output = $T>),+> BitAnd for BitwiseWrap<($($T),+,)> {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                BitwiseWrap(($(self.0.$idx & rhs.0.$idx),+,))
            }
        }

        impl<$($T: BitAndAssign),+> BitAndAssign for BitwiseWrap<($($T),+,)> {
            fn bitand_assign(&mut self, rhs: Self) {
                $(self.0.$idx &= rhs.0.$idx;)+
            }
        }

        impl<$($T: BitOr<Output = $T>),+> BitOr for BitwiseWrap<($($T),+,)> {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                BitwiseWrap(($(self.0.$idx | rhs.0.$idx),+,))
            }
        }

        impl<$($T: BitOrAssign),+> BitOrAssign for BitwiseWrap<($($T),+,)> {
            fn bitor_assign(&mut self, rhs: Self) {
                $(self.0.$idx |= rhs.0.$idx;)+
            }
        }

        impl<$($T: BitXor<Output = $T>),+> BitXor for BitwiseWrap<($($T),+,)> {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output {
                BitwiseWrap(($(self.0.$idx ^ rhs.0.$idx),+,))
            }
        }

        impl<$($T: BitXorAssign),+> BitXorAssign for BitwiseWrap<($($T),+,)> {
            fn bitxor_assign(&mut self, rhs: Self) {
                $(self.0.$idx ^= rhs.0.$idx;)+
            }
        }

        impl<$($T: Not<Output = $T>),+> Not for BitwiseWrap<($($T),+,)> {
            type Output = Self;

            fn not(self) -> Self::Output {
                BitwiseWrap(($(!self.0.$idx),+,))
            }
        }
    };
}

tuple_impls!(0 -> T0);
tuple_impls!(0 -> T0, 1 -> T1);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6, 7 -> T7);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6, 7 -> T7, 8 -> T8);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6, 7 -> T7, 8 -> T8, 9 -> T9);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6, 7 -> T7, 8 -> T8, 9 -> T9, 10 -> T10);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6, 7 -> T7, 8 -> T8, 9 -> T9, 10 -> T10, 11 -> T11);
tuple_impls!(0 -> T0, 1 -> T1, 2 -> T2, 3 -> T3, 4 -> T4, 5 -> T5, 6 -> T6, 7 -> T7, 8 -> T8, 9 -> T9, 10 -> T10, 11 -> T11, 12 -> T12);
