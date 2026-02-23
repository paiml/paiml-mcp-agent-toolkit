#![cfg_attr(coverage_nightly, coverage(off))]
/// EXTREME TDD coverage tests for perfection_score module
#[cfg(test)]
mod coverage_tests {
    use super::super::calculator::PerfectionScoreCalculator;
    use super::super::types::{
        CategoryScore, CategoryWeights, PerfectionScoreResult, MAX_PERFECTION_SCORE,
    };
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // Test Fixture Helpers
    // ============================================================================

    /// Create a test fixture with configurable project structure
    fn create_test_project(
        readme: bool,
        changelog: bool,
        docs: bool,
        contributing: bool,
        benches: bool,
        mutants_toml: bool,
        mutants_dir: bool,
    ) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        if readme {
            fs::write(root.join("README.md"), "# Test Project").unwrap();
        }
        if changelog {
            fs::write(root.join("CHANGELOG.md"), "# Changelog").unwrap();
        }
        if docs {
            fs::create_dir(root.join("docs")).unwrap();
        }
        if contributing {
            fs::write(root.join("CONTRIBUTING.md"), "# Contributing").unwrap();
        }
        if benches {
            fs::create_dir(root.join("benches")).unwrap();
        }
        if mutants_toml {
            fs::write(root.join("mutants.toml"), "[mutants]").unwrap();
        }
        if mutants_dir {
            fs::create_dir(root.join(".mutants")).unwrap();
        }

        temp_dir
    }

    /// Create a Rust project fixture with test files
    fn create_rust_project_fixture(test_count: usize, source_files: usize) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create src directory
        fs::create_dir(root.join("src")).unwrap();

        // Create source files with tests
        for i in 0..source_files {
            let mut content = format!("// Source file {}\n", i);
            let tests_in_file = if i < test_count { 1 } else { 0 };
            for j in 0..tests_in_file {
                content.push_str(&format!("\n#[test]\nfn test_{}_{} () {{}}\n", i, j));
            }
            fs::write(root.join("src").join(format!("mod_{}.rs", i)), content).unwrap();
        }

        // Create Cargo.toml
        fs::write(
            root.join("Cargo.toml"),
            r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        temp_dir
    }

    /// Create coverage metrics cache file
    fn create_coverage_cache(temp_dir: &TempDir, coverage: f64) {
        let metrics_dir = temp_dir.path().join(".pmat-metrics");
        fs::create_dir_all(&metrics_dir).unwrap();
        let cache_content = format!(r#"{{"coverage": {}}}"#, coverage);
        fs::write(metrics_dir.join("coverage.json"), cache_content).unwrap();
    }

    // ============================================================================
    // CategoryWeights Tests
    // ============================================================================

    #[test]
    fn test_category_weights_default_values() {
        let weights = CategoryWeights::default();
        assert_eq!(weights.tdg, 40);
        assert_eq!(weights.repo_score, 30);
        assert_eq!(weights.rust_score, 30);
        assert_eq!(weights.popper_score, 25);
        assert_eq!(weights.test_coverage, 25);
        assert_eq!(weights.mutation, 20);
        assert_eq!(weights.documentation, 15);
        assert_eq!(weights.performance, 15);
    }

    #[test]
    fn test_category_weights_clone() {
        let weights = CategoryWeights::default();
        let cloned = weights;
        assert_eq!(weights.tdg, cloned.tdg);
        assert_eq!(weights.repo_score, cloned.repo_score);
    }

    #[test]
    fn test_category_weights_debug() {
        let weights = CategoryWeights::default();
        let debug_str = format!("{:?}", weights);
        assert!(debug_str.contains("CategoryWeights"));
        assert!(debug_str.contains("40"));
    }

    // ============================================================================
    // CategoryScore Tests
    // ============================================================================

    #[test]
    fn test_category_score_new_zero_score() {
        let score = CategoryScore::new("Test", 0.0, 40);
        assert_eq!(score.name, "Test");
        assert_eq!(score.raw_score, 0.0);
        assert_eq!(score.max_points, 40);
        assert_eq!(score.earned_points, 0.0);
        assert_eq!(score.grade, "F");
        assert!(score.details.is_none());
    }

    #[test]
    fn test_category_score_new_fifty_percent() {
        let score = CategoryScore::new("Test", 50.0, 40);
        assert_eq!(score.earned_points, 20.0);
        assert_eq!(score.grade, "F");
    }

    #[test]
    fn test_category_score_new_hundred_percent() {
        let score = CategoryScore::new("Test", 100.0, 40);
        assert_eq!(score.earned_points, 40.0);
        assert_eq!(score.grade, "A+");
    }

    #[test]
    fn test_category_score_with_details() {
        let score = CategoryScore::new("Test", 75.0, 30).with_details("Some details");
        assert_eq!(score.details, Some("Some details".to_string()));
    }

    #[test]
    fn test_category_score_grade_a_plus() {
        let score = CategoryScore::new("Test", 97.0, 10);
        assert_eq!(score.grade, "A+");
        let score = CategoryScore::new("Test", 100.0, 10);
        assert_eq!(score.grade, "A+");
    }

    #[test]
    fn test_category_score_grade_a() {
        let score = CategoryScore::new("Test", 93.0, 10);
        assert_eq!(score.grade, "A");
        let score = CategoryScore::new("Test", 96.0, 10);
        assert_eq!(score.grade, "A");
    }

    #[test]
    fn test_category_score_grade_a_minus() {
        let score = CategoryScore::new("Test", 90.0, 10);
        assert_eq!(score.grade, "A-");
        let score = CategoryScore::new("Test", 92.0, 10);
        assert_eq!(score.grade, "A-");
    }

    #[test]
    fn test_category_score_grade_b_plus() {
        let score = CategoryScore::new("Test", 87.0, 10);
        assert_eq!(score.grade, "B+");
        let score = CategoryScore::new("Test", 89.0, 10);
        assert_eq!(score.grade, "B+");
    }

    #[test]
    fn test_category_score_grade_b() {
        let score = CategoryScore::new("Test", 83.0, 10);
        assert_eq!(score.grade, "B");
        let score = CategoryScore::new("Test", 86.0, 10);
        assert_eq!(score.grade, "B");
    }

    #[test]
    fn test_category_score_grade_b_minus() {
        let score = CategoryScore::new("Test", 80.0, 10);
        assert_eq!(score.grade, "B-");
        let score = CategoryScore::new("Test", 82.0, 10);
        assert_eq!(score.grade, "B-");
    }

    #[test]
    fn test_category_score_grade_c_plus() {
        let score = CategoryScore::new("Test", 77.0, 10);
        assert_eq!(score.grade, "C+");
        let score = CategoryScore::new("Test", 79.0, 10);
        assert_eq!(score.grade, "C+");
    }

    #[test]
    fn test_category_score_grade_c() {
        let score = CategoryScore::new("Test", 73.0, 10);
        assert_eq!(score.grade, "C");
        let score = CategoryScore::new("Test", 76.0, 10);
        assert_eq!(score.grade, "C");
    }

    #[test]
    fn test_category_score_grade_c_minus() {
        let score = CategoryScore::new("Test", 70.0, 10);
        assert_eq!(score.grade, "C-");
        let score = CategoryScore::new("Test", 72.0, 10);
        assert_eq!(score.grade, "C-");
    }

    #[test]
    fn test_category_score_grade_d_plus() {
        let score = CategoryScore::new("Test", 67.0, 10);
        assert_eq!(score.grade, "D+");
        let score = CategoryScore::new("Test", 69.0, 10);
        assert_eq!(score.grade, "D+");
    }

    #[test]
    fn test_category_score_grade_d() {
        let score = CategoryScore::new("Test", 63.0, 10);
        assert_eq!(score.grade, "D");
        let score = CategoryScore::new("Test", 66.0, 10);
        assert_eq!(score.grade, "D");
    }

    #[test]
    fn test_category_score_grade_d_minus() {
        let score = CategoryScore::new("Test", 60.0, 10);
        assert_eq!(score.grade, "D-");
        let score = CategoryScore::new("Test", 62.0, 10);
        assert_eq!(score.grade, "D-");
    }

    #[test]
    fn test_category_score_grade_f() {
        let score = CategoryScore::new("Test", 59.0, 10);
        assert_eq!(score.grade, "F");
        let score = CategoryScore::new("Test", 0.0, 10);
        assert_eq!(score.grade, "F");
    }

    #[test]
    fn test_category_score_serialization() {
        let score = CategoryScore::new("Test", 85.0, 25);
        let json = serde_json::to_string(&score).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"raw_score\":85.0"));
        assert!(json.contains("\"max_points\":25"));
    }

    #[test]
    fn test_category_score_deserialization() {
        let json = r#"{"name":"Test","raw_score":85.0,"max_points":25,"earned_points":21.25,"grade":"B","details":null}"#;
        let score: CategoryScore = serde_json::from_str(json).unwrap();
        assert_eq!(score.name, "Test");
        assert_eq!(score.raw_score, 85.0);
        assert_eq!(score.max_points, 25);
    }

    // ============================================================================
    // PerfectionScoreResult Tests
    // ============================================================================

    #[test]
    fn test_perfection_score_result_empty_categories() {
        let result = PerfectionScoreResult::new(vec![]);
        assert_eq!(result.total_score, 0.0);
        assert_eq!(result.max_score, MAX_PERFECTION_SCORE);
        assert_eq!(result.grade, "F");
        assert_eq!(result.recommendations.len(), 1);
        assert!(result.recommendations[0].contains("All categories are healthy"));
    }

    #[test]
    fn test_perfection_score_result_perfect_score() {
        let categories = vec![
            CategoryScore::new("TDG", 100.0, 40),
            CategoryScore::new("Repo", 100.0, 30),
            CategoryScore::new("Rust", 100.0, 30),
            CategoryScore::new("Popper", 100.0, 25),
            CategoryScore::new("Coverage", 100.0, 25),
            CategoryScore::new("Mutation", 100.0, 20),
            CategoryScore::new("Docs", 100.0, 15),
            CategoryScore::new("Performance", 100.0, 15),
        ];
        let result = PerfectionScoreResult::new(categories);
        assert_eq!(result.total_score, 200.0);
        assert_eq!(result.grade, "A+");
    }

    #[test]
    fn test_perfection_score_result_with_target() {
        let categories = vec![CategoryScore::new("TDG", 80.0, 40)];
        let result = PerfectionScoreResult::new(categories).with_target(100);
        assert!(result.target_gap.is_some());
        assert_eq!(result.target_gap.unwrap(), 100.0 - 32.0);
    }

    #[test]
    fn test_perfection_score_result_recommendations_critical() {
        let categories = vec![
            CategoryScore::new("TDG", 50.0, 40), // 50% - critical
        ];
        let result = PerfectionScoreResult::new(categories);
        assert!(result
            .recommendations
            .iter()
            .any(|r| r.contains("critical")));
    }

    #[test]
    fn test_perfection_score_result_recommendations_needs_attention() {
        let categories = vec![
            CategoryScore::new("TDG", 70.0, 40), // 70% - needs attention
        ];
        let result = PerfectionScoreResult::new(categories);
        assert!(result
            .recommendations
            .iter()
            .any(|r| r.contains("needs attention")));
    }

    #[test]
    fn test_perfection_score_result_recommendations_healthy() {
        let categories = vec![
            CategoryScore::new("TDG", 90.0, 40), // 90% - healthy
        ];
        let result = PerfectionScoreResult::new(categories);
        assert!(result.recommendations.iter().any(|r| r.contains("healthy")));
    }

    #[test]
    fn test_overall_grade_boundary_a_plus() {
        // 95%+ = A+: 190/200=95%, 200/200=100%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(190.0), "A+");
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(200.0), "A+");
    }

    #[test]
    fn test_overall_grade_boundary_a() {
        // 90-94% = A: 180/200=90%, 188/200=94%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(180.0), "A");
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(188.0), "A");
    }

    #[test]
    fn test_overall_grade_boundary_f() {
        // <50% = F: 0/200=0%, 98/200=49%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(0.0), "F");
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(98.0), "F");
    }

    #[test]
    fn test_perfection_score_result_serialization() {
        let categories = vec![CategoryScore::new("TDG", 80.0, 40)];
        let result = PerfectionScoreResult::new(categories);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total_score\""));
        assert!(json.contains("\"max_score\""));
        assert!(json.contains("\"grade\""));
    }

}
