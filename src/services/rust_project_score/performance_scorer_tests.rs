// Tests for PerformanceScorer: benchmark detection, CI workflow scoring,
// custom harness detection, cache integration, recommendations, and scoring modes.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = PerformanceScorer::new();
        assert_eq!(scorer.name(), "Performance & Benchmarking");
        assert_eq!(scorer.max_points(), 10.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = PerformanceScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = PerformanceScorer::default();
        assert_eq!(scorer.name(), "Performance & Benchmarking");
        assert_eq!(scorer.max_points(), 10.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = PerformanceScorer::new();

        let result = scorer.score(temp_dir.path());
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml found"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    #[test]
    fn test_benchmarks_no_bench_section() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmarks(temp_dir.path(), None).unwrap();

        // No [[bench]] = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_benchmarks_with_bench_section() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"

[[bench]]
name = "my_benchmark"
harness = true
"#,
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmarks(temp_dir.path(), None).unwrap();

        // Has [[bench]] = full points
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_benchmarks_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content = "[package]\nname = \"test\"\n\n[[bench]]\nname = \"perf\"\n";
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = PerformanceScorer::new();
        let result = scorer
            .score_benchmarks(temp_dir.path(), Some(&cache))
            .unwrap();

        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_benchmark_ci_no_workflows() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmark_ci(temp_dir.path()).unwrap();

        // No workflows = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_benchmark_ci_no_bench_workflow() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/ci.yml"),
            "name: CI\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmark_ci(temp_dir.path()).unwrap();

        // No benchmark workflow = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_benchmark_ci_with_cargo_bench() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/benchmark.yml"),
            "name: Benchmark\non: push\njobs:\n  bench:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo bench\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmark_ci(temp_dir.path()).unwrap();

        // Has cargo bench = 3 points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_benchmark_ci_with_benchmark_keyword() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/perf.yml"),
            "name: Performance\non: push\njobs:\n  benchmark:\n    runs-on: ubuntu-latest\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmark_ci(temp_dir.path()).unwrap();

        // Has benchmark keyword = 3 points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_benchmark_ci_yaml_extension() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/bench.yaml"),
            "name: Bench\non: push\njobs:\n  bench:\n    steps:\n      - run: cargo bench\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmark_ci(temp_dir.path()).unwrap();

        // .yaml extension also works
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_custom_harness_none() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_custom_harness(temp_dir.path(), None).unwrap();

        // No [[bench]] = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_custom_harness_without_false() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[[bench]]
name = "my_bench"
"#,
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_custom_harness(temp_dir.path(), None).unwrap();

        // [[bench]] without harness = false = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_custom_harness_with_false() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[[bench]]
name = "my_bench"
harness = false
"#,
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_custom_harness(temp_dir.path(), None).unwrap();

        // Has harness = false = 2 points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_custom_harness_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content =
            "[package]\nname = \"test\"\n\n[[bench]]\nname = \"b\"\nharness = false\n";
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = PerformanceScorer::new();
        let result = scorer
            .score_custom_harness(temp_dir.path(), Some(&cache))
            .unwrap();

        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_score_full_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[[bench]]
name = "my_bench"
harness = false
"#,
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/bench.yml"),
            "name: Bench\nsteps:\n  - run: cargo bench",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // bench(5) + ci(3) + harness(2) = 10
        assert_eq!(result.earned, 10.0);
        assert_eq!(result.max, 10.0);
    }

    #[test]
    fn test_score_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // No benchmarks = 0 points
        assert_eq!(result.earned, 0.0);
        assert_eq!(result.max, 10.0);
    }

    #[test]
    fn test_score_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_content = "[package]\nname = \"test\"\n\n[[bench]]\nname = \"b\"\n";
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("Cargo.toml"),
            cargo_content.to_string(),
        );

        let scorer = PerformanceScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        // Just [[bench]] = 5 points
        assert_eq!(result.earned, 5.0);
        assert_eq!(result.max, 10.0);
    }

    #[test]
    fn test_recommendations_no_benchmarks() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend all areas
        assert!(recommendations.iter().any(|r| r.contains("[[bench]]")));
        assert!(recommendations.iter().any(|r| r.contains("CI")));
        assert!(recommendations.iter().any(|r| r.contains("harness")));
    }

    #[test]
    fn test_recommendations_with_benchmarks() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[[bench]]\nname = \"b\"\nharness = false\n",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/bench.yml"),
            "cargo bench",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should have no recommendations for well-configured project
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_score_with_mode_fast() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[[bench]]\nname = \"b\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Mode doesn't affect performance scorer
        assert_eq!(result.earned, 5.0);
        assert_eq!(result.max, 10.0);
    }

    #[test]
    fn test_score_with_mode_full() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[[bench]]\nname = \"b\"\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Full)
            .unwrap();

        // Mode doesn't affect performance scorer
        assert_eq!(result.earned, 5.0);
        assert_eq!(result.max, 10.0);
    }

    #[test]
    fn test_benchmark_ci_bench_baseline() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/perf.yml"),
            "name: Performance\nsteps:\n  - uses: bench-baseline/action\n",
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmark_ci(temp_dir.path()).unwrap();

        // Has bench-baseline = 3 points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_multiple_bench_sections() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"

[[bench]]
name = "bench1"
harness = false

[[bench]]
name = "bench2"
harness = false
"#,
        )
        .unwrap();

        let scorer = PerformanceScorer::new();
        let result = scorer.score_benchmarks(temp_dir.path(), None).unwrap();

        // Multiple [[bench]] still = 5 points (presence check)
        assert_eq!(result, 5.0);
    }
}
