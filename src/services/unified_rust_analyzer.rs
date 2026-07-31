#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified Rust Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing Rust files twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x `syn::parse_file()` calls per file
//! After: 1x `syn::parse_file()` call per file
//! Expected gain: 40-50% reduction in parse time
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use pmat::services::unified_rust_analyzer::UnifiedRustAnalyzer;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let analyzer = UnifiedRustAnalyzer::new(PathBuf::from("src/main.rs"));
//!     let result = analyzer.analyze().await?;
//!
//!     println!("Found {} AST items", result.ast_items.len());
//!     println!("Analyzed {} functions", result.file_metrics.functions.len());
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::services::accurate_complexity_analyzer::measure_block;
use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::AstItem;
use crate::services::enhanced_ast_visitor::EnhancedAstVisitor;
use crate::services::source_line_index::{FunctionSpans, LineSpan};

/// Unified analyzer that parses once, extracts twice
pub struct UnifiedRustAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, structs, enums, traits)
    pub ast_items: Vec<AstItem>,

    /// File-level complexity metrics
    pub file_metrics: FileComplexityMetrics,

    /// Parse timestamp (for cache validation)
    pub parsed_at: std::time::Instant,
}

/// Error type for unified analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse Rust syntax: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

// --- Submodule includes ---

// Core implementation: new(), file_path(), analyze(), parse_count(), extract_ast_items()
include!("unified_rust_analyzer_core.rs");

// Complexity extraction: extract_complexity_metrics() with SimpleComplexityVisitor
include!("unified_rust_analyzer_complexity.rs");

// Unit tests and property tests
include!("unified_rust_analyzer_unit_tests.rs");
