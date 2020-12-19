use fieldmask::Maskable;

#[derive(Debug, PartialEq, Eq)]
struct FlatStruct {
    a: u32,
    b: u32,
}

impl Maskable for FlatStruct {
    type Mask = u8;

    fn deserialize_mask_impl<S: AsRef<str>, T: IntoIterator<Item = S>>(
        field_mask: T,
    ) -> Result<Self::Mask, S> {
        let mut mask = 0;
        for entry in field_mask {
            match entry.as_ref() {
                "a" => mask |= 0b1 << 0,
                "b" => mask |= 0b1 << 1,
                _ => return Err(entry),
            }
        }
        Ok(mask)
    }

    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask, _seal: fieldmask::Seal) {
        if mask & 0b1 != 0 {
            self.a = other.a;
        }
        if mask & 0b1 << 1 != 0 {
            self.b = other.b;
        }
    }
}

#[test]
fn test_flat_struct() {
    let mut struct1 = FlatStruct { a: 1, b: 2 };
    let struct2 = FlatStruct { a: 3, b: 4 };

    let expected_struct = FlatStruct { a: 1, b: 4 };
    struct1.apply_mask(
        struct2,
        FlatStruct::deserialize_mask(&["b"]).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}
