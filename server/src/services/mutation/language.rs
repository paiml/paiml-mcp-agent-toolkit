//! Language adapter system for mutation testing

use super::operators::MutationOperator;
use super::language_detector::Language; // Sprint 63: Use centralized language detection
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Trait for language-specific AST operations
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    /// Language name (e.g., "rust", "python")
    fn name(&self) -> &str;

    /// File extensions for this language
    fn extensions(&self) -> &[&str];

    /// Parse source code to AST
    async fn parse(&self, source: &str) -> Result<String>;

    /// Unparse AST back to source code
    async fn unparse(&self, ast: &str) -> Result<String>;

    /// Get mutation operators for this language
    fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>>;

    /// Run tests for this language
    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult>;
}

/// Test run result
#[derive(Debug, Clone)]
pub struct TestRunResult {
    pub passed: bool,
    pub failures: Vec<String>,
    pub execution_time_ms: u64,
    pub stdout: String,
    pub stderr: String,
}

/// Language registry for detecting and managing language adapters
pub struct LanguageRegistry {
    adapters: HashMap<String, Arc<dyn LanguageAdapter>>,
}

impl LanguageRegistry {
    /// Create a new language registry
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a language adapter
    pub fn register(&mut self, adapter: Arc<dyn LanguageAdapter>) {
        self.adapters.insert(adapter.name().to_string(), adapter);
    }

    /// Register TypeScript/JavaScript adapter
    pub fn register_typescript(&mut self) {
        use crate::services::mutation::TypeScriptAdapter;
        self.register(Arc::new(TypeScriptAdapter::new()));
    }

    /// Register Python adapter
    pub fn register_python(&mut self) {
        use crate::services::mutation::PythonAdapter;
        self.register(Arc::new(PythonAdapter::new()));
    }

    /// Register Go adapter
    pub fn register_go(&mut self) {
        use crate::services::mutation::GoAdapter;
        self.register(Arc::new(GoAdapter::new()));
    }

    /// Register C/C++ adapter
    pub fn register_cpp(&mut self) {
        use crate::services::mutation::CppAdapter;
        self.register(Arc::new(CppAdapter::new()));
    }

    /// Register WASM adapter
    pub fn register_wasm(&mut self) {
        use crate::services::mutation::WasmAdapter;
        self.register(Arc::new(WasmAdapter::new()));
    }

    /// Detect language from file path using centralized Language enum (Sprint 63)
    pub fn detect_language(&self, path: &Path) -> Option<Arc<dyn LanguageAdapter>> {
        // Use centralized language detection from language_detector module
        let detected_language = Language::from_extension(path);

        match detected_language {
            Language::Rust => self.get_adapter("rust"),
            Language::Python => self.get_adapter("python"),
            Language::TypeScript | Language::JavaScript => self.get_adapter("typescript"),
            Language::Go => self.get_adapter("go"),
            Language::Cpp => self.get_adapter("cpp"),
            Language::Unsupported => None,
        }
    }

    /// Detect language from file path (legacy extension-based detection)
    pub fn detect_language_by_extension(&self, path: &Path) -> Option<Arc<dyn LanguageAdapter>> {
        let extension = path.extension()?.to_str()?;

        for adapter in self.adapters.values() {
            if adapter.extensions().contains(&extension) {
                return Some(Arc::clone(adapter));
            }
        }

        None
    }

    /// Get adapter by name
    pub fn get_adapter(&self, name: &str) -> Option<Arc<dyn LanguageAdapter>> {
        self.adapters.get(name).map(Arc::clone)
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter;

    #[async_trait]
    impl LanguageAdapter for MockAdapter {
        fn name(&self) -> &str {
            "mock"
        }

        fn extensions(&self) -> &[&str] {
            &["mock"]
        }

        async fn parse(&self, source: &str) -> Result<String> {
            Ok(source.to_string())
        }

        async fn unparse(&self, ast: &str) -> Result<String> {
            Ok(ast.to_string())
        }

        fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>> {
            vec![]
        }

        async fn run_tests(&self, _source_file: &Path) -> Result<TestRunResult> {
            Ok(TestRunResult {
                passed: true,
                failures: vec![],
                execution_time_ms: 100,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn test_language_registry_register() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));

        assert_eq!(registry.languages(), vec!["mock"]);
    }

    #[test]
    fn test_language_registry_detect() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));

        let adapter = registry.detect_language(Path::new("test.mock"));
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().name(), "mock");
    }

    #[test]
    fn test_language_registry_detect_unknown() {
        let registry = LanguageRegistry::new();
        let adapter = registry.detect_language(Path::new("test.unknown"));
        assert!(adapter.is_none());
    }

    #[test]
    fn test_language_registry_get_adapter() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));

        let adapter = registry.get_adapter("mock");
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().name(), "mock");
    }

    // Sprint 25: Dogfooding - Additional edge case tests

    #[test]
    fn test_language_registry_get_adapter_unknown() {
        let registry = LanguageRegistry::new();
        let adapter = registry.get_adapter("unknown");
        assert!(adapter.is_none(), "Unknown adapter should return None");
    }

    #[test]
    fn test_language_registry_languages_empty() {
        let registry = LanguageRegistry::new();
        assert_eq!(registry.languages().len(), 0, "Empty registry should have no languages");
    }

    #[test]
    fn test_language_registry_languages_multiple() {
        struct MockAdapter2;

        #[async_trait]
        impl LanguageAdapter for MockAdapter2 {
            fn name(&self) -> &str {
                "mock2"
            }

            fn extensions(&self) -> &[&str] {
                &["m2"]
            }

            async fn parse(&self, source: &str) -> Result<String> {
                Ok(source.to_string())
            }

            async fn unparse(&self, ast: &str) -> Result<String> {
                Ok(ast.to_string())
            }

            fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>> {
                vec![]
            }

            async fn run_tests(&self, _source_file: &Path) -> Result<TestRunResult> {
                Ok(TestRunResult {
                    passed: true,
                    failures: vec![],
                    execution_time_ms: 100,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            }
        }

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));
        registry.register(Arc::new(MockAdapter2));

        let languages = registry.languages();
        assert_eq!(languages.len(), 2);
        assert!(languages.contains(&"mock"));
        assert!(languages.contains(&"mock2"));
    }

    #[test]
    fn test_language_registry_default() {
        let registry = LanguageRegistry::default();
        assert_eq!(registry.languages().len(), 0, "Default registry should be empty");
    }

    #[test]
    fn test_language_registry_detect_no_extension() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));

        let adapter = registry.detect_language(Path::new("noextension"));
        assert!(adapter.is_none(), "File without extension should not match");
    }

    #[test]
    fn test_language_registry_detect_case_sensitive() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));

        // Extensions are case-sensitive
        let adapter = registry.detect_language(Path::new("test.MOCK"));
        assert!(adapter.is_none(), "Extension matching should be case-sensitive");
    }

    #[test]
    fn test_test_run_result_construction() {
        let result = TestRunResult {
            passed: true,
            failures: vec!["test1".to_string()],
            execution_time_ms: 250,
            stdout: "output".to_string(),
            stderr: "errors".to_string(),
        };

        assert!(result.passed);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.execution_time_ms, 250);
        assert_eq!(result.stdout, "output");
        assert_eq!(result.stderr, "errors");
    }

    // Sprint 63: Multi-Language Detection Integration Tests

    #[test]
    fn test_language_enum_integration_rust() {
        use crate::services::mutation::RustAdapter;

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(RustAdapter::new()));

        // Test .rs extension detection via Language enum
        let adapter = registry.detect_language(Path::new("test.rs"));
        assert!(adapter.is_some(), "Rust file should be detected");
        assert_eq!(adapter.unwrap().name(), "rust");
    }

    #[test]
    fn test_language_enum_integration_python() {
        use crate::services::mutation::PythonAdapter;

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(PythonAdapter::new()));

        // Test .py extension detection via Language enum
        let adapter = registry.detect_language(Path::new("test.py"));
        assert!(adapter.is_some(), "Python file should be detected");
        assert_eq!(adapter.unwrap().name(), "python");
    }

    #[test]
    fn test_language_enum_integration_typescript() {
        use crate::services::mutation::TypeScriptAdapter;

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(TypeScriptAdapter::new()));

        // Test .ts extension detection via Language enum
        let adapter = registry.detect_language(Path::new("test.ts"));
        assert!(adapter.is_some(), "TypeScript file should be detected");
        assert_eq!(adapter.unwrap().name(), "typescript");

        // Test .tsx extension
        let adapter_tsx = registry.detect_language(Path::new("component.tsx"));
        assert!(adapter_tsx.is_some(), "TSX file should be detected");
        assert_eq!(adapter_tsx.unwrap().name(), "typescript");
    }

    #[test]
    fn test_language_enum_integration_javascript() {
        use crate::services::mutation::TypeScriptAdapter;

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(TypeScriptAdapter::new()));

        // Test .js extension detection via Language enum (handled by TypeScript adapter)
        let adapter = registry.detect_language(Path::new("test.js"));
        assert!(adapter.is_some(), "JavaScript file should be detected");
        assert_eq!(adapter.unwrap().name(), "typescript");

        // Test .jsx extension
        let adapter_jsx = registry.detect_language(Path::new("component.jsx"));
        assert!(adapter_jsx.is_some(), "JSX file should be detected");
        assert_eq!(adapter_jsx.unwrap().name(), "typescript");
    }

    #[test]
    fn test_language_enum_integration_go() {
        use crate::services::mutation::GoAdapter;

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(GoAdapter::new()));

        // Test .go extension detection via Language enum
        let adapter = registry.detect_language(Path::new("main.go"));
        assert!(adapter.is_some(), "Go file should be detected");
        assert_eq!(adapter.unwrap().name(), "go");
    }

    #[test]
    fn test_language_enum_integration_cpp() {
        use crate::services::mutation::CppAdapter;

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(CppAdapter::new()));

        // Test multiple C++ extensions via Language enum
        let extensions = vec!["cpp", "cc", "cxx", "hpp", "hxx", "h"];

        for ext in extensions {
            let filename = format!("test.{}", ext);
            let adapter = registry.detect_language(Path::new(&filename));
            assert!(adapter.is_some(), "{} file should be detected", ext);
            assert_eq!(adapter.unwrap().name(), "cpp");
        }
    }

    #[test]
    fn test_language_enum_integration_unsupported() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(MockAdapter));

        // Test unsupported extension returns None
        let adapter = registry.detect_language(Path::new("test.xyz"));
        assert!(adapter.is_none(), "Unsupported extension should return None");
    }

    #[test]
    fn test_language_enum_integration_multi_language() {
        use crate::services::mutation::{RustAdapter, PythonAdapter, TypeScriptAdapter};

        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(RustAdapter::new()));
        registry.register(Arc::new(PythonAdapter::new()));
        registry.register(Arc::new(TypeScriptAdapter::new()));

        // Verify all languages detected correctly
        assert!(registry.detect_language(Path::new("test.rs")).is_some());
        assert!(registry.detect_language(Path::new("test.py")).is_some());
        assert!(registry.detect_language(Path::new("test.ts")).is_some());
        assert!(registry.detect_language(Path::new("test.js")).is_some());

        // Verify languages list
        let languages = registry.languages();
        assert_eq!(languages.len(), 3);
        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"python"));
        assert!(languages.contains(&"typescript"));
    }
}
