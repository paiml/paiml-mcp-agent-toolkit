//! Factory for creating language-specific mappers
//!
//! This module provides a factory for creating language-specific mappers
//! that can transform language-specific ASTs into the unified representation.

use crate::ast::polyglot::language_mapper::TypeScriptMapper;
// Sprint 46 Phase 6: Feature-gated unused mapper imports
#[cfg(feature = "csharp-ast")]
use crate::ast::polyglot::language_mapper::CSharpMapper;
#[cfg(feature = "java-ast")]
use crate::ast::polyglot::language_mapper::JavaMapper;
#[cfg(feature = "javascript-ast")]
use crate::ast::polyglot::language_mapper::JavaScriptMapper;
#[cfg(feature = "kotlin-ast")]
use crate::ast::polyglot::language_mapper::KotlinMapper;
#[cfg(feature = "ruby-ast")]
use crate::ast::polyglot::language_mapper::RubyMapper;
#[cfg(feature = "scala-ast")]
use crate::ast::polyglot::language_mapper::ScalaMapper;
use crate::ast::polyglot::{Language, LanguageMapper, PolyglotPathValidator, UnifiedNode};
use crate::services::context::AstItem;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Factory for creating language mappers
pub struct LanguageMapperFactory;

impl LanguageMapperFactory {
    /// Create a language mapper for a specific language
    pub fn create(language: Language) -> Result<Arc<dyn LanguageMapper>> {
        match language {
            #[cfg(feature = "polyglot-java")]
            Language::Java => Ok(Arc::new(JavaMapper::new())),

            #[cfg(feature = "polyglot-kotlin")]
            Language::Kotlin => Ok(Arc::new(KotlinMapper::new())),

            #[cfg(feature = "polyglot-scala")]
            Language::Scala => Ok(Arc::new(ScalaMapper::new())),

            #[cfg(feature = "polyglot-typescript")]
            Language::TypeScript => Ok(Arc::new(TypeScriptMapper::new())),

            #[cfg(feature = "polyglot-javascript")]
            Language::JavaScript => Ok(Arc::new(JavaScriptMapper::new())),

            #[cfg(feature = "polyglot-csharp")]
            Language::CSharp => Ok(Arc::new(CSharpMapper::new())),

            #[cfg(feature = "polyglot-ruby")]
            Language::Ruby => Ok(Arc::new(RubyMapper::new())),

            _ => {
                // For now, use a stub mapper for testing
                Ok(Arc::new(StubMapper::new(language)))
            }
        }
    }
}

/// Stub mapper for testing and languages without full implementation
#[derive(Clone)]
pub struct StubMapper {
    language: Language,
}

impl StubMapper {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

#[async_trait]
impl LanguageMapper for StubMapper {
    fn language(&self) -> Language {
        self.language
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        // Validate path first
        PolyglotPathValidator::validate_file_path(path)?;
        // Return an empty list for now - this is just a stub
        Ok(Vec::new())
    }

    async fn map_directory(&self, path: &Path, _recursive: bool) -> Result<Vec<UnifiedNode>> {
        // Validate path first
        PolyglotPathValidator::validate_directory_path(path)?;
        // Return an empty list for now - this is just a stub
        Ok(Vec::new())
    }

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        // Return an empty list for now - this is just a stub
        Ok(Vec::new())
    }

    fn convert_ast_items(&self, _items: &[AstItem], _path: &Path) -> Vec<UnifiedNode> {
        // Return an empty list for now - this is just a stub
        Vec::new()
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
}
