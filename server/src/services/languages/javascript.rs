//! JavaScript language support module
//!
//! This module provides support for analyzing JavaScript code, including
//! AST parsing, syntax analysis, and code structure extraction.

use anyhow::Result;
use std::path::Path;
use crate::services::context::AstItem;

#[cfg(feature = "typescript-ast")]
use crate::services::ast_typescript::analyze_typescript_file;

/// Visitor for JavaScript AST analysis
pub struct JavaScriptAstVisitor {
    #[allow(dead_code)]
    path: std::path::PathBuf,
}

impl JavaScriptAstVisitor {
    /// Create a new JavaScript AST visitor
    pub fn new(path: &Path) -> Self {
        Self { path: path.to_path_buf() }
    }

    /// Analyze JavaScript source code
    ///
    /// This method parses JavaScript source code and extracts AST items.
    /// It creates a temporary file to leverage the existing TypeScript parser
    /// (which also handles JavaScript).
    #[cfg(feature = "typescript-ast")]
    pub fn analyze_javascript_source(&self, source: &str) -> Result<Vec<AstItem>> {
        // Create temporary file with .js extension (builder pattern)
        let temp_file = tempfile::Builder::new()
            .suffix(".js")
            .tempfile()
            .map_err(|e| anyhow::anyhow!("Failed to create temp file: {}", e))?;

        // Write source code to temporary file
        std::fs::write(temp_file.path(), source.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write source to temp file: {}", e))?;

        // Use existing TypeScript parser (handles both TS and JS)
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;

        runtime.block_on(async {
            let context = analyze_typescript_file(temp_file.path()).await
                .map_err(|e| anyhow::anyhow!("JavaScript parsing failed: {}", e))?;
            Ok(context.items)
        })
    }

    /// Analyze JavaScript source code (feature not enabled)
    #[cfg(not(feature = "typescript-ast"))]
    pub fn analyze_javascript_source(&self, _source: &str) -> Result<Vec<AstItem>> {
        // Return empty result when TypeScript AST feature is not enabled
        Ok(Vec::new())
    }
}