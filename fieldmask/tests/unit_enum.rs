use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, OptionMaskable, ProjectOptions, SelfMaskable};

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Parent {
    primitive: u32,
    optional: Option<UnitEnumNoDefault>,
    required: UnitEnumWithDefault,
    optional_with_default: Option<UnitEnumWithDefault>,
}

#[derive(Debug, Maskable, OptionMaskable, PartialEq)]
enum UnitEnumNoDefault {
    One = 1,
    Two = 2,
}

#[derive(Debug, Default, Maskable, OptionMaskable, PartialEq, SelfMaskable)]
#[fieldmask(normalize_some_default)]
enum UnitEnumWithDefault {
    #[default]
    One = 1,
    Two = 2,
}

mod project {
    use super::*;

    #[test]
    fn optional_selected() {
        let source = Parent {
            primitive: 1,
            optional: Some(UnitEnumNoDefault::Two),
            required: UnitEnumWithDefault::Two,
            optional_with_default: Some(UnitEnumWithDefault::One),
        };
        let mask = vec!["optional"];
        let expected = Parent {
            primitive: Default::default(),
            optional: Some(UnitEnumNoDefault::Two),
            required: UnitEnumWithDefault::One,
            optional_with_default: None,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn optional_with_default_selected() {
        let source = Parent {
            primitive: 1,
            optional: Some(UnitEnumNoDefault::Two),
            required: UnitEnumWithDefault::Two,
            optional_with_default: Some(UnitEnumWithDefault::One),
        };
        let mask = vec!["optional_with_default"];
        let expected = Parent {
            primitive: Default::default(),
            optional: None,
            required: UnitEnumWithDefault::One,
            optional_with_default: Some(UnitEnumWithDefault::One),
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);

        // Test normalization.
        let expected = Parent {
            primitive: Default::default(),
            optional: None,
            required: UnitEnumWithDefault::One,
            optional_with_default: None,
        };

        let actual = Mask::<Parent>::empty()
            .project_with_options(actual, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(actual, expected);
    }

    #[test]
    fn optional_not_selected() {
        let source = Parent {
            primitive: 1,
            optional: Some(UnitEnumNoDefault::One),
            required: UnitEnumWithDefault::Two,
            optional_with_default: Some(UnitEnumWithDefault::One),
        };
        let mask = vec!["primitive"];
        let expected = Parent {
            primitive: 1,
            optional: None,
            required: UnitEnumWithDefault::One,
            optional_with_default: None,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }
}
