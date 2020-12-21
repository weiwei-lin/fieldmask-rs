use fieldmask::Maskable;

#[derive(Debug, PartialEq, Maskable)]
struct Flat {
    a: u32,
    b: u32,
}

#[test]
fn flat() {
    let mut struct1 = Flat { a: 1, b: 2 };
    let struct2 = Flat { a: 3, b: 4 };

    let expected_struct = Flat { a: 1, b: 4 };
    struct1.apply_mask(
        struct2,
        Flat::deserialize_mask(vec!["b"].into_iter()).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}

#[test]
fn empty_mask() {
    let mut struct1 = Flat { a: 1, b: 2 };
    let struct2 = Flat { a: 3, b: 4 };

    let expected_struct = Flat { a: 1, b: 2 };
    struct1.apply_mask(
        struct2,
        Flat::deserialize_mask(vec![].into_iter()).expect("unable to deserialize mask"),
    );
    assert_eq!(struct1, expected_struct);
}
