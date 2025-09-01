// Toyota Way: Unified C AST Strategy

use super::super::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// C AST analysis strategy using tree-sitter
#[cfg(feature = "c-ast")]
pub struct CStrategy;

#[cfg(feature = "c-ast")]
impl CStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "c-ast")]
#[async_trait]
impl AstStrategy for CStrategy {
    async fn analyze(
        &self,
        file_path: &Path,
        _classifier: &FileClassifier,
    ) -> Result<FileContext> {
        // Create basic context for C files
        // TODO: Integrate with actual C AST parsing when available
        let context = FileContext {
            path: file_path.to_string_lossy().to_string(),
            language: "C".to_string(),
            items: Vec::new(),
            complexity_metrics: None,
        };
        Ok(context)
    }
    
    fn primary_extension(&self) -> &'static str {
        "c"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["c", "h"]
    }
    
    fn language_name(&self) -> &'static str {
        "C"
    }
}