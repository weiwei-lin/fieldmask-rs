use std::convert::TryFrom;

use fieldmask::{BitwiseWrap, FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Eq, Default)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Parent {
    child: Option<Child>,
    c: u32,
}

impl Maskable for Child {
    type Mask = BitwiseWrap<(bool, bool)>;

    fn deserialize_mask<'a>(mask: &mut Self::Mask, field_mask: &'a str) -> Result<(), ()> {
        match field_mask {
            "a" => mask.0 .0 |= true,
            "b" => mask.0 .1 |= true,
            _ => return Err(()),
        }
        Ok(())
    }

    fn apply_mask(&mut self, other: Self, mask: Self::Mask) {
        if mask.0 .0 {
            self.a = other.a;
        }
        if mask.0 .1 {
            self.b = other.b;
        }
    }
}

impl Maskable for Parent {
    type Mask = BitwiseWrap<(FieldMask<Option<Child>>, bool)>;

    fn deserialize_mask<'a>(mask: &mut Self::Mask, field_mask: &'a str) -> Result<(), ()> {
        match field_mask {
            "child" => mask.0 .0 |= !FieldMask::<Option<Child>>::default(),
            child if child.starts_with("child.") => {
                let child_attr = &child["child.".len()..];
                mask.0 .0 |= FieldMask::try_from(FieldMaskInput(vec![child_attr].into_iter()))
                    .map_err(|_| ())?;
            }
            "c" => mask.0 .1 |= true,
            _ => return Err(()),
        }
        Ok(())
    }

    fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
        mask.0 .0.apply(&mut self.child, src.child);
        if mask.0 .1 {
            self.c = src.c;
        }
    }
}

#[test]
fn optional_child() {
    let mut struct1 = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = Parent {
        child: Some(Child { a: 4, b: 5 }),
        c: 6,
    };

    let expected_struct = Parent {
        child: Some(Child { a: 1, b: 5 }),
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn other_child_is_none() {
    let mut struct1 = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = Parent { child: None, c: 6 };

    let expected_struct = Parent { child: None, c: 6 };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn self_child_is_none() {
    let mut struct1 = Parent { child: None, c: 3 };
    let struct2 = Parent {
        child: Some(Child { a: 4, b: 5 }),
        c: 6,
    };

    let expected_struct = Parent {
        child: Some(Child { a: 0, b: 5 }),
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn both_children_are_none() {
    let mut struct1 = Parent { child: None, c: 3 };
    let struct2 = Parent { child: None, c: 6 };

    let expected_struct = Parent { child: None, c: 6 };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn no_mask_applied_to_child() {
    let mut struct1 = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = Parent { child: None, c: 6 };

    let expected_struct = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn full_child_mask() {
    let mut struct1 = Parent {
        child: Some(Child { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = Parent { child: None, c: 6 };

    let expected_struct = Parent { child: None, c: 6 };

    FieldMask::try_from(FieldMaskInput(vec!["child", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}
