//! TypeScript language support module
//!
//! This module provides support for analyzing TypeScript code, including
//! AST parsing, syntax analysis, and code structure extraction.

use anyhow::Result;
use std::path::Path;
use crate::services::context::AstItem;

/// Visitor for TypeScript AST analysis
pub struct TypeScriptAstVisitor {
    path: std::path::PathBuf,
}

impl TypeScriptAstVisitor {
    /// Create a new TypeScript AST visitor
    pub fn new(path: &Path) -> Self {
        Self { path: path.to_path_buf() }
    }

    /// Analyze TypeScript source code
    #[cfg(feature = "typescript-ast")]
    pub fn analyze_typescript_source(&self, source: &str) -> Result<Vec<AstItem>> {
        // Use the TypeScript strategy to parse the source
        use crate::services::ast::languages::typescript::TypeScriptStrategy;
        use crate::services::ast::strategy::AstStrategy;

        let strategy = TypeScriptStrategy::new();
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            strategy.parse_file(&self.path, source).await
        })
    }

    /// Analyze TypeScript source code (feature not enabled)
    #[cfg(not(feature = "typescript-ast"))]
    pub fn analyze_typescript_source(&self, _source: &str) -> Result<Vec<AstItem>> {
        // Return empty result when TypeScript AST feature is not enabled
        Ok(Vec::new())
    }
}