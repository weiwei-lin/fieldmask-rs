use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Maskable)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    child: Child,
    c: u32,
}

#[test]
fn nested() {
    let mut target = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let update = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    let expected = Parent {
        child: Child { a: 1, b: 5 },
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn full_child_mask() {
    let mut target = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let update = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    let expected = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}
