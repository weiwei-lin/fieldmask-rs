use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, Default, Maskable, PartialEq)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, Maskable, PartialEq)]
struct Parent {
    child: Child,
    c: u32,
}

#[test]
fn nested() {
    let target = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let mask = vec!["child.b", "c"];
    let expected = Parent {
        child: Child {
            a: Default::default(),
            b: 2,
        },
        c: 3,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(actual, expected);
}

#[test]
fn full_child_mask() {
    let target = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let mask = vec!["child", "c"];
    let expected = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(actual, expected);
}

#[test]
fn explicit_full_child_mask() {
    let target = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let mask = vec!["child.a", "child.b", "c"];
    let expected = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(actual, expected);
}
