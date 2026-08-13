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

/// Shape of `AccurateDeadCodeReport` this build can read out of a cache entry.
///
/// The cache was keyed on (tree hash, pmat version) alone, so a cache written
/// by an EARLIER BUILD OF THE SAME VERSION was accepted whole. When
/// `FileDeadCode::unreachable_items` was added, those entries deserialised it as
/// empty via `#[serde(default)]` and `--include-unreachable` reported nothing at
/// all on any tree with a warm cache — the flag would have looked inert again,
/// for a reason nothing in the report disclosed. Bump this whenever the cached
/// report gains or changes a field.
/// 3: `DeadCodeKind::Suppressed` was removed. A cache entry written by an
/// earlier build carries `"Suppressed"` kinds, which this build cannot
/// deserialise — and if it could, it would restore exactly the erased-kind
/// report (`dead_functions: 0` over six dead functions) that removing the
/// variant fixed.
pub const DEAD_CODE_CACHE_SCHEMA: u32 = 3;

/// Cached dead code result with metadata for O(1) invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDeadCodeResult {
    /// Shape of the cached report; entries that do not match this build's
    /// `DEAD_CODE_CACHE_SCHEMA` are a miss. `0` for entries written before the
    /// field existed.
    #[serde(default)]
    pub report_schema: u32,
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
    /// Source files actually scanned: `.rs` files walked, minus ignored/hidden
    /// dirs and minus the trees this run was configured to skip.
    #[serde(default)]
    pub total_files: usize,
    /// Every `.rs` file the walk saw, including the test/example/bench trees
    /// `total_files` leaves out. The two differ exactly when the scan was
    /// narrowed, and that difference is the only record of the narrowing a
    /// consumer gets -- without it a default run reports all zeros over files
    /// it never opened.
    #[serde(default)]
    pub project_files: usize,
    /// Dead lines count
    pub dead_lines: usize,
    /// Summary by type
    pub dead_by_type: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// File dead code.
pub struct FileDeadCode {
    pub file_path: PathBuf,
    pub dead_items: Vec<DeadItem>,
    /// Statements rustc reported as `unreachable_code`, kept OUT of
    /// `dead_items`.
    ///
    /// They are a different finding from "never used": unreachable code is
    /// reachable-from-nowhere code inside a live item. Mixing them into
    /// `dead_items` would move `total_dead_items`, `dead_by_type`,
    /// `file_dead_percentage` and every estimated line count, so a default run
    /// would change; they are carried alongside and surfaced only by
    /// `analyze dead-code --include-unreachable`.
    ///
    /// The flag reached `DeadCodeAnalysisConfig` and stopped there, and the one
    /// site that honours it (`analysis_ranking.rs`) is reachable only from the
    /// MCP tool — so the CLI printed "Unreachable blocks: 0" for a file with
    /// four statements after a `return` while the MCP equivalent was live.
    #[serde(default)]
    pub unreachable_items: Vec<DeadItem>,
    pub file_dead_percentage: f64,
    /// Physical lines in the file, counted while computing
    /// `file_dead_percentage`. `None` when the file could not be read.
    ///
    /// The consumer used to substitute the literal `100` for every file
    /// (`FileDeadCodeMetrics.total_lines`), so a 1287-line file and a 370-line
    /// file both reported `total_lines: 100` next to a `dead_percentage`
    /// computed from the real count — 24 dead lines "of 100" printed as 6.49%.
    /// The count is measured here, once, and carried instead of re-invented.
    #[serde(default)]
    pub total_lines: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Dead item.
pub struct DeadItem {
    pub name: String,
    pub kind: DeadCodeKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Dead code kind.
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
    // There is deliberately no `Suppressed` variant. Layer 1 (the scan for
    // explicit `allow(dead_code)` admissions) used to tag every item it found
    // with one, which erased the item's real kind: "suppressed" is a category
    // `count_dead_items_by_kind` and `dead_by_type` know nothing about, so a
    // suppressed function could not increment `dead_functions`, could not be
    // typed `function` in the report, and was billed the 2-line "anything else"
    // estimate instead of a function's 5. How an item was DISCOVERED is
    // provenance and belongs in `DeadItem::message`; WHAT it is belongs in
    // `kind`, and one must not overwrite the other.
    /// rustc's `unreachable_code` lint: a statement that can never execute.
    ///
    /// Only ever stored in `FileDeadCode::unreachable_items`, never in
    /// `dead_items`, so no existing counter can see it.
    UnreachableCode,
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
    /// How long the analysis may run before the `cargo check` child is killed.
    ///
    /// This was a hardcoded `Duration::from_secs(90)` inside `analyze()`, so
    /// `analyze dead-code --timeout 300` was silently capped at 90 even once the
    /// timer worked. The caller that has a user-facing budget owns it; 90s
    /// remains the default for callers that do not.
    timeout: std::time::Duration,
}

impl CargoDeadCodeAnalyzer {
    /// Create a new analyzer for the given project path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            exclude_tests: true,
            // `--include-tests` is the only scope flag the CLI (and MCP) ever
            // exposes, so defaulting these two to `true` did not narrow the
            // default report — it made `examples/` and `benches/` unreachable
            // from EVERY invocation, with no flag able to put them back and
            // `--include 'examples/**'` returning an empty list because the
            // glob ran over a set the tree had already been cut from. They are
            // ordinary first-party source that ships with the crate, so they
            // stay in scope; only the test tree is gated.
            exclude_examples: false,
            exclude_benches: false,
            max_depth: 8,
            use_cache: true,
            force_refresh: false,
            timeout: std::time::Duration::from_secs(90),
        }
    }

    /// Set how long the analysis may run before `cargo check` is killed.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Include test code in analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn include_tests(mut self) -> Self {
        self.exclude_tests = false;
        self
    }

    /// Include example code in analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn include_examples(mut self) -> Self {
        self.exclude_examples = false;
        self
    }

    /// Include benchmark code in analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn include_benches(mut self) -> Self {
        self.exclude_benches = false;
        self
    }

    /// Set maximum directory traversal depth
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Disable caching (force fresh analysis every time)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn without_cache(mut self) -> Self {
        self.use_cache = false;
        self
    }

    /// Force cache refresh even if cache is valid
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
        self
    }
}

/// Public API for backward compatibility
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dead_code(project_path: impl AsRef<Path>) -> Result<AccurateDeadCodeReport> {
    let analyzer = CargoDeadCodeAnalyzer::new(project_path);
    analyzer.analyze().await
}

// Include implementation files
include!("cargo_dead_code_analyzer/cache_operations.rs");
include!("cargo_dead_code_analyzer/analysis.rs");
include!("cargo_dead_code_analyzer/parsing.rs");
include!("cargo_dead_code_analyzer/tests.rs");

#[cfg(test)]
#[path = "cargo_dead_code_analyzer/dead_line_bound_tests.rs"]
mod dead_line_bound_tests;

#[cfg(test)]
#[path = "cargo_dead_code_analyzer/cargo_target_scope_tests.rs"]
mod cargo_target_scope_tests;
