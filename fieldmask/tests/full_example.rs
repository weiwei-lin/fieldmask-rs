use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, OptionMaskable, SelfMaskable};

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Parent {
    primitive: String,

    child: Child,

    // Child fields can be flattened into the parent.
    #[fieldmask(flatten)]
    flatten_child: Child,

    one_of_field: Option<OneOfField>,

    // You can use an enum to represent oneof fields from a protobuf message.
    // Each variant in the enum must be tuple variant with a single inner type.
    //
    // If you flatten it, the behavior will be exactly the same as the protobuf message.
    #[fieldmask(flatten)]
    flatten_one_of_field: Option<OneOfField>,

    // You can use an enum to enums from a protobuf message.
    // Each variant in the enum must be a unit variant.
    unit_enum: Option<UnitEnum>,
    unit_enum_with_default: UnitEnumWithDefault,
}

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
struct Child {
    field_one: String,
    field_two: u32,
}

// You can derive `OptionMaskable` on tuple enums.
// If you do so, `Option<MyTupleEnum>` will be `SelfMaskable`.
#[derive(Debug, Maskable, OptionMaskable, PartialEq)]
enum OneOfField {
    VariantOne(String),
    VariantTwo(u32),
}

// You can derive `OptionMaskable` on unit enums.
// If you do so, `Option<UnitEnum>` will be `SelfMaskable`.
#[derive(Debug, Maskable, OptionMaskable, PartialEq)]
#[allow(dead_code)]
enum UnitEnum {
    One = 1,
    Two = 2,
}

// If the unit enum implements `Default` and `PartialEq`, you can derive `SelfMaskable` for it.
#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
enum UnitEnumWithDefault {
    #[default]
    One = 1,
    Two = 2,
}

mod project {
    use super::*;

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

            unit_enum: Some(UnitEnum::Two),
            unit_enum_with_default: UnitEnumWithDefault::Two,
        };
        let mask = vec![
            "primitive",
            "child.field_two",
            "field_one",
            "one_of_field.variant_one",
            "variant_two",
            "unit_enum",
            "unit_enum_with_default",
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

            unit_enum: Some(UnitEnum::Two),
            unit_enum_with_default: UnitEnumWithDefault::Two,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }
}
