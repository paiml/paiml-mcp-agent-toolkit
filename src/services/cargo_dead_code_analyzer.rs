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
/// 4: `compiler_scan` was added. A schema-3 entry deserialises it as `None`,
/// which is the one value that means "this engine has no compiler layer" — so
/// a cached report would deny having a compiler layer at all, and the reduced
/// scan this field exists to disclose would be invisible again.
/// 5: the cache is keyed on the WORKING tree (scratch-index `git write-tree`)
/// instead of `git rev-parse HEAD:`, and the report carries `cache`. Every
/// schema-4 entry is keyed on a commit tree and must be a miss, or an existing
/// cache keeps serving pre-fix answers after upgrade (CRUX-04, #1153).
pub const DEAD_CODE_CACHE_SCHEMA: u32 = 5;

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
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
    /// Whether Layer 2 — rustc's dead-code lint, via `cargo check` — actually
    /// ran, and what stopped it when it did not.
    ///
    /// `calculate_metrics` builds the report before it can know, so it is
    /// `None` there and filled in by `analyze`. A `None` that escapes to a
    /// consumer means the same thing it means everywhere else: no compiler
    /// layer was involved, NOT that one ran.
    #[serde(default)]
    pub compiler_scan: Option<crate::models::dead_code::CompilerScanReport>,
    /// Whether this report was replayed from the cache, and the working-tree
    /// hash it is keyed on. Set by `analyze()`, never persisted as a hit: an
    /// entry is written with `hit: false` and marked `hit: true` when served.
    #[serde(default)]
    pub cache: Option<crate::models::dead_code::DeadCodeCacheReport>,
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
    /// The path the caller asked about, exactly as given. Everything the report
    /// describes is measured over this: the walk that counts files and lines,
    /// the suppression scan, and the paths the rows are relative to.
    project_path: PathBuf,
    /// `project_path` made absolute, which is what a cargo diagnostic's file
    /// name is compared against to decide whether it is inside the requested
    /// tree. Kept beside `project_path` rather than replacing it so the cache
    /// location and the reported row paths do not move.
    report_root: PathBuf,
    /// The directory the reported row paths are relative to: `report_root`
    /// itself, or its parent when the requested path is a single FILE.
    ///
    /// Membership and naming are two different questions and a file answers
    /// them differently: `<crate>/src/inner/mod.rs` is in scope only if it IS
    /// that file, but naming a row relative to itself yields the empty string —
    /// a row that names no file at all, next to a `total_lines: 0` that reads
    /// as a measurement. So the row is named against the directory holding it
    /// while scope stays pinned to the file.
    report_base: PathBuf,
    /// The crate `cargo check` is run in: `project_path` itself when it holds a
    /// `Cargo.toml` declaring a `[package]`, otherwise the nearest ancestor that
    /// does.
    ///
    /// These are the same directory for the ordinary "analyse this crate" call
    /// and they differ for "analyse this subdirectory of a crate", which is the
    /// case that used to compile nothing at all — see
    /// [`enclosing_crate_root`]. A `[workspace]` manifest declares no package
    /// and so is not one of those ancestors: a workspace is not a compilation
    /// unit, and treating its root as a crate is what published a zero over a
    /// workspace whose members were never compiled. When no ancestor declares a
    /// package this falls back to `report_root`, and `cargo check` then fails
    /// the way it always did; the CLI refuses before reaching that point.
    cargo_root: PathBuf,
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
    /// timer worked. The caller that has a user-facing budget owns it;
    /// [`DEFAULT_ANALYSIS_TIMEOUT_SECS`] is the default for callers that do not.
    timeout: std::time::Duration,
}

/// How long a dead-code analysis may run before its `cargo check` child is
/// killed, for any caller that does not name its own budget.
///
/// #929 CONSEQUENCE. The budget only started binding when the blocking
/// `Command::output()` was replaced by a child this analyzer can kill, and the
/// moment it bound, every default that had never been tested became a real
/// failure mode: the CLI shipped 60s and this constructor shipped 90s for the
/// SAME work, so `pmat analyze dead-code -p .` on this repo exited 5 at 60.4s
/// where the same command used to run 245s to completion.
///
/// 900s is what the measurement supports, not a round number: this repo takes
/// 245s for a COLD `cargo check` (67.6s warm), and a monorepo several times its
/// size on a slower machine is the case the default has to survive, since the
/// first run after a dependency bump is always cold. The result is cached, so
/// the budget is paid once. A caller that wants a tighter bound passes
/// [`CargoDeadCodeAnalyzer::with_timeout`]; the CLI's `--timeout` does exactly
/// that.
///
/// One constant, so the library default and the CLI default cannot drift apart
/// again the way 90 and 60 did.
pub const DEFAULT_ANALYSIS_TIMEOUT_SECS: u64 = 900;

impl CargoDeadCodeAnalyzer {
    /// Create a new analyzer for the given project path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        let project_path = project_path.as_ref().to_path_buf();
        let report_root = absolutize(&project_path);
        // Resolved once, here, so that the cargo invocation and everything that
        // reasons about the crate's shape cannot disagree about which crate is
        // being compiled.
        let cargo_root = enclosing_crate_root(&project_path).unwrap_or_else(|| report_root.clone());
        let report_base = if report_root.is_file() {
            report_root
                .parent()
                .map_or_else(|| report_root.clone(), Path::to_path_buf)
        } else {
            report_root.clone()
        };
        Self {
            project_path,
            report_root,
            report_base,
            cargo_root,
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
            timeout: std::time::Duration::from_secs(DEFAULT_ANALYSIS_TIMEOUT_SECS),
        }
    }

    /// The crate this analysis compiles, which is the requested path itself
    /// only when that path is a crate root.
    ///
    /// Readable because the difference between it and the requested path is the
    /// whole of the subtree case: a caller that wants to know which crate
    /// answered must be able to ask rather than re-derive it.
    #[must_use]
    pub fn cargo_root(&self) -> &Path {
        &self.cargo_root
    }

    /// How long this analysis may run before `cargo check` is killed.
    ///
    /// Readable so the shipped default is testable: it was a literal inside
    /// `new()` that nothing could observe, which is how it came to disagree
    /// with the CLI's own default (#929).
    #[must_use]
    pub fn timeout(&self) -> std::time::Duration {
        self.timeout
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
include!("cargo_dead_code_analyzer/crate_root.rs");
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

#[cfg(test)]
#[path = "cargo_dead_code_analyzer/crate_root_tests.rs"]
mod crate_root_tests;

/// Give a TEST fixture crate the `Cargo.lock` a real project would have.
///
/// `analyze dead-code` passes `--locked` (#1076), so it no longer creates one —
/// and a fixture that has none is now analysed at REDUCED fidelity, with the
/// compiler layer skipped. Without this the tests below would keep passing
/// their own assertions while silently measuring nothing, which is the exact
/// failure mode the disclosure exists to prevent.
///
/// `--offline` keeps it hermetic: every fixture that calls this is
/// dependency-free, so the resolution needs no registry.
#[cfg(test)]
pub(crate) fn write_fixture_lockfile(crate_root: &Path) {
    let output = Command::new("cargo")
        .current_dir(crate_root)
        .args(["generate-lockfile", "--offline"])
        .output()
        .expect("cargo generate-lockfile runs");
    assert!(
        output.status.success(),
        "could not give the fixture at {} a lockfile: {}",
        crate_root.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// The analyser must not write a Cargo.lock into the tree it measures (#1076).
#[cfg(test)]
#[path = "cargo_dead_code_analyzer/lockfile_tests.rs"]
mod lockfile_tests;
