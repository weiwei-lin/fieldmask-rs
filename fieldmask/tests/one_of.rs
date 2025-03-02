use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Maskable)]
enum OneOf {
    A(String),
    B(String),
    AnotherCase(String),
}

impl Default for OneOf {
    fn default() -> Self {
        Self::A(String::default())
    }
}

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    #[fieldmask(flatten)]
    one_of: Option<OneOf>,
    c: u32,
}

#[test]
fn one_of() {
    let mut target = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let update = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    let expected = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn different_variant() {
    let mut target = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let update = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    let expected = Parent { one_of: None, c: 2 };

    FieldMask::try_from(FieldMaskInput(vec!["a", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn different_variant_both_in_mask() {
    let mut target = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let update = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    let expected = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["a", "b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn no_field() {
    let mut target = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let update = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    let expected = Parent { one_of: None, c: 2 };

    FieldMask::try_from(FieldMaskInput(vec!["b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn matched_field() {
    let mut target = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let update = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    let expected = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["a", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn self_none() {
    let mut target = Parent { one_of: None, c: 1 };
    let update = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    let expected = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["a", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}

#[test]
fn snake_case() {
    let mut target = Parent { one_of: None, c: 1 };
    let update = Parent {
        one_of: Some(OneOf::AnotherCase("a2".into())),
        c: 2,
    };

    let expected = Parent {
        one_of: Some(OneOf::AnotherCase("a2".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["another_case", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut target, update);
    assert_eq!(target, expected);
}
