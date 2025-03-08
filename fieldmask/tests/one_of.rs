use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, PartialEq, Maskable)]
enum OneOf {
    A(String),
    B(String),
    AnotherCase(String),
}

impl Default for OneOf {
    fn default() -> Self {
        Self::A("".into())
    }
}

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    one_of: Option<OneOf>,
    c: u32,
}

#[test]
fn selected_variant_is_the_same() {
    let target = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 1,
    };
    let mask = vec!["one_of.b", "c"];
    let expected = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 1,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn selected_variant_is_different() {
    let target = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 1,
    };
    let mask = vec!["one_of.a", "c"];
    let expected = Parent { one_of: None, c: 1 };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn both_variants_selected() {
    let target = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 1,
    };
    let mask = vec!["one_of.a", "one_of.b", "c"];
    let expected = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 1,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn no_field_selected() {
    let target = Parent {
        one_of: Some(OneOf::B("b".into())),
        c: 1,
    };
    let mask = vec!["c"];
    let expected = Parent {
        one_of: Default::default(),
        c: 1,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn snake_case() {
    let target = Parent {
        one_of: Some(OneOf::AnotherCase("another".into())),
        c: 1,
    };
    let mask = vec!["one_of.another_case", "c"];
    let expected = Parent {
        one_of: Some(OneOf::AnotherCase("another".into())),
        c: 1,
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}
