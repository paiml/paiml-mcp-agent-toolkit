//! C AST analysis - MIGRATION IN PROGRESS
//!
//! This module is being migrated to the new unified AST architecture.
//! It now acts as a facade, delegating to the compatibility layer.
//!
//! Migration status: Using compatibility shim
//! Target: server/src/ast/languages/c.rs

use crate::models::unified_ast::AstDag;
use anyhow::Result;
use std::path::Path;

// Re-export compatibility functions
pub use super::ast_c_compat::{
    analyze_c_file, analyze_c_file_with_classifier, analyze_c_file_with_complexity,
    analyze_c_file_with_complexity_and_classifier,
};

// Dispatch parser removed - functionality moved to new AST module

// Legacy compatibility types (may be referenced by other modules)
/// C ast parser.
pub struct CAstParser {}

impl Default for CAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CAstParser {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {}
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Parse file.
    pub fn parse_file(&mut self, _path: &Path, _content: &str) -> Result<AstDag> {
        // Placeholder - use new AST module for C parsing
        Err(anyhow::anyhow!(
            "C AST parsing has been moved to the new AST module"
        ))
    }
}

// Keep any other types that were exported
// (The rest of the original implementation is preserved in ast_c_compat.rs)
