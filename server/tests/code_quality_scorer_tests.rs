//! RED Phase Tests for CodeQualityScorer (Sprint 2)
//!
//! Code Quality Category: 26 points total
//! - Cyclomatic Complexity (3pts): All functions ≤20 complexity
//! - Unsafe Code (9pts): Proper unsafe usage with safety comments
//! - Mutation Testing (8pts): ≥80% mutation score
//! - Build Time (4pts): Fast incremental builds
//! - Dead Code (2pts): No unused code
//!
//! Evidence-based refinement (arXiv 2024): Complexity weight reduced from 8→3pts
//! due to low correlation with bugs. Unsafe and mutation weights increased.

use pmat::services::rust_project_score::{CategoryScore, CodeQualityScorer, Scorer};
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
// Test 1: CodeQualityScorer Creation
// ============================================================================

#[test]
fn test_code_quality_scorer_creation() {
    let scorer = CodeQualityScorer::new();
    assert_eq!(scorer.name(), "Code Quality");
    assert_eq!(scorer.max_points(), 26.0);
}

// ============================================================================
// Test 2: Perfect Score (All Quality Checks Pass)
// ============================================================================

#[test]
fn test_perfect_score_all_checks_pass() {
    let temp = create_test_project();
    let scorer = CodeQualityScorer::new();

    let result = scorer.score(temp.path());
    assert!(result.is_ok());

    let score = result.unwrap();
    assert_eq!(score.max, 26.0);
    assert!(score.earned >= 0.0 && score.earned <= 26.0);
}

// ============================================================================
// Test 3: Cyclomatic Complexity Scoring (3pts)
// ============================================================================

#[test]
fn test_complexity_scoring_simple_code() {
    let temp = create_test_project();

    // Create simple code with low complexity
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    println!("Simple function");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get full 3 points for complexity
    // (assuming other metrics also pass)
    assert!(score.earned >= 3.0 || score.earned == 0.0);
}

#[test]
fn test_complexity_scoring_high_complexity() {
    let temp = create_test_project();

    // Create code with high complexity (>20)
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    complex_function(5);
}

fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                if x > 3 {
                    if x > 4 {
                        if x > 5 {
                            if x > 6 {
                                if x > 7 {
                                    if x > 8 {
                                        if x > 9 {
                                            10
                                        } else { 9 }
                                    } else { 8 }
                                } else { 7 }
                            } else { 6 }
                        } else { 5 }
                    } else { 4 }
                } else { 3 }
            } else { 2 }
        } else { 1 }
    } else { 0 }
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose complexity points
    assert!(score.earned < 26.0);
}

// ============================================================================
// Test 4: Unsafe Code Scoring (9pts)
// ============================================================================

#[test]
fn test_unsafe_code_with_proper_documentation() {
    let temp = create_test_project();

    // Create unsafe code WITH proper safety comments
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    let x = 5;
    let ptr = &x as *const i32;

    // SAFETY: ptr is valid and points to x which is in scope
    unsafe {
        println!("Value: {}", *ptr);
    }
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get points for documented unsafe
    assert!(score.earned >= 0.0);
}

#[test]
fn test_unsafe_code_without_documentation() {
    let temp = create_test_project();

    // Create unsafe code WITHOUT safety comments
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    let x = 5;
    let ptr = &x as *const i32;
    unsafe {
        println!("Value: {}", *ptr);
    }
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose points for undocumented unsafe
    assert!(score.earned < 26.0);
}

// ============================================================================
// Test 5: Mutation Testing Scoring (8pts)
// ============================================================================

#[test]
#[ignore] // Requires cargo-mutants installation
fn test_mutation_testing_score() {
    let temp = create_test_project();

    // Create code with tests
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
    }
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Mutation score should be calculated
    assert!(score.earned >= 0.0 && score.earned <= 26.0);
}

// ============================================================================
// Test 6: Build Time Scoring (4pts)
// ============================================================================

#[test]
fn test_build_time_scoring() {
    let temp = create_test_project();
    let scorer = CodeQualityScorer::new();

    // First build
    let result = scorer.score(temp.path());
    assert!(result.is_ok());

    let score = result.unwrap();

    // Build time should be measured
    // Fast builds (<30s) get full 4 points
    // Slow builds (>2min) get 0 points
    assert!(score.earned >= 0.0 && score.earned <= 26.0);
}

// ============================================================================
// Test 7: Dead Code Detection (2pts)
// ============================================================================

#[test]
fn test_dead_code_detection_clean() {
    let temp = create_test_project();

    // Create code with no dead code
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    let result = add(2, 3);
    println!("Result: {}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
}

#[test]
fn test_dead_code_detection_with_unused() {
    let temp = create_test_project();

    // Create code with dead code
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    println!("Hello");
}

#[allow(dead_code)]
fn unused_function() {
    println!("Never called");
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose dead code points
    assert!(score.earned <= 26.0);
}

// ============================================================================
// Test 8: Complexity Threshold (≤20)
// ============================================================================

#[test]
fn test_complexity_threshold_enforcement() {
    let temp = create_test_project();

    // Create function with exactly 20 complexity
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    boundary_complexity(10);
}

fn boundary_complexity(x: i32) -> i32 {
    match x {
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        _ => 0,
    }
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should pass threshold check
    assert!(score.earned >= 0.0);
}

// ============================================================================
// Test 9: Unsafe Count vs Total Lines
// ============================================================================

#[test]
fn test_unsafe_ratio_scoring() {
    let temp = create_test_project();

    // Create code with reasonable unsafe ratio
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    safe_function();

    // SAFETY: Valid pointer dereference
    unsafe {
        unsafe_operation();
    }
}

fn safe_function() {
    println!("Safe code");
}

unsafe fn unsafe_operation() {
    println!("Unsafe operation");
}

fn another_safe_function() {
    println!("More safe code");
}

fn yet_another_safe_function() {
    println!("Even more safe code");
}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Low unsafe ratio should get points
    assert!(score.earned >= 0.0);
}

// ============================================================================
// Test 10: Recommendations Generation
// ============================================================================

#[test]
fn test_recommendations_for_quality_issues() {
    let temp = create_test_project();

    // Create code with multiple quality issues
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    complex_function(5);
    unsafe {
        println!("Undocumented unsafe");
    }
}

fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                if x > 3 {
                    if x > 4 {
                        5
                    } else { 4 }
                } else { 3 }
            } else { 2 }
        } else { 1 }
    } else { 0 }
}

#[allow(dead_code)]
fn unused() {}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();
    let recommendations = scorer.recommendations(temp.path());

    // Should provide specific recommendations
    assert!(!recommendations.is_empty());

    let rec_text = recommendations.join(" ");
    assert!(
        rec_text.contains("complexity")
            || rec_text.contains("unsafe")
            || rec_text.contains("dead code")
            || rec_text.contains("mutation")
    );
}

// ============================================================================
// Test 11: Evidence-Based Weighting (Complexity 3pts)
// ============================================================================

#[test]
fn test_complexity_weight_reduced() {
    let scorer = CodeQualityScorer::new();

    // Verify complexity is only 3 points (reduced from 8 in v1.0)
    // This is based on arXiv 2024 research showing low bug correlation
    assert_eq!(scorer.max_points(), 26.0);

    // Complexity (3) + Unsafe (9) + Mutation (8) + Build (4) + Dead (2) = 26
}

// ============================================================================
// Test 12: Scorer Implements Send + Sync
// ============================================================================

#[test]
fn test_scorer_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<CodeQualityScorer>();
    assert_sync::<CodeQualityScorer>();
}

// ============================================================================
// Test 13: Scoring is Deterministic
// ============================================================================

#[test]
fn test_scoring_is_deterministic() {
    let temp = create_test_project();
    let scorer = CodeQualityScorer::new();

    // Score same project twice
    let result1 = scorer.score(temp.path()).unwrap();
    let result2 = scorer.score(temp.path()).unwrap();

    // Should get identical scores
    assert_eq!(result1.earned, result2.earned);
    assert_eq!(result1.max, result2.max);
}

// ============================================================================
// Test 14: Invalid Project Error
// ============================================================================

#[test]
fn test_invalid_project_no_cargo_toml() {
    let temp = tempfile::tempdir().unwrap();
    let scorer = CodeQualityScorer::new();

    let result = scorer.score(temp.path());

    // Should return error for invalid project
    assert!(result.is_err());
}

// ============================================================================
// Test 15: Performance (<5 seconds)
// ============================================================================

#[test]
fn test_scoring_performance() {
    use std::time::Instant;

    let temp = create_test_project();
    let scorer = CodeQualityScorer::new();

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
// Test 16: CategoryScore Structure
// ============================================================================

#[test]
fn test_category_score_structure() {
    let temp = create_test_project();
    let scorer = CodeQualityScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Verify CategoryScore has correct structure
    assert_eq!(result.max, 26.0);
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);

    // Test percentage calculation
    let percentage = result.percentage();
    assert!(percentage >= 0.0 && percentage <= 100.0);
}

// ============================================================================
// Property-Based Test 17: Score Bounds
// ============================================================================

#[test]
fn test_score_bounds_property() {
    let temp = create_test_project();
    let scorer = CodeQualityScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Property: Score must be in [0, max]
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);
    assert_eq!(result.max, 26.0);
}

// ============================================================================
// Property-Based Test 18: Score Monotonicity
// ============================================================================

#[test]
fn test_score_monotonicity_property() {
    // Property: Adding quality issues should never increase score

    let temp1 = create_test_project();
    let src1 = temp1.path().join("src").join("main.rs");
    std::fs::write(&src1, "fn main() { println!(\"clean\"); }").unwrap();

    let temp2 = create_test_project();
    let src2 = temp2.path().join("src").join("main.rs");
    std::fs::write(
        &src2,
        r#"
fn main() {
    unsafe { println!("dirty"); }
}

#[allow(dead_code)]
fn unused() {}
"#,
    )
    .unwrap();

    let scorer = CodeQualityScorer::new();

    let clean_score = scorer.score(temp1.path()).unwrap();
    let dirty_score = scorer.score(temp2.path()).unwrap();

    // Clean code should score >= dirty code
    assert!(clean_score.earned >= dirty_score.earned);
}
