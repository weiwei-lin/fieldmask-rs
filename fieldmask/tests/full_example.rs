use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    primitive: String,

    child: Child,
    #[fieldmask(flatten)]
    flatten_child: Child,

    one_of_field: Option<OneOfField>,
    #[fieldmask(flatten)]
    flatten_one_of_field: Option<OneOfField>,

    unit_field: Option<UnitField>,
}

#[derive(Debug, Default, Maskable, PartialEq)]
struct Child {
    field_one: String,
    field_two: u32,
}

#[derive(Debug, Maskable, PartialEq)]
enum OneOfField {
    VariantOne(String),
    VariantTwo(u32),
}

impl Default for OneOfField {
    fn default() -> Self {
        Self::VariantOne(String::default())
    }
}

#[derive(Debug, Maskable, PartialEq)]
enum UnitField {
    One = 1,
    Two = 2,
}

#[test]
fn case_1() {
    let target = Parent {
        primitive: "string".into(),

        child: Child {
            field_one: "child field one".into(),
            field_two: 1,
        },
        flatten_child: Child {
            field_one: "flatten child field one".into(),
            field_two: 2,
        },

        one_of_field: Some(OneOfField::VariantOne("variant one".into())),
        flatten_one_of_field: Some(OneOfField::VariantTwo(3)),

        unit_field: Some(UnitField::Two),
    };
    let mask = vec![
        "primitive",
        "child.field_two",
        "field_one",
        "one_of_field.variant_one",
        "variant_two",
        "unit_field",
    ];
    let expected = Parent {
        primitive: "string".into(),

        child: Child {
            field_one: Default::default(),
            field_two: 1,
        },
        flatten_child: Child {
            field_one: "flatten child field one".into(),
            field_two: Default::default(),
        },

        one_of_field: Some(OneOfField::VariantOne("variant one".into())),
        flatten_one_of_field: Some(OneOfField::VariantTwo(3)),

        unit_field: Some(UnitField::Two),
    };

    let mask =
        Mask::<Parent>::try_from(MaskInput(mask.into_iter())).expect("unable to deserialize mask");
    let actual = target.project(&mask);

    assert_eq!(expected, actual);
}
