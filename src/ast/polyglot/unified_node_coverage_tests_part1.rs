    // ==========================================================================
    // UnifiedNode::new tests
    // ==========================================================================

    #[test]
    fn test_unified_node_new_basic() {
        let node = UnifiedNode::new(NodeKind::Function, "my_function", Language::Rust);

        assert_eq!(node.id, "Rust:function:my_function");
        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "my_function");
        assert_eq!(node.fqn, "my_function");
        assert_eq!(node.language, Language::Rust);
        assert!(node.file_path.as_os_str().is_empty());
        assert_eq!(node.position.start_line, 0);
        assert_eq!(node.position.start_col, 0);
        assert!(node.attributes.is_empty());
        assert!(node.children.is_empty());
        assert!(node.parent.is_none());
        assert!(node.references.is_empty());
        assert!(node.type_info.is_none());
        assert!(node.signature.is_none());
        assert!(node.documentation.is_none());
        assert!(node.original_item.is_none());
        assert!(node.metadata.is_empty());
    }

    #[test]
    fn test_unified_node_new_all_kinds() {
        let test_cases = vec![
            (NodeKind::Class, "MyClass", Language::Java),
            (NodeKind::Function, "my_func", Language::Python),
            (NodeKind::Method, "do_something", Language::TypeScript),
            (NodeKind::Struct, "DataStruct", Language::Rust),
            (NodeKind::Enum, "Status", Language::Go),
            (NodeKind::Interface, "IService", Language::CSharp),
            (NodeKind::Trait, "MyTrait", Language::Scala),
            (NodeKind::Module, "my_mod", Language::Ruby),
            (NodeKind::LanguageSpecific(42), "custom", Language::Other(1)),
        ];

        for (kind, name, language) in test_cases {
            let node = UnifiedNode::new(kind, name, language);
            assert_eq!(node.kind, kind);
            assert_eq!(node.name, name);
            assert_eq!(node.language, language);
        }
    }

    // ==========================================================================
    // UnifiedNode::from_ast_item tests
    // ==========================================================================

    #[test]
    fn test_from_ast_item_function() {
        let item = AstItem::Function {
            name: "process_data".to_string(),
            visibility: "pub".to_string(),
            is_async: true,
            line: 42,
        };
        let path = Path::new("/src/lib.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "process_data");
        assert_eq!(node.position.start_line, 42);
        assert_eq!(node.access(), Some("pub"));
        assert!(node.has_modifier("async"));
    }

    #[test]
    fn test_from_ast_item_function_sync() {
        let item = AstItem::Function {
            name: "sync_func".to_string(),
            visibility: "private".to_string(),
            is_async: false,
            line: 10,
        };
        let path = Path::new("/src/main.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert!(!node.has_modifier("async"));
    }

    #[test]
    fn test_from_ast_item_struct_with_derives() {
        let item = AstItem::Struct {
            name: "MyStruct".to_string(),
            visibility: "pub".to_string(),
            fields_count: 5,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Serialize".to_string(),
            ],
            line: 1,
        };
        let path = Path::new("/src/models.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Struct);
        assert!(node.attributes.contains_key("derive:Debug"));
        assert!(node.attributes.contains_key("derive:Clone"));
        assert!(node.attributes.contains_key("derive:Serialize"));
    }

    #[test]
    fn test_from_ast_item_struct_same_as_file() {
        let item = AstItem::Struct {
            name: "MyModule".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        // When struct name matches file name, FQN should just be the name
        let path = Path::new("/src/MyModule.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.fqn, "MyModule");
    }

    #[test]
    fn test_from_ast_item_struct_different_from_file() {
        let item = AstItem::Struct {
            name: "InnerStruct".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        let path = Path::new("/src/outer.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        // FQN should include file name prefix
        assert_eq!(node.fqn, "outer.InnerStruct");
    }

    #[test]
    fn test_from_ast_item_enum() {
        let item = AstItem::Enum {
            name: "Status".to_string(),
            visibility: "pub(crate)".to_string(),
            variants_count: 3,
            line: 20,
        };
        let path = Path::new("/src/types.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Enum);
        assert_eq!(node.name, "Status");
        assert_eq!(node.access(), Some("pub(crate)"));
        assert_eq!(node.fqn, "types.Status");
    }

    #[test]
    fn test_from_ast_item_trait() {
        let item = AstItem::Trait {
            name: "Processor".to_string(),
            visibility: "pub".to_string(),
            line: 5,
        };
        let path = Path::new("/src/traits.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Trait);
        assert_eq!(node.name, "Processor");
        assert_eq!(node.fqn, "traits.Processor");
    }

    #[test]
    fn test_from_ast_item_impl() {
        let item = AstItem::Impl {
            type_name: "MyStruct".to_string(),
            trait_name: Some("Display".to_string()),
            line: 15,
        };
        let path = Path::new("/src/impls.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Implements);
        assert_eq!(node.name, "MyStruct");
        assert_eq!(node.access(), Some("public")); // Impls default to public
    }

    #[test]
    fn test_from_ast_item_impl_no_trait() {
        let item = AstItem::Impl {
            type_name: "MyStruct".to_string(),
            trait_name: None,
            line: 15,
        };
        let path = Path::new("/src/impls.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Implements);
        assert_eq!(node.name, "MyStruct");
    }

    #[test]
    fn test_from_ast_item_module() {
        let item = AstItem::Module {
            name: "submodule".to_string(),
            visibility: "pub(super)".to_string(),
            line: 1,
        };
        let path = Path::new("/src/lib.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Module);
        assert_eq!(node.name, "submodule");
        assert_eq!(node.access(), Some("pub(super)"));
    }

    #[test]
    fn test_from_ast_item_use() {
        let item = AstItem::Use {
            path: "std::collections::HashMap".to_string(),
            line: 3,
        };
        let path = Path::new("/src/main.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Uses);
        assert_eq!(node.name, "std::collections::HashMap");
        assert_eq!(node.access(), Some("public")); // Use defaults to public
    }

    #[test]
    fn test_from_ast_item_import() {
        let item = AstItem::Import {
            module: "external_lib".to_string(),
            items: vec![],
            alias: Some("ext".to_string()),
            line: 1,
        };
        let path = Path::new("/src/main.py");

        let node = UnifiedNode::from_ast_item(&item, Language::Python, path, None);

        assert_eq!(node.kind, NodeKind::Import);
        assert_eq!(node.name, "external_lib");
    }

    #[test]
    fn test_from_ast_item_import_no_alias() {
        let item = AstItem::Import {
            module: "json".to_string(),
            items: vec![],
            alias: None,
            line: 1,
        };
        let path = Path::new("/src/main.py");

        let node = UnifiedNode::from_ast_item(&item, Language::Python, path, None);

        assert_eq!(node.kind, NodeKind::Import);
        assert_eq!(node.name, "json");
    }

    #[test]
    fn test_from_ast_item_with_custom_prefix() {
        let item = AstItem::Function {
            name: "test_fn".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        let path = Path::new("/src/lib.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, Some("custom_prefix"));

        assert!(node.id.starts_with("custom_prefix:"));
    }

    // ==========================================================================
    // extract_name_from_item tests
    // ==========================================================================

    #[test]
    fn test_extract_name_from_all_item_types() {
        let test_cases = vec![
            (
                AstItem::Function {
                    name: "func_name".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
                "func_name",
            ),
            (
                AstItem::Struct {
                    name: "StructName".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 0,
                    derives: vec![],
                    line: 1,
                },
                "StructName",
            ),
            (
                AstItem::Enum {
                    name: "EnumName".to_string(),
                    visibility: "pub".to_string(),
                    variants_count: 0,
                    line: 1,
                },
                "EnumName",
            ),
            (
                AstItem::Trait {
                    name: "TraitName".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                "TraitName",
            ),
            (
                AstItem::Impl {
                    type_name: "TypeName".to_string(),
                    trait_name: None,
                    line: 1,
                },
                "TypeName",
            ),
            (
                AstItem::Use {
                    path: "std::io".to_string(),
                    line: 1,
                },
                "std::io",
            ),
            (
                AstItem::Module {
                    name: "mod_name".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                "mod_name",
            ),
            (
                AstItem::Import {
                    module: "import_name".to_string(),
                    items: vec![],
                    alias: None,
                    line: 1,
                },
                "import_name",
            ),
        ];

        for (item, expected_name) in test_cases {
            assert_eq!(
                UnifiedNode::extract_name_from_item(&item),
                expected_name,
                "Failed for item: {:?}",
                item
            );
        }
    }

