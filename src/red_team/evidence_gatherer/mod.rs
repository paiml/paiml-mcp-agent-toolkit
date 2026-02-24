// Evidence Gatherer: Multi-source validation for hallucination detection
//
// Specification: Section 3.2 - Claim Categories
// Implements empirical evidence gathering for 8 claim categories

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{Claim, ClaimCategory};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceSource {
    GitHistory,       // Subsequent commits contradicting claim
    TestExecution,    // Running tests to verify claim
    CoverageReport,   // Actual coverage vs claimed
    LinkValidation,   // Checking documentation links
    CargoAudit,       // Security audit results
    BenchmarkResults, // Performance measurements
    IssueTracker,     // GitHub issue status
    CodeGrep,         // Searching codebase for references
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceResult {
    pub source: EvidenceSource,
    pub supports_claim: bool,
    pub confidence: f64, // 0.0 to 1.0
    pub details: String,
    pub timestamp: Option<i64>,
}

pub struct EvidenceGatherer {
    // Configuration for evidence gathering (future use)
    #[allow(dead_code)]
    git_history_window_days: u32,
    #[allow(dead_code)]
    confidence_threshold: f64,
}

include!("evidence_impl.rs");

impl Default for EvidenceGatherer {
    fn default() -> Self {
        Self::new()
    }
}

// Supporting types for repository context
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub message: String,
    pub timestamp: i64,
    pub author: String,
}

#[derive(Debug, Clone, Default)]
pub struct TestExecutionInfo {
    pub has_results: bool,
    pub passed_count: usize,
    pub failed_count: usize,
    pub ignored_count: usize,
}

// RepositoryContext: Mock-friendly context for evidence gathering
#[derive(Debug, Clone)]
pub struct RepositoryContext {
    pub subsequent_commits: Option<Vec<String>>,
    pub test_results: Option<(bool, usize)>, // (passing, ignored_count)
    pub actual_coverage: Option<f64>,
    pub coverage_error: Option<String>,
    pub broken_links_count: Option<usize>,
    pub vulnerabilities_count: Option<usize>,
    pub benchmark_results: Option<String>,
    pub issue_status: Option<String>,
    pub code_grep_results: Option<(String, usize)>, // (search_term, count)
    pub latest_commit_timestamp: Option<i64>,
    pub commit_timestamps: Option<Vec<i64>>,

    // Real repository data (populated by from_path)
    git_repo: Option<PathBuf>,
    test_files: Vec<PathBuf>,
    coverage_path: Option<PathBuf>,
    test_results_path: Option<PathBuf>,
    repo_path: PathBuf, // Original path passed to from_path
}

include!("repository_impl.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_gatherer_compiles() {
        let gatherer = EvidenceGatherer::new();
        assert!(gatherer.git_history_window_days == 30);
    }

    #[test]
    fn test_repository_context_builder() {
        let context = RepositoryContext::new_mock()
            .with_coverage(85.0)
            .with_vulnerabilities(0);

        assert_eq!(context.actual_coverage, Some(85.0));
        assert_eq!(context.vulnerabilities_count, Some(0));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    include!("tests_claim_verification.rs");
    include!("tests_feature_and_bugfix.rs");
    include!("tests_security_and_builder.rs");
}
