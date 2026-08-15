//! Kotlin AST Strategy Implementation
//!
//! This module provides the Kotlin language strategy implementation for the unified
//! AST framework, following the same pattern as C/C++ strategies.

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

use crate::services::ast::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use crate::services::languages::kotlin::KotlinAstVisitor;

/// Kotlin AST Strategy for unified analysis
pub struct KotlinStrategy;

impl KotlinStrategy {
    /// Creates a new Kotlin language strategy
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AstStrategy for KotlinStrategy {
    /// Analyzes a Kotlin file and returns a FileContext with AST information
    async fn analyze(&self, path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        use tokio::fs;

        // Read file content
        let content = fs::read_to_string(path).await?;

        // Use KotlinAstVisitor to analyze the Kotlin file
        let visitor = KotlinAstVisitor::new(path);
        match visitor.analyze_kotlin_source(&content) {
            Ok(items) => {
                // Create a FileContext with the extracted items
                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "kotlin".to_string(),
                    items,
                    complexity_metrics: None, // Complexity metrics could be added here
                })
            }
            Err(e) => {
                // Log error and return empty context
                tracing::warn!("Failed to parse Kotlin file {}: {}", path.display(), e);
                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "kotlin".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }

    fn primary_extension(&self) -> &'static str {
        "kt"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["kt", "kts"]
    }

    fn language_name(&self) -> &'static str {
        "Kotlin"
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kotlin_strategy_supports_extension() {
        let strategy = KotlinStrategy;

        assert!(strategy.can_handle("kt"), "Should support .kt files");
        assert!(strategy.can_handle("kts"), "Should support .kts files");
        assert!(
            !strategy.can_handle("java"),
            "Should not support .java files"
        );
        assert!(!strategy.can_handle("py"), "Should not support .py files");
    }

    // Add more tests as needed for the Kotlin strategy
}
