#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified TypeScript/JavaScript Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing TypeScript/JavaScript files twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x parse calls per file (AST + Complexity)
//! After: 1x parse call per file
//! Expected gain: 40-50% reduction in parse time
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use pmat::services::unified_typescript_analyzer::UnifiedTypeScriptAnalyzer;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let analyzer = UnifiedTypeScriptAnalyzer::new(PathBuf::from("src/main.ts"));
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

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::AstItem;
use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;

#[cfg(feature = "typescript-ast")]
use swc_common::{sync::Lrc, FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_ast::Module;
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

/// Unified analyzer that parses TypeScript/JavaScript once, extracts twice
pub struct UnifiedTypeScriptAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, classes, interfaces, etc.)
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

    #[error("Failed to parse TypeScript/JavaScript syntax: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

// --- Implementation split into include files ---
include!("unified_typescript_analyzer_impl.rs");
include!("unified_typescript_analyzer_test_suite.rs");
