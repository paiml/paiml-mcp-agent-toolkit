// Toyota Way: Unified AST Module for Structural Complexity Reduction
//
// This module consolidates all AST-related functionality under a single, unified
// framework to reduce structural complexity and achieve A+ grade.
//
// Consolidates:
// - ast_strategies.rs (strategy pattern)
// - ast_rust.rs, ast_python.rs, ast_typescript.rs (individual parsers)  
// - ast_c_dispatch.rs, ast_typescript_dispatch.rs (dispatch logic)
// - unified_ast_parser.rs, unified_ast_engine.rs (unified parsers)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

pub mod languages;
pub mod strategy;
pub mod unified;

// use crate::models::unified_ast::AstDag; // Not needed in unified framework
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;

/// Core AST analysis trait for all language strategies
#[async_trait]
pub trait AstStrategy: Send + Sync {
    /// Analyze a source file and extract AST information
    async fn analyze(
        &self,
        file_path: &Path,
        classifier: &FileClassifier,
    ) -> Result<FileContext>;
    
    /// Get the primary file extension this strategy handles
    fn primary_extension(&self) -> &'static str;
    
    /// Get all file extensions this strategy can handle
    fn supported_extensions(&self) -> Vec<&'static str>;
    
    /// Get the language name
    fn language_name(&self) -> &'static str;
    
    /// Check if this strategy can handle the given file extension
    fn can_handle(&self, extension: &str) -> bool {
        self.supported_extensions().contains(&extension)
    }
}

/// Registry for managing AST strategies across languages
pub struct AstRegistry {
    strategies: std::collections::HashMap<String, Arc<dyn AstStrategy>>,
}

impl AstRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            strategies: std::collections::HashMap::new(),
        };
        
        // Register all available language strategies
        registry.register_defaults();
        registry
    }
    
    fn register_defaults(&mut self) {
        // Register Rust strategy (always available)
        self.register(Arc::new(languages::rust::RustStrategy::new()));
        
        // Register optional language strategies based on features
        #[cfg(feature = "typescript-ast")]
        {
            self.register(Arc::new(languages::typescript::TypeScriptStrategy::new()));
            self.register(Arc::new(languages::javascript::JavaScriptStrategy::new()));
        }
        
        #[cfg(feature = "python-ast")]
        {
            self.register(Arc::new(languages::python::PythonStrategy::new()));
        }
        
        #[cfg(feature = "c-ast")]
        {
            self.register(Arc::new(languages::c::CStrategy::new()));
        }
        
        #[cfg(feature = "cpp-ast")]
        {
            self.register(Arc::new(languages::cpp::CppStrategy::new()));
        }
    }
    
    pub fn register(&mut self, strategy: Arc<dyn AstStrategy>) {
        for ext in strategy.supported_extensions() {
            self.strategies.insert(ext.to_string(), strategy.clone());
        }
    }
    
    pub fn get_strategy(&self, extension: &str) -> Option<Arc<dyn AstStrategy>> {
        self.strategies.get(extension).cloned()
    }
    
    pub fn list_supported_extensions(&self) -> Vec<&str> {
        self.strategies.keys().map(|s| s.as_str()).collect()
    }
    
    /// Analyze a file using the appropriate strategy
    pub async fn analyze_file(
        &self,
        file_path: &Path,
        classifier: &FileClassifier,
    ) -> Result<FileContext> {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        if let Some(strategy) = self.get_strategy(extension) {
            strategy.analyze(file_path, classifier).await
        } else {
            // Fallback to generic analysis
            Ok(FileContext {
                path: file_path.to_string_lossy().to_string(),
                language: "Unknown".to_string(),
                items: Vec::new(),
                complexity_metrics: None,
            })
        }
    }
}

impl Default for AstRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified AST analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstConfig {
    pub include_complexity: bool,
    pub include_functions: bool,
    pub include_types: bool,
    pub include_imports: bool,
    pub max_depth: Option<usize>,
}

impl Default for AstConfig {
    fn default() -> Self {
        Self {
            include_complexity: true,
            include_functions: true,
            include_types: true,
            include_imports: true,
            max_depth: None,
        }
    }
}

/// Result from AST analysis
#[derive(Debug, Clone)]
pub struct AstAnalysisResult {
    pub file_path: std::path::PathBuf,
    pub language: String,
    pub context: FileContext,
    pub analysis_duration_ms: u64,
}

/// High-level AST analyzer that uses the registry
pub struct UnifiedAstAnalyzer {
    registry: AstRegistry,
    classifier: FileClassifier,
}

impl UnifiedAstAnalyzer {
    pub fn new() -> Self {
        Self {
            registry: AstRegistry::new(),
            classifier: FileClassifier::default(),
        }
    }
    
    pub async fn analyze_file(&self, file_path: &Path) -> Result<AstAnalysisResult> {
        let start = std::time::Instant::now();
        
        let context = self.registry.analyze_file(file_path, &self.classifier).await?;
        let language = context.language.clone();
        
        let duration = start.elapsed();
        
        Ok(AstAnalysisResult {
            file_path: file_path.to_path_buf(),
            language,
            context,
            analysis_duration_ms: duration.as_millis() as u64,
        })
    }
    
    pub fn supported_languages(&self) -> Vec<&str> {
        self.registry.list_supported_extensions()
    }
}

impl Default for UnifiedAstAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_ast_registry_creation() {
        let registry = AstRegistry::new();
        let extensions = registry.list_supported_extensions();
        
        // Should always support Rust
        assert!(extensions.contains(&"rs"));
        assert!(!extensions.is_empty());
    }
    
    #[tokio::test]
    async fn test_rust_strategy() {
        let registry = AstRegistry::new();
        let rust_strategy = registry.get_strategy("rs").unwrap();
        
        assert_eq!(rust_strategy.primary_extension(), "rs");
        assert_eq!(rust_strategy.language_name(), "Rust");
        assert!(rust_strategy.can_handle("rs"));
    }
    
    #[tokio::test]
    async fn test_unified_analyzer() {
        let analyzer = UnifiedAstAnalyzer::new();
        let languages = analyzer.supported_languages();
        
        assert!(languages.contains(&"rs"));
        assert!(!languages.is_empty());
    }
    
    #[tokio::test]
    async fn test_file_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fn main() { println!(\"Hello, world!\"); }").unwrap();
        temp_file.flush().unwrap();
        
        // Change extension to .rs
        let rust_file_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &rust_file_path).unwrap();
        
        let analyzer = UnifiedAstAnalyzer::new();
        let result = analyzer.analyze_file(&rust_file_path).await.unwrap();
        
        assert_eq!(result.language, "Rust");
        assert!(result.analysis_duration_ms > 0);
        assert_eq!(result.file_path, rust_file_path);
        
        // Clean up
        std::fs::remove_file(&rust_file_path).unwrap();
    }
    
    #[test]
    fn test_ast_config_default() {
        let config = AstConfig::default();
        
        assert!(config.include_complexity);
        assert!(config.include_functions);
        assert!(config.include_types);
        assert!(config.include_imports);
        assert!(config.max_depth.is_none());
    }
}