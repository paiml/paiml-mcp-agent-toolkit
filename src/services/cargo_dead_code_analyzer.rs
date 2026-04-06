//! Accurate dead code analyzer using cargo/rustc integration
//!
//! This module provides accurate dead code detection by leveraging
//! the Rust compiler's built-in dead code analysis, replacing the
//! previous heuristic-based approach that produced false positives.
//!
//! ## Performance (CB-128 O(1) Caching)
//!
//! Uses git tree-hash for O(1) cache invalidation:
//! - Cache hit: ~5ms (read JSON from .pmat/dead-code-cache/)
//! - Cache miss: ~30-60s (full cargo check with -W dead_code)
//!
//! Cache is invalidated when:
//! - Git tree hash changes (code modified)
//! - PMAT version changes
//! - Cache file is missing or corrupted

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Cached dead code result with metadata for O(1) invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDeadCodeResult {
    /// Git tree hash when this cache was computed
    pub tree_hash: String,
    /// PMAT version that computed this cache
    pub pmat_version: String,
    /// Timestamp of cache computation
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The actual dead code report
    pub report: AccurateDeadCodeReport,
}

/// Dead code analysis result with accurate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccurateDeadCodeReport {
    /// Files with dead code
    pub files_with_dead_code: Vec<FileDeadCode>,
    /// Total dead code items
    pub total_dead_items: usize,
    /// Accurate dead code percentage
    pub dead_code_percentage: f64,
    /// Total lines analyzed
    pub total_lines: usize,
    /// Dead lines count
    pub dead_lines: usize,
    /// Summary by type
    pub dead_by_type: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeadCode {
    pub file_path: PathBuf,
    pub dead_items: Vec<DeadItem>,
    pub file_dead_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadItem {
    pub name: String,
    pub kind: DeadCodeKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeadCodeKind {
    Function,
    Method,
    Struct,
    Enum,
    Variant,
    Field,
    Constant,
    Static,
    Module,
    Trait,
    TypeAlias,
    /// Layer 1: Code explicitly marked with #[allow(dead_code)]
    /// This is an admission that the code is unused
    Suppressed,
    Other(String),
}

/// Cargo-based dead code analyzer for accurate detection with O(1) caching
pub struct CargoDeadCodeAnalyzer {
    project_path: PathBuf,
    exclude_tests: bool,
    exclude_examples: bool,
    exclude_benches: bool,
    max_depth: usize,
    /// Enable caching (default: true)
    use_cache: bool,
    /// Force cache refresh even if valid
    force_refresh: bool,
}

impl CargoDeadCodeAnalyzer {
    /// Create a new analyzer for the given project path
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            exclude_tests: true,
            exclude_examples: true,
            exclude_benches: true,
            max_depth: 8,
            use_cache: true,
            force_refresh: false,
        }
    }

    /// Include test code in analysis
    #[must_use]
    pub fn include_tests(mut self) -> Self {
        self.exclude_tests = false;
        self
    }

    /// Include example code in analysis
    #[must_use]
    pub fn include_examples(mut self) -> Self {
        self.exclude_examples = false;
        self
    }

    /// Include benchmark code in analysis
    #[must_use]
    pub fn include_benches(mut self) -> Self {
        self.exclude_benches = false;
        self
    }

    /// Set maximum directory traversal depth
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        debug_assert!(max_depth > 0, "max_depth must be positive");
        self.max_depth = max_depth;
        self
    }

    /// Disable caching (force fresh analysis every time)
    #[must_use]
    pub fn without_cache(mut self) -> Self {
        self.use_cache = false;
        self
    }

    /// Force cache refresh even if cache is valid
    #[must_use]
    pub fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
        self
    }
}

/// Public API for backward compatibility
pub async fn analyze_dead_code(project_path: impl AsRef<Path>) -> Result<AccurateDeadCodeReport> {
    let analyzer = CargoDeadCodeAnalyzer::new(project_path);
    analyzer.analyze().await
}

// Include implementation files
include!("cargo_dead_code_analyzer/cache_operations.rs");
include!("cargo_dead_code_analyzer/analysis.rs");
include!("cargo_dead_code_analyzer/parsing.rs");
include!("cargo_dead_code_analyzer/tests.rs");
