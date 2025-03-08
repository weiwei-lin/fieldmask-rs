use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, OptionMaskable, SelfMaskable};

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Parent {
    primitive: u32,
    optional: Option<UnitEnumNoDefault>,
    required: UnitEnumWithDefault,
}

#[derive(Debug, Maskable, OptionMaskable, PartialEq)]
enum UnitEnumNoDefault {
    One = 1,
    Two = 2,
}

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
enum UnitEnumWithDefault {
    #[default]
    One = 1,
    Two = 2,
}

mod project {
    use super::*;

    #[test]
    fn optional_selected() {
        let target = Parent {
            primitive: 1,
            optional: Some(UnitEnumNoDefault::Two),
            required: UnitEnumWithDefault::Two,
        };
        let mask = vec!["optional"];
        let expected = Parent {
            primitive: Default::default(),
            optional: Some(UnitEnumNoDefault::Two),
            required: UnitEnumWithDefault::One,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }

    #[test]
    fn optional_not_selected() {
        let target = Parent {
            primitive: 1,
            optional: Some(UnitEnumNoDefault::One),
            required: UnitEnumWithDefault::Two,
        };
        let mask = vec!["primitive"];
        let expected = Parent {
            primitive: 1,
            optional: None,
            required: UnitEnumWithDefault::One,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }
}
