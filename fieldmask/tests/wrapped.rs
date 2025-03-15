use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, SelfMaskable};

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Parent {
    child: Child,
}

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
struct Child {
    a: u32,
    b: Option<u32>,
    c: Option<Option<u32>>,
    d: Box<u32>,
    #[allow(clippy::redundant_allocation)]
    e: Box<Box<u32>>,
    f: Box<Option<u32>>,
    g: Option<Box<u32>>,
}

mod project {
    use super::*;

    #[test]
    fn regular_mask() {
        let source = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec!["child.a"];
        let expected = Parent {
            child: Child {
                a: 1,
                b: Default::default(),
                c: Default::default(),
                d: Default::default(),
                e: Default::default(),
                f: Default::default(),
                g: Default::default(),
            },
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn full_mask() {
        let source = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec![
            "child.a", "child.b", "child.c", "child.d", "child.e", "child.f", "child.g",
        ];
        let expected = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn full_mask_with_unnormalized() {
        let source = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(None),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec![
            "child.a", "child.b", "child.c", "child.d", "child.e", "child.f", "child.g",
        ];
        let expected = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(None),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };

        let actual = Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_mask() {
        let source = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec![];
        let expected = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
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
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec!["child.a"];
        let source = Parent {
            child: Child {
                a: 11,
                b: Some(3),
                c: Some(Some(4)),
                d: Box::new(5),
                e: Box::new(Box::new(6)),
                f: Box::new(Some(7)),
                g: Some(Box::new(8)),
            },
        };
        let options = Default::default();
        let expected = Parent {
            child: Child {
                a: 11,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update(&mut target, source, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn empty_mask() {
        let mut target = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec![];
        let source = Parent {
            child: Child {
                a: 2,
                b: Some(3),
                c: Some(Some(4)),
                d: Box::new(5),
                e: Box::new(Box::new(6)),
                f: Box::new(Some(7)),
                g: Some(Box::new(8)),
            },
        };
        let options = Default::default();
        let expected = Parent {
            child: Child {
                a: 2,
                b: Some(3),
                c: Some(Some(4)),
                d: Box::new(5),
                e: Box::new(Box::new(6)),
                f: Box::new(Some(7)),
                g: Some(Box::new(8)),
            },
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update(&mut target, source, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn empty_mask_with_unnormalized() {
        let mut target = Parent {
            child: Child {
                a: 1,
                b: Some(2),
                c: Some(Some(3)),
                d: Box::new(4),
                e: Box::new(Box::new(5)),
                f: Box::new(Some(6)),
                g: Some(Box::new(7)),
            },
        };
        let mask = vec![];
        let source = Parent {
            child: Child {
                a: 2,
                b: Some(3),
                c: Some(None),
                d: Box::new(5),
                e: Box::new(Box::new(6)),
                f: Box::new(Some(7)),
                g: Some(Box::new(8)),
            },
        };
        let options = Default::default();
        let expected = Parent {
            child: Child {
                a: 2,
                b: Some(3),
                c: Some(Some(0)),
                d: Box::new(5),
                e: Box::new(Box::new(6)),
                f: Box::new(Some(7)),
                g: Some(Box::new(8)),
            },
        };

        Mask::<Parent>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update(&mut target, source, &options);

        assert_eq!(target, expected);
    }
}
