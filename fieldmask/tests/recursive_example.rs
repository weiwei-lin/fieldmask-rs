use std::convert::TryFrom;

use fieldmask::{Mask, MaskInput, Maskable, OptionMaskable, SelfMaskable};

#[derive(Debug, Default, Maskable, OptionMaskable, PartialEq, SelfMaskable, Clone)]
struct RecursiveNodeA {
    id: String,
    data: String,
    child: Option<Box<RenamedRecursiveNodeB>>,
    metadata: NodeMetadata,
}

#[derive(Debug, Default, Maskable, OptionMaskable, PartialEq, SelfMaskable, Clone)]
struct RecursiveNodeB {
    id: String,
    data: String,
    child: Option<Box<RenamedRecursiveNodeA>>,
    metadata: NodeMetadata,
}

type RenamedRecursiveNodeA = RecursiveNodeA;
type RenamedRecursiveNodeB = RecursiveNodeB;

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable, Clone)]
struct NodeMetadata {
    created_at: String,
    modified_at: String,
}

#[derive(Debug, Default, Maskable, PartialEq, SelfMaskable)]
struct TreeStructure {
    root: RecursiveNodeA,
    description: String,
    nodes_count: u32,
}

mod project {
    use super::*;

    fn create_test_tree() -> TreeStructure {
        TreeStructure {
            root: RecursiveNodeA {
                id: "root".into(),
                data: "root data".into(),
                child: Some(Box::new(RecursiveNodeB {
                    id: "child1".into(),
                    data: "child1 data".into(),
                    child: Some(Box::new(RecursiveNodeA {
                        id: "grandchild1".into(),
                        data: "grandchild1 data".into(),
                        child: None,
                        metadata: NodeMetadata {
                            created_at: "2023-01-03".into(),
                            modified_at: "2023-01-03".into(),
                        },
                    })),
                    metadata: NodeMetadata {
                        created_at: "2023-01-02".into(),
                        modified_at: "2023-01-02".into(),
                    },
                })),
                metadata: NodeMetadata {
                    created_at: "2023-01-01".into(),
                    modified_at: "2023-01-01".into(),
                },
            },
            description: "A recursive tree example".into(),
            nodes_count: 3,
        }
    }

    #[test]
    fn project_only_root_id() {
        let source = create_test_tree();
        let mask = vec!["root.id"];

        let expected = TreeStructure {
            root: RecursiveNodeA {
                id: "root".into(),
                data: "".into(),
                child: None,
                metadata: NodeMetadata::default(),
            },
            description: "".into(),
            nodes_count: 0,
        };

        let actual = Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn project_nested_child() {
        let source = create_test_tree();
        let mask = vec!["root.child"];

        let expected = TreeStructure {
            root: RecursiveNodeA {
                id: "".into(),
                data: "".into(),
                child: Some(Box::new(RecursiveNodeB {
                    id: "child1".into(),
                    data: "child1 data".into(),
                    child: Some(Box::new(RecursiveNodeA {
                        id: "grandchild1".into(),
                        data: "grandchild1 data".into(),
                        child: None,
                        metadata: NodeMetadata {
                            created_at: "2023-01-03".into(),
                            modified_at: "2023-01-03".into(),
                        },
                    })),
                    metadata: NodeMetadata {
                        created_at: "2023-01-02".into(),
                        modified_at: "2023-01-02".into(),
                    },
                })),
                metadata: NodeMetadata::default(),
            },
            description: "".into(),
            nodes_count: 0,
        };

        let actual = Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn project_nested_fields() {
        let source = create_test_tree();
        let mask = vec![
            "root.id",
            "root.child.id",
            "root.child.child.id",
            "description",
        ];

        let expected = TreeStructure {
            root: RecursiveNodeA {
                id: "root".into(),
                data: "".into(),
                child: Some(Box::new(RecursiveNodeB {
                    id: "child1".into(),
                    data: "".into(),
                    child: Some(Box::new(RecursiveNodeA {
                        id: "grandchild1".into(),
                        data: "".into(),
                        child: None,
                        metadata: NodeMetadata::default(),
                    })),
                    metadata: NodeMetadata::default(),
                })),
                metadata: NodeMetadata::default(),
            },
            description: "A recursive tree example".into(),
            nodes_count: 0,
        };

        let actual = Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }

    #[test]
    fn project_nested_metadata() {
        let source = create_test_tree();
        let mask = vec!["root.metadata", "root.child.metadata.created_at"];

        let expected = TreeStructure {
            root: RecursiveNodeA {
                id: "".into(),
                data: "".into(),
                child: Some(Box::new(RecursiveNodeB {
                    id: "".into(),
                    data: "".into(),
                    child: None,
                    metadata: NodeMetadata {
                        created_at: "2023-01-02".into(),
                        modified_at: "".into(),
                    },
                })),
                metadata: NodeMetadata {
                    created_at: "2023-01-01".into(),
                    modified_at: "2023-01-01".into(),
                },
            },
            description: "".into(),
            nodes_count: 0,
        };

        let actual = Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .project(source);

        assert_eq!(actual, expected);
    }
}

mod update {
    use super::*;

    fn create_base_tree() -> TreeStructure {
        TreeStructure {
            root: RecursiveNodeA {
                id: "root".into(),
                data: "root data".into(),
                child: Some(Box::new(RecursiveNodeB {
                    id: "child1".into(),
                    data: "child1 data".into(),
                    child: None,
                    metadata: NodeMetadata {
                        created_at: "2023-01-02".into(),
                        modified_at: "2023-01-02".into(),
                    },
                })),
                metadata: NodeMetadata {
                    created_at: "2023-01-01".into(),
                    modified_at: "2023-01-01".into(),
                },
            },
            description: "Original tree".into(),
            nodes_count: 2,
        }
    }

    fn create_updated_tree() -> TreeStructure {
        TreeStructure {
            root: RecursiveNodeA {
                id: "updated-root".into(),
                data: "updated root data".into(),
                child: Some(Box::new(RecursiveNodeB {
                    id: "updated-child1".into(),
                    data: "updated child1 data".into(),
                    child: Some(Box::new(RecursiveNodeA {
                        id: "new-grandchild".into(),
                        data: "new grandchild data".into(),
                        child: None,
                        metadata: NodeMetadata {
                            created_at: "2023-02-03".into(),
                            modified_at: "2023-02-03".into(),
                        },
                    })),
                    metadata: NodeMetadata {
                        created_at: "2023-02-02".into(),
                        modified_at: "2023-02-02".into(),
                    },
                })),
                metadata: NodeMetadata {
                    created_at: "2023-02-01".into(),
                    modified_at: "2023-02-01".into(),
                },
            },
            description: "Updated tree".into(),
            nodes_count: 3,
        }
    }

    #[test]
    fn update_root_id_only() {
        let mut target = create_base_tree();
        let source = create_updated_tree();
        let mask = vec!["root.id"];
        let options = Default::default();

        let mut expected = create_base_tree();
        expected.root.id = "updated-root".into();

        Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn update_nested_child() {
        let mut target = create_base_tree();
        let source = create_updated_tree();
        let mask = vec!["root.child"];
        let options = Default::default();

        let mut expected = create_base_tree();
        expected.root.child = Some(Box::new(RecursiveNodeB {
            id: "updated-child1".into(),
            data: "updated child1 data".into(),
            child: Some(Box::new(RecursiveNodeA {
                id: "new-grandchild".into(),
                data: "new grandchild data".into(),
                child: None,
                metadata: NodeMetadata {
                    created_at: "2023-02-03".into(),
                    modified_at: "2023-02-03".into(),
                },
            })),
            metadata: NodeMetadata {
                created_at: "2023-02-02".into(),
                modified_at: "2023-02-02".into(),
            },
        }));

        Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);

        assert_eq!(target, expected);
    }

    #[test]
    fn update_metadata_only() {
        let mut target = create_base_tree();
        let source = create_updated_tree();
        let mask = vec!["root.metadata.modified_at"];
        let options = Default::default();

        let mut expected = create_base_tree();
        expected.root.metadata.modified_at = "2023-02-01".into();

        Mask::<TreeStructure>::try_from(MaskInput(mask.into_iter()))
            .expect("unable to deserialize mask")
            .update_with_options(&mut target, source, &options);

        assert_eq!(target, expected);
    }
}
