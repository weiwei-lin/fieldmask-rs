use std::convert::TryFrom;

use fieldmask::{BitwiseWrap, FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq)]
enum OneOf {
    A(String),
    B(String),
}

#[derive(Debug, PartialEq)]
struct Parent {
    one_of: Option<OneOf>,
    c: u32,
}

impl Maskable for Parent {
    type Mask = BitwiseWrap<(FieldMask<String>, FieldMask<String>, FieldMask<u32>)>;

    fn deserialize_mask<'a, I: Iterator<Item = &'a str>>(
        mask: &mut Self::Mask,
        mut field_mask_segs: I,
    ) -> Result<(), ()> {
        let seg = field_mask_segs.next();
        match seg {
            None => *mask = !Self::Mask::default(),
            Some("a") => mask.0 .0.try_bitand_assign(field_mask_segs)?,
            Some("b") => mask.0 .1.try_bitand_assign(field_mask_segs)?,
            Some("c") => mask.0 .2.try_bitand_assign(field_mask_segs)?,
            Some(_) => return Err(()),
        }
        Ok(())
    }

    fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
        match src.one_of {
            Some(OneOf::A(a)) if mask.0 .0 != FieldMask::default() => {
                self.one_of = Some(OneOf::A(a))
            }
            Some(OneOf::B(b)) if mask.0 .1 != FieldMask::default() => {
                self.one_of = Some(OneOf::B(b))
            }
            _ if mask.0 .0 != FieldMask::default() || mask.0 .1 != FieldMask::default() => {
                self.one_of = None
            }
            _ => (),
        }
        if mask.0 .2 != FieldMask::default() {
            self.c = src.c;
        }
    }
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
