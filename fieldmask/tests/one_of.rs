use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable, OptionMaskable, SelfMaskable};

#[derive(Debug, PartialEq, Maskable, OptionMaskable)]
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

#[derive(Debug, PartialEq, Maskable, SelfMaskable)]
struct Parent {
    #[fieldmask(flatten)]
    one_of: Option<OneOf>,
    c: u32,
}

#[test]
fn one_of() {
    let mut struct1 = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let struct2 = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    let expected_struct = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn no_field() {
    let mut struct1 = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let struct2 = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    let expected_struct = Parent { one_of: None, c: 2 };

    FieldMask::try_from(FieldMaskInput(vec!["b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn matched_field() {
    let mut struct1 = Parent {
        one_of: Some(OneOf::A("a".into())),
        c: 1,
    };
    let struct2 = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    let expected_struct = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["a", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn self_none() {
    let mut struct1 = Parent { one_of: None, c: 1 };
    let struct2 = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    let expected_struct = Parent {
        one_of: Some(OneOf::A("a2".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["a", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn snake_case() {
    let mut struct1 = Parent { one_of: None, c: 1 };
    let struct2 = Parent {
        one_of: Some(OneOf::AnotherCase("a2".into())),
        c: 2,
    };

    let expected_struct = Parent {
        one_of: Some(OneOf::AnotherCase("a2".into())),
        c: 2,
    };

    FieldMask::try_from(FieldMaskInput(vec!["another_case", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}
