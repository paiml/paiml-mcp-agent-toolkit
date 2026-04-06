impl DocumentationScorer {
    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    fn score_internal(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Rustdoc coverage (7pts)
        match self.score_rustdoc(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // README quality (5pts)
        match self.score_readme(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Changelog presence (3pts)
        match self.score_changelog(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}

impl Scorer for DocumentationScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // Backward compatibility: call without cache
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
        // Kaizen Round 4: Use FileCache for README, CHANGELOG, and src/*.rs
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut recommendations = Vec::new();

        // Check rustdoc (no cache - backward compatibility)
        if let Ok(score) = self.score_rustdoc(project_path, None) {
            if score < 7.0 {
                recommendations.push(
                    "Improve rustdoc coverage: Add /// documentation to public API items with examples".to_string(),
                );
            }
        }

        // Check README (no cache - backward compatibility)
        if let Ok(score) = self.score_readme(project_path, None) {
            if score < 5.0 {
                recommendations.push(
                    "Improve README: Add Installation, Usage, Examples, and License sections"
                        .to_string(),
                );
            }
        }

        // Check changelog (no cache - backward compatibility)
        if let Ok(score) = self.score_changelog(project_path, None) {
            if score == 0.0 {
                recommendations.push(
                    "Add CHANGELOG.md: Document version history and changes between releases"
                        .to_string(),
                );
            } else if score < 3.0 {
                recommendations.push(
                    "Expand CHANGELOG.md: Add more version entries for full documentation credit"
                        .to_string(),
                );
            }
        }

        recommendations
    }
}
