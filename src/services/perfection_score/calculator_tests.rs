#![cfg_attr(coverage_nightly, coverage(off))]
/// Calculator unit tests for perfection_score module
#[cfg(test)]
mod calculator_tests {
    use super::super::calculator::{normalize_rps_percentage, PerfectionScoreCalculator};
    use super::super::types::CategoryScore;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // PerfectionScoreCalculator Tests
    // ============================================================================

    #[test]
    fn test_calculator_new() {
        let calc = PerfectionScoreCalculator::new();
        assert!(!calc.fast_mode);
        assert_eq!(calc.weights.tdg, 40);
    }

    #[test]
    fn test_calculator_default() {
        let calc = PerfectionScoreCalculator::default();
        assert!(!calc.fast_mode);
    }

    #[test]
    fn test_calculator_fast_mode_setter() {
        let calc = PerfectionScoreCalculator::new().fast_mode(true);
        assert!(calc.fast_mode);

        let calc = PerfectionScoreCalculator::new().fast_mode(false);
        assert!(!calc.fast_mode);
    }

    #[tokio::test]
    async fn test_get_documentation_score_all_docs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("README.md"), "# Test Project").unwrap();
        fs::write(root.join("CHANGELOG.md"), "# Changelog").unwrap();
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(root.join("CONTRIBUTING.md"), "# Contributing").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 100.0); // 40 + 20 + 25 + 15 = 100
    }

    #[tokio::test]
    async fn test_get_documentation_score_readme_only() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 40.0);
    }

    #[tokio::test]
    async fn test_get_documentation_score_no_docs() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 0.0);
    }

    #[tokio::test]
    async fn test_get_documentation_score_lowercase_readme() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("readme.md"), "# Test").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 40.0);
    }

    #[tokio::test]
    async fn test_get_performance_score_with_benches() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("benches")).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 80.0); // 50 base + 30 for benches
    }

    #[tokio::test]
    async fn test_get_performance_score_with_criterion() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
criterion = "0.5"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for criterion
    }

    #[tokio::test]
    async fn test_get_performance_score_with_both() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("benches")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
criterion = "0.5"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 100.0); // 50 base + 30 benches + 20 criterion = 100 (capped)
    }

    #[tokio::test]
    async fn test_get_mutation_score_with_mutants_config() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("mutants.toml"), "[mutants]").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for config
    }

    #[tokio::test]
    async fn test_get_mutation_score_with_mutants_dir() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join(".mutants")).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for results dir
    }

    #[tokio::test]
    async fn test_get_mutation_score_with_all_indicators() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("mutants.toml"), "[mutants]").unwrap();
        fs::create_dir(temp_dir.path().join(".mutants")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
cargo-mutants = "1.0"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 100.0); // 50 + 20 + 20 + 10 = 100 (capped)
    }

    #[tokio::test]
    async fn test_get_coverage_score_from_cache() {
        let temp_dir = TempDir::new().unwrap();
        let metrics_dir = temp_dir.path().join(".pmat-metrics");
        fs::create_dir_all(&metrics_dir).unwrap();
        fs::write(metrics_dir.join("coverage.json"), r#"{"coverage": 85.5}"#).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        assert_eq!(score, 85.5);
    }

    #[tokio::test]
    async fn test_get_coverage_score_heuristic() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir(root.join("src")).unwrap();
        for i in 0..5_usize {
            let mut content = format!("// Source file {}\n", i);
            if i < 10 {
                content.push_str(&format!("\n#[test]\nfn test_{}_0 () {{}}\n", i));
            }
            fs::write(root.join("src").join(format!("mod_{}.rs", i)), content).unwrap();
        }
        fs::write(
            root.join("Cargo.toml"),
            r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        // Score based on test density heuristic
        assert!((50.0..=95.0).contains(&score));
    }

    #[tokio::test]
    async fn test_get_coverage_score_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // Default moderate estimate
    }

    // ============================================================================
    // Calculator Fast Mode Integration Test
    // ============================================================================

    #[tokio::test]
    async fn test_calculator_fast_mode_mutation_default_inline() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new().fast_mode(true);

        // In fast mode, mutation score should be 50.0 (default credit)
        let result = calc.calculate(temp_dir.path()).await.unwrap();

        let mutation_cat = result
            .categories
            .iter()
            .find(|c| c.name == "Mutation Testing")
            .unwrap();
        assert_eq!(mutation_cat.raw_score, 50.0);
        assert!(mutation_cat
            .details
            .as_ref()
            .is_some_and(|d| d.contains("fast mode")));
    }

    #[test]
    fn test_category_score_in_calculator_context() {
        // Test that CategoryScore created via calculator uses correct weights
        let score = CategoryScore::new("Technical Debt Grade", 75.0, 40);
        assert_eq!(score.earned_points, 30.0);
        assert_eq!(score.grade, "C");
    }

    // ============================================================================
    // RPS Normalization Tests (raw points → percentage, not raw → category)
    // ============================================================================

    #[test]
    fn test_rps_raw_points_normalize_to_category_fraction() {
        // RPS raw 246.6/289 = 85.3% → 0.853 * 30 ≈ 25.6/30, NOT 246.6 treated
        // as a percentage (which earned 55.2/30 and clamped total at 200 A+)
        let pct = normalize_rps_percentage(246.6, 289.0);
        assert!((pct - 85.328).abs() < 0.01, "pct was {}", pct);

        let score = CategoryScore::new("Rust Project Quality", pct, 30);
        assert!(
            (score.earned_points - 25.6).abs() < 0.01,
            "earned was {}",
            score.earned_points
        );
    }

    #[test]
    fn test_rps_perfect_input_earns_exactly_max() {
        let pct = normalize_rps_percentage(289.0, 289.0);
        assert_eq!(pct, 100.0);

        let score = CategoryScore::new("Rust Project Quality", pct, 30);
        assert_eq!(score.earned_points, 30.0);
    }

    #[test]
    fn test_rps_normalize_degenerate_inputs() {
        // Zero/negative max must not divide by zero or go negative
        assert_eq!(normalize_rps_percentage(100.0, 0.0), 0.0);
        assert_eq!(normalize_rps_percentage(-5.0, 289.0), 0.0);
        // Earned above max (should not happen) clamps to 100%
        assert_eq!(normalize_rps_percentage(300.0, 289.0), 100.0);
    }

    #[test]
    fn test_category_never_exceeds_max_points() {
        // Regression: raw 184.03 fed as a percentage must clamp at the
        // category max instead of earning 55.2/30
        let score = CategoryScore::new("Rust Project Quality", 184.03, 30);
        assert_eq!(score.earned_points, 30.0);
        let perfect = CategoryScore::new("Rust Project Quality", 100.0, 30);
        assert_eq!(
            score.grade, perfect.grade,
            "over-max raw must grade as the clamped 100%, not via overflow"
        );

        let negative = CategoryScore::new("Rust Project Quality", -10.0, 30);
        assert_eq!(negative.earned_points, 0.0);
        let zero = CategoryScore::new("Rust Project Quality", 0.0, 30);
        assert_eq!(
            negative.grade, zero.grade,
            "negative raw must grade as the clamped 0%"
        );
    }
}
