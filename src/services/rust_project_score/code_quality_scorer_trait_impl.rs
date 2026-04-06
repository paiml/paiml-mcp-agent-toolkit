impl Default for CodeQualityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for CodeQualityScorer {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // Backward compatibility: call without cache
        self.score_internal(project_path, mode, None)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Kaizen Round 4: Use FileCache to eliminate 3 redundant src/*.rs reads
        self.score_internal(project_path, mode, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut recommendations = Vec::new();

        // Check complexity - USE SIMPLE FALLBACK (no subprocess, no cache)
        if let Ok(score) = self.score_complexity_simple(project_path, None) {
            if score < 3.0 {
                recommendations.push(
                    "Reduce cyclomatic complexity: refactor functions with >20 complexity into smaller units".to_string(),
                );
            }
        }

        // Check unsafe - Fast (filesystem only, no cache)
        if let Ok(score) = self.score_unsafe(project_path, None) {
            if score < 9.0 {
                recommendations.push(
                    "Add SAFETY comments for all unsafe blocks explaining invariants".to_string(),
                );
            }
        }

        // Check mutation - SKIP subprocess, always recommend
        recommendations.push(
            "Improve test quality: install cargo-mutants and aim for >=80% mutation score"
                .to_string(),
        );

        // Check dead code - Fast (filesystem only, no cache)
        if let Ok(score) = self.score_dead_code(project_path, None) {
            if score < 2.0 {
                recommendations.push(
                    "Remove dead code: delete or document unused functions with #[allow(dead_code)]".to_string(),
                );
            }
        }

        recommendations
    }
}

// SAFETY: CodeQualityScorer holds only a PathBuf (owned, Send+Sync) and no interior mutability,
// making it safe to send between and share across threads for parallel scoring.
unsafe impl Send for CodeQualityScorer {}
unsafe impl Sync for CodeQualityScorer {}
