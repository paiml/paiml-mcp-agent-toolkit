// Toyota Way: Unified Rust AST Strategy
//
// Consolidates functionality from ast_rust.rs and related files

use super::super::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Rust AST analysis strategy using syn parser
pub struct RustStrategy;

impl RustStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AstStrategy for RustStrategy {
    async fn analyze(
        &self,
        file_path: &Path,
        _classifier: &FileClassifier,
    ) -> Result<FileContext> {
        // Delegate to the existing ast_rust functionality that returns FileContext
        // Convert TemplateError to anyhow::Error
        let context = crate::services::ast_rust::analyze_rust_file(file_path).await
            .map_err(|e| anyhow::anyhow!("Rust analysis failed: {}", e))?;
        Ok(context)
    }
    
    fn primary_extension(&self) -> &'static str {
        "rs"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["rs"]
    }
    
    fn language_name(&self) -> &'static str {
        "Rust"
    }
}