    fn create_test_ast_item(kind: &str, name: &str) -> AstItem {
        // Map string kind to AstItem enum variant
        match kind {
            "function" | "method" => AstItem::Function {
                name: name.to_string(),
                visibility: "public".to_string(),
                is_async: false,
                line: 1,
            },
            "class" | "struct" => AstItem::Struct {
                name: name.to_string(),
                visibility: "public".to_string(),
                fields_count: 0,
                derives: vec![],
                line: 1,
            },
            "trait" | "interface" => AstItem::Trait {
                name: name.to_string(),
                visibility: "public".to_string(),
                line: 1,
            },
            "enum" => AstItem::Enum {
                name: name.to_string(),
                visibility: "public".to_string(),
                variants_count: 0,
                line: 1,
            },
            "module" | "namespace" => AstItem::Module {
                name: name.to_string(),
                visibility: "public".to_string(),
                line: 1,
            },
            _ => AstItem::Struct {
                name: name.to_string(),
                visibility: "public".to_string(),
                fields_count: 0,
                derives: vec![],
                line: 1,
            },
        }
    }

    #[test]
    fn test_language_mapper_factory() {
        // Test creating mappers for supported languages
        let java_mapper = LanguageMapperFactory::create(Language::Java);
        assert!(java_mapper.is_ok());
        assert_eq!(java_mapper.unwrap().language(), Language::Java);

        let scala_mapper = LanguageMapperFactory::create(Language::Scala);
        assert!(scala_mapper.is_ok());
        assert_eq!(scala_mapper.unwrap().language(), Language::Scala);

        // Test creating mapper for unsupported language
        let unsupported = LanguageMapperFactory::create(Language::Other(0));
        assert!(unsupported.is_err());

        // Test creating mapper for file
        let file_path = Path::new("test.java");
        let file_mapper = LanguageMapperFactory::create_for_file(file_path);
        assert!(file_mapper.is_ok());
        assert_eq!(file_mapper.unwrap().language(), Language::Java);
    }

    #[test]
    fn test_convert_ast_items() {
        let java_mapper = JavaMapper::new();
        let file_path = Path::new("/path/to/Test.java");

        let items = vec![
            create_test_ast_item("class", "TestClass"),
            create_test_ast_item("method", "testMethod"),
        ];

        let nodes = java_mapper.convert_ast_items(&items, file_path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].kind, NodeKind::Struct); // Java classes are represented as Struct in AstItem
        assert_eq!(nodes[0].name, "TestClass");
        assert_eq!(nodes[1].kind, NodeKind::Function); // Methods are represented as Function in AstItem
        assert_eq!(nodes[1].name, "testMethod");
    }

    /// Exercise every arm of `create_test_ast_item` to stop the helper
    /// from reporting 23 uncovered lines. Each returned `AstItem` variant
    /// is fed through `JavaMapper::convert_ast_items` so the assertion
    /// also validates the mapper handles every kind the helper can produce.
    #[test]
    fn test_create_test_ast_item_covers_all_variant_arms() {
        let mapper = JavaMapper::new();
        let file_path = Path::new("/path/to/Test.java");

        let items = vec![
            create_test_ast_item("trait", "MyTrait"),
            create_test_ast_item("interface", "MyIface"),
            create_test_ast_item("enum", "MyEnum"),
            create_test_ast_item("module", "MyMod"),
            create_test_ast_item("namespace", "MyNs"),
            create_test_ast_item("unknown_kind", "Fallback"),
        ];

        // trait/interface → Trait
        if let AstItem::Trait { name, .. } = &items[0] {
            assert_eq!(name, "MyTrait");
        } else {
            panic!("Expected Trait for 'trait'");
        }
        if let AstItem::Trait { name, .. } = &items[1] {
            assert_eq!(name, "MyIface");
        } else {
            panic!("Expected Trait for 'interface'");
        }
        // enum → Enum
        if let AstItem::Enum { name, .. } = &items[2] {
            assert_eq!(name, "MyEnum");
        } else {
            panic!("Expected Enum for 'enum'");
        }
        // module/namespace → Module
        if let AstItem::Module { name, .. } = &items[3] {
            assert_eq!(name, "MyMod");
        } else {
            panic!("Expected Module for 'module'");
        }
        if let AstItem::Module { name, .. } = &items[4] {
            assert_eq!(name, "MyNs");
        } else {
            panic!("Expected Module for 'namespace'");
        }
        // default `_` → Struct
        if let AstItem::Struct { name, .. } = &items[5] {
            assert_eq!(name, "Fallback");
        } else {
            panic!("Expected Struct for default arm");
        }

        // Drive through the mapper to confirm each variant is convertible.
        let nodes = mapper.convert_ast_items(&items, file_path);
        assert_eq!(nodes.len(), 6);
    }
