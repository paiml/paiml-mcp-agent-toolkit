// Scorer trait implementation for RustToolingScorer
// Included into rust_tooling_scorer.rs

impl RustToolingScorer {
    /// Internal scoring logic that accepts optional cache and mode
    pub(super) fn score_internal(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        total_earned += self.score_clippy_by_mode(project_path, mode)?;
        total_earned += self.score_rustfmt_by_mode(project_path, mode)?;
        total_earned += self.score_audit_by_mode(project_path, mode)?;
        total_earned += self.score_cargo_deny(project_path)?;
        total_earned += self.score_workspace_lints(project_path, cache)?;
        total_earned += self.score_ci_cd_integration(project_path, cache)?;
        total_earned += self.score_docs_rs_metadata(project_path, cache)?;
        total_earned += self.score_workspace_organization(project_path, cache)?;
        total_earned += self.score_release_automation(project_path, cache)?;
        total_earned += self.score_msrv_tracking(project_path, cache)?;
        total_earned += self.score_release_profiles(project_path, cache)?;

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}

impl Scorer for RustToolingScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        // Kaizen Round 4: Cache support added for API consistency
        // Note: This scorer only does file existence checks and subprocess calls,
        // so cache is not actually used (no file reads to optimize)
        self.score_internal(project_path, mode, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut recommendations = vec![
            "Run 'cargo clippy --fix' to automatically fix clippy warnings".to_string(),
            "Run 'cargo fmt' to format code according to Rust style guidelines".to_string(),
            "Run 'cargo audit' and update vulnerable dependencies".to_string(),
        ];

        if self.score_cargo_deny(project_path).is_ok_and(|s| s < 3.0) {
            recommendations
                .push("Add deny.toml configuration for dependency policy enforcement".to_string());
        }

        if self
            .score_workspace_lints(project_path, None)
            .is_ok_and(|s| s < 12.0)
        {
            recommendations.extend(Self::lint_recommendations(project_path));
        }

        recommendations
    }
}

// SAFETY: RustToolingScorer holds only a PathBuf (owned, Send+Sync) and no interior mutability,
// making it safe to send between and share across threads for parallel scoring.
unsafe impl Send for RustToolingScorer {}
unsafe impl Sync for RustToolingScorer {}
