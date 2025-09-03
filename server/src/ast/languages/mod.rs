//! Language-specific AST parsing strategies
//!
//! This module implements the strategy pattern for different programming languages,
//! providing a unified interface for AST parsing and analysis.

use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;

use crate::ast::core::{AstDag, Language, UnifiedAstNode};

pub mod rust;
pub mod python;
pub mod typescript;
pub mod c_cpp;
pub mod others;

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
    pub fn new() -> Self {
        let strategies: Vec<Arc<dyn LanguageStrategy>> = vec![
            Arc::new(rust::RustStrategy::new()),
            Arc::new(python::PythonStrategy::new()),
            Arc::new(typescript::TypeScriptStrategy::new()),
            Arc::new(typescript::JavaScriptStrategy::new()),
            Arc::new(c_cpp::CStrategy::new()),
            Arc::new(c_cpp::CppStrategy::new()),
        ];
        
        Self { strategies }
    }
    
    /// Register a custom strategy
    pub fn register(&mut self, strategy: Arc<dyn LanguageStrategy>) {
        self.strategies.push(strategy);
    }
    
    /// Find a strategy for the given file
    pub fn find_strategy(&self, path: &Path) -> Option<Arc<dyn LanguageStrategy>> {
        self.strategies
            .iter()
            .find(|s| s.can_parse(path))
            .cloned()
    }
    
    /// Get strategy for a specific language
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