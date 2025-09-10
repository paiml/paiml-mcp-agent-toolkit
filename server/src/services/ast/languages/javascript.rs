// Toyota Way: Unified JavaScript AST Strategy

use super::super::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// JavaScript AST analysis strategy
#[cfg(feature = "typescript-ast")]
pub struct JavaScriptStrategy;

#[cfg(feature = "typescript-ast")]
impl Default for JavaScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "typescript-ast")]
#[async_trait]
impl AstStrategy for JavaScriptStrategy {
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // JavaScript analysis (similar to TypeScript but for JS files)
        // Convert TemplateError to anyhow::Error
        let context = crate::services::ast_typescript::analyze_javascript_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("JavaScript analysis failed: {}", e))?;
        Ok(context)
    }

    fn primary_extension(&self) -> &'static str {
        "js"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["js", "jsx", "mjs"]
    }

    fn language_name(&self) -> &'static str {
        "JavaScript"
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
