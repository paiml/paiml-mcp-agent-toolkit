//! Language adapter system for mutation testing

use super::operators::MutationOperator;
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

    /// Detect language from file path
    pub fn detect_language(&self, path: &Path) -> Option<Arc<dyn LanguageAdapter>> {
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
}
