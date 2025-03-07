use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, Maskable, PartialEq)]
struct Flat {
    a: u32,
    b: u32,
}

#[test]
fn flat() {
    let target = Flat { a: 1, b: 2 };
    let mask = vec!["b"];
    let expected = Flat {
        a: Default::default(),
        b: 2,
    };

    let mask =
        Mask::<Flat>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn empty_mask() {
    let target = Flat { a: 1, b: 2 };
    let mask = vec![];
    let expected = Flat { a: 1, b: 2 };

    let mask =
        Mask::<Flat>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}

#[test]
fn nested_mask() {
    let mask = vec!["a.b"];

    assert_eq!(
        Mask::<Flat>::try_from(MaskInput(mask.into_iter()))
            .expect_err("should fail to parse fieldmask")
            .to_string(),
        "\
        error in field \"a\":\n\
        \ttype `u32` has no field named \"b\"\
        ",
    );
}
