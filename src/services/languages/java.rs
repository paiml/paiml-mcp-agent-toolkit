#![cfg_attr(coverage_nightly, coverage(off))]
//! Java Language Support for PMAT
//!
//! This module provides Java-specific analysis capabilities using tree-sitter-java parser
//! for AST extraction and complexity analysis aligned with Java best practices.
//!
//! # Module Structure
//!
//! The implementation is split across include files for maintainability:
//! - `java_visitor.rs` - JavaAstVisitor impl (extraction logic)
//! - `java_complexity.rs` - JavaComplexityAnalyzer impl
//! - `java_tests.rs` - Unit tests and property tests

#[cfg(feature = "java-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "java-ast")]
use std::path::{Path, PathBuf};

/// Java AST visitor that extracts Java-specific AST information
#[cfg(feature = "java-ast")]
pub struct JavaAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    class_count: usize,
}

/// Java complexity analyzer for extracting Java-specific metrics (complexity ≤10)
#[cfg(feature = "java-ast")]
pub struct JavaComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

// --- Implementation split across include files ---

#[cfg(feature = "java-ast")]
include!("java_visitor.rs");

#[cfg(feature = "java-ast")]
include!("java_complexity.rs");

include!("java_tests.rs");
