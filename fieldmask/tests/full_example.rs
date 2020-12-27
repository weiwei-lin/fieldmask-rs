use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable};

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    primitive: String,
    child_1: Child,
    child_2: Child,
    #[fieldmask(flatten)]
    one_of_field: Option<OneOfField>,
}

#[derive(Debug, PartialEq, Maskable)]
struct Child {
    field_one: String,
    field_two: u32,
}

#[derive(Debug, PartialEq, Maskable)]
enum OneOfField {
    VariantOne(String),
    VariantTwo(u32),
}

impl Default for OneOfField {
    fn default() -> Self {
        Self::VariantOne(String::default())
    }
}

#[test]
fn case_1() {
    let mut target_struct = Parent {
        primitive: "string".into(),
        child_1: Child {
            field_one: "child_1 field one".into(),
            field_two: 1,
        },
        child_2: Child {
            field_one: "child_2 field one".into(),
            field_two: 2,
        },
        one_of_field: Some(OneOfField::VariantOne("variant one".into())),
    };
    let src_struct = Parent {
        primitive: "updated string".into(),
        child_1: Child {
            field_one: "updated child_1 field one".into(),
            field_two: 10,
        },
        child_2: Child {
            field_one: "updated child_2 field one".into(),
            field_two: 20,
        },
        one_of_field: Some(OneOfField::VariantTwo(50)),
    };

    let expected_struct = Parent {
        primitive: "updated string".into(),
        child_1: Child {
            field_one: "child_1 field one".into(),
            field_two: 10,
        },
        child_2: Child {
            field_one: "updated child_2 field one".into(),
            field_two: 20,
        },
        one_of_field: Some(OneOfField::VariantTwo(50)),
    };

    FieldMask::try_from(FieldMaskInput(
        vec![
            "primitive",
            "child_1.field_two",
            "child_2", // if child properties are not specified, all properties are included.
            "variant_two", // if a field is marked with `flatten`, it's properties are merged with its parents properties.
        ]
        .into_iter(),
    ))
    .expect("unable to deserialize mask")
    .apply(&mut target_struct, src_struct);

    assert_eq!(target_struct, expected_struct);
}
