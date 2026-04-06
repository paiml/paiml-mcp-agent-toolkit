// Scoring and trait implementations for KnownDefectsScorer
// included from known_defects_scorer.rs - shares parent module scope

impl KnownDefectsScorer {
    /// Calculate score based on unwrap count
    ///
    /// Scoring:
    /// - 0-99 unwraps: 20 points (perfect)
    /// - 100-199 unwraps: 15 points (-5)
    /// - 200-299 unwraps: 10 points (-10)
    /// - 300-399 unwraps: 5 points (-15)
    /// - 400+ unwraps: 0 points (-20)
    fn calculate_unwrap_score(&self, production_unwraps: usize) -> f64 {
        let penalty = (production_unwraps / 100) as f64 * 5.0;
        let score = self.max_points - penalty;
        score.max(0.0) // Cannot go negative
    }

    /// Internal scoring logic
    fn score_internal(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let (production_unwraps, _test_unwraps) = self.count_unwraps(project_path, cache)?;
        let score = self.calculate_unwrap_score(production_unwraps);

        // Create category score
        let category_score = CategoryScore::new(score, self.max_points);

        Ok(category_score)
    }
}

impl Default for KnownDefectsScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for KnownDefectsScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        self.score_internal(project_path, None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        self.score(project_path)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut recommendations = Vec::new();

        if let Ok((production_unwraps, _test_unwraps)) = self.count_unwraps(project_path, None) {
            if production_unwraps > 0 {
                recommendations.push(format!(
                    "CRITICAL: {} unwrap() calls in production code - replace with .expect() or proper error handling (Cloudflare-class defect)",
                    production_unwraps
                ));
                recommendations.push(
                    "Run: cargo clippy -- -D clippy::disallowed-methods to enforce unwrap() ban"
                        .to_string(),
                );
                recommendations.push(
                    "See Cloudflare outage 2025-11-18: unwrap() panic caused 3+ hour network outage".to_string()
                );
            }
        }

        recommendations
    }
}

// SAFETY: KnownDefectsScorer holds only a PathBuf (owned, Send+Sync) and no interior mutability,
// making it safe to send between and share across threads for parallel scoring.
unsafe impl Send for KnownDefectsScorer {}
unsafe impl Sync for KnownDefectsScorer {}
