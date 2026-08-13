#![allow(unused)]
// Evidence Gatherer: Multi-source validation for hallucination detection
//
// Specification: Section 3.2 - Claim Categories
// Implements empirical evidence gathering for 8 claim categories

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{Claim, ClaimCategory};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Evidence source.
pub enum EvidenceSource {
    GitHistory,       // Subsequent commits contradicting claim
    TestExecution,    // Running tests to verify claim
    CoverageReport,   // Actual coverage vs claimed
    LinkValidation,   // Checking documentation links
    CargoAudit,       // Security audit results
    BenchmarkResults, // Performance measurements
    IssueTracker,     // GitHub issue status
    CodeGrep,         // Searching codebase for references
    /// A source that could not adjudicate this claim: the artefact it reads was
    /// absent or unreadable, or pmat does not run that measurement at all.
    ///
    /// This exists because absence had no representation. A missing benchmark
    /// scored `supports_claim: false` and failed every Performance claim,
    /// including ones true by construction; a missing coverage report produced
    /// no entry at all and let "✅ All claims verified" stand over a number
    /// nothing had read. Neither is a finding about the claim, so neither may
    /// be rendered as one.
    NotMeasured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Result of evidence operation.
pub struct EvidenceResult {
    pub source: EvidenceSource,
    /// Meaningless unless [`EvidenceResult::measured`] — read it through
    /// [`EvidenceResult::contradicts`] rather than directly.
    pub supports_claim: bool,
    pub confidence: f64, // 0.0 to 1.0
    pub details: String,
    pub timestamp: Option<i64>,
}

impl EvidenceResult {
    /// Did this source actually observe something about the claim?
    #[must_use]
    pub fn measured(&self) -> bool {
        !matches!(self.source, EvidenceSource::NotMeasured)
    }

    /// The single definition of "this evidence contradicts the claim".
    ///
    /// Every consumer — the verdict, the exit code, the contradicting-evidence
    /// list — must ask here, so that a check which never ran cannot fail a
    /// commit on one surface while reading as silence on another.
    #[must_use]
    pub fn contradicts(&self) -> bool {
        self.measured() && !self.supports_claim
    }

    /// A check that did not run, and the artefact that would make it run.
    #[must_use]
    pub fn not_measured(reason: impl Into<String>) -> Self {
        Self {
            source: EvidenceSource::NotMeasured,
            // Not support: an unread artefact certifies nothing. Not a
            // contradiction either — see `contradicts`, which gates on source.
            supports_claim: false,
            confidence: 0.0,
            details: format!("NOT MEASURED: {}", reason.into()),
            timestamp: None,
        }
    }
}

/// Evidence gatherer.
pub struct EvidenceGatherer {
    // Configuration for evidence gathering (future use)
    git_history_window_days: u32,

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
/// Information about commit.
pub struct CommitInfo {
    pub message: String,
    pub timestamp: i64,
    pub author: String,
}

#[derive(Debug, Clone, Default)]
/// Information about test execution.
pub struct TestExecutionInfo {
    pub has_results: bool,
    pub passed_count: usize,
    pub failed_count: usize,
    pub ignored_count: usize,
}

// RepositoryContext: Mock-friendly context for evidence gathering
#[derive(Debug, Clone)]
/// Context for repository operations.
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
