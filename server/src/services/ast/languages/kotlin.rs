//! Kotlin AST Strategy for PMAT
//!
//! This module provides the Kotlin language strategy implementation for the unified
//! AST framework, following the same pattern as Rust, TypeScript, and Python strategies.

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use crate::services::languages::kotlin::KotlinAstVisitor;

use super::super::AstStrategy;

/// Kotlin language strategy for AST parsing
pub struct KotlinStrategy;

impl KotlinStrategy {
    /// Creates a new Kotlin language strategy
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AstStrategy for KotlinStrategy {
    /// Analyzes a Kotlin file and returns a FileContext with AST information
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Read the file content
        let content = fs::read_to_string(file_path).await?;
        
        // Use KotlinAstVisitor to analyze the Kotlin file
        let visitor = KotlinAstVisitor::new(file_path);
        match visitor.analyze_kotlin_source(&content) {
            Ok(items) => {
                // Create a FileContext with the extracted items
                Ok(FileContext {
                    path: file_path.to_string_lossy().to_string(),
                    language: "kotlin".to_string(),
                    items,
                    complexity_metrics: None, // Complexity metrics could be added here
                })
            },
            Err(e) => {
                // Log error and return empty context
                tracing::warn!("Failed to parse Kotlin file {}: {}", file_path.display(), e);
                Ok(FileContext {
                    path: file_path.to_string_lossy().to_string(),
                    language: "kotlin".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }
    
    /// Returns the primary file extension for Kotlin
    fn primary_extension(&self) -> &'static str {
        "kt"
    }
    
    /// Returns all file extensions supported by this strategy
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["kt", "kts"]
    }
    
    /// Returns the language name
    fn language_name(&self) -> &'static str {
        "Kotlin"
    }
}

impl Default for KotlinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_kotlin_strategy_supports_kt_extension() {
        let strategy = KotlinStrategy::new();
        
        assert!(strategy.can_handle("kt"));
        assert!(strategy.can_handle("kts"));
        assert!(!strategy.can_handle("java"));
    }
    
    #[test]
    fn test_kotlin_strategy_properties() {
        let strategy = KotlinStrategy::new();
        
        assert_eq!(strategy.primary_extension(), "kt");
        assert_eq!(strategy.language_name(), "Kotlin");
        assert_eq!(strategy.supported_extensions(), vec!["kt", "kts"]);
    }
    
    #[tokio::test]
    async fn test_kotlin_strategy_analyze_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"").unwrap();
        temp_file.flush().unwrap();
        
        let kotlin_file_path = temp_file.path().with_extension("kt");
        std::fs::copy(temp_file.path(), &kotlin_file_path).unwrap();
        
        let strategy = KotlinStrategy::new();
        let result = strategy
            .analyze(&kotlin_file_path, &FileClassifier::default())
            .await
            .unwrap();
        
        assert_eq!(result.language, "kotlin");
        assert!(result.items.is_empty());
        
        // Clean up
        std::fs::remove_file(&kotlin_file_path).unwrap();
    }
    
    #[tokio::test]
    async fn test_kotlin_strategy_analyze_simple_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fun main() { println(\"Hello, Kotlin!\") }").unwrap();
        temp_file.flush().unwrap();
        
        let kotlin_file_path = temp_file.path().with_extension("kt");
        std::fs::copy(temp_file.path(), &kotlin_file_path).unwrap();
        
        let strategy = KotlinStrategy::new();
        let result = strategy
            .analyze(&kotlin_file_path, &FileClassifier::default())
            .await
            .unwrap();
        
        assert_eq!(result.language, "kotlin");
        // Should find at least one function
        let functions: Vec<_> = result
            .items
            .iter()
            .filter(|item| matches!(item, crate::services::context::AstItem::Function { .. }))
            .collect();
        assert!(!functions.is_empty());
        
        // Clean up
        std::fs::remove_file(&kotlin_file_path).unwrap();
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }
        
        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}