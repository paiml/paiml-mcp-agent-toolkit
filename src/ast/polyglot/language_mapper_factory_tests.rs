    // ==========================================================================
    // LanguageMapperFactory Tests
    // ==========================================================================

    #[test]
    fn test_factory_create_typescript() {
        // TypeScript should use TypeScriptMapper (always available)
        let result = LanguageMapperFactory::create(Language::TypeScript);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::TypeScript);
    }

    #[test]
    fn test_factory_create_python_uses_stub() {
        // Python falls through to StubMapper (no polyglot-python feature)
        let result = LanguageMapperFactory::create(Language::Python);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Python);
    }

    #[test]
    fn test_factory_create_rust_uses_stub() {
        // Rust falls through to StubMapper
        let result = LanguageMapperFactory::create(Language::Rust);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Rust);
    }

    #[test]
    fn test_factory_create_go_uses_stub() {
        // Go falls through to StubMapper
        let result = LanguageMapperFactory::create(Language::Go);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Go);
    }

    #[test]
    fn test_factory_create_cpp_uses_stub() {
        // C++ falls through to StubMapper
        let result = LanguageMapperFactory::create(Language::Cpp);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Cpp);
    }

    #[test]
    fn test_factory_create_swift_uses_stub() {
        // Swift falls through to StubMapper
        let result = LanguageMapperFactory::create(Language::Swift);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Swift);
    }

    #[test]
    fn test_factory_create_php_uses_stub() {
        // PHP falls through to StubMapper
        let result = LanguageMapperFactory::create(Language::Php);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Php);
    }

    #[test]
    fn test_factory_create_other_uses_stub() {
        // Other(n) falls through to StubMapper
        let result = LanguageMapperFactory::create(Language::Other(42));
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Other(42));
    }

    // ==========================================================================
    // StubMapper Tests
    // ==========================================================================

    #[test]
    fn test_stub_mapper_new() {
        let mapper = StubMapper::new(Language::Python);
        assert_eq!(mapper.language, Language::Python);
    }

    #[test]
    fn test_stub_mapper_language() {
        let mapper = StubMapper::new(Language::Go);
        assert_eq!(mapper.language(), Language::Go);
    }

    #[test]
    fn test_stub_mapper_clone() {
        let mapper = StubMapper::new(Language::Rust);
        let cloned = mapper.clone();
        assert_eq!(cloned.language, Language::Rust);
    }

    #[test]
    fn test_stub_mapper_clone_box() {
        let mapper = StubMapper::new(Language::Cpp);
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Cpp);
    }

    #[tokio::test]
    async fn test_stub_mapper_map_file_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, "def hello(): pass").unwrap();

        let mapper = StubMapper::new(Language::Python);
        let result = mapper.map_file(&file_path).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_file_invalid() {
        let mapper = StubMapper::new(Language::Python);
        let result = mapper.map_file(Path::new("/nonexistent/file.py")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_directory_valid() {
        let temp_dir = TempDir::new().unwrap();

        let mapper = StubMapper::new(Language::Go);
        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();

        let mapper = StubMapper::new(Language::Go);
        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_directory_invalid() {
        let mapper = StubMapper::new(Language::Go);
        let result = mapper
            .map_directory(Path::new("/nonexistent/dir"), false)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_source() {
        let mapper = StubMapper::new(Language::Rust);
        let result = mapper
            .map_source("fn main() {}", Path::new("test.rs"))
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stub_mapper_map_source_empty() {
        let mapper = StubMapper::new(Language::Swift);
        let result = mapper.map_source("", Path::new("test.swift")).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_stub_mapper_convert_ast_items_empty() {
        let mapper = StubMapper::new(Language::Php);
        let items: Vec<AstItem> = vec![];
        let result = mapper.convert_ast_items(&items, Path::new("test.php"));
        assert!(result.is_empty());
    }

    #[test]
    fn test_stub_mapper_convert_ast_items_with_items() {
        let mapper = StubMapper::new(Language::Python);
        let items = vec![
            AstItem::Function {
                name: "test_func".to_string(),
                visibility: "public".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Struct {
                name: "TestClass".to_string(),
                visibility: "public".to_string(),
                fields_count: 0,
                derives: vec![],
                line: 10,
            },
        ];
        let result = mapper.convert_ast_items(&items, Path::new("test.py"));
        // StubMapper always returns empty - it's a stub
        assert!(result.is_empty());
    }

    // ==========================================================================
    // Feature-gated language tests (always test the fallback behavior)
    // ==========================================================================

    #[test]
    fn test_factory_create_java() {
        // Java may use JavaMapper or StubMapper depending on feature flags
        let result = LanguageMapperFactory::create(Language::Java);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Java);
    }

    #[test]
    fn test_factory_create_kotlin() {
        // Kotlin may use KotlinMapper or StubMapper depending on feature flags
        let result = LanguageMapperFactory::create(Language::Kotlin);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Kotlin);
    }

    #[test]
    fn test_factory_create_scala() {
        // Scala may use ScalaMapper or StubMapper depending on feature flags
        let result = LanguageMapperFactory::create(Language::Scala);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Scala);
    }

    #[test]
    fn test_factory_create_javascript() {
        // JavaScript may use JavaScriptMapper or StubMapper depending on feature flags
        let result = LanguageMapperFactory::create(Language::JavaScript);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::JavaScript);
    }

    #[test]
    fn test_factory_create_csharp() {
        // C# may use CSharpMapper or StubMapper depending on feature flags
        let result = LanguageMapperFactory::create(Language::CSharp);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::CSharp);
    }

    #[test]
    fn test_factory_create_ruby() {
        // Ruby may use RubyMapper or StubMapper depending on feature flags
        let result = LanguageMapperFactory::create(Language::Ruby);
        assert!(result.is_ok());
        let mapper = result.unwrap();
        assert_eq!(mapper.language(), Language::Ruby);
    }

    // ==========================================================================
    // Edge case tests
    // ==========================================================================

    #[test]
    fn test_stub_mapper_with_various_other_values() {
        // Test Other variant with different numeric identifiers
        for id in [0, 1, 100, 999, u32::MAX] {
            let mapper = StubMapper::new(Language::Other(id));
            assert_eq!(mapper.language(), Language::Other(id));
        }
    }

    #[tokio::test]
    async fn test_stub_mapper_map_source_with_complex_code() {
        let mapper = StubMapper::new(Language::Python);
        let complex_source = r#"
class Calculator:
    def __init__(self):
        self.value = 0

    def add(self, x):
        self.value += x
        return self

    async def fetch_data(self):
        return await some_api()
"#;
        let result = mapper
            .map_source(complex_source, Path::new("calc.py"))
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_factory_returns_arc_compatible_mapper() {
        let mapper = LanguageMapperFactory::create(Language::Python).unwrap();
        // Verify it can be cloned via Arc
        let cloned = Arc::clone(&mapper);
        assert_eq!(cloned.language(), Language::Python);
    }

