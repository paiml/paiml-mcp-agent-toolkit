// EXTREME TDD: Red Team Mode Integration Tests
//
// Integration tests for the Red Team Mode hallucination detection system
// Specification: docs/specifications/red-team-mode-spec.md v1.1

use pmat::red_team::{ClaimExtractor, Claim, ClaimCategory};

// RED Test 1: Extract "all tests passing" claim
#[test]
fn test_extract_all_tests_passing_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "fix: All tests passing";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::TestStatus);
    assert_eq!(claims[0].text, "All tests passing");
    assert!(claims[0].is_absolute);
}

// RED Test 2: Extract coverage percentage claim
#[test]
fn test_extract_coverage_percentage_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "test: Coverage stable at 85%";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::Coverage);
    assert_eq!(claims[0].text, "Coverage stable at 85%");
    assert_eq!(claims[0].numeric_value, Some(85.0));
}

// RED Test 3: Extract feature completion claim
#[test]
fn test_extract_feature_completion_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "feat: Complete user authentication (MVP - Sprint 42)";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::FeatureCompletion);
    assert_eq!(claims[0].text, "Complete user authentication");
    assert!(claims[0].has_scope_qualifier); // "MVP" qualifier
}

// RED Test 4: Extract "fixed all broken links" claim
#[test]
fn test_extract_documentation_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "docs: Fix all broken documentation links (18 → 0)";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::Documentation);
    assert_eq!(claims[0].text, "Fix all broken documentation links");
    assert!(claims[0].is_absolute); // "all" keyword
}

// RED Test 5: Extract migration completion claim
#[test]
fn test_extract_migration_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "feat: Complete migration to libsql";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::Migration);
    assert_eq!(claims[0].text, "Complete migration to libsql");
    assert!(claims[0].is_absolute);
}

// RED Test 6: Extract bug fix claim
#[test]
fn test_extract_bug_fix_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "fix: Resolve issue #42 - parser bug";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::BugFix);
    assert_eq!(claims[0].text, "Resolve issue #42");
    assert_eq!(claims[0].issue_number, Some(42));
}

// RED Test 7: Extract performance claim
#[test]
fn test_extract_performance_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "perf: 50% faster parsing";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::Performance);
    assert_eq!(claims[0].text, "50% faster parsing");
    assert_eq!(claims[0].numeric_value, Some(50.0));
}

// RED Test 8: Extract security claim
#[test]
fn test_extract_security_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "security: Zero vulnerabilities after audit";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::Security);
    assert_eq!(claims[0].text, "Zero vulnerabilities");
    assert!(claims[0].is_absolute);
}

// RED Test 9: Multiple claims in single commit
#[test]
fn test_extract_multiple_claims() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "feat: Complete X feature\n\nAll tests passing\nCoverage at 90%";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 3);
    assert_eq!(claims[0].category, ClaimCategory::FeatureCompletion);
    assert_eq!(claims[1].category, ClaimCategory::TestStatus);
    assert_eq!(claims[2].category, ClaimCategory::Coverage);
}

// RED Test 10: No testable claims (vague commit message)
#[test]
fn test_extract_no_claims_from_vague_message() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "refactor: Update code";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 0);
}

// RED Test 11: Qualified claim (not absolute)
#[test]
fn test_extract_qualified_claim() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "fix: Most tests passing (293/309)";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert!(!claims[0].is_absolute); // "Most" is not absolute
    assert_eq!(claims[0].numeric_value, Some(293.0));
}

// RED Test 12: Phase marker detection
#[test]
fn test_detect_phase_marker() {
    let extractor = ClaimExtractor::new();
    let commit_msg = "feat: Phase 1 complete - read operations";

    let claims = extractor.extract(commit_msg);

    assert_eq!(claims.len(), 1);
    assert!(claims[0].has_scope_qualifier);
    assert_eq!(claims[0].scope, Some("Phase 1".to_string()));
}
