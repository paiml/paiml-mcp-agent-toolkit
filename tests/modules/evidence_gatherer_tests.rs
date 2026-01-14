// EXTREME TDD: Evidence Gatherer Tests (RED Phase)
//
// Test-Driven Development for multi-source evidence validation
// Specification: Section 3.2 - Claim Categories
// Target: Gather empirical evidence for hallucination detection

use pmat::red_team::{Claim, ClaimCategory, EvidenceGatherer, EvidenceSource};

// RED Test 1: Gather evidence for test status claim
#[test]
fn test_gather_test_status_evidence() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_test_status_claim("all tests passing", true);

    let evidence = gatherer.gather_evidence(&claim, &create_mock_repo_context());

    // Should check multiple sources
    assert!(evidence.len() >= 2);

    // Should include git history check
    assert!(evidence
        .iter()
        .any(|e| e.source == EvidenceSource::GitHistory));

    // Should include test execution
    assert!(evidence
        .iter()
        .any(|e| e.source == EvidenceSource::TestExecution));

    // Each evidence should have confidence score
    assert!(evidence
        .iter()
        .all(|e| e.confidence >= 0.0 && e.confidence <= 1.0));
}

// RED Test 2: Evidence supports claim (positive case)
#[test]
fn test_evidence_supports_claim() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_coverage_claim("coverage at 85%", Some(85.0));

    // Mock: Actual coverage is 86% (supports claim)
    let context = create_mock_context_with_coverage(86.0);

    let evidence = gatherer.gather_evidence(&claim, &context);

    // Should find supporting evidence
    let coverage_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::CoverageReport)
        .expect("Coverage evidence should exist");

    assert!(coverage_evidence.supports_claim);
    assert!(coverage_evidence.confidence > 0.8); // High confidence
}

// RED Test 3: Evidence contradicts claim (hallucination detected)
#[test]
fn test_evidence_contradicts_claim() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_coverage_claim("coverage at 85%", Some(85.0));

    // Mock: Actual coverage is 65% (contradicts claim)
    let context = create_mock_context_with_coverage(65.0);

    let evidence = gatherer.gather_evidence(&claim, &context);

    // Should find contradicting evidence
    let coverage_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::CoverageReport)
        .expect("Coverage evidence should exist");

    assert!(!coverage_evidence.supports_claim);
    assert!(coverage_evidence.confidence > 0.8); // High confidence in contradiction
    assert!(coverage_evidence.details.contains("65")); // Show actual value
}

// RED Test 4: Git history evidence - subsequent fixes
#[test]
fn test_git_history_evidence_subsequent_fixes() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_feature_completion_claim("Complete feature X", true);

    // Mock: Subsequent commits fixing feature X
    let context = create_mock_context_with_subsequent_fixes(vec![
        "fix: Bug in feature X",
        "fix: Edge case in feature X",
    ]);

    let evidence = gatherer.gather_evidence(&claim, &context);

    let git_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::GitHistory)
        .expect("Git history evidence should exist");

    assert!(!git_evidence.supports_claim); // Subsequent fixes contradict "complete"
    assert!(git_evidence.confidence > 0.7);
    assert!(git_evidence.details.contains("2 subsequent fixes"));
}

// RED Test 5: No contradicting evidence = supports claim
#[test]
fn test_no_contradicting_evidence_supports_claim() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_test_status_claim("tests passing", false); // Not absolute

    // Mock: No subsequent test fixes, tests currently passing
    let context = create_mock_context_clean();

    let evidence = gatherer.gather_evidence(&claim, &context);

    // All evidence should support claim
    assert!(evidence.iter().all(|e| e.supports_claim));
}

// RED Test 6: Documentation link validation
#[test]
fn test_documentation_link_evidence() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_documentation_claim("fixed all broken links", true);

    // Mock: 3 broken links still exist
    let context = create_mock_context_with_broken_links(3);

    let evidence = gatherer.gather_evidence(&claim, &context);

    let link_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::LinkValidation)
        .expect("Link validation evidence should exist");

    assert!(!link_evidence.supports_claim);
    assert!(link_evidence.details.contains("3 broken links"));
}

// RED Test 7: Security audit evidence
#[test]
fn test_security_audit_evidence() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_security_claim("zero vulnerabilities", true);

    // Mock: Cargo audit found 2 vulnerabilities
    let context = create_mock_context_with_vulnerabilities(2);

    let evidence = gatherer.gather_evidence(&claim, &context);

    let audit_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::CargoAudit)
        .expect("Cargo audit evidence should exist");

    assert!(!audit_evidence.supports_claim);
    assert!(audit_evidence.details.contains("2 vulnerabilities"));
}

// RED Test 8: Performance benchmark evidence
#[test]
fn test_performance_benchmark_evidence() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_performance_claim("50% faster", Some(50.0));

    // Mock: No benchmark data available
    let context = create_mock_context_no_benchmarks();

    let evidence = gatherer.gather_evidence(&claim, &context);

    let benchmark_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::BenchmarkResults)
        .expect("Benchmark evidence should exist");

    assert!(!benchmark_evidence.supports_claim); // No data = cannot verify
    assert!(benchmark_evidence.details.contains("No benchmark data"));
}

// RED Test 9: Bug fix verification via issue tracker
#[test]
fn test_bug_fix_issue_tracker_evidence() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_bugfix_claim("fixes issue #42", Some(42));

    // Mock: Issue #42 was reopened (regression)
    let context = create_mock_context_with_issue_reopened(42);

    let evidence = gatherer.gather_evidence(&claim, &context);

    let issue_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::IssueTracker)
        .expect("Issue tracker evidence should exist");

    assert!(!issue_evidence.supports_claim);
    assert!(issue_evidence.details.contains("reopened"));
}

// RED Test 10: Migration evidence - old system still referenced
#[test]
fn test_migration_evidence_old_system_referenced() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_migration_claim("Complete migration to libsql", true);

    // Mock: Old system (sled) still referenced in 15 files
    let context = create_mock_context_with_old_system_refs("sled", 15);

    let evidence = gatherer.gather_evidence(&claim, &context);

    let migration_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::CodeGrep)
        .expect("Code grep evidence should exist");

    assert!(!migration_evidence.supports_claim);
    assert!(migration_evidence.details.contains("15 files"));
    assert!(migration_evidence.details.contains("sled"));
}

// RED Test 11: Evidence aggregation confidence
#[test]
fn test_aggregate_evidence_confidence() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_test_status_claim("all tests passing", true);

    // Mock: Multiple contradicting sources
    let context = create_mock_context_with_multiple_contradictions();

    let evidence = gatherer.gather_evidence(&claim, &context);

    // When all sources contradict, aggregate confidence should be high
    let all_contradict = evidence.iter().all(|e| !e.supports_claim);
    if all_contradict {
        let avg_confidence: f64 =
            evidence.iter().map(|e| e.confidence).sum::<f64>() / evidence.len() as f64;
        assert!(avg_confidence > 0.7);
    }
}

// RED Test 12: Qualified claim (has scope) requires less evidence
#[test]
fn test_qualified_claim_evidence_gathering() {
    let gatherer = EvidenceGatherer::new();

    // Claim with scope qualifier ("MVP - Sprint 42")
    let mut claim = create_feature_completion_claim("Complete feature X", false);
    claim.has_scope_qualifier = true;
    claim.scope = Some("MVP - Sprint 42".to_string());

    let context = create_mock_repo_context();

    let evidence = gatherer.gather_evidence(&claim, &context);

    // Qualified claims should still gather evidence but interpret it differently
    // (This is tested in interpretation, not gathering)
    assert!(!evidence.is_empty());
}

// RED Test 13: Evidence includes timestamps for temporal analysis
#[test]
fn test_evidence_includes_timestamps() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_test_status_claim("all tests passing", true);
    let context = create_mock_context_with_timestamps();

    let evidence = gatherer.gather_evidence(&claim, &context);

    // Git history evidence should include timestamps
    let git_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::GitHistory);

    if let Some(git_ev) = git_evidence {
        assert!(git_ev.timestamp.is_some());
    }
}

// RED Test 14: Evidence gathering handles errors gracefully
#[test]
fn test_evidence_gathering_error_handling() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_coverage_claim("coverage at 85%", Some(85.0));

    // Mock: Coverage tool failed (error context)
    let context = create_mock_context_with_error();

    let evidence = gatherer.gather_evidence(&claim, &context);

    // Should still return evidence (with error details)
    let coverage_evidence = evidence
        .iter()
        .find(|e| e.source == EvidenceSource::CoverageReport);

    if let Some(cov_ev) = coverage_evidence {
        assert!(!cov_ev.supports_claim); // Error = cannot verify
        assert!(cov_ev.details.contains("error") || cov_ev.details.contains("failed"));
    }
}

// RED Test 15: Evidence details are human-readable
#[test]
fn test_evidence_details_human_readable() {
    let gatherer = EvidenceGatherer::new();

    let claim = create_test_status_claim("all tests passing", true);
    let context = create_mock_repo_context();

    let evidence = gatherer.gather_evidence(&claim, &context);

    // All evidence should have non-empty, descriptive details
    assert!(evidence.iter().all(|e| !e.details.is_empty()));
    assert!(evidence.iter().all(|e| e.details.len() > 10)); // Reasonably descriptive
}

// Helper functions to create test data
fn create_test_status_claim(text: &str, is_absolute: bool) -> Claim {
    Claim {
        category: ClaimCategory::TestStatus,
        text: text.to_string(),
        is_absolute,
        numeric_value: None,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_coverage_claim(text: &str, value: Option<f64>) -> Claim {
    Claim {
        category: ClaimCategory::Coverage,
        text: text.to_string(),
        is_absolute: false,
        numeric_value: value,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_feature_completion_claim(text: &str, is_absolute: bool) -> Claim {
    Claim {
        category: ClaimCategory::FeatureCompletion,
        text: text.to_string(),
        is_absolute,
        numeric_value: None,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_documentation_claim(text: &str, is_absolute: bool) -> Claim {
    Claim {
        category: ClaimCategory::Documentation,
        text: text.to_string(),
        is_absolute,
        numeric_value: None,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_security_claim(text: &str, is_absolute: bool) -> Claim {
    Claim {
        category: ClaimCategory::Security,
        text: text.to_string(),
        is_absolute,
        numeric_value: None,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_performance_claim(text: &str, value: Option<f64>) -> Claim {
    Claim {
        category: ClaimCategory::Performance,
        text: text.to_string(),
        is_absolute: false,
        numeric_value: value,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_bugfix_claim(text: &str, issue_num: Option<u32>) -> Claim {
    Claim {
        category: ClaimCategory::BugFix,
        text: text.to_string(),
        is_absolute: false,
        numeric_value: None,
        issue_number: issue_num,
        has_scope_qualifier: false,
        scope: None,
    }
}

fn create_migration_claim(text: &str, is_absolute: bool) -> Claim {
    Claim {
        category: ClaimCategory::Migration,
        text: text.to_string(),
        is_absolute,
        numeric_value: None,
        issue_number: None,
        has_scope_qualifier: false,
        scope: None,
    }
}

// Mock context creators (implementations will vary based on EvidenceGatherer design)
use pmat::red_team::RepositoryContext;

fn create_mock_repo_context() -> RepositoryContext {
    RepositoryContext::new_mock()
}

fn create_mock_context_with_coverage(coverage: f64) -> RepositoryContext {
    RepositoryContext::new_mock().with_coverage(coverage)
}

fn create_mock_context_with_subsequent_fixes(fixes: Vec<&str>) -> RepositoryContext {
    RepositoryContext::new_mock()
        .with_subsequent_commits(fixes.iter().map(|s| s.to_string()).collect())
}

fn create_mock_context_clean() -> RepositoryContext {
    RepositoryContext::new_mock()
        .with_subsequent_commits(vec![])
        .with_test_results(true, 0) // All passing, 0 ignored
}

fn create_mock_context_with_broken_links(count: usize) -> RepositoryContext {
    RepositoryContext::new_mock().with_broken_links(count)
}

fn create_mock_context_with_vulnerabilities(count: usize) -> RepositoryContext {
    RepositoryContext::new_mock().with_vulnerabilities(count)
}

fn create_mock_context_no_benchmarks() -> RepositoryContext {
    RepositoryContext::new_mock().with_benchmarks(None)
}

fn create_mock_context_with_issue_reopened(issue_num: u32) -> RepositoryContext {
    RepositoryContext::new_mock().with_issue_status(issue_num, "reopened")
}

fn create_mock_context_with_old_system_refs(old_system: &str, count: usize) -> RepositoryContext {
    RepositoryContext::new_mock().with_code_grep_results(old_system, count)
}

fn create_mock_context_with_multiple_contradictions() -> RepositoryContext {
    RepositoryContext::new_mock()
        .with_subsequent_commits(vec!["fix: tests".to_string()])
        .with_test_results(false, 5) // 5 ignored tests
        .with_coverage(65.0) // Low coverage
}

fn create_mock_context_with_timestamps() -> RepositoryContext {
    RepositoryContext::new_mock().with_commit_timestamps(vec![1000, 2000, 3000])
}

fn create_mock_context_with_error() -> RepositoryContext {
    RepositoryContext::new_mock().with_coverage_error("Coverage tool failed")
}
