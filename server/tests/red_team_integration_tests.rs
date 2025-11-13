// EXTREME TDD: Red Team Mode Integration Tests (RED Phase)
//
// Integration tests combining ClaimExtractor + IntentClassifier
// Tests end-to-end hallucination detection workflow

use pmat::red_team::{
    ClaimCategory, ClaimExtractor, CommitInfo, CommitIntent, IntentClassifier, TestChanges,
};

// Helper to create commit info
fn create_commit(
    message: &str,
    timestamp: i64,
    files: Vec<&str>,
    issue_num: Option<u32>,
    issue_created: Option<i64>,
    branch: &str,
    added_tests: usize,
    fixed_tests: usize,
) -> CommitInfo {
    CommitInfo {
        message: message.to_string(),
        timestamp_seconds: timestamp,
        modified_files: files.iter().map(|s| s.to_string()).collect(),
        issue_number: issue_num,
        issue_created_timestamp: issue_created,
        branch: branch.to_string(),
        test_changes: TestChanges {
            added_tests,
            fixed_tests,
            modified_test_files: vec![],
        },
    }
}

// Integration Test 1: Detect "all tests passing" hallucination
#[test]
fn test_detect_all_tests_passing_hallucination() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit claims all tests passing
    let original_commit = create_commit(
        "feat: Complete feature X - all tests passing",
        1000,
        vec!["src/feature_x.rs"],
        None,
        None,
        "feature/x",
        0,
        0,
    );

    // Extract claims
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 2); // FeatureCompletion + TestStatus
    assert_eq!(claims[0].category, ClaimCategory::FeatureCompletion);
    assert_eq!(claims[1].category, ClaimCategory::TestStatus);
    assert!(claims[1].is_absolute); // "all tests passing" is absolute

    // Follow-up commit fixes failing tests
    let followup_commit = create_commit(
        "fix: Fix failing tests in feature X",
        1000 + (100 * 3600), // 100 hours later (after grace period)
        vec!["src/feature_x.rs", "tests/feature_x_tests.rs"],
        Some(42),
        Some(5000), // Issue created after original commit
        "hotfix/feature-x-tests",
        0,
        5, // Fixed 5 tests
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Should detect as hallucination fix
    assert_eq!(classification.intent, CommitIntent::HallucinationFix);
    assert!(classification.confidence > 0.7); // High confidence
    assert_eq!(classification.signals.len(), 5); // All 5 signals present

    // This scenario indicates a HALLUCINATION: claimed "all tests passing" but tests were broken
}

// Integration Test 2: Planned iteration should NOT be flagged
#[test]
fn test_planned_iteration_not_flagged_as_hallucination() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit with qualified claim
    let original_commit = create_commit(
        "feat: Complete feature X (MVP - Sprint 42)\n\nScope: Basic functionality\nDeferred: Advanced features (Sprint 43)",
        1000,
        vec!["src/feature_x.rs"],
        None,
        None,
        "feature/x",
        0,
        0,
    );

    // Extract claims
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 1); // FeatureCompletion
    assert!(claims[0].has_scope_qualifier); // Has "MVP - Sprint 42" qualifier

    // Follow-up commit adds planned advanced features
    let followup_commit = create_commit(
        "feat: Add advanced features to feature X (Sprint 43)",
        1000 + (14 * 24 * 3600), // 14 days later (next sprint)
        vec!["src/feature_x.rs", "src/feature_x_advanced.rs"],
        Some(43),
        Some(500), // Issue existed before original commit (pre-planned)
        "feature/x",
        10, // Added 10 new tests
        0,
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Should detect as planned iteration
    assert_eq!(classification.intent, CommitIntent::PlannedIteration);

    // This scenario is CORRECT: original claim was qualified, follow-up was planned
}

// Integration Test 3: Detect coverage hallucination
#[test]
fn test_detect_coverage_hallucination() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit claims 85% coverage
    let original_commit = create_commit(
        "test: Coverage stable at 85%",
        1000,
        vec!["tests/integration_tests.rs"],
        None,
        None,
        "master",
        0,
        0,
    );

    // Extract claims
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::Coverage);
    assert_eq!(claims[0].numeric_value, Some(85.0));

    // Follow-up commit reveals coverage was actually lower
    let followup_commit = create_commit(
        "fix: Add missing tests to reach 85% coverage",
        1000 + (48 * 3600), // 48 hours later (right at grace period boundary)
        vec!["tests/integration_tests.rs", "tests/unit_tests.rs"],
        None,
        None,
        "master",
        15, // Adding many missing tests
        0,
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Should likely detect as hallucination (adding missing tests suggests coverage wasn't 85%)
    // But it's within grace period, so might be uncertain or planned iteration
    assert!(
        classification.intent == CommitIntent::HallucinationFix
            || classification.intent == CommitIntent::PlannedIteration
    );
}

// Integration Test 4: Multiple claims with mixed validity
#[test]
fn test_multiple_claims_mixed_validity() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit with multiple claims
    let original_commit = create_commit(
        "feat: Complete user authentication\n\nAll tests passing\nCoverage at 90%\nZero vulnerabilities",
        1000,
        vec!["src/auth.rs"],
        None,
        None,
        "feature/auth",
        0,
        0,
    );

    // Extract claims
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 4); // FeatureCompletion, TestStatus, Coverage, Security

    // All claims are absolute
    assert!(claims.iter().all(|c| c.is_absolute || c.numeric_value.is_some()));

    // Follow-up commit fixes multiple issues
    let followup_commit = create_commit(
        "fix: Fix auth bugs and security issues",
        1000 + (200 * 3600), // Many days later
        vec!["src/auth.rs"],
        Some(123),
        Some(5000), // Issue created after
        "hotfix/auth-bugs",
        0,
        8, // Fixed many tests
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Should detect as hallucination fix (multiple false claims)
    assert_eq!(classification.intent, CommitIntent::HallucinationFix);
    assert!(classification.confidence > 0.7);
}

// Integration Test 5: Refactoring with no hallucination
#[test]
fn test_refactoring_no_hallucination() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit with basic implementation
    let original_commit = create_commit(
        "feat: Basic parser implementation",
        1000,
        vec!["src/parser.rs"],
        None,
        None,
        "feature/parser",
        0,
        0,
    );

    // Extract claims - should be minimal (no absolute claims)
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 0); // No testable claims (vague message)

    // Follow-up commit improves code
    let followup_commit = create_commit(
        "refactor: Improve parser performance and error handling",
        1000 + (5 * 24 * 3600), // 5 days later
        vec!["src/parser.rs", "src/parser_errors.rs"],
        None,
        None,
        "feature/parser",
        5, // Adding tests
        0,
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Should detect as planned iteration (refactoring)
    assert_eq!(classification.intent, CommitIntent::PlannedIteration);

    // No hallucination here - original claim was modest, follow-up is planned improvement
}

// Integration Test 6: Grace period prevents false positive
#[test]
fn test_grace_period_prevents_false_positive() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit with absolute claim
    let original_commit = create_commit(
        "feat: Complete feature Y",
        1000,
        vec!["src/feature_y.rs"],
        None,
        None,
        "feature/y",
        0,
        0,
    );

    // Extract claims
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 1);
    assert!(claims[0].is_absolute); // "Complete" is absolute

    // Quick follow-up within grace period (developer catches issue immediately)
    let followup_commit = create_commit(
        "chore: Polish feature Y implementation", // Neutral keyword (not "fix")
        1000 + (6 * 3600), // 6 hours later (within 48-hour grace period)
        vec!["src/feature_y.rs", "src/feature_y_utils.rs"], // Some new files
        None,
        None,
        "feature/y", // Same branch
        2, // Adding tests
        0,
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Grace period + same branch + adding tests should favor planned iteration
    assert_eq!(classification.intent, CommitIntent::PlannedIteration);

    // Not a hallucination - just part of active development
}

// Integration Test 7: Cross-branch fix indicates hallucination
#[test]
fn test_cross_branch_fix_indicates_hallucination() {
    let claim_extractor = ClaimExtractor::new();
    let intent_classifier = IntentClassifier::new();

    // Original commit merged to master
    let original_commit = create_commit(
        "feat: Complete feature Z - production ready",
        1000,
        vec!["src/feature_z.rs"],
        None,
        None,
        "master",
        0,
        0,
    );

    // Extract claims
    let claims = claim_extractor.extract(&original_commit.message);
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].category, ClaimCategory::FeatureCompletion);

    // Hotfix from different branch (indicates production issue)
    let followup_commit = create_commit(
        "fix: Critical bug in feature Z",
        1000 + (72 * 3600), // 3 days later
        vec!["src/feature_z.rs"],
        Some(999),
        Some(1000 + (70 * 3600)), // Issue created 70 hours after original
        "hotfix/feature-z-critical",
        0,
        3,
    );

    // Classify intent
    let classification = intent_classifier.classify(&original_commit, &followup_commit);

    // Should strongly indicate hallucination (hotfix branch, issue created after, after grace period)
    assert_eq!(classification.intent, CommitIntent::HallucinationFix);
    assert!(classification.confidence > 0.75);
}
