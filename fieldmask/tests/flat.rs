use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, ProjectOptions, SelfMaskable};

#[derive(Debug, Maskable, PartialEq, SelfMaskable)]
struct Flat {
    a: u32,
    b: u32,
}

mod project {
    use super::*;

    #[test]
    fn regular_mask() {
        let source = Flat { a: 1, b: 2 };
        let mask = vec!["b"];
        let expected = Flat {
            a: Default::default(),
            b: 2,
        };

        let actual = Mask::<Flat>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_mask() {
        let source = Flat { a: 1, b: 2 };
        let mask = vec![];
        let expected = Flat { a: 1, b: 2 };

        let actual = Mask::<Flat>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn nested_mask() {
        let mask = vec!["a.b"];

        assert_eq!(
            Mask::<Flat>::try_from(MaskInput(mask.into_iter()))
                .expect_err("should fail to parse fieldmask")
                .to_string(),
            "\
        error in field \"a\":\n\
        \ttype `u32` has no field named \"b\"\
        ",
        );
    }
}

mod update {

    use super::*;

    #[test]
    fn regular_mask() {
        let mut target = Flat { a: 1, b: 2 };
        let source = Flat { a: 2, b: 3 };
        let mask = vec!["b"];
        let options = Default::default();
        let expected = Flat { a: 1, b: 3 };

        Mask::<Flat>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);
        let target = Mask::<Flat>::empty()
            .project_with_options(target, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(target, expected);
    }

    #[test]
    fn empty_mask() {
        let mut target = Flat { a: 1, b: 2 };
        let mask = vec![];
        let source = Flat { a: 2, b: 3 };
        let options = Default::default();
        let expected = Flat { a: 2, b: 3 };

        Mask::<Flat>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);
        let target = Mask::<Flat>::empty()
            .project_with_options(target, &ProjectOptions::builder().normalize(true).build());

        assert_eq!(target, expected);
    }
}
