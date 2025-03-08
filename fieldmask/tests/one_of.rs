use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, PartialEq, Maskable)]
enum OneOf {
    A(String),
    B(String),
    AnotherCase(String),
}

#[derive(Debug, PartialEq, Maskable)]
struct Parent {
    one_of: Option<OneOf>,
    c: u32,
}

mod project {
    use super::*;

    #[test]
    fn source_variant_is_the_same() {
        let target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let mask = vec!["one_of.b", "c"];
        let expected = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }

    #[test]
    fn source_variant_is_different() {
        let target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let mask = vec!["one_of.a", "c"];
        let expected = Parent { one_of: None, c: 1 };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }

    #[test]
    fn both_variants_selected() {
        let target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let mask = vec!["one_of.a", "one_of.b", "c"];
        let expected = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_variant_selected() {
        let target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let mask = vec!["c"];
        let expected = Parent {
            one_of: Default::default(),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }

    #[test]
    fn snake_case() {
        let target = Parent {
            one_of: Some(OneOf::AnotherCase("another".into())),
            c: 1,
        };
        let mask = vec!["one_of.another_case", "c"];
        let expected = Parent {
            one_of: Some(OneOf::AnotherCase("another".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        let actual = target.project(&mask);

        assert_eq!(actual, expected);
    }
}

mod update {
    use super::*;

    #[test]
    fn source_variant_is_the_same() {
        let mut target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let source = Parent {
            one_of: Some(OneOf::B("updated-b".into())),
            c: 1,
        };
        let mask = vec!["one_of.b", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::B("updated-b".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn source_variant_is_different_and_selected() {
        let mut target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let source = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };
        let mask = vec!["one_of.a", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn source_variant_is_different_and_not_selected() {
        let mut target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let source = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };
        let mask = vec!["one_of.another_case", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn source_variant_is_different_and_selected_target_variant() {
        let mut target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let source = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };
        let mask = vec!["one_of.b", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::B(Default::default())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn target_is_none_source_variant_selected() {
        let mut target = Parent { one_of: None, c: 1 };
        let source = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };
        let mask = vec!["one_of.a", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn target_is_none_source_variant_not_selected() {
        let mut target = Parent { one_of: None, c: 1 };
        let source = Parent {
            one_of: Some(OneOf::A("updated-a".into())),
            c: 1,
        };
        let mask = vec!["one_of.another_case", "c"];
        let options = Default::default();
        let expected = Parent { one_of: None, c: 1 };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn source_is_none_target_selected() {
        let mut target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let source = Parent { one_of: None, c: 1 };
        let mask = vec!["one_of.b", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::B(Default::default())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn source_is_none_target_not_selected() {
        let mut target = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };
        let source = Parent { one_of: None, c: 1 };
        let mask = vec!["one_of.a", "c"];
        let options = Default::default();
        let expected = Parent {
            one_of: Some(OneOf::B("b".into())),
            c: 1,
        };

        let mask = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask");
        target.update(source, &mask, &options);

        assert_eq!(target, expected);
    }
}
