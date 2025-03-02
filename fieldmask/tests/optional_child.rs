use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Default, Maskable)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    child: Option<Child>,
    c: u32,
}

#[test]
fn optional_child() {
    let mut target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let update = Parent {
        child: Some(Child { a: 4, b: 5 }),
        c: 6,
    };

    let expected = Parent {
        child: Some(Child { a: 1, b: 5 }),
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn other_child_is_none() {
    let mut target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let update = Parent { child: None, c: 6 };

    let expected = Parent { child: None, c: 6 };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn self_child_is_none() {
    let mut target = Parent { child: None, c: 3 };
    let update = Parent {
        child: Some(Child { a: 4, b: 5 }),
        c: 6,
    };

    let expected = Parent {
        child: Some(Child { a: 0, b: 5 }),
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn both_children_are_none() {
    let mut target = Parent { child: None, c: 3 };
    let update = Parent { child: None, c: 6 };

    let expected = Parent { child: None, c: 6 };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn no_mask_applied_to_child() {
    let mut target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let update = Parent { child: None, c: 6 };

    let expected = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn full_child_mask() {
    let mut target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let update = Parent { child: None, c: 6 };

    let expected = Parent { child: None, c: 6 };

    FieldMask::try_from(FieldMaskInput(vec!["child", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}
