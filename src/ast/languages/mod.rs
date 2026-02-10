#![cfg_attr(coverage_nightly, coverage(off))]
//! Language-specific AST parsing strategies
//!
//! This module implements the strategy pattern for different programming languages,
//! providing a unified interface for AST parsing and analysis.

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use crate::ast::core::{AstDag, Language, UnifiedAstNode};

pub mod c_cpp;
#[cfg(feature = "lua-ast")]
pub mod lua;
pub mod others;
pub mod python;
pub mod rust;
pub mod typescript;

/// Trait for language-specific AST parsing strategies
#[async_trait]
pub trait LanguageStrategy: Send + Sync {
    /// Get the language this strategy handles
    fn language(&self) -> Language;

    /// Check if this strategy can parse the given file
    fn can_parse(&self, path: &Path) -> bool;

    /// Parse a file into a unified AST
    async fn parse_file(&self, path: &Path, content: &str) -> Result<AstDag>;

    /// Extract imports from the AST
    fn extract_imports(&self, ast: &AstDag) -> Vec<String>;

    /// Extract function definitions
    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode>;

    /// Extract type definitions (classes, structs, interfaces)
    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode>;

    /// Calculate complexity metrics
    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32); // (cyclomatic, cognitive)
}

/// Registry for language strategies
pub struct LanguageRegistry {
    strategies: Vec<Arc<dyn LanguageStrategy>>,
}

impl LanguageRegistry {
    /// Create a new registry with all default strategies
    #[must_use]
    pub fn new() -> Self {
        let mut strategies: Vec<Arc<dyn LanguageStrategy>> = vec![
            Arc::new(rust::RustStrategy::new()),
            Arc::new(python::PythonStrategy::new()),
            Arc::new(typescript::TypeScriptStrategy::new()),
            Arc::new(typescript::JavaScriptStrategy::new()),
            Arc::new(c_cpp::CStrategy::new()),
            Arc::new(c_cpp::CppStrategy::new()),
        ];

        #[cfg(feature = "lua-ast")]
        strategies.push(Arc::new(lua::LuaStrategy::new()));

        Self { strategies }
    }

    /// Register a custom strategy
    pub fn register(&mut self, strategy: Arc<dyn LanguageStrategy>) {
        self.strategies.push(strategy);
    }

    /// Find a strategy for the given file
    #[must_use]
    pub fn find_strategy(&self, path: &Path) -> Option<Arc<dyn LanguageStrategy>> {
        self.strategies.iter().find(|s| s.can_parse(path)).cloned()
    }

    /// Get strategy for a specific language
    #[must_use]
    pub fn get_strategy(&self, language: Language) -> Option<Arc<dyn LanguageStrategy>> {
        self.strategies
            .iter()
            .find(|s| s.language() == language)
            .cloned()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== LanguageRegistry Tests ====================

    #[test]
    fn test_registry_new() {
        let registry = LanguageRegistry::new();
        // 6 default strategies: Rust, Python, TypeScript, JavaScript, C, C++
        // +1 when lua-ast feature is enabled
        let expected = if cfg!(feature = "lua-ast") { 7 } else { 6 };
        assert_eq!(registry.strategies.len(), expected);
    }

    #[test]
    fn test_registry_default() {
        let registry = LanguageRegistry::default();
        let expected = if cfg!(feature = "lua-ast") { 7 } else { 6 };
        assert_eq!(registry.strategies.len(), expected);
    }

    #[test]
    fn test_find_strategy_rust() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.rs");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Rust);
    }

    #[test]
    fn test_find_strategy_python() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.py");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Python);
    }

    #[test]
    fn test_find_strategy_typescript() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.ts");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_find_strategy_tsx() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("component.tsx");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_find_strategy_javascript() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.js");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_find_strategy_jsx() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("component.jsx");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_find_strategy_mjs() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("module.mjs");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_find_strategy_c() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.c");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::C);
    }

    #[test]
    fn test_find_strategy_c_header() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.h");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::C);
    }

    #[test]
    fn test_find_strategy_cpp() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.cpp");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Cpp);
    }

    #[test]
    fn test_find_strategy_cpp_cc() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.cc");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Cpp);
    }

    #[test]
    fn test_find_strategy_cpp_hpp() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.hpp");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Cpp);
    }

    #[test]
    fn test_find_strategy_unknown() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("test.unknown");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_none());
    }

    #[test]
    fn test_find_strategy_no_extension() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("Makefile");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_none());
    }

    #[test]
    fn test_get_strategy_rust() {
        let registry = LanguageRegistry::new();
        let strategy = registry.get_strategy(Language::Rust);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Rust);
    }

    #[test]
    fn test_get_strategy_python() {
        let registry = LanguageRegistry::new();
        let strategy = registry.get_strategy(Language::Python);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Python);
    }

    #[test]
    fn test_get_strategy_typescript() {
        let registry = LanguageRegistry::new();
        let strategy = registry.get_strategy(Language::TypeScript);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_get_strategy_javascript() {
        let registry = LanguageRegistry::new();
        let strategy = registry.get_strategy(Language::JavaScript);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_get_strategy_c() {
        let registry = LanguageRegistry::new();
        let strategy = registry.get_strategy(Language::C);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::C);
    }

    #[test]
    fn test_get_strategy_cpp() {
        let registry = LanguageRegistry::new();
        let strategy = registry.get_strategy(Language::Cpp);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Cpp);
    }

    #[test]
    fn test_get_strategy_not_registered() {
        let registry = LanguageRegistry::new();
        // Kotlin is not in the default registry
        let strategy = registry.get_strategy(Language::Kotlin);
        assert!(strategy.is_none());
    }

    #[test]
    fn test_register_custom_strategy() {
        let mut registry = LanguageRegistry::new();
        let initial_count = registry.strategies.len();

        // Register a custom strategy
        let custom_strategy = Arc::new(others::PlaceholderStrategy::kotlin());
        registry.register(custom_strategy);

        assert_eq!(registry.strategies.len(), initial_count + 1);

        // Now Kotlin should be findable
        let strategy = registry.get_strategy(Language::Kotlin);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Kotlin);
    }

    #[test]
    fn test_register_multiple_strategies() {
        let mut registry = LanguageRegistry::new();
        let initial_count = registry.strategies.len();

        registry.register(Arc::new(others::PlaceholderStrategy::kotlin()));
        registry.register(Arc::new(others::PlaceholderStrategy::shell()));
        registry.register(Arc::new(others::PlaceholderStrategy::makefile()));

        assert_eq!(registry.strategies.len(), initial_count + 3);
    }

    #[test]
    fn test_find_strategy_after_register() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(others::PlaceholderStrategy::kotlin()));

        let path = PathBuf::from("main.kt");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Kotlin);
    }

    #[test]
    fn test_find_strategy_shell_after_register() {
        let mut registry = LanguageRegistry::new();
        registry.register(Arc::new(others::PlaceholderStrategy::shell()));

        let path = PathBuf::from("script.sh");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Shell);
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_find_strategy_empty_path() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_none());
    }

    #[test]
    fn test_find_strategy_full_path() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("/home/user/project/src/main.rs");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Rust);
    }

    #[test]
    fn test_find_strategy_relative_path() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from("./src/lib.rs");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Rust);
    }

    #[test]
    fn test_find_strategy_hidden_file() {
        let registry = LanguageRegistry::new();
        let path = PathBuf::from(".hidden.rs");
        let strategy = registry.find_strategy(&path);
        assert!(strategy.is_some());
        assert_eq!(strategy.unwrap().language(), Language::Rust);
    }

    #[test]
    fn test_strategies_are_unique_languages() {
        let registry = LanguageRegistry::new();
        let languages: Vec<_> = registry.strategies.iter().map(|s| s.language()).collect();

        // Check no duplicates
        let mut unique = languages.clone();
        unique.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        unique.dedup_by(|a, b| format!("{a:?}") == format!("{b:?}"));
        assert_eq!(
            languages.len(),
            unique.len(),
            "All languages should be unique"
        );
    }
}
