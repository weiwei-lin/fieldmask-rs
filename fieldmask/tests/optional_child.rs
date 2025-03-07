use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, Default, PartialEq, Maskable)]
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
fn optional_child_selected_field() {
    let target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let mask = vec!["child.b", "c"];
    let expected = Parent {
        child: Some(Child {
            a: Default::default(),
            b: 2,
        }),
        c: 3,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn optional_child_selected_whole() {
    let target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let mask = vec!["child", "c"];
    let expected = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn child_is_none() {
    let target = Parent { child: None, c: 3 };
    let mask = vec!["child.b", "c"];
    let expected = Parent { child: None, c: 3 };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn child_not_selected() {
    let target = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let mask = vec!["c"];
    let expected = Parent {
        child: Default::default(),
        c: 3,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}
