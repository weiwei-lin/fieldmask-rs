use std::convert::TryFrom;

use fieldmask::{AbsoluteMaskable, FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Maskable, AbsoluteMaskable)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Maskable, AbsoluteMaskable)]
struct Parent {
    child: Child,
    c: u32,
}

#[test]
fn nested() {
    let mut struct1 = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let struct2 = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    let expected_struct = Parent {
        child: Child { a: 1, b: 5 },
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn full_child_mask() {
    let mut struct1 = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let struct2 = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    let expected_struct = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}
