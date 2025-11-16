//! RustProjectScore Orchestrator Integration Tests
//!
//! EXTREME TDD tests for the orchestrator that aggregates all 6 category scorers.

use pmat::services::rust_project_score::*;
use std::fs;
use tempfile::TempDir;

// ==================== Test Fixtures ====================

/// Create a minimal valid Rust project for testing
fn create_minimal_project() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create basic project structure
    let src_dir = temp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn example() {}").unwrap();

    // Create minimal Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    fs::write(temp.path().join("Cargo.toml"), cargo_toml).unwrap();

    temp
}

/// Create a high-quality Rust project (should score high)
fn create_high_quality_project() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create basic structure
    let src_dir = temp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(
        src_dir.join("lib.rs"),
        r#"
//! High-quality Rust library
//!
//! This is a well-documented library with good practices.

/// Example function with documentation
pub fn example() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(example(), 42);
    }
}
"#,
    )
    .unwrap();

    // High-quality Cargo.toml with features
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"], default-features = false }
tokio = { version = "1.0", optional = true }

[dev-dependencies]
criterion = "0.5"

[features]
default = []
async-runtime = ["tokio"]

[[bench]]
name = "benchmarks"
harness = false
"#;
    fs::write(temp.path().join("Cargo.toml"), cargo_toml).unwrap();

    // Add README
    let readme = r#"# Test Project

A high-quality Rust project for testing.

## Features

- Well-documented
- Comprehensive tests
- Performance benchmarks
"#;
    fs::write(temp.path().join("README.md"), readme).unwrap();

    // Add CHANGELOG
    let changelog = r#"# Changelog

## [0.1.0] - 2025-01-16

### Added
- Initial release
"#;
    fs::write(temp.path().join("CHANGELOG.md"), changelog).unwrap();

    // Add benchmarks
    let benches_dir = temp.path().join("benches");
    fs::create_dir(&benches_dir).unwrap();
    fs::write(
        benches_dir.join("benchmarks.rs"),
        r#"
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn example_bench(c: &mut Criterion) {
    c.bench_function("example", |b| b.iter(|| black_box(42)));
}

criterion_group!(benches, example_bench);
criterion_main!(benches);
"#,
    )
    .unwrap();

    temp
}

// ==================== Basic Orchestrator Tests ====================

#[test]
fn test_orchestrator_creation() {
    let orchestrator = RustProjectScoreOrchestrator::new();
    assert_eq!(orchestrator.name(), "Rust Project Score v1.1");
    assert_eq!(orchestrator.max_points(), 106.0);
}

#[test]
fn test_orchestrator_has_all_six_scorers() {
    let orchestrator = RustProjectScoreOrchestrator::new();
    let scorer_names = orchestrator.scorer_names();

    assert_eq!(scorer_names.len(), 6);
    assert!(scorer_names.contains(&"Rust Tooling Compliance"));
    assert!(scorer_names.contains(&"Code Quality"));
    assert!(scorer_names.contains(&"Testing Excellence"));
    assert!(scorer_names.contains(&"Documentation"));
    assert!(scorer_names.contains(&"Performance & Benchmarking"));
    assert!(scorer_names.contains(&"Dependency Health"));
}

#[test]
fn test_max_points_distribution() {
    let orchestrator = RustProjectScoreOrchestrator::new();
    let max_points_by_category = orchestrator.max_points_by_category();

    assert_eq!(
        max_points_by_category.get("Rust Tooling Compliance"),
        Some(&25.0)
    );
    assert_eq!(max_points_by_category.get("Code Quality"), Some(&26.0));
    assert_eq!(
        max_points_by_category.get("Testing Excellence"),
        Some(&20.0)
    );
    assert_eq!(max_points_by_category.get("Documentation"), Some(&15.0));
    assert_eq!(
        max_points_by_category.get("Performance & Benchmarking"),
        Some(&10.0)
    );
    assert_eq!(max_points_by_category.get("Dependency Health"), Some(&12.0));
}

// ==================== Scoring Tests ====================

#[test]
fn test_score_minimal_project() {
    let temp = create_minimal_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok(), "Scoring should succeed for valid project");

    let project_score = result.unwrap();
    assert!(project_score.total_earned >= 0.0);
    assert!(project_score.total_earned <= 106.0);
    assert_eq!(project_score.total_possible, 106.0);
}

#[test]
fn test_score_high_quality_project() {
    let temp = create_high_quality_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    // High-quality project should score significantly higher than minimal
    assert!(
        project_score.total_earned > 10.0,
        "High-quality project should earn >10 points, got {}",
        project_score.total_earned
    );
}

#[test]
fn test_score_invalid_project() {
    let temp = TempDir::new().unwrap();
    // Empty directory - no Cargo.toml

    let orchestrator = RustProjectScoreOrchestrator::new();
    let result = orchestrator.score(temp.path());

    assert!(result.is_err(), "Should fail for invalid Rust project");
    assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
}

#[test]
fn test_category_scores_populated() {
    let temp = create_high_quality_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    // All 6 categories should have scores
    assert!(project_score
        .categories
        .contains_key("Rust Tooling Compliance"));
    assert!(project_score.categories.contains_key("Code Quality"));
    assert!(project_score.categories.contains_key("Testing Excellence"));
    assert!(project_score.categories.contains_key("Documentation"));
    assert!(project_score
        .categories
        .contains_key("Performance & Benchmarking"));
    assert!(project_score.categories.contains_key("Dependency Health"));
}

// ==================== Grade Calculation Tests ====================

#[test]
fn test_grade_calculation_a_plus() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // Create mock score with ≥95% (100.7+ points)
    let grade = orchestrator.calculate_grade(101.0, 106.0);
    assert_eq!(grade, Grade::APlus);
}

#[test]
fn test_grade_calculation_a() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // 90-94 points
    let grade = orchestrator.calculate_grade(92.0, 106.0);
    assert_eq!(grade, Grade::A);
}

#[test]
fn test_grade_calculation_a_minus() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // 85-89 points
    let grade = orchestrator.calculate_grade(87.0, 106.0);
    assert_eq!(grade, Grade::AMinus);
}

#[test]
fn test_grade_calculation_b_plus() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // 80-84 points
    let grade = orchestrator.calculate_grade(82.0, 106.0);
    assert_eq!(grade, Grade::BPlus);
}

#[test]
fn test_grade_calculation_b() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // 70-79 points
    let grade = orchestrator.calculate_grade(75.0, 106.0);
    assert_eq!(grade, Grade::B);
}

#[test]
fn test_grade_calculation_c() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // 60-69 points
    let grade = orchestrator.calculate_grade(65.0, 106.0);
    assert_eq!(grade, Grade::C);
}

#[test]
fn test_grade_calculation_d() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // 50-59 points
    let grade = orchestrator.calculate_grade(55.0, 106.0);
    assert_eq!(grade, Grade::D);
}

#[test]
fn test_grade_calculation_f() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // <50 points
    let grade = orchestrator.calculate_grade(30.0, 106.0);
    assert_eq!(grade, Grade::F);
}

#[test]
fn test_grade_boundary_conditions() {
    let orchestrator = RustProjectScoreOrchestrator::new();

    // Test exact boundaries
    assert_eq!(orchestrator.calculate_grade(95.0, 106.0), Grade::APlus); // Exactly 95
    assert_eq!(orchestrator.calculate_grade(90.0, 106.0), Grade::A); // Exactly 90
    assert_eq!(orchestrator.calculate_grade(85.0, 106.0), Grade::AMinus); // Exactly 85
}

// ==================== Recommendation Tests ====================

#[test]
fn test_recommendations_generated() {
    let temp = create_minimal_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    // Minimal project should have recommendations
    assert!(!project_score.recommendations.is_empty());
}

#[test]
fn test_high_quality_project_fewer_recommendations() {
    let minimal = create_minimal_project();
    let high_quality = create_high_quality_project();

    let orchestrator = RustProjectScoreOrchestrator::new();

    let minimal_score = orchestrator.score(minimal.path()).unwrap();
    let hq_score = orchestrator.score(high_quality.path()).unwrap();

    // High-quality project should have fewer or equal recommendations
    assert!(
        hq_score.recommendations.len() <= minimal_score.recommendations.len(),
        "High-quality project should have <= recommendations than minimal"
    );
}

// ==================== Percentage Calculation Tests ====================

#[test]
fn test_percentage_calculation() {
    let temp = create_minimal_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    assert!(project_score.percentage >= 0.0);
    assert!(project_score.percentage <= 100.0);

    // Verify percentage matches earned/possible ratio
    let expected_percentage = (project_score.total_earned / project_score.total_possible) * 100.0;
    assert!((project_score.percentage - expected_percentage).abs() < 0.01);
}

// ==================== Property-Based Tests ====================

#[test]
fn test_score_monotonicity() {
    // Adding good practices should never decrease score
    let minimal = create_minimal_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let minimal_score = orchestrator.score(minimal.path()).unwrap().total_earned;

    // Add README (should increase Documentation score)
    fs::write(minimal.path().join("README.md"), "# Test\n\nDocumentation").unwrap();
    let with_readme_score = orchestrator.score(minimal.path()).unwrap().total_earned;

    assert!(
        with_readme_score >= minimal_score,
        "Adding README should not decrease score"
    );
}

#[test]
fn test_score_bounds() {
    let temp = create_minimal_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    // Score must be within bounds
    assert!(project_score.total_earned >= 0.0);
    assert!(project_score.total_earned <= project_score.total_possible);
}

#[test]
fn test_category_scores_sum_to_total() {
    let temp = create_minimal_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    // Sum of category earned scores should equal total earned
    let category_sum: f64 = project_score
        .categories
        .values()
        .map(|cat| cat.earned)
        .sum();

    assert!(
        (category_sum - project_score.total_earned).abs() < 0.01,
        "Category scores should sum to total: {} vs {}",
        category_sum,
        project_score.total_earned
    );
}

// ==================== Error Handling Tests ====================

#[test]
fn test_score_nonexistent_path() {
    let orchestrator = RustProjectScoreOrchestrator::new();
    let result = orchestrator.score(std::path::Path::new("/nonexistent/path"));

    assert!(result.is_err());
}

#[test]
fn test_score_file_instead_of_directory() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("file.txt");
    fs::write(&file_path, "not a directory").unwrap();

    let orchestrator = RustProjectScoreOrchestrator::new();
    let result = orchestrator.score(&file_path);

    assert!(result.is_err());
}

// ==================== Integration Tests ====================

#[test]
fn test_end_to_end_scoring() {
    let temp = create_high_quality_project();
    let orchestrator = RustProjectScoreOrchestrator::new();

    let result = orchestrator.score(temp.path());
    assert!(result.is_ok());

    let project_score = result.unwrap();

    // Verify all expected fields are present
    assert!(project_score.total_earned > 0.0);
    assert_eq!(project_score.total_possible, 106.0);
    assert!(project_score.percentage > 0.0);
    assert_eq!(project_score.categories.len(), 6);
}

#[test]
fn test_grade_display() {
    assert_eq!(Grade::APlus.to_string(), "A+");
    assert_eq!(Grade::A.to_string(), "A");
    assert_eq!(Grade::AMinus.to_string(), "A-");
    assert_eq!(Grade::BPlus.to_string(), "B+");
    assert_eq!(Grade::B.to_string(), "B");
    assert_eq!(Grade::C.to_string(), "C");
    assert_eq!(Grade::D.to_string(), "D");
    assert_eq!(Grade::F.to_string(), "F");
}
