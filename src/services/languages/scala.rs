#![cfg_attr(coverage_nightly, coverage(off))]
//! Scala Language Support for PMAT
//!
//! This module provides Scala-specific analysis capabilities using tree-sitter-scala parser
//! for AST extraction and complexity analysis aligned with Scala best practices.

#[cfg(feature = "scala-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "scala-ast")]
use std::path::{Path, PathBuf};

/// Scala AST visitor that extracts Scala-specific AST information
#[cfg(feature = "scala-ast")]
pub struct ScalaAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    class_count: usize,
    trait_count: usize,
    object_count: usize,
    case_class_count: usize,
}

/// Scala complexity analyzer for extracting Scala-specific metrics (complexity ≤10)
#[cfg(feature = "scala-ast")]
pub struct ScalaComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

// ScalaAstVisitor implementation (extraction, parsing, declarations)
include!("scala_visitor.rs");

// ScalaComplexityAnalyzer implementation (Default, complexity analysis)
include!("scala_complexity.rs");

// Tests (unit tests for visitor and complexity analyzer)
include!("scala_tests.rs");
