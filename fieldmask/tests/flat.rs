use fieldmask::{BitwiseWrap, Maskable};

#[derive(Debug, PartialEq, Eq)]
struct FlatStruct {
    a: u32,
    b: u32,
}

impl Maskable for FlatStruct {
    type Mask = BitwiseWrap<(bool, bool)>;

    fn deserialize_mask_impl<'a, T: Iterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<Self::Mask, &'a str> {
        let mut mask = Self::Mask::default();
        for entry in field_mask {
            match entry {
                "a" => mask.0 .0 |= true,
                "b" => mask.0 .1 |= true,
                _ => return Err(entry),
            }
        }
        Ok(mask)
    }

    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask, _seal: fieldmask::Seal) {
        if mask.0 .0 {
            self.a = other.a;
        }
        if mask.0 .1 {
            self.b = other.b;
        }
    }
}

#[test]
fn flat_struct() {
    let mut struct1 = FlatStruct { a: 1, b: 2 };
    let struct2 = FlatStruct { a: 3, b: 4 };

    let expected_struct = FlatStruct { a: 1, b: 4 };
    struct1.apply_mask(
        struct2,
        FlatStruct::deserialize_mask(vec!["b"].into_iter()).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn empty_mask() {
    let mut struct1 = FlatStruct { a: 1, b: 2 };
    let struct2 = FlatStruct { a: 3, b: 4 };

    let expected_struct = FlatStruct { a: 1, b: 2 };
    struct1.apply_mask(
        struct2,
        FlatStruct::deserialize_mask(vec![].into_iter()).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}