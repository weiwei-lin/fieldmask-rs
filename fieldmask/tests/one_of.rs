use fieldmask::{BitwiseWrap, Maskable};

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
    type Mask = BitwiseWrap<(bool, bool, bool)>;

    fn deserialize_mask_impl<'a, T: Iterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<Self::Mask, &'a str> {
        let mut mask = Self::Mask::default();
        for entry in field_mask {
            match entry {
                "a" => mask.0 .0 |= true,
                "b" => mask.0 .1 |= true,
                "c" => mask.0 .2 |= true,
                _ => return Err(entry),
            }
        }
        Ok(mask)
    }

    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask) {
        match other.one_of {
            Some(OneOf::A(a)) if mask.0 .0 => self.one_of = Some(OneOf::A(a)),
            Some(OneOf::B(b)) if mask.0 .1 => self.one_of = Some(OneOf::B(b)),
            _ if mask.0 .0 || mask.0 .1 => self.one_of = None,
            _ => (),
        }
        if mask.0 .2 {
            self.c = other.c;
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
    struct1.apply_mask(
        struct2,
        Parent::deserialize_mask(vec!["b", "c"].into_iter()).expect("unable to deserialize mask"),
    );
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
    struct1.apply_mask(
        struct2,
        Parent::deserialize_mask(vec!["b", "c"].into_iter()).expect("unable to deserialize mask"),
    );
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
    struct1.apply_mask(
        struct2,
        Parent::deserialize_mask(vec!["a", "c"].into_iter()).expect("unable to deserialize mask"),
    );
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
    struct1.apply_mask(
        struct2,
        Parent::deserialize_mask(vec!["a", "c"].into_iter()).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}
