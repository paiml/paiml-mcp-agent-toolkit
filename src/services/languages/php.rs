#![cfg_attr(coverage_nightly, coverage(off))]
//! PHP Script Analysis Support for PMAT
//!
//! This module provides PHP-specific analysis capabilities using lexical analysis
//! and partial AST extraction for PHP scripts within static analysis constraints.
//!
//! # Module Organization
//! - `php_analysis.rs`: PhpScriptAnalyzer impl (extraction and analysis methods)
//! - `php_complexity.rs`: PhpComplexityAnalyzer impl (complexity metrics)
//! - `php_tests.rs`: Unit tests

#[cfg(feature = "php-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};

/// PHP script analyzer that extracts PHP-specific information
pub struct PhpScriptAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    script_name: String,
    function_count: usize,
    class_count: usize,
    method_count: usize,
}

/// PHP complexity analyzer for PHP-specific metrics
pub struct PhpComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

// PhpScriptAnalyzer implementation: new(), analyze, extract_*, get_qualified_name
include!("php_analysis.rs");

// PhpComplexityAnalyzer implementation: Default, new(), analyze_complexity
include!("php_complexity.rs");

// Unit tests
include!("php_tests.rs");
