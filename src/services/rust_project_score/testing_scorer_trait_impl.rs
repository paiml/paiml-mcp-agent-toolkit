// Scorer trait implementation for TestingScorer
// Included from testing_scorer.rs - shares parent module scope

impl Scorer for TestingScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        // Backward compatibility: call with default mode and no cache
        self.score_internal(project_path, ScoringMode::default(), None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        // Backward compatibility: call with no cache
        self.score_internal(project_path, mode, None)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Kaizen Round 4: Use FileCache to eliminate 2 redundant src/*.rs reads
        self.score_internal(project_path, mode, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for nextest coverage anti-pattern (Five Whys discovery)
        if let Some(warning) = self.check_coverage_config_warning(project_path) {
            recommendations.push(warning);
        }

        // Check coverage - USE FALLBACK (no subprocess, no cache - backward compatibility)
        if let Ok(score) = self.score_coverage_fallback(project_path, None) {
            if score < 8.0 {
                recommendations.push(
                    "Improve test coverage: Install cargo-llvm-cov and aim for ≥85% line coverage"
                        .to_string(),
                );
            }
        }

        // Check integration tests
        if let Ok(score) = self.score_integration_tests(project_path) {
            if score < 4.0 {
                recommendations.push(
                    "Add integration tests: Create tests/ directory with end-to-end test files"
                        .to_string(),
                );
            }
        }

        // Check doc tests (no cache - backward compatibility)
        if let Ok(score) = self.score_doc_tests(project_path, None) {
            if score < 3.0 {
                recommendations.push(
                    "Add doc tests: Include runnable examples in /// documentation comments"
                        .to_string(),
                );
            }
        }

        // Note: mutation testing recommendation is in code_quality_scorer to avoid duplicates

        recommendations
    }
}
