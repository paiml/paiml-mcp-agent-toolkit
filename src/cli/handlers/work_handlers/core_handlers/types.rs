#![cfg_attr(coverage_nightly, coverage(off))]
// Types and constants for work command handlers

use serde::{Deserialize, Serialize};

/// Commit metadata structure for linking commits to work items and quality scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CommitMetadata {
    pub(super) commit_sha: Option<String>,
    pub(super) work_item_id: String,
    pub(super) prompt: String,
    pub(super) tdg_score: f64,
    pub(super) repo_score: f64,
    pub(super) rust_project_score: Option<f64>,
    pub(super) timestamp: chrono::DateTime<chrono::Utc>,
}

/// Minimal issue info for API-agnostic GitHub operations
/// Works with either octocrab (github-api feature) or gh CLI fallback
#[derive(Debug, Clone)]
pub struct GitHubIssueInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<String>,
}

/// Hypothesis pattern to CLI override name mapping (CB-040 complexity refactor)
pub(super) const CLAIM_PATTERNS: &[(&[&str], &str)] = &[
    (&["manifest", "files deleted"], "manifest"),
    (&["meta-falsification", "falsification system"], "meta-falsification"),
    (&["coverage gaming", "coverage exclusion", "cfg(not(coverage))"], "coverage-gaming"),
    (&["differential coverage", "new code", "changed lines"], "differential-coverage"),
    (&["total coverage", "absolute coverage", "coverage does not decrease", "coverage >= 95"], "coverage"),
    (&["tdg", "test-driven grade"], "tdg"),
    (&["complexity", "cyclomatic"], "complexity"),
    (&["supply chain", "dependencies", "vulnerable dependencies"], "supply-chain"),
    (&["file size", "500 lines"], "file-size"),
    (&["spec", "specification"], "spec-quality"),
    (&["github", "sync", "changes pushed", "uncommitted"], "github-sync"),
    (&["examples", "compile"], "examples"),
    (&["book", "pmat-book"], "book"),
    (&["satd", "todo/fixme/hack"], "satd"),
    (&["dead code introduced", "dead code detected"], "dead-code"),
    (&["per-file coverage", "files have >= 95%", "all files have"], "per-file-coverage"),
    (&["lint passes", "make lint"], "lint"),
    // v3.1 defect churn prevention
    (&["match arm", "variant"], "variant-coverage"),
    (&["fix-after-fix", "fix chain"], "fix-chain"),
    (&["cross-crate", "sibling project", "integration tests pass"], "cross-crate"),
    (&["regression", "performance"], "regression-gate"),
];
