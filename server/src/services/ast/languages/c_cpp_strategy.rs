// Toyota Way: C/C++ Strategy Implementation for AST Service Layer
// 
// This adapter module implements the services::ast::AstStrategy trait
// for the ast::languages::c_cpp::CStrategy and CppStrategy types.

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

// Re-export these types to make them available to consumers of this module
pub use crate::ast::languages::c_cpp::{CStrategy, CppStrategy};
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;

// Import service-layer AST Strategy trait
use crate::services::ast::AstStrategy;

// Import our own C/C++ analyzers
use super::c;
use super::cpp;

#[async_trait]
impl AstStrategy for CStrategy {
    /// Analyze a C source file and extract AST information
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Use our comprehensive C analyzer
        let file_context = c::analyze_c_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("C analysis error: {}", e))?;
            
        Ok(file_context)
    }

    /// Get the primary file extension this strategy handles
    fn primary_extension(&self) -> &'static str {
        "c"
    }

    /// Get all file extensions this strategy can handle
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["c", "h"]
    }

    /// Get the language name
    fn language_name(&self) -> &'static str {
        "C"
    }
}

#[async_trait]
impl AstStrategy for CppStrategy {
    /// Analyze a C++ source file and extract AST information
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Use our comprehensive C++ analyzer
        let file_context = cpp::analyze_cpp_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("C++ analysis error: {}", e))?;
            
        Ok(file_context)
    }

    /// Get the primary file extension this strategy handles
    fn primary_extension(&self) -> &'static str {
        "cpp"
    }

    /// Get all file extensions this strategy can handle
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["cpp", "cc", "cxx", "hpp", "hxx", "hh"]
    }

    /// Get the language name
    fn language_name(&self) -> &'static str {
        "C++"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_strategy_supported_extensions() {
        let strategy = CStrategy::new();
        assert_eq!(strategy.primary_extension(), "c");
        assert!(strategy.supported_extensions().contains(&"c"));
        assert!(strategy.supported_extensions().contains(&"h"));
        assert_eq!(strategy.language_name(), "C");
    }

    #[test]
    fn test_cpp_strategy_supported_extensions() {
        let strategy = CppStrategy::new();
        assert_eq!(strategy.primary_extension(), "cpp");
        assert!(strategy.supported_extensions().contains(&"cpp"));
        assert!(strategy.supported_extensions().contains(&"hpp"));
        assert_eq!(strategy.language_name(), "C++");
    }

    #[test]
    fn test_c_strategy_can_handle() {
        let strategy = CStrategy::new();
        assert!(strategy.can_handle("c"));
        assert!(strategy.can_handle("h"));
        assert!(!strategy.can_handle("cpp"));
    }

    #[test]
    fn test_cpp_strategy_can_handle() {
        let strategy = CppStrategy::new();
        assert!(strategy.can_handle("cpp"));
        assert!(strategy.can_handle("hpp"));
        assert!(strategy.can_handle("cxx"));
        assert!(!strategy.can_handle("c"));
    }
}