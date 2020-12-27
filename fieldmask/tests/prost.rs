#![cfg(feature = "prost-integration")]

use std::convert::TryFrom;

use fieldmask::{FieldMask, FieldMaskInput, Maskable};

#[derive(PartialEq, Maskable, ::prost::Message)]
struct Parent {
    #[prost(string, tag = "1")]
    primitive: String,
    #[prost(message, tag = "2")]
    child_1: Option<Child>,
    #[prost(message, tag = "3")]
    child_2: Option<Child>,
    #[prost(oneof = "OneOfField", tags = "4, 5")]
    one_of_field: Option<OneOfField>,
}

#[derive(PartialEq, Maskable, ::prost::Message)]
struct Child {
    #[prost(string, tag = "1")]
    field_one: String,
    #[prost(uint32, tag = "2")]
    field_two: u32,
}

#[derive(PartialEq, Maskable, ::prost::Oneof)]
enum OneOfField {
    #[prost(string, tag = "3")]
    VariantOne(String),
    #[prost(uint32, tag = "4")]
    VariantTwo(u32),
}

impl Default for OneOfField {
    fn default() -> Self {
        Self::VariantOne("".into())
    }
}

#[test]
fn prost() {
    let mut target_struct = Parent {
        primitive: "string".into(),
        child_1: Some(Child {
            field_one: "child_1 field one".into(),
            field_two: 1,
        }),
        child_2: Some(Child {
            field_one: "child_2 field one".into(),
            field_two: 2,
        }),
        one_of_field: Some(OneOfField::VariantOne("variant one".into())),
    };
    let src_struct = Parent {
        primitive: "updated string".into(),
        child_1: Some(Child {
            field_one: "updated child_1 field one".into(),
            field_two: 10,
        }),
        child_2: Some(Child {
            field_one: "updated child_2 field one".into(),
            field_two: 20,
        }),
        one_of_field: Some(OneOfField::VariantTwo(50)),
    };

    let expected_struct = Parent {
        primitive: "updated string".into(),
        child_1: Some(Child {
            field_one: "child_1 field one".into(),
            field_two: 10,
        }),
        child_2: Some(Child {
            field_one: "updated child_2 field one".into(),
            field_two: 20,
        }),
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
