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

    // ========================================================================
    // #938: three categories were a 50.0 constant plus file-existence bonuses
    //
    // Every assertion below used to encode the arithmetic of that constant
    // ("50 base + 30 for benches"). A benchmark that was never run, a mutant
    // that was never generated and a line of code that was never executed are
    // all *not measured*, and now say so.
    // ========================================================================

    #[tokio::test]
    async fn test_performance_is_not_measured_from_a_benches_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("benches")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[dev-dependencies]\ncriterion = \"0.5\"\n",
        )
        .unwrap();

        let calc = PerfectionScoreCalculator::new();
        let err = calc
            .get_performance_score(temp_dir.path())
            .await
            .expect_err("an empty benches/ dir is not a benchmark run");
        assert!(err.contains("cargo bench"), "unhelpful reason: {err}");
    }

    #[tokio::test]
    async fn test_performance_scores_criterion_comparisons() {
        let temp_dir = TempDir::new().unwrap();
        let criterion = temp_dir.path().join("target/criterion");
        for (name, mean) in [("fast", -0.02), ("slow", 0.40), ("steady", 0.001)] {
            let dir = criterion.join(name).join("change");
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("estimates.json"),
                format!("{{\"mean\":{{\"point_estimate\":{mean}}}}}"),
            )
            .unwrap();
        }

        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await.unwrap();
        // 1 of 3 benchmarks regressed past the 5% noise threshold.
        assert!(
            (score - (2.0 / 3.0 * 100.0)).abs() < 0.001,
            "score was {score}"
        );
    }

    #[tokio::test]
    async fn test_mutation_is_not_measured_from_config_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("mutants.toml"), "[mutants]").unwrap();
        fs::create_dir(temp_dir.path().join(".mutants")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[dev-dependencies]\ncargo-mutants = \"1.0\"\n",
        )
        .unwrap();

        let calc = PerfectionScoreCalculator::new();
        let err = calc
            .get_mutation_score(temp_dir.path())
            .await
            .expect_err("configuration is not a mutation run");
        assert!(err.contains("cargo mutants"), "unhelpful reason: {err}");
    }

    #[tokio::test]
    async fn test_mutation_scores_cargo_mutants_outcomes() {
        let temp_dir = TempDir::new().unwrap();
        let out = temp_dir.path().join("mutants.out");
        fs::create_dir_all(&out).unwrap();
        fs::write(
            out.join("outcomes.json"),
            r#"{"outcomes":[
                {"scenario":"Baseline","summary":"Success"},
                {"scenario":{"Mutant":{}},"summary":"CaughtMutant"},
                {"scenario":{"Mutant":{}},"summary":"CaughtMutant"},
                {"scenario":{"Mutant":{}},"summary":"CaughtMutant"},
                {"scenario":{"Mutant":{}},"summary":"MissedMutant"},
                {"scenario":{"Mutant":{}},"summary":"Unviable"}
            ]}"#,
        )
        .unwrap();

        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await.unwrap();
        // 3 caught of 4 viable; the unviable mutant and the baseline are excluded.
        assert_eq!(score, 75.0);
    }

    #[tokio::test]
    async fn test_get_coverage_score_from_cache() {
        let temp_dir = TempDir::new().unwrap();
        let metrics_dir = temp_dir.path().join(".pmat-metrics");
        fs::create_dir_all(&metrics_dir).unwrap();
        fs::write(metrics_dir.join("coverage.json"), r#"{"coverage": 85.5}"#).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await.unwrap();
        assert_eq!(score, 85.5);
    }

    /// Counting `#[test]` attributes measures how many tests were written, not
    /// how much code they execute. It used to produce
    /// `50 + test_count * 0.1 + density * 5` and call it coverage.
    #[tokio::test]
    async fn test_coverage_is_not_estimated_from_test_attribute_counts() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir(root.join("src")).unwrap();
        for i in 0..5_usize {
            fs::write(
                root.join("src").join(format!("mod_{i}.rs")),
                format!("// Source file {i}\n\n#[test]\nfn test_{i}_0 () {{}}\n"),
            )
            .unwrap();
        }
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"t\"\n").unwrap();

        let calc = PerfectionScoreCalculator::new();
        let err = calc
            .get_coverage_score(root)
            .await
            .expect_err("test attributes are not coverage");
        assert!(err.contains("llvm-cov"), "unhelpful reason: {err}");
    }

    #[tokio::test]
    async fn test_get_coverage_score_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new();
        assert!(
            calc.get_coverage_score(temp_dir.path()).await.is_err(),
            "an empty project has no coverage, not 70%"
        );
    }

    /// #938's reproduction, as a test: four empty files must not move the score.
    #[tokio::test]
    async fn test_empty_files_do_not_move_the_total() {
        fn bare_crate() -> TempDir {
            let dir = TempDir::new().unwrap();
            fs::create_dir(dir.path().join("src")).unwrap();
            fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"t\"\n").unwrap();
            fs::write(dir.path().join("src/lib.rs"), "//! x\n").unwrap();
            dir
        }

        let bare = bare_crate();
        let dressed = bare_crate();
        fs::write(dressed.path().join("mutants.toml"), "").unwrap();
        fs::create_dir(dressed.path().join(".mutants")).unwrap();
        fs::create_dir(dressed.path().join("benches")).unwrap();
        fs::write(
            dressed.path().join("Cargo.toml"),
            "[package]\nname = \"t\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n",
        )
        .unwrap();

        let calc = PerfectionScoreCalculator::new().fast_mode(true);
        let bare_result = calc.calculate(bare.path()).await.unwrap();
        let dressed_result = calc.calculate(dressed.path()).await.unwrap();

        let category = |r: &super::super::types::PerfectionScoreResult, name: &str| {
            r.categories
                .iter()
                .find(|c| c.name == name)
                .unwrap_or_else(|| panic!("missing category {name}"))
                .earned_points
        };
        for name in ["Mutation Testing", "Performance", "Test Coverage"] {
            assert_eq!(
                category(&bare_result, name),
                category(&dressed_result, name),
                "{name} moved on four empty files"
            );
            assert_eq!(
                category(&bare_result, name),
                0.0,
                "{name} was scored without evidence"
            );
        }
    }

    // ============================================================================
    // Calculator Fast Mode Integration Test
    // ============================================================================

    /// Fast mode cannot run mutation testing, so it must not award points for it.
    /// This test previously asserted the opposite — that the category came back
    /// with a flat raw_score of 50.0 ("default credit") — which is exactly how ten
    /// unearned points ended up inside a total presented as a grade, identically
    /// for a real repo and for a path that does not exist.
    #[tokio::test]
    async fn test_calculator_fast_mode_mutation_earns_nothing() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new().fast_mode(true);

        let result = calc.calculate(temp_dir.path()).await.unwrap();

        let mutation_cat = result
            .categories
            .iter()
            .find(|c| c.name == "Mutation Testing")
            .unwrap();
        assert_eq!(
            mutation_cat.earned_points, 0.0,
            "an unmeasured category must not contribute points"
        );
        assert_eq!(
            mutation_cat.max_points, 0,
            "an unmeasured category must not sit in the denominator"
        );
        assert!(mutation_cat
            .details
            .as_ref()
            .is_some_and(|d| d.contains("Not measured")));

        // The reported denominator must equal what was actually measured.
        // (This asserted a fixed 180 when Mutation Testing was the only
        // category that could be N/A; Test Coverage and Performance now drop
        // out too when no coverage or benchmark run left evidence — #938.)
        let summed: u16 = result.categories.iter().map(|c| c.max_points).sum();
        assert_eq!(summed, result.max_score);
        assert!(
            result.max_score <= 180,
            "mutation must have left the denominator, got {}",
            result.max_score
        );
    }

    // ========================================================================
    // #941: Technical Debt Grade contradicted `pmat tdg` on the same path
    // ========================================================================

    /// A file `pmat tdg` grades F must not be graded C+ by the same binary's
    /// perfection-score. The category used to run a separate `TDGCalculator` on
    /// a 0-5 debt scale converted as `100 - average_tdg * 20`, which reported
    /// 77.2 (C+, 30.9 of 40 points) for the fixture below while `pmat tdg` and
    /// `pmat analyze tdg` both reported 0.0/100 (F).
    #[tokio::test]
    async fn test_tdg_category_agrees_with_the_tdg_command() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"awful\"\n").unwrap();

        let mut src = String::from(
            "// TODO: this whole file is a mess\n// FIXME: rewrite\n// HACK: do not ship\n",
        );
        src.push_str(
            "pub fn monster(a: i32, b: i32, c: i32, d: i32) -> i32 {\n    let mut r = 0;\n",
        );
        for i in 0..60 {
            src.push_str(&format!(
                "    if a > {i} && b < {i} || c == {i} {{ r += {i}; }} else if d != {i} {{ r -= {i}; }}\n"
            ));
        }
        src.push_str("    r\n}\n");
        for i in 0..40 {
            src.push_str(&format!(
                "pub fn bad{i}(s: &str) -> i32 {{ // TODO fix bad{i}\n    let v: i32 = s.parse().unwrap();\n    if v < 0 {{ panic!(\"negative\"); }}\n    v\n}}\n"
            ));
        }
        fs::write(root.join("src/lib.rs"), src).unwrap();

        // What `pmat tdg` / `pmat analyze tdg` report for this tree.
        let expected = crate::tdg::TdgAnalyzer::new()
            .unwrap()
            .analyze_project(root)
            .await
            .unwrap()
            .average_score
            .expect("the fixture has a gradable file");

        let calc = PerfectionScoreCalculator::new();
        let measured = calc.get_tdg_score(root).await.expect("a gradable tree");

        assert!(
            (measured - f64::from(expected)).abs() < 0.001,
            "perfection-score says {measured}, `pmat tdg` says {expected}"
        );
    }

    /// A tree with nothing to grade is not a tree that scored 100 — or 0.
    #[tokio::test]
    async fn test_tdg_category_is_not_measured_without_source() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new();
        let err = calc
            .get_tdg_score(temp_dir.path())
            .await
            .expect_err("nothing to grade");
        assert!(err.contains("gradable"), "unhelpful reason: {err}");
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
