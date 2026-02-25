
    // Strategy for generating valid language names
    fn language_strategy() -> impl Strategy<Value = Language> {
        prop_oneof![
            Just(Language::Java),
            Just(Language::Kotlin),
            Just(Language::Scala),
            Just(Language::TypeScript),
            Just(Language::JavaScript),
            Just(Language::Python),
            Just(Language::Rust),
            Just(Language::Go),
            Just(Language::Cpp),
            Just(Language::CSharp),
            Just(Language::Ruby),
            Just(Language::Swift),
            Just(Language::Php),
            (0u32..1000).prop_map(Language::Other),
        ]
    }

    // Strategy for generating valid identifiers
    fn identifier_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_map(|s| s)
    }

    // Strategy for generating node kinds
    fn node_kind_strategy() -> impl Strategy<Value = NodeKind> {
        prop_oneof![
            Just(NodeKind::Class),
            Just(NodeKind::Interface),
            Just(NodeKind::Trait),
            Just(NodeKind::Function),
            Just(NodeKind::Method),
            Just(NodeKind::Module),
            Just(NodeKind::Enum),
            Just(NodeKind::Struct),
        ]
    }

    proptest! {
        #[test]
        fn test_base_mapper_language_preserved(lang in language_strategy()) {
            let mapper = BaseLanguageMapper::new(lang);
            prop_assert_eq!(mapper.language(), lang);
        }

        #[test]
        fn test_base_mapper_clone_preserves_language(lang in language_strategy()) {
            let mapper = BaseLanguageMapper::new(lang);
            let cloned = mapper.clone();
            prop_assert_eq!(cloned.language(), lang);
        }

        #[test]
        fn test_base_mapper_clone_box_preserves_language(lang in language_strategy()) {
            let mapper = BaseLanguageMapper::new(lang);
            let boxed = mapper.clone_box();
            prop_assert_eq!(boxed.language(), lang);
        }

        #[test]
        fn test_create_test_node_preserves_properties(
            kind in node_kind_strategy(),
            name in identifier_strategy(),
            lang in language_strategy()
        ) {
            let mapper = BaseLanguageMapper::new(lang);
            let node = mapper.create_test_node(kind, &name);

            prop_assert_eq!(node.kind, kind);
            prop_assert_eq!(node.name, name);
            prop_assert_eq!(node.language, lang);
        }

        #[test]
        fn test_unified_node_new_generates_valid_id(
            kind in node_kind_strategy(),
            name in identifier_strategy(),
            lang in language_strategy()
        ) {
            let node = UnifiedNode::new(kind, &name, lang);

            // ID should contain language name, kind, and name
            prop_assert!(node.id.contains(lang.name()));
            prop_assert!(node.id.contains(kind.as_str()));
            prop_assert!(node.id.contains(&name));
        }

        #[test]
        fn test_factory_create_supported_languages(lang in prop_oneof![
            Just(Language::Java),
            Just(Language::Kotlin),
            Just(Language::Scala),
            Just(Language::TypeScript),
            Just(Language::JavaScript),
        ]) {
            let result = LanguageMapperFactory::create(lang);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap().language(), lang);
        }

        #[test]
        fn test_factory_create_unsupported_returns_error(id in 0u32..1000) {
            let result = LanguageMapperFactory::create(Language::Other(id));
            prop_assert!(result.is_err());
        }

        #[test]
        fn test_convert_ast_items_count_matches(
            count in 0usize..20,
        ) {
            let mapper = BaseLanguageMapper::new(Language::Rust);
            let path = std::path::Path::new("/test/file.rs");

            let items: Vec<AstItem> = (0..count)
                .map(|i| AstItem::Function {
                    name: format!("func_{}", i),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: i + 1,
                })
                .collect();

            let nodes = mapper.convert_ast_items(&items, path);
            prop_assert_eq!(nodes.len(), count);
        }
    }
