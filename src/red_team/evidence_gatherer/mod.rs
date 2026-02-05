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

    // ==================== EvidenceGatherer Tests ====================

    #[test]
    fn test_evidence_gatherer_new_sets_defaults() {
        let gatherer = EvidenceGatherer::new();
        assert_eq!(gatherer.git_history_window_days, 30);
        assert!((gatherer.confidence_threshold - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evidence_gatherer_default_trait() {
        let gatherer = EvidenceGatherer::default();
        assert_eq!(gatherer.git_history_window_days, 30);
        assert!((gatherer.confidence_threshold - 0.7).abs() < f64::EPSILON);
    }

    // ==================== Test Status Evidence Tests ====================

    #[test]
    fn test_gather_test_status_evidence_with_no_subsequent_commits() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock();
        let evidence = gatherer.gather_evidence(&claim, &context);

        // Should have git history evidence (empty commits) and test results
        assert!(!evidence.is_empty());
    }

    #[test]
    fn test_gather_test_status_evidence_with_test_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_subsequent_commits(vec![
            "fix test failure".to_string(),
            "update docs".to_string(),
        ]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        // Should find git history evidence that doesn't support claim
        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!((git_evidence.confidence - 0.85).abs() < 0.01);
        assert!(git_evidence
            .details
            .contains("1 subsequent test fixes found"));
    }

    #[test]
    fn test_gather_test_status_evidence_with_ignore_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix: ignore flaky test".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_test_status_evidence_with_test_results_all_passing() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(true, 0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        assert!(test_evidence.supports_claim);
        assert!((test_evidence.confidence - 0.9).abs() < 0.01);
        assert!(test_evidence.details.contains("All tests passing"));
    }

    #[test]
    fn test_gather_test_status_evidence_with_ignored_tests_absolute_claim() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(true, 5); // 5 ignored tests
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        // Absolute claim should not be supported if tests are ignored
        assert!(!test_evidence.supports_claim);
        assert!(test_evidence.details.contains("5 tests ignored"));
    }

    #[test]
    fn test_gather_test_status_evidence_with_ignored_tests_qualified_claim() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "tests passing".to_string(),
            is_absolute: false, // Not absolute
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(true, 5); // 5 ignored tests
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        // Qualified claim should be supported even with ignored tests
        assert!(test_evidence.supports_claim);
    }

    #[test]
    fn test_gather_test_status_evidence_with_failing_tests() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_test_results(false, 0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let test_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::TestExecution)
            .expect("Should have test execution evidence");
        assert!(!test_evidence.supports_claim);
        assert!(test_evidence.details.contains("Tests failing"));
    }

    // ==================== Documentation Evidence Tests ====================

    #[test]
    fn test_gather_documentation_evidence_with_doc_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["docs: fix broken link".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!(git_evidence
            .details
            .contains("1 subsequent documentation fixes found"));
    }

    #[test]
    fn test_gather_documentation_evidence_with_404_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix 404 in readme".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_documentation_evidence_with_broken_links() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_broken_links(3);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let link_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::LinkValidation)
            .expect("Should have link validation evidence");
        assert!(!link_evidence.supports_claim);
        assert!(link_evidence.details.contains("3 broken links found"));
    }

    #[test]
    fn test_gather_documentation_evidence_with_no_broken_links() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Documentation,
            text: "fixed all broken links".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_broken_links(0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let link_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::LinkValidation)
            .expect("Should have link validation evidence");
        assert!(link_evidence.supports_claim);
        assert!(link_evidence.details.contains("All links valid"));
    }

    // ==================== Coverage Evidence Tests ====================

    #[test]
    fn test_gather_coverage_evidence_with_coverage_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix coverage regression".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_coverage_evidence_actual_matches_claimed() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_coverage(85.5); // Within 2% tolerance
        let evidence = gatherer.gather_evidence(&claim, &context);

        let coverage_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CoverageReport)
            .expect("Should have coverage report evidence");
        assert!(coverage_evidence.supports_claim);
        assert!((coverage_evidence.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_gather_coverage_evidence_actual_differs_from_claimed() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_coverage(80.0); // Differs by 5%, outside tolerance
        let evidence = gatherer.gather_evidence(&claim, &context);

        let coverage_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CoverageReport)
            .expect("Should have coverage report evidence");
        assert!(!coverage_evidence.supports_claim);
        assert!(coverage_evidence
            .details
            .contains("Claimed: 85.0%, Actual: 80.0%"));
    }

    #[test]
    fn test_gather_coverage_evidence_with_coverage_error() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage stable at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_coverage_error("Tool failed to run");
        let evidence = gatherer.gather_evidence(&claim, &context);

        let coverage_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CoverageReport)
            .expect("Should have coverage report evidence");
        assert!(!coverage_evidence.supports_claim);
        assert!(coverage_evidence.details.contains("Coverage tool error"));
    }

    // ==================== Feature Completion Evidence Tests ====================

    #[test]
    fn test_gather_feature_completion_evidence_with_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::FeatureCompletion,
            text: "complete implementation".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["fix: handle edge case".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_feature_completion_evidence_with_reverts() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::FeatureCompletion,
            text: "complete implementation".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["revert: broken feature".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_feature_completion_evidence_no_issues() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::FeatureCompletion,
            text: "complete implementation".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["chore: update deps".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(git_evidence.supports_claim);
        assert!(git_evidence.details.contains("No subsequent fixes found"));
    }

    // ==================== Migration Evidence Tests ====================

    #[test]
    fn test_gather_migration_evidence_with_rollbacks() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Migration,
            text: "migration to v2 complete".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["rollback: migration broke prod".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!(git_evidence.details.contains("1 rollback commits found"));
    }

    #[test]
    fn test_gather_migration_evidence_with_old_system_references() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Migration,
            text: "migration to v2 complete".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_code_grep_results("old_api", 5);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let grep_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CodeGrep)
            .expect("Should have code grep evidence");
        assert!(!grep_evidence.supports_claim);
        assert!(grep_evidence
            .details
            .contains("5 files still reference 'old_api'"));
    }

    #[test]
    fn test_gather_migration_evidence_no_old_references() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Migration,
            text: "migration to v2 complete".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_code_grep_results("old_api", 0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let grep_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CodeGrep)
            .expect("Should have code grep evidence");
        assert!(grep_evidence.supports_claim);
        assert!(grep_evidence
            .details
            .contains("No references to 'old_api' found"));
    }

    // ==================== Bug Fix Evidence Tests ====================

    #[test]
    fn test_gather_bugfix_evidence_issue_closed() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_issue_status(123, "closed");
        let evidence = gatherer.gather_evidence(&claim, &context);

        let issue_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::IssueTracker)
            .expect("Should have issue tracker evidence");
        assert!(issue_evidence.supports_claim);
        assert!(issue_evidence.details.contains("Issue #123 is closed"));
    }

    #[test]
    fn test_gather_bugfix_evidence_issue_reopened() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_issue_status(123, "reopened");
        let evidence = gatherer.gather_evidence(&claim, &context);

        let issue_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::IssueTracker)
            .expect("Should have issue tracker evidence");
        assert!(!issue_evidence.supports_claim);
        assert!(issue_evidence.details.contains("Issue #123 was reopened"));
    }

    #[test]
    fn test_gather_bugfix_evidence_with_regressions() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["re-fix: regression in fix #123".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
        assert!(git_evidence.details.contains("1 regression commits found"));
    }

    // ==================== Performance Evidence Tests ====================

    #[test]
    fn test_gather_performance_evidence_with_benchmark_data() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_benchmarks(Some("baseline: 100ms, new: 50ms".to_string()));
        let evidence = gatherer.gather_evidence(&claim, &context);

        let bench_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::BenchmarkResults)
            .expect("Should have benchmark evidence");
        assert!(bench_evidence.supports_claim);
        assert!(bench_evidence.details.contains("baseline: 100ms"));
    }

    #[test]
    fn test_gather_performance_evidence_without_benchmark_data() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock();
        let evidence = gatherer.gather_evidence(&claim, &context);

        let bench_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::BenchmarkResults)
            .expect("Should have benchmark evidence");
        assert!(!bench_evidence.supports_claim);
        assert!(bench_evidence
            .details
            .contains("No benchmark data found to support numeric claim"));
    }

    #[test]
    fn test_gather_performance_evidence_with_regressions() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_subsequent_commits(vec![
            "fix: performance regression causing timeout".to_string(),
        ]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    #[test]
    fn test_gather_performance_evidence_no_numeric_no_benchmark() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "performance improved".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock();
        let evidence = gatherer.gather_evidence(&claim, &context);

        let bench_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::BenchmarkResults)
            .expect("Should have benchmark evidence");
        assert!(!bench_evidence.supports_claim);
        assert!(bench_evidence
            .details
            .contains("No benchmark data available"));
    }

    // ==================== Security Evidence Tests ====================

    #[test]
    fn test_gather_security_evidence_no_vulnerabilities() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Security,
            text: "zero vulnerabilities".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_vulnerabilities(0);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let audit_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CargoAudit)
            .expect("Should have cargo audit evidence");
        assert!(audit_evidence.supports_claim);
        assert!(audit_evidence.details.contains("No vulnerabilities found"));
    }

    #[test]
    fn test_gather_security_evidence_with_vulnerabilities() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Security,
            text: "zero vulnerabilities".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock().with_vulnerabilities(3);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let audit_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::CargoAudit)
            .expect("Should have cargo audit evidence");
        assert!(!audit_evidence.supports_claim);
        assert!(audit_evidence.details.contains("3 vulnerabilities found"));
    }

    #[test]
    fn test_gather_security_evidence_with_security_fixes() {
        let gatherer = EvidenceGatherer::new();
        let claim = Claim {
            category: ClaimCategory::Security,
            text: "zero vulnerabilities".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let context = RepositoryContext::new_mock()
            .with_subsequent_commits(vec!["security: patch CVE-2024-1234".to_string()]);
        let evidence = gatherer.gather_evidence(&claim, &context);

        let git_evidence = evidence
            .iter()
            .find(|e| e.source == EvidenceSource::GitHistory)
            .expect("Should have git history evidence");
        assert!(!git_evidence.supports_claim);
    }

    // ==================== RepositoryContext Builder Tests ====================

    #[test]
    fn test_repository_context_with_commit_timestamps() {
        let timestamps = vec![1000, 2000, 3000];
        let context = RepositoryContext::new_mock().with_commit_timestamps(timestamps.clone());

        assert_eq!(context.commit_timestamps, Some(timestamps));
        assert_eq!(context.latest_commit_timestamp, Some(3000));
    }

    #[test]
    fn test_repository_context_with_empty_commit_timestamps() {
        let context = RepositoryContext::new_mock().with_commit_timestamps(vec![]);

        assert_eq!(context.commit_timestamps, Some(vec![]));
        assert_eq!(context.latest_commit_timestamp, None);
    }

    #[test]
    fn test_repository_context_new_mock_defaults() {
        let context = RepositoryContext::new_mock();

        assert_eq!(context.subsequent_commits, Some(vec![]));
        assert_eq!(context.test_results, Some((true, 0)));
        assert_eq!(context.actual_coverage, None);
        assert_eq!(context.coverage_error, None);
        assert_eq!(context.broken_links_count, None);
        assert_eq!(context.vulnerabilities_count, None);
        assert_eq!(context.benchmark_results, None);
        assert_eq!(context.issue_status, None);
        assert_eq!(context.code_grep_results, None);
    }

    #[test]
    fn test_repository_context_has_git_history() {
        let context = RepositoryContext::new_mock();
        // Mock context has no git_repo set
        assert!(!context.has_git_history());
    }

    #[test]
    fn test_repository_context_has_coverage_report() {
        let context = RepositoryContext::new_mock();
        // Mock context has no coverage_path set
        assert!(!context.has_coverage_report());
    }

    #[test]
    fn test_repository_context_get_test_files() {
        let context = RepositoryContext::new_mock();
        // Mock context has empty test_files
        assert!(context.get_test_files().is_empty());
    }

    #[test]
    fn test_repository_context_get_coverage_percentage_no_report() {
        let context = RepositoryContext::new_mock();
        // Mock context has no coverage_path, should return 0.0
        assert!((context.get_coverage_percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_repository_context_get_test_execution_info_no_results() {
        let context = RepositoryContext::new_mock();
        let info = context.get_test_execution_info();
        // Mock context has no test_results_path
        assert!(!info.has_results);
        assert_eq!(info.passed_count, 0);
        assert_eq!(info.failed_count, 0);
        assert_eq!(info.ignored_count, 0);
    }

    #[test]
    fn test_repository_context_grep_codebase() {
        let context = RepositoryContext::new_mock();
        // Should return empty since repo_path is "."
        let results = context.grep_codebase("nonexistent_pattern_xyz123");
        // May or may not find matches depending on current directory
        assert!(results.is_empty() || !results.is_empty()); // Just verify it doesn't panic
    }

    #[test]
    fn test_repository_context_get_recent_commits_no_repo() {
        let context = RepositoryContext::new_mock();
        // Mock context has no git_repo set
        let commits = context.get_recent_commits(10);
        assert!(commits.is_empty());
    }

    // ==================== EvidenceSource Tests ====================

    #[test]
    fn test_evidence_source_serialization() {
        let sources = vec![
            EvidenceSource::GitHistory,
            EvidenceSource::TestExecution,
            EvidenceSource::CoverageReport,
            EvidenceSource::LinkValidation,
            EvidenceSource::CargoAudit,
            EvidenceSource::BenchmarkResults,
            EvidenceSource::IssueTracker,
            EvidenceSource::CodeGrep,
        ];

        for source in sources {
            let serialized = serde_json::to_string(&source).expect("Should serialize");
            let deserialized: EvidenceSource =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(source, deserialized);
        }
    }

    // ==================== EvidenceResult Tests ====================

    #[test]
    fn test_evidence_result_serialization() {
        let result = EvidenceResult {
            source: EvidenceSource::GitHistory,
            supports_claim: true,
            confidence: 0.85,
            details: "No issues found".to_string(),
            timestamp: Some(1234567890),
        };

        let serialized = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: EvidenceResult =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(result.source, deserialized.source);
        assert_eq!(result.supports_claim, deserialized.supports_claim);
        assert!((result.confidence - deserialized.confidence).abs() < f64::EPSILON);
        assert_eq!(result.details, deserialized.details);
        assert_eq!(result.timestamp, deserialized.timestamp);
    }

    // ==================== CommitInfo Tests ====================

    #[test]
    fn test_commit_info_fields() {
        let info = CommitInfo {
            message: "Fix bug".to_string(),
            timestamp: 1234567890,
            author: "Test Author".to_string(),
        };

        assert_eq!(info.message, "Fix bug");
        assert_eq!(info.timestamp, 1234567890);
        assert_eq!(info.author, "Test Author");
    }

    // ==================== TestExecutionInfo Tests ====================

    #[test]
    fn test_test_execution_info_default() {
        let info = TestExecutionInfo::default();

        assert!(!info.has_results);
        assert_eq!(info.passed_count, 0);
        assert_eq!(info.failed_count, 0);
        assert_eq!(info.ignored_count, 0);
    }
}
