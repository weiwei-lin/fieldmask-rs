use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, ProjectOptions, SelfMaskable, UpdateOptions};

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
struct Child {
    a: u32,
    b: u32,
    field_with_default_source: String,
}

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Parent {
    child: Child,
    c: u32,
}

mod project {
    use super::*;

    #[test]
    fn regular_mask() {
        let source = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let mask = vec!["child.b", "c"];
        let expected = Parent {
            child: Child {
                a: Default::default(),
                b: 2,
                field_with_default_source: Default::default(),
            },
            c: 3,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn full_child_mask() {
        let source = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let mask = vec!["child", "c"];
        let expected = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn explicit_full_child_mask() {
        let source = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let mask = vec!["child.a", "child.b", "child.field_with_default_source", "c"];
        let expected = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }
}

mod update {
    use super::*;

    #[test]
    fn regular_mask() {
        let mut target = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let source = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: Default::default(),
            },
            c: 6,
        };
        let mask = vec!["child.b", "c"];
        let options = Default::default();
        let expected = Parent {
            child: Child {
                a: 1,
                b: 5,
                field_with_default_source: "init".to_string(),
            },
            c: 6,
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);
        let target = Mask::<Parent>::empty()
            .project_with_options(target, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(target, expected);
    }

    #[test]
    fn full_child_mask() {
        let mut target = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let source = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: Default::default(),
            },
            c: 6,
        };
        let mask = vec!["child", "c"];
        let options = Default::default();
        let expected = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: "init".to_string(),
            },
            c: 6,
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);
        let target = Mask::<Parent>::empty()
            .project_with_options(target, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(target, expected);
    }

    #[test]
    fn full_child_mask_with_replace_message() {
        let mut target = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let source = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: Default::default(),
            },
            c: 6,
        };
        let mask = vec!["child", "c"];
        let options = UpdateOptions::builder().replace_message(true).build();
        let expected = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: Default::default(),
            },
            c: 6,
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);
        let target = Mask::<Parent>::empty()
            .project_with_options(target, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(target, expected);
    }

    #[test]
    fn explicit_full_child_mask() {
        let mut target = Parent {
            child: Child {
                a: 1,
                b: 2,
                field_with_default_source: "init".to_string(),
            },
            c: 3,
        };
        let source = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: Default::default(),
            },
            c: 6,
        };
        let mask = vec!["child.a", "child.b", "child.field_with_default_source", "c"];
        let options = Default::default();
        let expected = Parent {
            child: Child {
                a: 4,
                b: 5,
                field_with_default_source: Default::default(),
            },
            c: 6,
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);
        let target = Mask::<Parent>::empty()
            .project_with_options(target, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(target, expected);
    }
}
