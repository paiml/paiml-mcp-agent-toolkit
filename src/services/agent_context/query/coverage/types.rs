#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── LLVM Coverage Export JSON Structs ───────────────────────────────────────

/// Top-level LLVM coverage export format (cargo llvm-cov export --format=json)
#[derive(Debug, Deserialize)]
pub(super) struct LlvmCoverageExport {
    pub(super) data: Vec<LlvmCoverageData>,
    #[serde(rename = "type")]
    pub(super) export_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LlvmCoverageData {
    pub(super) files: Vec<LlvmFileCoverage>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LlvmFileCoverage {
    pub(super) filename: String,
    pub(super) segments: Vec<Vec<serde_json::Value>>,

    pub(super) summary: Option<LlvmSummary>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LlvmSummary {
    pub(super) lines: Option<LlvmLineSummary>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LlvmLineSummary {
    pub(super) count: u32,
    pub(super) covered: u32,
}

// ── Coverage Cache ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct CoverageCache {
    pub(super) git_hash: String,
    /// mtime (seconds since epoch) of profdata source when cache was built
    #[serde(default)]
    pub(super) coverage_mtime: Option<u64>,
    /// Path to the llvm-cov-target directory used when cache was built
    #[serde(default)]
    pub(super) profdata_dir: Option<String>,
    pub(super) files: HashMap<String, HashMap<usize, u64>>,
}
