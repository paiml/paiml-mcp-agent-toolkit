// EXTREME TDD: Intent Classifier Tests (RED Phase)
//
// Test-Driven Development for distinguishing hallucination fixes from planned iterations
// Specification: Section 2.1 - Multi-Signal Temporal Analysis
// Target: <5% false positive rate

use pmat::red_team::{CommitInfo, CommitIntent, IntentClassifier, TestChanges};

// Helper function to create test commits
fn create_commit(
    message: &str,
    timestamp_seconds: i64,
    files: Vec<&str>,
    issue_num: Option<u32>,
    issue_created: Option<i64>,
    branch: &str,
    added_tests: usize,
    fixed_tests: usize,
) -> CommitInfo {
    CommitInfo {
        message: message.to_string(),
        timestamp_seconds,
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

// RED Test 1: Hallucination fix detected via "fix" keyword
#[test]
fn test_detect_hallucination_fix_via_keyword() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: All tests passing",
        1000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/auth",
        0,
        0,
    );

    let followup = create_commit(
        "fix: Actually fix failing tests",
        2000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/auth",
        0,
        5, // Fixed 5 tests
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::HallucinationFix);
    assert!(result.confidence > 0.5);
}

// RED Test 2: Planned iteration detected via "refactor" keyword
#[test]
fn test_detect_planned_iteration_via_keyword() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Complete user authentication",
        1000,
        vec!["src/auth.rs"],
        None,
        None,
        "feature/auth",
        0,
        0,
    );

    let followup = create_commit(
        "refactor: Improve authentication error handling",
        2000,
        vec!["src/auth.rs"],
        None,
        None,
        "feature/auth",
        3, // Added 3 tests
        0,
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::PlannedIteration);
    assert!(result.confidence > 0.5);
}

// RED Test 3: Issue created AFTER original commit = hallucination
#[test]
fn test_issue_created_after_commit_indicates_hallucination() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Bug-free implementation",
        1000,
        vec!["src/lib.rs"],
        None,
        None,
        "master",
        0,
        0,
    );

    let followup = create_commit(
        "fix: Resolve issue #42",
        5000,
        vec!["src/lib.rs"],
        Some(42),
        Some(2000), // Issue created at 2000, after original commit at 1000
        "master",
        0,
        1,
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::HallucinationFix);
    assert!(result.confidence > 0.7); // High confidence from issue tracker signal
}

// RED Test 4: Pre-existing issue = planned work
#[test]
fn test_preexisting_issue_indicates_planned_work() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Implement feature X",
        5000,
        vec!["src/feature_x.rs"],
        None,
        None,
        "feature/x",
        0,
        0,
    );

    let followup = create_commit(
        "feat: Add error handling for feature X (closes #42)",
        10000,
        vec!["src/feature_x.rs"],
        Some(42),
        Some(1000), // Issue created at 1000, before original commit at 5000
        "feature/x",
        2,
        0,
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::PlannedIteration);
    assert!(result.confidence > 0.6);
}

// RED Test 5: High file overlap (>80%) = hallucination fix
#[test]
fn test_high_file_overlap_indicates_hallucination_fix() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Complete module implementation",
        1000,
        vec!["src/mod1.rs", "src/mod2.rs", "src/mod3.rs"],
        None,
        None,
        "feature/module",
        0,
        0,
    );

    let followup = create_commit(
        "fix: Correct module logic",        // Hallucination keyword
        1000 + (100 * 3600),                // After grace period
        vec!["src/mod1.rs", "src/mod2.rs"], // 100% of followup files overlap
        None,
        None,
        "hotfix/module-bug", // Different branch
        0,
        2, // Fixed tests
    );

    let result = classifier.classify(&original, &followup);

    // High overlap + hallucination signals = hallucination fix
    assert_eq!(result.intent, CommitIntent::HallucinationFix);
}

// RED Test 6: Low file overlap (<20%) = planned iteration
#[test]
fn test_low_file_overlap_indicates_planned_iteration() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Complete module A",
        1000,
        vec!["src/mod_a.rs"],
        None,
        None,
        "feature/modules",
        0,
        0,
    );

    let followup = create_commit(
        "feat: Add module B",
        2000,
        vec![
            "src/mod_b.rs",
            "src/mod_c.rs",
            "src/mod_d.rs",
            "src/mod_e.rs",
            "src/mod_f.rs",
        ],
        None,
        None,
        "feature/modules",
        10, // Added tests
        0,
    );

    let result = classifier.classify(&original, &followup);

    // 0% overlap suggests new work
    assert_eq!(result.intent, CommitIntent::PlannedIteration);
}

// RED Test 7: More test fixes than additions = hallucination
#[test]
fn test_test_fixes_indicate_hallucination() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "test: All tests passing",
        1000,
        vec!["tests/integration.rs"],
        None,
        None,
        "master",
        0,
        0,
    );

    let followup = create_commit(
        "test: Fix test failures",
        2000,
        vec!["tests/integration.rs"],
        None,
        None,
        "master",
        1,  // Added 1 test
        10, // Fixed 10 tests
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::HallucinationFix);
}

// RED Test 8: More test additions than fixes = planned iteration
#[test]
fn test_test_additions_indicate_planned_iteration() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Basic feature implementation",
        1000,
        vec!["src/feature.rs"],
        None,
        None,
        "feature/new",
        0,
        0,
    );

    let followup = create_commit(
        "test: Expand test coverage",
        2000,
        vec!["tests/feature_tests.rs"],
        None,
        None,
        "feature/new",
        15, // Added 15 tests
        2,  // Fixed 2 tests
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::PlannedIteration);
}

// RED Test 9: Within 48-hour grace period = planned iteration
#[test]
fn test_grace_period_indicates_planned_iteration() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Complete feature X",
        1000,
        vec!["src/feature_x.rs"],
        None,
        None,
        "feature/x",
        0,
        0,
    );

    let followup = create_commit(
        "refactor: Improve error handling in feature X", // Iteration keyword
        1000 + (24 * 3600), // 24 hours later (within 48-hour grace period)
        vec!["src/feature_x.rs", "src/feature_x_errors.rs"], // Some overlap, some new
        None,
        None,
        "feature/x", // Same branch
        5,           // Adding tests
        0,
    );

    let result = classifier.classify(&original, &followup);

    // Grace period + iteration signals should favor planned iteration
    assert_eq!(result.intent, CommitIntent::PlannedIteration);
}

// RED Test 10: After grace period + different branch = hallucination
#[test]
fn test_after_grace_period_different_branch_indicates_hallucination() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Complete feature X",
        1000,
        vec!["src/feature_x.rs"],
        None,
        None,
        "feature/x",
        0,
        0,
    );

    let followup = create_commit(
        "fix: Bug in feature X",
        1000 + (72 * 3600), // 72 hours later (after 48-hour grace period)
        vec!["src/feature_x.rs"],
        None,
        None,
        "hotfix/feature-x-bug", // Different branch
        0,
        3,
    );

    let result = classifier.classify(&original, &followup);

    // After grace period + different branch suggests hallucination fix
    assert_eq!(result.intent, CommitIntent::HallucinationFix);
}

// RED Test 11: Same branch (within grace period) = planned iteration
#[test]
fn test_same_branch_indicates_planned_iteration() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Initial implementation",
        1000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/new-module",
        0,
        0,
    );

    let followup = create_commit(
        "feat: Add error handling",
        2000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/new-module", // Same branch
        3,
        0,
    );

    let result = classifier.classify(&original, &followup);

    // Same branch suggests related planned work
    assert_eq!(result.intent, CommitIntent::PlannedIteration);
}

// RED Test 12: Mixed signals = uncertain
#[test]
fn test_mixed_signals_result_in_uncertain() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: New feature",
        1000,
        vec!["src/feature.rs"],
        None,
        None,
        "feature/new",
        0,
        0,
    );

    let followup = create_commit(
        "chore: Update code", // Neutral keyword
        1000 + (100 * 3600),  // After grace period
        vec!["src/other.rs"], // No file overlap
        None,
        None,
        "master", // Different branch
        5,        // Equal additions
        5,        // and fixes
    );

    let result = classifier.classify(&original, &followup);

    // Conflicting signals should result in uncertain
    assert_eq!(result.intent, CommitIntent::Uncertain);
}

// RED Test 13: All signals agree on hallucination = high confidence
#[test]
fn test_unanimous_hallucination_signals_high_confidence() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: All tests passing, bug-free",
        1000,
        vec!["src/lib.rs", "src/mod.rs"],
        None,
        None,
        "master",
        0,
        0,
    );

    let followup = create_commit(
        "fix: Critical bug in lib.rs (fixes #123)",
        1000 + (100 * 3600),              // After grace period
        vec!["src/lib.rs", "src/mod.rs"], // 100% overlap
        Some(123),
        Some(5000),            // Issue created after original commit
        "hotfix/critical-bug", // Different branch
        0,
        10, // Fixed many tests
    );

    let result = classifier.classify(&original, &followup);

    assert_eq!(result.intent, CommitIntent::HallucinationFix);
    assert!(result.confidence > 0.75); // High confidence when all signals agree
}

// RED Test 14: Confidence score is between 0 and 1
#[test]
fn test_confidence_score_bounds() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Some feature",
        1000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/test",
        0,
        0,
    );

    let followup = create_commit(
        "refactor: Improve code",
        2000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/test",
        1,
        0,
    );

    let result = classifier.classify(&original, &followup);

    assert!(result.confidence >= 0.0);
    assert!(result.confidence <= 1.0);
}

// RED Test 15: Classification includes all 5 signals
#[test]
fn test_classification_includes_all_signals() {
    let classifier = IntentClassifier::new();

    let original = create_commit(
        "feat: Feature complete",
        1000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/test",
        0,
        0,
    );

    let followup = create_commit(
        "fix: Bug fix",
        2000,
        vec!["src/lib.rs"],
        None,
        None,
        "feature/test",
        0,
        1,
    );

    let result = classifier.classify(&original, &followup);

    // Should have 5 signals:
    // 1. commit_message, 2. issue_linkage, 3. code_churn, 4. test_changes, 5. temporal_context
    assert_eq!(result.signals.len(), 5);

    let signal_names: Vec<_> = result
        .signals
        .iter()
        .map(|s| s.signal_name.as_str())
        .collect();
    assert!(signal_names.contains(&"commit_message"));
    assert!(signal_names.contains(&"issue_linkage"));
    assert!(signal_names.contains(&"code_churn"));
    assert!(signal_names.contains(&"test_changes"));
    assert!(signal_names.contains(&"temporal_context"));
}
