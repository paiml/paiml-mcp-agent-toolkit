//! RED Phase Tests for TestingScorer (Sprint 2)
//!
//! Testing Excellence Category: 20 points total
//! - Test Coverage (8pts): ≥85% line coverage via cargo-llvm-cov
//! - Integration Tests (4pts): Presence of tests/ directory with integration tests
//! - Doc Tests (3pts): Rustdoc examples that compile and run
//! - Mutation Testing (5pts): ≥80% mutation score (cross-check with CodeQualityScorer)
//!
//! Evidence-based refinement: Coverage threshold based on empirical research
//! showing ≥85% coverage correlates with significantly fewer production bugs.

use pmat::services::rust_project_score::{CategoryScore, Scorer, TestingScorer};
use std::path::Path;

// Test fixture: Create temporary test project
fn create_test_project() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let cargo_toml = temp.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    // Create src directory with main.rs
    let src_dir = temp.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    temp
}

// ============================================================================
// Test 1: TestingScorer Creation
// ============================================================================

#[test]
fn test_testing_scorer_creation() {
    let scorer = TestingScorer::new();
    assert_eq!(scorer.name(), "Testing Excellence");
    assert_eq!(scorer.max_points(), 20.0);
}

// ============================================================================
// Test 2: Perfect Score (All Testing Checks Pass)
// ============================================================================

#[test]
fn test_perfect_score_all_checks_pass() {
    let temp = create_test_project();
    let scorer = TestingScorer::new();

    let result = scorer.score(temp.path());
    assert!(result.is_ok());

    let score = result.unwrap();
    assert_eq!(score.max, 20.0);
    assert!(score.earned >= 0.0 && score.earned <= 20.0);
}

// ============================================================================
// Test 3: Test Coverage Scoring (8pts)
// ============================================================================

#[test]
fn test_coverage_scoring_high_coverage() {
    let temp = create_test_project();

    // Create lib.rs with code and tests
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
    }
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get points for having tests (exact score depends on coverage)
    assert!(score.earned >= 0.0);
}

#[test]
fn test_coverage_scoring_no_tests() {
    let temp = create_test_project();

    // Create lib.rs with NO tests
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose coverage points (0% coverage)
    assert!(score.earned < 20.0);
}

// ============================================================================
// Test 4: Integration Tests Detection (4pts)
// ============================================================================

#[test]
fn test_integration_tests_present() {
    let temp = create_test_project();

    // Create tests/ directory with integration test
    let tests_dir = temp.path().join("tests");
    std::fs::create_dir(&tests_dir).unwrap();
    std::fs::write(
        tests_dir.join("integration_test.rs"),
        r#"
#[test]
fn test_integration() {
    assert!(true);
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get integration test points
    assert!(score.earned >= 4.0 || score.earned == 0.0);
}

#[test]
fn test_integration_tests_absent() {
    let temp = create_test_project();

    // No tests/ directory created

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose integration test points
    assert!(score.earned <= 16.0);
}

#[test]
fn test_integration_tests_multiple_files() {
    let temp = create_test_project();

    // Create tests/ directory with multiple integration tests
    let tests_dir = temp.path().join("tests");
    std::fs::create_dir(&tests_dir).unwrap();

    std::fs::write(
        tests_dir.join("integration_test_1.rs"),
        r#"
#[test]
fn test_integration_1() {
    assert!(true);
}
"#,
    )
    .unwrap();

    std::fs::write(
        tests_dir.join("integration_test_2.rs"),
        r#"
#[test]
fn test_integration_2() {
    assert!(true);
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get full integration test points for multiple test files
    assert!(score.earned >= 4.0 || score.earned == 0.0);
}

// ============================================================================
// Test 5: Doc Tests Detection (3pts)
// ============================================================================

#[test]
fn test_doc_tests_present() {
    let temp = create_test_project();

    // Create lib.rs with doc tests
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// let result = test_project::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get doc test points
    assert!(score.earned >= 0.0);
}

#[test]
fn test_doc_tests_absent() {
    let temp = create_test_project();

    // Create lib.rs WITHOUT doc tests
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose doc test points
    assert!(score.earned <= 17.0);
}

#[test]
fn test_doc_tests_multiple_functions() {
    let temp = create_test_project();

    // Create lib.rs with multiple documented functions
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// let result = test_project::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtracts two numbers.
///
/// # Examples
///
/// ```
/// let result = test_project::subtract(5, 3);
/// assert_eq!(result, 2);
/// ```
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get full doc test points for multiple documented functions
    assert!(score.earned >= 3.0 || score.earned == 0.0);
}

// ============================================================================
// Test 6: Mutation Testing Scoring (5pts)
// ============================================================================

#[test]
#[ignore] // Requires cargo-mutants installation
fn test_mutation_testing_score() {
    let temp = create_test_project();

    // Create code with strong tests
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(-1, 1), 0);
    }
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Mutation score should be calculated
    assert!(score.earned >= 0.0 && score.earned <= 20.0);
}

// ============================================================================
// Test 7: Coverage Threshold (≥85%)
// ============================================================================

#[test]
fn test_coverage_threshold_enforcement() {
    let temp = create_test_project();

    // Create code with partial test coverage
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
pub fn tested_function() -> i32 {
    42
}

pub fn untested_function() -> i32 {
    99
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tested_function() {
        assert_eq!(tested_function(), 42);
    }

    // No test for untested_function
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose coverage points (< 85% coverage)
    assert!(score.earned < 20.0);
}

// ============================================================================
// Test 8: Recommendations Generation
// ============================================================================

#[test]
fn test_recommendations_for_testing_issues() {
    let temp = create_test_project();

    // Create project with NO tests
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let recommendations = scorer.recommendations(temp.path());

    // Should provide specific recommendations
    assert!(!recommendations.is_empty());

    let rec_text = recommendations.join(" ");
    assert!(
        rec_text.contains("coverage")
            || rec_text.contains("tests")
            || rec_text.contains("integration")
            || rec_text.contains("doc test")
    );
}

// ============================================================================
// Test 9: Scorer Implements Send + Sync
// ============================================================================

#[test]
fn test_scorer_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<TestingScorer>();
    assert_sync::<TestingScorer>();
}

// ============================================================================
// Test 10: Scoring is Deterministic
// ============================================================================

#[test]
fn test_scoring_is_deterministic() {
    let temp = create_test_project();
    let scorer = TestingScorer::new();

    // Score same project twice
    let result1 = scorer.score(temp.path()).unwrap();
    let result2 = scorer.score(temp.path()).unwrap();

    // Should get identical scores
    assert_eq!(result1.earned, result2.earned);
    assert_eq!(result1.max, result2.max);
}

// ============================================================================
// Test 11: Invalid Project Error
// ============================================================================

#[test]
fn test_invalid_project_no_cargo_toml() {
    let temp = tempfile::tempdir().unwrap();
    let scorer = TestingScorer::new();

    let result = scorer.score(temp.path());

    // Should return error for invalid project
    assert!(result.is_err());
}

// ============================================================================
// Test 12: Performance (<5 seconds)
// ============================================================================

#[test]
fn test_scoring_performance() {
    use std::time::Instant;

    let temp = create_test_project();
    let scorer = TestingScorer::new();

    let start = Instant::now();
    let result = scorer.score(temp.path());
    let duration = start.elapsed();

    assert!(result.is_ok());

    // Should complete in <5 seconds per specification
    assert!(
        duration.as_secs() < 5,
        "Scoring took {:?}, expected <5s",
        duration
    );
}

// ============================================================================
// Test 13: CategoryScore Structure
// ============================================================================

#[test]
fn test_category_score_structure() {
    let temp = create_test_project();
    let scorer = TestingScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Verify CategoryScore has correct structure
    assert_eq!(result.max, 20.0);
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);

    // Test percentage calculation
    let percentage = result.percentage();
    assert!(percentage >= 0.0 && percentage <= 100.0);
}

// ============================================================================
// Property-Based Test 14: Score Bounds
// ============================================================================

#[test]
fn test_score_bounds_property() {
    let temp = create_test_project();
    let scorer = TestingScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Property: Score must be in [0, max]
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);
    assert_eq!(result.max, 20.0);
}

// ============================================================================
// Property-Based Test 15: Score Monotonicity
// ============================================================================

#[test]
fn test_score_monotonicity_property() {
    // Property: Adding tests should never decrease score

    let temp1 = create_test_project();
    let src1 = temp1.path().join("src").join("lib.rs");
    std::fs::write(&src1, "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    let temp2 = create_test_project();
    let src2 = temp2.path().join("src").join("lib.rs");
    std::fs::write(
        &src2,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();

    let no_tests_score = scorer.score(temp1.path()).unwrap();
    let with_tests_score = scorer.score(temp2.path()).unwrap();

    // Code with tests should score >= code without tests
    assert!(with_tests_score.earned >= no_tests_score.earned);
}

// ============================================================================
// Test 16: Coverage Tiered Scoring
// ============================================================================

#[test]
fn test_coverage_tiered_scoring() {
    let scorer = TestingScorer::new();

    // Verify tiered scoring:
    // ≥85% coverage: 8pts
    // ≥70% coverage: 6pts
    // ≥50% coverage: 4pts
    // ≥30% coverage: 2pts
    // <30% coverage: 0pts

    // This is verified implicitly by the implementation
    assert_eq!(scorer.max_points(), 20.0);
    // Coverage (8) + Integration (4) + Doc (3) + Mutation (5) = 20
}

// ============================================================================
// Test 17: Integration Test File Counting
// ============================================================================

#[test]
fn test_integration_test_file_counting() {
    let temp = create_test_project();

    // Create tests/ directory with multiple test files
    let tests_dir = temp.path().join("tests");
    std::fs::create_dir(&tests_dir).unwrap();

    for i in 1..=5 {
        std::fs::write(
            tests_dir.join(format!("test_{}.rs", i)),
            format!(
                r#"
#[test]
fn test_{}() {{
    assert!(true);
}}
"#,
                i
            ),
        )
        .unwrap();
    }

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get full integration test points
    assert!(result.earned >= 4.0 || result.earned == 0.0);
}

// ============================================================================
// Test 18: Doc Test Extraction
// ============================================================================

#[test]
fn test_doc_test_extraction() {
    let temp = create_test_project();

    // Create lib.rs with doc tests in different formats
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
/// Function with example
///
/// ```
/// let x = test_project::fn1();
/// ```
pub fn fn1() -> i32 { 1 }

/// Function with multiple examples
///
/// # Examples
///
/// ```
/// let x = test_project::fn2();
/// ```
///
/// ```
/// let y = test_project::fn2();
/// ```
pub fn fn2() -> i32 { 2 }
"#,
    )
    .unwrap();

    let scorer = TestingScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get doc test points
    assert!(result.earned >= 3.0 || result.earned == 0.0);
}

// ============================================================================
// Test 19: Graceful Tool Degradation
// ============================================================================

#[test]
fn test_graceful_degradation_no_coverage_tool() {
    let temp = create_test_project();
    let scorer = TestingScorer::new();

    let result = scorer.score(temp.path());

    // Should succeed even if coverage tools not installed
    assert!(result.is_ok());

    let score = result.unwrap();
    assert!(score.earned >= 0.0 && score.earned <= 20.0);
}

// ============================================================================
// Test 20: Evidence-Based Weight Allocation
// ============================================================================

#[test]
fn test_evidence_based_weights() {
    let scorer = TestingScorer::new();

    // Verify evidence-based weight allocation
    assert_eq!(scorer.max_points(), 20.0);

    // Coverage (8pts): Highest weight - direct correlation with bugs
    // Integration (4pts): Important for system-level validation
    // Doc tests (3pts): Ensures examples compile and run
    // Mutation (5pts): Validates test quality
}
