use std::convert::TryFrom;

use fieldmask::{BitwiseWrap, FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Eq)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Parent {
    child: Child,
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
    type Mask = BitwiseWrap<(FieldMask<Child>, bool)>;

    fn deserialize_mask<'a>(mask: &mut Self::Mask, field_mask: &'a str) -> Result<(), ()> {
        match field_mask {
            "child" => mask.0 .0 |= !FieldMask::<Child>::default(),
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

    fn apply_mask(&mut self, other: Self, mask: Self::Mask) {
        mask.0 .0.apply(&mut self.child, other.child);
        if mask.0 .1 {
            self.c = other.c;
        }
    }
}

#[test]
fn nested() {
    let mut struct1 = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let struct2 = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    let expected_struct = Parent {
        child: Child { a: 1, b: 5 },
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child.b", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}

#[test]
fn full_child_mask() {
    let mut struct1 = Parent {
        child: Child { a: 1, b: 2 },
        c: 3,
    };
    let struct2 = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    let expected_struct = Parent {
        child: Child { a: 4, b: 5 },
        c: 6,
    };

    FieldMask::try_from(FieldMaskInput(vec!["child", "c"].into_iter()))
        .expect("unable to deserialize mask")
        .apply(&mut struct1, struct2);
    assert_eq!(struct1, expected_struct);
}
