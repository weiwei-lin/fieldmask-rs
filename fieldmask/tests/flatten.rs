use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
struct Child {
    a: u32,
    b: u32,
}

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Parent {
    #[fieldmask(flatten)]
    child: Child,
    c: u32,
}

mod project {
    use super::*;

    #[test]
    fn regular_mask() {
        let source = Parent {
            child: Child { a: 1, b: 2 },
            c: 3,
        };
        let mask = vec!["b", "c"];
        let expected = Parent {
            child: Child {
                a: Default::default(),
                b: 2,
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
            child: Child { a: 1, b: 2 },
            c: 3,
        };
        let mask = vec!["a", "b", "c"];
        let expected = Parent {
            child: Child { a: 1, b: 2 },
            c: 3,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn child_not_selected() {
        let source = Parent {
            child: Child { a: 1, b: 2 },
            c: 3,
        };
        let mask = vec!["c"];
        let expected = Parent {
            child: Child { a: 0, b: 0 },
            c: 3,
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }
}
