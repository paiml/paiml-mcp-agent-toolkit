impl Default for CodeQualityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for CodeQualityScorer {
    fn name(&self) -> &str {
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
        let mut recommendations = Vec::new();

        // Complexity — measured, not guessed. This advice used to come from the
        // deep-nesting proxy (#937): a project could be told to "refactor
        // functions with >20 complexity" because eight lines somewhere were
        // indented past column 40, and an agent climbing the score would
        // reformat instead of refactor. It now names the functions that are
        // actually over the threshold `analyze complexity` flags.
        if let Some(profile) = self.measure_cyclomatic(project_path, None) {
            if score_from_cyclomatic(profile) < 3.0 {
                let error_threshold = crate::services::complexity::ComplexityThresholds::default()
                    .cyclomatic_error;
                recommendations.push(format!(
                    "Reduce cyclomatic complexity: {} of {} functions exceed {} (worst is {}); split them into smaller units",
                    profile.over_error, profile.functions, error_threshold, profile.max
                ));
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
                    "Remove dead code: delete or document unused functions with ".to_string(),
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
