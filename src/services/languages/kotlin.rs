#![cfg_attr(coverage_nightly, coverage(off))]
//! Enhanced Kotlin Language Support for PMAT
//!
//! This module provides enhanced Kotlin-specific analysis capabilities using tree-sitter-kotlin parser
//! for AST extraction and complexity analysis with focus on coroutines and Kotlin-specific features.

#[cfg(feature = "kotlin-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "kotlin-ast")]
use std::path::{Path, PathBuf};

/// Enhanced Kotlin AST visitor that extracts Kotlin-specific AST information
#[cfg(feature = "kotlin-ast")]
pub struct KotlinAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    package_name: String,
    class_count: usize,
    coroutine_count: usize,
}

/// Kotlin complexity analyzer with enhanced coroutine analysis (complexity ≤10)
#[cfg(feature = "kotlin-ast")]
pub struct KotlinComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    coroutine_complexity: u32,
}

// KotlinAstVisitor implementation (extraction, parsing, declarations)
include!("kotlin_visitor.rs");

// KotlinComplexityAnalyzer implementation (Default, complexity analysis)
include!("kotlin_complexity.rs");

/// Analyze a Kotlin file and return AST items
#[cfg(feature = "kotlin-ast")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_kotlin_file(file_path: &Path) -> anyhow::Result<Vec<AstItem>> {
    debug_assert!(
        file_path.exists(),
        "file_path must exist: {}",
        file_path.display()
    );
    let source = std::fs::read_to_string(file_path)?;
    let visitor = KotlinAstVisitor::new(file_path);
    visitor
        .analyze_kotlin_source(&source)
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Stub when kotlin-ast feature is disabled
#[cfg(not(feature = "kotlin-ast"))]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_kotlin_file(
    _file_path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::context::AstItem>> {
    debug_assert!(
        _file_path.exists(),
        "_file_path must exist: {}",
        _file_path.display()
    );
    Ok(Vec::new())
}

// Tests (unit tests and property-based tests)
include!("kotlin_tests.rs");
