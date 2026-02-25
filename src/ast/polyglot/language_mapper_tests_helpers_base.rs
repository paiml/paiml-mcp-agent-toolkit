
    // Helper functions for test data creation

    fn create_function_item(name: &str, is_async: bool, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "public".to_string(),
            is_async,
            line,
        }
    }

    fn create_struct_item(name: &str, fields: usize, derives: Vec<String>, line: usize) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: "public".to_string(),
            fields_count: fields,
            derives,
            line,
        }
    }

    fn create_enum_item(name: &str, variants: usize, line: usize) -> AstItem {
        AstItem::Enum {
            name: name.to_string(),
            visibility: "public".to_string(),
            variants_count: variants,
            line,
        }
    }

    fn create_trait_item(name: &str, line: usize) -> AstItem {
        AstItem::Trait {
            name: name.to_string(),
            visibility: "public".to_string(),
            line,
        }
    }

    fn create_module_item(name: &str, line: usize) -> AstItem {
        AstItem::Module {
            name: name.to_string(),
            visibility: "public".to_string(),
            line,
        }
    }

    fn create_use_item(path: &str, line: usize) -> AstItem {
        AstItem::Use {
            path: path.to_string(),
            line,
        }
    }

    fn create_impl_item(type_name: &str, trait_name: Option<&str>, line: usize) -> AstItem {
        AstItem::Impl {
            type_name: type_name.to_string(),
            trait_name: trait_name.map(|s| s.to_string()),
            line,
        }
    }

    fn create_import_item(module: &str, line: usize) -> AstItem {
        AstItem::Import {
            module: module.to_string(),
            items: vec![],
            alias: None,
            line,
        }
    }

    // BaseLanguageMapper Tests

    #[test]
    fn test_base_language_mapper_new() {
        let mapper = BaseLanguageMapper::new(Language::Java);
        assert_eq!(mapper.language, Language::Java);
    }

    #[test]
    fn test_base_language_mapper_language() {
        let mapper = BaseLanguageMapper::new(Language::Kotlin);
        assert_eq!(mapper.language(), Language::Kotlin);
    }

    #[test]
    fn test_base_language_mapper_clone() {
        let mapper = BaseLanguageMapper::new(Language::Scala);
        let cloned = mapper.clone();
        assert_eq!(cloned.language, Language::Scala);
    }

    #[test]
    fn test_base_language_mapper_clone_box() {
        let mapper = BaseLanguageMapper::new(Language::TypeScript);
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::TypeScript);
    }

    #[test]
    fn test_base_language_mapper_convert_ast_items_comprehensive() {
        let mapper = BaseLanguageMapper::new(Language::Rust);
        let path = Path::new("/test/file.rs");

        // Test with various AstItem types
        let items = vec![
            create_function_item("test_func", false, 1),
            create_struct_item("TestStruct", 3, vec!["Debug".to_string()], 10),
            create_enum_item("TestEnum", 5, 20),
            create_trait_item("TestTrait", 30),
            create_module_item("test_module", 40),
            create_use_item("std::collections::HashMap", 50),
            create_impl_item("TestStruct", Some("TestTrait"), 60),
            create_import_item("external_crate", 70),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 8);
        assert_eq!(nodes[0].kind, NodeKind::Function);
        assert_eq!(nodes[0].name, "test_func");
        assert_eq!(nodes[1].kind, NodeKind::Struct);
        assert_eq!(nodes[1].name, "TestStruct");
        assert_eq!(nodes[2].kind, NodeKind::Enum);
        assert_eq!(nodes[2].name, "TestEnum");
        assert_eq!(nodes[3].kind, NodeKind::Trait);
        assert_eq!(nodes[3].name, "TestTrait");
        assert_eq!(nodes[4].kind, NodeKind::Module);
        assert_eq!(nodes[4].name, "test_module");
        assert_eq!(nodes[5].kind, NodeKind::Uses);
        assert_eq!(nodes[5].name, "std::collections::HashMap");
        assert_eq!(nodes[6].kind, NodeKind::Implements);
        assert_eq!(nodes[6].name, "TestStruct");
        assert_eq!(nodes[7].kind, NodeKind::Import);
        assert_eq!(nodes[7].name, "external_crate");
    }

    #[test]
    fn test_base_language_mapper_convert_ast_items_empty() {
        let mapper = BaseLanguageMapper::new(Language::Go);
        let path = Path::new("/test/file.go");
        let items: Vec<AstItem> = vec![];

        let nodes = mapper.convert_ast_items(&items, path);
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_source_returns_error() {
        let mapper = BaseLanguageMapper::new(Language::Rust);
        let result = mapper
            .map_source("fn main() {}", Path::new("test.rs"))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not implemented"));
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_file_not_found() {
        let mapper = BaseLanguageMapper::new(Language::Java);
        let result = mapper.map_file(Path::new("/nonexistent/file.java")).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_not_found() {
        let mapper = BaseLanguageMapper::new(Language::Java);
        let result = mapper
            .map_directory(Path::new("/nonexistent/dir"), false)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = BaseLanguageMapper::new(Language::Java);

        let result = mapper.map_directory(temp_dir.path(), false).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_with_non_matching_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with non-matching extensions
        fs::write(temp_dir.path().join("readme.txt"), "Hello").unwrap();
        fs::write(temp_dir.path().join("data.json"), "{}").unwrap();

        let mapper = BaseLanguageMapper::new(Language::Java);
        let result = mapper.map_directory(temp_dir.path(), false).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_with_nested_dirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("subdir");
        fs::create_dir(&nested_dir).unwrap();

        // Create files in both directories
        fs::write(temp_dir.path().join("file.txt"), "test").unwrap();
        fs::write(nested_dir.join("nested.txt"), "nested").unwrap();

        let mapper = BaseLanguageMapper::new(Language::Java);

        // Non-recursive should skip nested directories
        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());

        // Recursive should explore nested directories
        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_base_language_mapper_create_test_node() {
        let mapper = BaseLanguageMapper::new(Language::Python);
        let node = mapper.create_test_node(NodeKind::Function, "test_function");

        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "test_function");
        assert_eq!(node.language, Language::Python);
    }

    // LanguageMapperFactory Tests (in language_mapper.rs)

    #[test]
    fn test_factory_create_all_languages() {
        let mappers = LanguageMapperFactory::create_all();

        // Should contain all supported languages
        assert!(mappers.contains_key(&Language::Java));
        assert!(mappers.contains_key(&Language::Kotlin));
        assert!(mappers.contains_key(&Language::Scala));
        assert!(mappers.contains_key(&Language::TypeScript));
        assert!(mappers.contains_key(&Language::JavaScript));

        // Verify each mapper returns correct language
        for (lang, mapper) in &mappers {
            assert_eq!(mapper.language(), *lang);
        }
    }

    #[test]
    fn test_factory_create_kotlin() {
        let result = LanguageMapperFactory::create(Language::Kotlin);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::Kotlin);
    }

    #[test]
    fn test_factory_create_typescript() {
        let result = LanguageMapperFactory::create(Language::TypeScript);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_factory_create_javascript() {
        let result = LanguageMapperFactory::create(Language::JavaScript);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_factory_create_for_file_kotlin() {
        let result = LanguageMapperFactory::create_for_file(Path::new("Test.kt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::Kotlin);
    }

    #[test]
    fn test_factory_create_for_file_typescript() {
        let result = LanguageMapperFactory::create_for_file(Path::new("app.ts"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_factory_create_for_file_javascript() {
        let result = LanguageMapperFactory::create_for_file(Path::new("script.js"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_factory_create_for_file_scala() {
        let result = LanguageMapperFactory::create_for_file(Path::new("Main.scala"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::Scala);
    }

    #[test]
    fn test_factory_create_for_file_unsupported() {
        let result = LanguageMapperFactory::create_for_file(Path::new("readme.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_create_for_file_no_extension() {
        let result = LanguageMapperFactory::create_for_file(Path::new("Makefile"));
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_create_unsupported_other() {
        let result = LanguageMapperFactory::create(Language::Other(999));
        assert!(result.is_err());
    }

