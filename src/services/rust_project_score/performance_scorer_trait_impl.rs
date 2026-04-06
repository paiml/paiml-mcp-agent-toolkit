// Scorer trait implementation for PerformanceScorer: delegates to internal scoring
// methods, provides backward-compatible score/score_with_mode/score_with_cache,
// and generates improvement recommendations.

impl Scorer for PerformanceScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        // Backward compatibility: call with no cache
        self.score_internal(project_path, None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        // This scorer doesn't have expensive operations, so mode doesn't affect it
        self.score(project_path)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Kaizen Round 4: Use FileCache to eliminate 2 redundant Cargo.toml reads
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check [[bench]] sections (no cache - backward compatibility)
        if let Ok(score) = self.score_benchmarks(project_path, None) {
            if score < 5.0 {
                recommendations.push(
                    "Add [[bench]] sections: Configure benchmark targets in Cargo.toml with Criterion".to_string(),
                );
            }
        }

        // Check benchmark CI workflow
        if let Ok(score) = self.score_benchmark_ci(project_path) {
            if score < 3.0 {
                recommendations.push(
                    "Add benchmark CI: Create .github/workflows with 'cargo bench' for automated performance testing".to_string(),
                );
            }
        }

        // Check custom harness
        if let Ok(score) = self.score_custom_harness(project_path, None) {
            if score < 2.0 {
                recommendations.push(
                    "Use custom harness: Add 'harness = false' to [[bench]] sections for Criterion integration".to_string(),
                );
            }
        }

        recommendations
    }
}
