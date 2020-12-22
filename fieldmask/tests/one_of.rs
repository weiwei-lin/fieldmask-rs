use std::{convert::TryFrom, iter::Peekable};

use fieldmask::{
    AbsoluteMaskable, BitwiseWrap, FieldMask, FieldMaskInput, Maskable, OptionalMaskable,
};

#[derive(Debug, PartialEq, Maskable, OptionalMaskable)]
enum OneOf {
    A(String),
    B(String),
}

impl Default for OneOf {
    fn default() -> Self {
        Self::A(String::default())
    }
}

#[derive(Debug, PartialEq)]
struct Parent {
    one_of: Option<OneOf>,
    c: u32,
}

impl Maskable for Parent {
    type Mask = BitwiseWrap<(FieldMask<Option<OneOf>>, FieldMask<u32>)>;

    fn deserialize_mask<'a, I: Iterator<Item = &'a str>>(
        mask: &mut Self::Mask,
        mut field_mask_segs: Peekable<I>,
    ) -> Result<(), ()> {
        let seg = field_mask_segs.peek();
        match seg {
            None => *mask = !Self::Mask::default(),
            Some(&"a") | Some(&"b") => mask.0 .0.try_bitand_assign(field_mask_segs)?,
            Some(&"c") => {
                field_mask_segs.next().expect("should not be None");
                mask.0 .1.try_bitand_assign(field_mask_segs)?
            }
            Some(_) => return Err(()),
        }
        Ok(())
    }
}

impl AbsoluteMaskable for Parent {
    fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
        mask.0 .0.apply(&mut self.one_of, src.one_of);
        mask.0 .1.apply(&mut self.c, src.c);
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
