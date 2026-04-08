#![cfg_attr(coverage_nightly, coverage(off))]

//! Coverage exclusion classification for --coverage-gaps filtering.
//!
//! Detects WHY a function appears as a coverage gap, so `--coverage-gaps`
//! can filter out intentionally excluded functions (coverage(off), Makefile
//! COVERAGE_EXCLUDE patterns, dead code) and only show genuinely testable gaps.

use super::types::QueryResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Why a function is excluded from coverage tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CoverageExclusion {
    /// Testable — genuine coverage gap
    #[default]
    None,
    /// File has `#![cfg_attr(coverage_nightly, coverage(off))]` at module level
    CoverageOff,
    /// File matches Makefile `COVERAGE_EXCLUDE` regex pattern
    MakefileExcluded,
    /// Function is in dead-code-cache.json
    DeadCode,
}

impl CoverageExclusion {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is none.
    pub fn is_none(&self) -> bool {
        matches!(self, CoverageExclusion::None)
    }

    /// Label.
    pub fn label(&self) -> &'static str {
        match self {
            CoverageExclusion::None => "testable",
            CoverageExclusion::CoverageOff => "coverage(off)",
            CoverageExclusion::DeadCode => "dead code",
            CoverageExclusion::MakefileExcluded => "Makefile pattern",
        }
    }
}

/// Parsed exclusion context: caches file-level checks so we don't re-read files.
pub(crate) struct ExclusionContext {
    /// Files that have module-level `coverage(off)` annotation
    coverage_off_files: HashSet<String>,
    /// Files already checked for coverage(off) — prevents redundant re-reads
    /// for files that do NOT have the annotation (negative cache).
    checked_files: HashSet<String>,
    /// Compiled regex from Makefile COVERAGE_EXCLUDE (if found)
    makefile_regex: Option<regex::Regex>,
    /// Dead function keys: "file_path::function_name"
    dead_functions: HashSet<String>,
    /// Whether coverage_off_files was pre-populated from index cache (skip file I/O)
    use_cached: bool,
}

/// Summary of excluded function counts by category.
#[derive(Default)]
pub struct ExclusionSummary {
    pub coverage_off_count: usize,
    pub coverage_off_files: usize,
    pub dead_code_count: usize,
    pub dead_code_files: usize,
    pub makefile_count: usize,
    pub makefile_files: usize,
}

// ── Implementation includes ──────────────────────────────────────────────────

include!("coverage_exclusion_context.rs");
include!("coverage_exclusion_summary.rs");
include!("coverage_exclusion_tests.rs");
