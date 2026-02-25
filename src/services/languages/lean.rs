#![cfg_attr(coverage_nightly, coverage(off))]
//! Lean 4 Language Support for PMAT
//!
//! This module provides Lean 4-specific analysis capabilities using pattern-based parsing
//! for AST extraction and proof quality metrics.
//!
//! Lean 4 constructs: def, theorem, lemma, structure, class, inductive, abbrev,
//! axiom, opaque, instance, namespace.

#[cfg(feature = "lean-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "lean-ast")]
use std::path::{Path, PathBuf};

/// Lean 4 AST visitor that extracts Lean-specific AST information
#[cfg(feature = "lean-ast")]
pub struct LeanAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    namespace: String,
}

/// Lean complexity analyzer for extracting Lean-specific metrics (complexity <=10)
#[cfg(feature = "lean-ast")]
pub struct LeanComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

// LeanAstVisitor impl: AST extraction methods for definitions, theorems, types, instances, axioms
#[cfg(feature = "lean-ast")]
include!("lean_visitor.rs");

// Analysis functions: sorry counting, theorem counting, complexity analysis, file analysis
#[cfg(feature = "lean-ast")]
include!("lean_analysis.rs");

// Unit tests and property-based tests for Lean language support
include!("lean_tests.rs");
