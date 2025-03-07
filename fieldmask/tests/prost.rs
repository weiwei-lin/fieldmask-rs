#![cfg(feature = "prost")]

use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

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
    let target = Parent {
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
    let mask = vec![
        "primitive",
        "child_1.field_two",
        "child_2", // if child properties are not specified, all properties are included.
        "variant_two", // if a field is marked with `flatten`, it's properties are merged with its parents properties.
    ];
    let expected = Parent {
        primitive: "string".into(),
        child_1: Some(Child {
            field_one: Default::default(),
            field_two: 1,
        }),
        child_2: Some(Child {
            field_one: "child_2 field one".into(),
            field_two: 2,
        }),
        one_of_field: Some(OneOfField::VariantOne(Default::default())),
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}
