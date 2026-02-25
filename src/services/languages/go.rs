#![cfg_attr(coverage_nightly, coverage(off))]
//! Go Language Support for PMAT
//!
//! This module provides Go-specific analysis capabilities using tree-sitter-go parser
//! for AST extraction and complexity analysis aligned with Go best practices.

#[cfg(feature = "go-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "go-ast")]
use std::path::{Path, PathBuf};

/// Go AST visitor that extracts Go-specific AST information
#[cfg(feature = "go-ast")]
pub struct GoAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    _current_type: Vec<String>,
}

/// Go complexity analyzer for extracting Go-specific metrics (complexity ≤10)
#[cfg(feature = "go-ast")]
pub struct GoComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "go-ast")]
impl Default for GoComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// --- Implementation (split into include files) ---

include!("go_analysis.rs");
include!("go_tests.rs");
