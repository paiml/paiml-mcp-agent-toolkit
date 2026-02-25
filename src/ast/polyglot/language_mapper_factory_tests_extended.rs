    // ==========================================================================
    // Additional coverage tests for StubMapper edge cases
    // ==========================================================================

    #[test]
    fn test_stub_mapper_all_language_variants() {
        // Exhaustively test all Language variants through StubMapper
        let languages = vec![
            Language::Java,
            Language::Kotlin,
            Language::Scala,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Rust,
            Language::Go,
            Language::Cpp,
            Language::CSharp,
            Language::Ruby,
            Language::Swift,
            Language::Php,
            Language::Other(0),
            Language::Other(100),
        ];

        for lang in languages {
            let mapper = StubMapper::new(lang);
            assert_eq!(mapper.language(), lang);

            // Test clone works for all variants
            let cloned = mapper.clone();
            assert_eq!(cloned.language(), lang);

            // Test clone_box works for all variants
            let boxed = mapper.clone_box();
            assert_eq!(boxed.language(), lang);
        }
    }

    #[tokio::test]
    async fn test_stub_mapper_map_file_with_directory_path() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = StubMapper::new(Language::Python);

        // Passing a directory path to map_file should fail validation
        let result = mapper.map_file(temp_dir.path()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_directory_with_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, "def hello(): pass").unwrap();

        let mapper = StubMapper::new(Language::Python);

        // Passing a file path to map_directory should fail validation
        let result = mapper.map_directory(&file_path, false).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stub_mapper_convert_ast_items_all_variants() {
        let mapper = StubMapper::new(Language::Rust);

        let items = vec![
            AstItem::Function {
                name: "my_func".to_string(),
                visibility: "pub".to_string(),
                is_async: true,
                line: 1,
            },
            AstItem::Struct {
                name: "MyStruct".to_string(),
                visibility: "pub".to_string(),
                fields_count: 3,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                line: 10,
            },
            AstItem::Enum {
                name: "MyEnum".to_string(),
                visibility: "pub".to_string(),
                variants_count: 5,
                line: 20,
            },
            AstItem::Trait {
                name: "MyTrait".to_string(),
                visibility: "pub".to_string(),
                line: 30,
            },
            AstItem::Impl {
                type_name: "MyStruct".to_string(),
                trait_name: Some("MyTrait".to_string()),
                line: 40,
            },
            AstItem::Module {
                name: "my_module".to_string(),
                visibility: "pub".to_string(),
                line: 50,
            },
            AstItem::Use {
                path: "std::collections::HashMap".to_string(),
                line: 60,
            },
            AstItem::Import {
                module: "external_crate".to_string(),
                items: vec![],
                alias: Some("ext".to_string()),
                line: 70,
            },
        ];

        // StubMapper always returns empty for all item types
        let result = mapper.convert_ast_items(&items, Path::new("test.rs"));
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_source_unicode() {
        let mapper = StubMapper::new(Language::Python);
        let unicode_source = r#"
def greet(name):
    return f"Hello, {name}! 你好！こんにちは！🎉"
"#;
        let result = mapper
            .map_source(unicode_source, Path::new("test.py"))
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_source_large() {
        let mapper = StubMapper::new(Language::Python);
        // Generate a large source file
        let large_source = (0..1000)
            .map(|i| format!("def func_{}(): pass\n", i))
            .collect::<String>();
        let result = mapper.map_source(&large_source, Path::new("test.py")).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_factory_create_all_languages_succeed() {
        // Ensure factory never returns an error for any valid Language
        let languages = vec![
            Language::Java,
            Language::Kotlin,
            Language::Scala,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Rust,
            Language::Go,
            Language::Cpp,
            Language::CSharp,
            Language::Ruby,
            Language::Swift,
            Language::Php,
            Language::Other(0),
            Language::Other(u32::MAX),
        ];

        for lang in languages {
            let result = LanguageMapperFactory::create(lang);
            assert!(result.is_ok(), "Factory failed for language: {:?}", lang);
            assert_eq!(result.unwrap().language(), lang);
        }
    }

    #[test]
    fn test_stub_mapper_implements_send_sync() {
        // Verify StubMapper can be sent between threads
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StubMapper>();
    }

    #[tokio::test]
    async fn test_stub_mapper_map_source_with_special_paths() {
        let mapper = StubMapper::new(Language::Python);

        // Test with various path patterns
        let paths = vec![
            Path::new("test.py"),
            Path::new("/absolute/path/test.py"),
            Path::new("relative/path/test.py"),
            Path::new("./test.py"),
            Path::new("../test.py"),
            Path::new("path with spaces/test.py"),
        ];

        for path in paths {
            let result = mapper.map_source("def test(): pass", path).await;
            assert!(result.is_ok(), "Failed for path: {:?}", path);
            assert!(result.unwrap().is_empty());
        }
    }
