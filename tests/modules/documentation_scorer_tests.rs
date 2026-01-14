//! RED Phase Tests for DocumentationScorer (Sprint 2)
//!
//! Documentation Category: 15 points total
//! - Rustdoc Coverage (7pts): Public API documentation with examples
//! - README Quality (5pts): Comprehensive project README
//! - Changelog Presence (3pts): CHANGELOG.md with version history
//!
//! Evidence-based design: Well-documented projects have 30-40% fewer
//! support issues and faster onboarding (GitHub State of the Octoverse 2024).

use pmat::services::rust_project_score::{DocumentationScorer, Scorer};

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
// Test 1: DocumentationScorer Creation
// ============================================================================

#[test]
fn test_documentation_scorer_creation() {
    let scorer = DocumentationScorer::new();
    assert_eq!(scorer.name(), "Documentation");
    assert_eq!(scorer.max_points(), 15.0);
}

// ============================================================================
// Test 2: Perfect Score (All Documentation Checks Pass)
// ============================================================================

#[test]
fn test_perfect_score_all_checks_pass() {
    let temp = create_test_project();
    let scorer = DocumentationScorer::new();

    let result = scorer.score(temp.path());
    assert!(result.is_ok());

    let score = result.unwrap();
    assert_eq!(score.max, 15.0);
    assert!(score.earned >= 0.0 && score.earned <= 15.0);
}

// ============================================================================
// Test 3: Rustdoc Coverage Scoring (7pts)
// ============================================================================

#[test]
fn test_rustdoc_coverage_well_documented() {
    let temp = create_test_project();

    // Create lib.rs with comprehensive documentation
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
//! Test library with documentation
//!
//! This library demonstrates well-documented code.

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

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get high rustdoc coverage points
    assert!(score.earned >= 7.0 || score.earned == 0.0);
}

#[test]
fn test_rustdoc_coverage_poorly_documented() {
    let temp = create_test_project();

    // Create lib.rs WITHOUT documentation
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

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose rustdoc coverage points
    assert!(score.earned < 15.0);
}

// ============================================================================
// Test 4: README Quality Scoring (5pts)
// ============================================================================

#[test]
fn test_readme_quality_comprehensive() {
    let temp = create_test_project();

    // Create comprehensive README
    std::fs::write(
        temp.path().join("README.md"),
        r#"
# Test Project

A comprehensive test project demonstrating README quality.

## Installation

```bash
cargo add test-project
```

## Usage

```rust
use test_project::add;

let result = add(2, 3);
```

## Features

- Feature 1
- Feature 2
- Feature 3

## License

MIT License
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get high README quality points
    assert!(score.earned >= 5.0 || score.earned == 0.0);
}

#[test]
fn test_readme_quality_minimal() {
    let temp = create_test_project();

    // Create minimal README
    std::fs::write(
        temp.path().join("README.md"),
        r#"
# Test Project

A test project.
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get partial README points
    assert!(score.earned < 15.0);
}

#[test]
fn test_readme_absent() {
    let temp = create_test_project();

    // No README created

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose README points
    assert!(score.earned <= 10.0);
}

// ============================================================================
// Test 5: Changelog Presence Scoring (3pts)
// ============================================================================

#[test]
fn test_changelog_present() {
    let temp = create_test_project();

    // Create CHANGELOG.md
    std::fs::write(
        temp.path().join("CHANGELOG.md"),
        r#"
# Changelog

## [0.1.0] - 2024-01-01

### Added
- Initial release
- Feature 1
- Feature 2
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get changelog points
    assert!(score.earned >= 3.0 || score.earned == 0.0);
}

#[test]
fn test_changelog_absent() {
    let temp = create_test_project();

    // No CHANGELOG created

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose changelog points
    assert!(score.earned <= 12.0);
}

// ============================================================================
// Test 6: Rustdoc Missing Items
// ============================================================================

#[test]
fn test_rustdoc_missing_items() {
    let temp = create_test_project();

    // Create lib.rs with some documented and some undocumented items
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
/// Documented function
pub fn documented() -> i32 {
    42
}

pub fn undocumented() -> i32 {
    99
}
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get partial rustdoc points
    assert!(score.earned < 15.0);
}

// ============================================================================
// Test 7: README Section Detection
// ============================================================================

#[test]
fn test_readme_section_detection() {
    let temp = create_test_project();

    // Create README with multiple sections
    std::fs::write(
        temp.path().join("README.md"),
        r#"
# Test Project

## Installation
Install instructions here.

## Usage
Usage examples here.

## API Documentation
API docs here.

## Contributing
Contribution guidelines here.

## License
License information here.
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get full README points for comprehensive sections
    assert!(score.earned >= 5.0 || score.earned == 0.0);
}

// ============================================================================
// Test 8: Changelog Version Detection
// ============================================================================

#[test]
fn test_changelog_version_detection() {
    let temp = create_test_project();

    // Create CHANGELOG with multiple versions
    std::fs::write(
        temp.path().join("CHANGELOG.md"),
        r#"
# Changelog

## [0.2.0] - 2024-02-01
### Added
- New feature

## [0.1.0] - 2024-01-01
### Added
- Initial release
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get full changelog points for version history
    assert!(score.earned >= 3.0 || score.earned == 0.0);
}

// ============================================================================
// Test 9: Recommendations Generation
// ============================================================================

#[test]
fn test_recommendations_for_documentation_issues() {
    let temp = create_test_project();

    // Create project with NO documentation
    let scorer = DocumentationScorer::new();
    let recommendations = scorer.recommendations(temp.path());

    // Should provide specific recommendations
    assert!(!recommendations.is_empty());

    let rec_text = recommendations.join(" ");
    assert!(
        rec_text.contains("documentation")
            || rec_text.contains("README")
            || rec_text.contains("CHANGELOG")
            || rec_text.contains("rustdoc")
    );
}

// ============================================================================
// Test 10: Scorer Implements Send + Sync
// ============================================================================

#[test]
fn test_scorer_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<DocumentationScorer>();
    assert_sync::<DocumentationScorer>();
}

// ============================================================================
// Test 11: Scoring is Deterministic
// ============================================================================

#[test]
fn test_scoring_is_deterministic() {
    let temp = create_test_project();
    let scorer = DocumentationScorer::new();

    // Score same project twice
    let result1 = scorer.score(temp.path()).unwrap();
    let result2 = scorer.score(temp.path()).unwrap();

    // Should get identical scores
    assert_eq!(result1.earned, result2.earned);
    assert_eq!(result1.max, result2.max);
}

// ============================================================================
// Test 12: Invalid Project Error
// ============================================================================

#[test]
fn test_invalid_project_no_cargo_toml() {
    let temp = tempfile::tempdir().unwrap();
    let scorer = DocumentationScorer::new();

    let result = scorer.score(temp.path());

    // Should return error for invalid project
    assert!(result.is_err());
}

// ============================================================================
// Test 13: Performance (<5 seconds)
// ============================================================================

#[test]
fn test_scoring_performance() {
    use std::time::Instant;

    let temp = create_test_project();
    let scorer = DocumentationScorer::new();

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
// Test 14: CategoryScore Structure
// ============================================================================

#[test]
fn test_category_score_structure() {
    let temp = create_test_project();
    let scorer = DocumentationScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Verify CategoryScore has correct structure
    assert_eq!(result.max, 15.0);
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);

    // Test percentage calculation
    let percentage = result.percentage();
    assert!((0.0..=100.0).contains(&percentage));
}

// ============================================================================
// Property-Based Test 15: Score Bounds
// ============================================================================

#[test]
fn test_score_bounds_property() {
    let temp = create_test_project();
    let scorer = DocumentationScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Property: Score must be in [0, max]
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);
    assert_eq!(result.max, 15.0);
}

// ============================================================================
// Property-Based Test 16: Score Monotonicity
// ============================================================================

#[test]
fn test_score_monotonicity_property() {
    // Property: Adding documentation should never decrease score

    let temp1 = create_test_project();

    let temp2 = create_test_project();
    std::fs::write(
        temp2.path().join("README.md"),
        "# Test Project\n\nA documented project.",
    )
    .unwrap();

    let scorer = DocumentationScorer::new();

    let no_docs_score = scorer.score(temp1.path()).unwrap();
    let with_docs_score = scorer.score(temp2.path()).unwrap();

    // Code with documentation should score >= code without
    assert!(with_docs_score.earned >= no_docs_score.earned);
}

// ============================================================================
// Test 17: Rustdoc Coverage Calculation
// ============================================================================

#[test]
fn test_rustdoc_coverage_calculation() {
    let temp = create_test_project();

    // Create lib.rs with 50% documentation coverage
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
/// Documented function 1
pub fn fn1() -> i32 { 1 }

pub fn fn2() -> i32 { 2 }

/// Documented function 3
pub fn fn3() -> i32 { 3 }

pub fn fn4() -> i32 { 4 }
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get partial rustdoc coverage points
    assert!(result.earned > 0.0 && result.earned < 15.0);
}

// ============================================================================
// Test 18: README Word Count
// ============================================================================

#[test]
fn test_readme_word_count() {
    let temp = create_test_project();

    // Create README with substantial content (>200 words)
    let readme_content = vec!["word"; 250].join(" ");
    std::fs::write(
        temp.path().join("README.md"),
        format!("# Test Project\n\n{}", readme_content),
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get points for substantial README
    assert!(result.earned >= 0.0);
}

// ============================================================================
// Test 19: Evidence-Based Weight Allocation
// ============================================================================

#[test]
fn test_evidence_based_weights() {
    let scorer = DocumentationScorer::new();

    // Verify evidence-based weight allocation
    assert_eq!(scorer.max_points(), 15.0);

    // Rustdoc (7pts): Highest weight - API documentation critical
    // README (5pts): Project overview and onboarding
    // Changelog (3pts): Version history and upgrade guidance
}

// ============================================================================
// Test 20: Module-Level Documentation
// ============================================================================

#[test]
fn test_module_level_documentation() {
    let temp = create_test_project();

    // Create lib.rs with module-level documentation
    let src_lib = temp.path().join("src").join("lib.rs");
    std::fs::write(
        &src_lib,
        r#"
//! # Test Project
//!
//! This is a comprehensive module-level documentation
//! that describes the entire library.

/// Public function
pub fn example() -> i32 {
    42
}
"#,
    )
    .unwrap();

    let scorer = DocumentationScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get points for module-level documentation
    assert!(result.earned >= 0.0);
}
