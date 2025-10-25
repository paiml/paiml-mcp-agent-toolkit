//! JavaScript language support module
//!
//! This module provides support for analyzing JavaScript code, including
//! AST parsing, syntax analysis, and code structure extraction.

use anyhow::Result;
use std::path::Path;
use crate::services::context::AstItem;

/// Visitor for JavaScript AST analysis
pub struct JavaScriptAstVisitor {
    path: std::path::PathBuf,
}

impl JavaScriptAstVisitor {
    /// Create a new JavaScript AST visitor
    pub fn new(path: &Path) -> Self {
        Self { path: path.to_path_buf() }
    }

    /// Analyze JavaScript source code
    #[cfg(feature = "typescript-ast")]
    pub fn analyze_javascript_source(&self, source: &str) -> Result<Vec<AstItem>> {
        // Use the TypeScript strategy which also handles JavaScript
        use crate::services::ast::languages::typescript::TypeScriptStrategy;
        use crate::services::ast::strategy::AstStrategy;

        let strategy = TypeScriptStrategy::new();
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            strategy.parse_file(&self.path, source).await
        })
    }

    /// Analyze JavaScript source code (feature not enabled)
    #[cfg(not(feature = "typescript-ast"))]
    pub fn analyze_javascript_source(&self, _source: &str) -> Result<Vec<AstItem>> {
        // Return empty result when TypeScript AST feature is not enabled
        Ok(Vec::new())
    }
}