use fieldmask::{BitwiseWrap, FieldMask, Maskable, Seal};

#[derive(Debug, PartialEq, Eq, Default)]
struct ChildStruct {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct ParentStruct {
    child: Option<ChildStruct>,
    c: u32,
}

impl Maskable for ChildStruct {
    type Mask = BitwiseWrap<(bool, bool)>;

    fn deserialize_mask_impl<'a, T: IntoIterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<Self::Mask, &'a str> {
        let mut mask = Self::Mask::default();
        for entry in field_mask {
            match entry {
                "a" => mask.0 .0 |= true,
                "b" => mask.0 .1 |= true,
                _ => return Err(entry),
            }
        }
        Ok(mask)
    }

    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask, _seal: Seal) {
        if mask.0 .0 {
            self.a = other.a;
        }
        if mask.0 .1 {
            self.b = other.b;
        }
    }
}

impl Maskable for ParentStruct {
    type Mask = BitwiseWrap<(FieldMask<Option<ChildStruct>>, bool)>;

    fn deserialize_mask_impl<'a, T: IntoIterator<Item = &'a str>>(
        field_mask: T,
    ) -> Result<Self::Mask, &'a str> {
        let mut mask = Self::Mask::default();

        for entry in field_mask {
            match entry {
                "child" => mask.0 .0 |= !FieldMask::<Option<ChildStruct>>::default(),
                child if child.starts_with("child.") => {
                    let child_attr = &child["child.".len()..];
                    mask.0 .0 |=
                        Option::<ChildStruct>::deserialize_mask(vec![child_attr].into_iter())?;
                }
                "c" => mask.0 .1 |= true,
                _ => return Err(entry),
            }
        }
        Ok(mask)
    }

    fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask, _seal: fieldmask::Seal) {
        self.child.apply_mask(other.child, mask.0 .0);
        if mask.0 .1 {
            self.c = other.c;
        }
    }
}

#[test]
fn optional_child() {
    let mut struct1 = ParentStruct {
        child: Some(ChildStruct { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = ParentStruct {
        child: Some(ChildStruct { a: 4, b: 5 }),
        c: 6,
    };

    let expected_struct = ParentStruct {
        child: Some(ChildStruct { a: 1, b: 5 }),
        c: 6,
    };

    struct1.apply_mask(
        struct2,
        ParentStruct::deserialize_mask(vec!["child.b", "c"].into_iter())
            .expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn other_child_is_none() {
    let mut struct1 = ParentStruct {
        child: Some(ChildStruct { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = ParentStruct { child: None, c: 6 };

    let expected_struct = ParentStruct { child: None, c: 6 };

    struct1.apply_mask(
        struct2,
        ParentStruct::deserialize_mask(vec!["child.b", "c"].into_iter())
            .expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn self_child_is_none() {
    let mut struct1 = ParentStruct { child: None, c: 3 };
    let struct2 = ParentStruct {
        child: Some(ChildStruct { a: 4, b: 5 }),
        c: 6,
    };

    let expected_struct = ParentStruct {
        child: Some(ChildStruct { a: 0, b: 5 }),
        c: 6,
    };

    struct1.apply_mask(
        struct2,
        ParentStruct::deserialize_mask(vec!["child.b", "c"].into_iter())
            .expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn both_children_are_none() {
    let mut struct1 = ParentStruct { child: None, c: 3 };
    let struct2 = ParentStruct { child: None, c: 6 };

    let expected_struct = ParentStruct { child: None, c: 6 };

    struct1.apply_mask(
        struct2,
        ParentStruct::deserialize_mask(vec!["child.b", "c"].into_iter())
            .expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn no_mask_applied_to_child() {
    let mut struct1 = ParentStruct {
        child: Some(ChildStruct { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = ParentStruct { child: None, c: 6 };

    let expected_struct = ParentStruct {
        child: Some(ChildStruct { a: 1, b: 2 }),
        c: 6,
    };

    struct1.apply_mask(
        struct2,
        ParentStruct::deserialize_mask(vec!["c"].into_iter()).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn full_child_mask() {
    let mut struct1 = ParentStruct {
        child: Some(ChildStruct { a: 1, b: 2 }),
        c: 3,
    };
    let struct2 = ParentStruct { child: None, c: 6 };

    let expected_struct = ParentStruct { child: None, c: 6 };

    struct1.apply_mask(
        struct2,
        ParentStruct::deserialize_mask(vec!["child", "c"].into_iter())
            .expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}
