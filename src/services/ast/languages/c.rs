#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! C Language Support for PMAT
//!
//! This module provides C-specific analysis capabilities using tree-sitter-c parser
//! for AST extraction and complexity analysis aligned with C best practices.

#[cfg(feature = "c-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "c-ast")]
use std::path::{Path, PathBuf};

/// C AST visitor that extracts C-specific AST information
#[cfg(feature = "c-ast")]
pub struct CAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,

    current_scope: Vec<String>,

    is_header: bool,
}

// CAstVisitor: constructor, source analysis, and all extraction/helper methods
include!("c_visitor.rs");

/// C complexity analyzer for extracting C-specific metrics (complexity ≤10)
#[cfg(feature = "c-ast")]
pub struct CComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "c-ast")]
impl Default for CComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// CComplexityAnalyzer implementation and analyze_c_file async function
include!("c_complexity.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[cfg(feature = "c-ast")]
    use super::*;
    #[cfg(feature = "c-ast")]
    use std::path::Path;

    // Test fixtures and unit tests for CAstVisitor and CComplexityAnalyzer
    include!("c_tests.rs");
}
