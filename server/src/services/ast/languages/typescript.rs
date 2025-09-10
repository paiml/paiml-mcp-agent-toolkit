// Toyota Way: Unified TypeScript AST Strategy
//
// Consolidates functionality from ast_typescript.rs and ast_typescript_dispatch.rs

use super::super::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// TypeScript AST analysis strategy
#[cfg(feature = "typescript-ast")]
pub struct TypeScriptStrategy;

#[cfg(feature = "typescript-ast")]
impl Default for TypeScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "typescript-ast")]
#[async_trait]
impl AstStrategy for TypeScriptStrategy {
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Delegate to existing TypeScript analysis
        // Convert TemplateError to anyhow::Error
        let context = crate::services::ast_typescript::analyze_typescript_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("TypeScript analysis failed: {}", e))?;
        Ok(context)
    }

    fn primary_extension(&self) -> &'static str {
        "ts"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ts", "tsx"]
    }

    fn language_name(&self) -> &'static str {
        "TypeScript"
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn strategy_extensions_consistent() {
            let strategy = TypeScriptStrategy::new();
            let primary = strategy.primary_extension();
            let supported = strategy.supported_extensions();
            
            prop_assert!(supported.contains(&primary));
            prop_assert!(!supported.is_empty());
        }

        #[test]
        fn language_name_not_empty() {
            let strategy = TypeScriptStrategy::new();
            prop_assert!(!strategy.language_name().is_empty());
        }
    }
}
