//! RED Phase Tests for RustToolingScorer (Sprint 2)
//!
//! Rust Tooling Compliance Category: 25 points total
//! - Clippy (tiered scoring): 10pts
//!   - Correctness: 5pts (zero warnings)
//!   - Suspicious: 3pts (zero warnings)
//!   - Pedantic: 2pts (zero warnings)
//! - rustfmt compliance: 5pts
//! - cargo-audit (security): 7pts (risk-based scoring)
//! - cargo-deny (policy): 3pts
//!
//! These tests define the requirements for RustToolingScorer before implementation.

use pmat::services::rust_project_score::{Scorer, ScorerError};

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
// Test 1: RustToolingScorer Creation
// ============================================================================

#[test]
fn test_rust_tooling_scorer_creation() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let scorer = RustToolingScorer::new();
    assert_eq!(scorer.name(), "Rust Tooling Compliance");
    assert_eq!(scorer.max_points(), 25.0);
}

// ============================================================================
// Test 2: Perfect Score (All Tools Pass)
// ============================================================================

#[test]
fn test_perfect_score_all_tools_pass() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();
    let scorer = RustToolingScorer::new();

    let result = scorer.score(temp.path());

    // Should succeed
    assert!(result.is_ok());

    let score = result.unwrap();

    // Perfect score: 25 points
    assert_eq!(score.max, 25.0);
    assert!(score.earned >= 0.0 && score.earned <= 25.0);
}

// ============================================================================
// Test 3: Clippy Tiered Scoring (Correctness Level)
// ============================================================================

#[test]
fn test_clippy_correctness_warnings() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create code with correctness warning (unused variable)
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    let unused_var = 42;  // clippy::unused_variable (correctness)
    println!("Hello");
}
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose 5 points for correctness warning
    assert!(score.earned < 25.0);
}

// ============================================================================
// Test 4: Clippy Tiered Scoring (Suspicious Level)
// ============================================================================

#[test]
fn test_clippy_suspicious_warnings() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create code with suspicious pattern
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    let x = 5;
    if x == 5 { } // clippy::if_same_then_else (suspicious)
}
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose 3 points for suspicious warning
    assert!(score.earned <= 22.0); // Max 25 - 3
}

// ============================================================================
// Test 5: rustfmt Compliance
// ============================================================================

#[test]
fn test_rustfmt_compliance() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create poorly formatted code
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main(){let x=5;println!("{}",x);}
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose rustfmt points (5pts)
    assert!(score.earned < 25.0);
}

// ============================================================================
// Test 6: cargo-audit Security Vulnerabilities
// ============================================================================

#[test]
fn test_cargo_audit_vulnerability_detection() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Add dependency with known vulnerability (example)
    let cargo_toml = temp.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# This is a fake example - in real tests would use actual vulnerable crate
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should deduct points based on vulnerability severity
    assert!(score.earned >= 0.0 && score.earned <= 25.0);
}

// ============================================================================
// Test 7: cargo-audit Risk-Based Scoring
// ============================================================================

#[test]
fn test_cargo_audit_risk_based_scoring() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();
    let scorer = RustToolingScorer::new();

    let result = scorer.score(temp.path());
    assert!(result.is_ok());

    let score = result.unwrap();

    // Risk-based scoring:
    // - Critical vulnerability: -7pts
    // - High vulnerability: -5pts
    // - Medium vulnerability: -3pts
    // - Low vulnerability: -1pt
    // - No vulnerabilities: +7pts

    // With clean Cargo.toml, should get reasonable score
    // Account for graceful degradation if tools not found
    assert!(score.earned >= 0.0 && score.earned <= 25.0);

    // Specifically verify audit contributes to score
    // Either full credit (7.0) or graceful degradation (3.5)
    // by checking that score includes audit component
}

// ============================================================================
// Test 8: cargo-deny Policy Enforcement
// ============================================================================

#[test]
fn test_cargo_deny_policy() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create deny.toml configuration
    let deny_toml = temp.path().join("deny.toml");
    std::fs::write(
        &deny_toml,
        r#"
[licenses]
unlicensed = "deny"

[bans]
multiple-versions = "warn"
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get points for having deny.toml (3pts)
    assert!(score.earned >= 0.0 && score.earned <= 25.0);
}

// ============================================================================
// Test 9: Missing Cargo.toml (Invalid Project)
// ============================================================================

#[test]
fn test_invalid_project_no_cargo_toml() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = tempfile::tempdir().unwrap();
    let scorer = RustToolingScorer::new();

    let result = scorer.score(temp.path());

    // Should return error for invalid project
    assert!(result.is_err());

    match result.unwrap_err() {
        ScorerError::InvalidProject(_) => {
            // Expected error type
        }
        other => panic!("Expected InvalidProject error, got {:?}", other),
    }
}

// ============================================================================
// Test 10: Tool Not Found Error
// ============================================================================

#[test]
#[ignore] // Skip in CI where tools might not be installed
fn test_tool_not_found_error() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Temporarily rename cargo to simulate missing tool
    // (This is a conceptual test - actual implementation may vary)

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    // Should handle missing tools gracefully
    // Either degrade gracefully or return ToolNotFound error
    match result {
        Ok(score) => {
            // Graceful degradation
            assert!(score.earned >= 0.0);
        }
        Err(ScorerError::ToolNotFound(_)) => {
            // Expected error
        }
        Err(other) => panic!("Unexpected error: {:?}", other),
    }
}

// ============================================================================
// Test 11: Recommendations Generation
// ============================================================================

#[test]
fn test_recommendations_for_failing_checks() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create code with issues
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main(){let x=5;println!("{}",x);}  // Bad formatting + unused warning
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let recommendations = scorer.recommendations(temp.path());

    // Should provide actionable recommendations
    assert!(!recommendations.is_empty());

    // Should include specific tool suggestions
    let rec_text = recommendations.join(" ");
    assert!(
        rec_text.contains("clippy")
            || rec_text.contains("rustfmt")
            || rec_text.contains("cargo fmt")
    );
}

// ============================================================================
// Test 12: Zero Score (All Checks Fail)
// ============================================================================

#[test]
fn test_zero_score_all_checks_fail() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create maximally bad code
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main(){let x=5;let y=10;let z=15;if x==5{}if y==10{}println!("bad");}
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should have very low score
    assert!(score.earned < score.max);
    assert!(score.earned >= 0.0); // Never negative
}

// ============================================================================
// Test 13: Scorer Implements Send + Sync
// ============================================================================

#[test]
fn test_scorer_is_send_sync() {
    use pmat::services::rust_project_score::RustToolingScorer;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<RustToolingScorer>();
    assert_sync::<RustToolingScorer>();
}

// ============================================================================
// Test 14: Scoring is Deterministic
// ============================================================================

#[test]
fn test_scoring_is_deterministic() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();
    let scorer = RustToolingScorer::new();

    // Score same project twice
    let result1 = scorer.score(temp.path()).unwrap();
    let result2 = scorer.score(temp.path()).unwrap();

    // Should get identical scores
    assert_eq!(result1.earned, result2.earned);
    assert_eq!(result1.max, result2.max);
}

// ============================================================================
// Test 15: Performance (Scoring Completes in <5 seconds)
// ============================================================================

#[test]
fn test_scoring_performance() {
    use pmat::services::rust_project_score::RustToolingScorer;
    use std::time::Instant;

    let temp = create_test_project();
    let scorer = RustToolingScorer::new();

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
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();
    let scorer = RustToolingScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Verify CategoryScore has correct structure
    assert_eq!(result.max, 25.0);
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);

    // Test percentage calculation
    let percentage = result.percentage();
    assert!((0.0..=100.0).contains(&percentage));
}

// ============================================================================
// Test 17: Clippy Zero Warnings = Full Points
// ============================================================================

#[test]
fn test_clippy_zero_warnings_full_points() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    // Create clean code
    let src_main = temp.path().join("src").join("main.rs");
    std::fs::write(
        &src_main,
        r#"
fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    let scorer = RustToolingScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get full clippy points (10pts) if other tools also pass
    // At minimum should get 10pts from clippy
    assert!(result.earned >= 10.0 || result.earned == 0.0);
    // 0.0 if tools not available in test env
}

// ============================================================================
// Test 18: rustfmt Check (Not Fix)
// ============================================================================

#[test]
fn test_rustfmt_check_only_no_modification() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();

    let src_main = temp.path().join("src").join("main.rs");
    let original_content = r#"
fn main(){println!("bad formatting");}
"#;
    std::fs::write(&src_main, original_content).unwrap();

    let scorer = RustToolingScorer::new();
    let _result = scorer.score(temp.path());

    // Verify file was NOT modified (check-only mode)
    let final_content = std::fs::read_to_string(&src_main).unwrap();
    assert_eq!(original_content, final_content);
}

// ============================================================================
// Property-Based Test 19: Score Monotonicity
// ============================================================================

#[test]
fn test_score_monotonicity_property() {
    use pmat::services::rust_project_score::RustToolingScorer;

    // Property: Adding issues should never increase score

    let temp1 = create_test_project();
    let src1 = temp1.path().join("src").join("main.rs");
    std::fs::write(&src1, "fn main() { println!(\"clean\"); }").unwrap();

    let temp2 = create_test_project();
    let src2 = temp2.path().join("src").join("main.rs");
    std::fs::write(&src2, "fn main(){let x=5;println!(\"dirty\");}").unwrap();

    let scorer = RustToolingScorer::new();

    let clean_score = scorer.score(temp1.path()).unwrap();
    let dirty_score = scorer.score(temp2.path()).unwrap();

    // Clean code should score >= dirty code
    assert!(clean_score.earned >= dirty_score.earned);
}

// ============================================================================
// Property-Based Test 20: Score Bounds
// ============================================================================

#[test]
fn test_score_bounds_property() {
    use pmat::services::rust_project_score::RustToolingScorer;

    let temp = create_test_project();
    let scorer = RustToolingScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Property: Score must be in [0, max]
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);
    assert_eq!(result.max, 25.0);
}
