#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified Python Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing Python files twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x parse calls per file (AST + Complexity)
//! After: 1x parse call per file
//! Expected gain: 40-50% reduction in parse time

use anyhow::Result;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::AstItem;

// Modern tree-sitter-python parsing (replaces rustpython-parser)
#[cfg(feature = "python-ast")]
use tree_sitter::{Parser as TsParser, Tree};

/// Unified analyzer that parses Python once, extracts twice
pub struct UnifiedPythonAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, classes, methods)
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

    #[error("Failed to parse Python syntax: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl UnifiedPythonAnalyzer {
    /// Create new analyzer for a file
    pub fn new(file_path: PathBuf) -> Self {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        Self {
            file_path,
            #[cfg(test)]
            parse_count: AtomicUsize::new(0),
        }
    }

    /// Get the file path being analyzed
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }
}

// Analysis entry point and complexity extraction
include!("unified_python_analyzer_analysis.rs");

// Tree-sitter parsing methods
include!("unified_python_analyzer_parser.rs");

// AST visitor for extracting items from parsed tree
include!("unified_python_analyzer_visitor.rs");

// Tests
include!("unified_python_analyzer_tests.rs");
