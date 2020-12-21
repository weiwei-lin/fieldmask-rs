use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Maskable)]
struct Flat {
    a: u32,
    b: u32,
}

#[test]
fn flat() {
    let mut struct1 = Flat { a: 1, b: 2 };
    let struct2 = Flat { a: 3, b: 4 };

    let expected_struct = Flat { a: 1, b: 4 };

    FieldMask::try_from(FieldMaskInput(vec!["b"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn empty_mask() {
    let mut struct1 = Flat { a: 1, b: 2 };
    let struct2 = Flat { a: 3, b: 4 };

    let expected_struct = Flat { a: 1, b: 2 };

    FieldMask::try_from(FieldMaskInput(vec![].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}
