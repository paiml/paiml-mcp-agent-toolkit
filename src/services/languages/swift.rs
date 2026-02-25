#![cfg_attr(coverage_nightly, coverage(off))]
//! Swift Source File Analysis Support for PMAT
//!
//! This module provides Swift-specific analysis capabilities using lexical analysis
//! and partial AST extraction for Swift files within static analysis constraints.

#[cfg(feature = "swift-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};

/// Swift source analyzer that extracts Swift-specific information
pub struct SwiftSourceAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    source_name: String,
    function_count: usize,
    class_count: usize,
    method_count: usize,
}

/// Swift complexity analyzer for Swift-specific metrics (complexity ≤10)
pub struct SwiftComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

impl Default for SwiftComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// SwiftSourceAnalyzer implementation: constructor, analysis, and extraction methods
include!("swift_analysis.rs");

// SwiftComplexityAnalyzer implementation: constructor and complexity analysis
include!("swift_complexity.rs");

// Unit tests for Swift analysis and complexity
#[cfg_attr(coverage_nightly, coverage(off))]
include!("swift_tests.rs");
