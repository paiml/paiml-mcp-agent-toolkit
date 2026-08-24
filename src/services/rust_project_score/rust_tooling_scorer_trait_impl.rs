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
        // Points for checks that did not run. They leave the DENOMINATOR
        // rather than being awarded at half marks (#1035) — the same rule
        // Code Quality applies via `UNMEASURED_IN_FAST_MODE`.
        let mut unmeasured = 0.0;

        for (measured, points) in [
            (
                self.score_clippy_by_mode(project_path, mode)?,
                Self::CLIPPY_POINTS,
            ),
            (
                self.score_rustfmt_by_mode(project_path, mode)?,
                Self::RUSTFMT_POINTS,
            ),
            (
                self.score_audit_by_mode(project_path, mode)?,
                Self::AUDIT_POINTS,
            ),
        ] {
            match measured {
                Some(score) => total_earned += score,
                None => unmeasured += points,
            }
        }

        total_earned += self.score_cargo_deny(project_path)?;
        total_earned += self.score_workspace_lints(project_path, cache)?;
        total_earned += self.score_ci_cd_integration(project_path, cache)?;
        total_earned += self.score_docs_rs_metadata(project_path, cache)?;
        total_earned += self.score_workspace_organization(project_path, cache)?;
        total_earned += self.score_release_automation(project_path, cache)?;
        total_earned += self.score_msrv_tracking(project_path, cache)?;
        total_earned += self.score_release_profiles(project_path, cache)?;

        Ok(CategoryScore::new(
            total_earned,
            self.max_points - unmeasured,
        ))
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

#[cfg(test)]
mod fast_mode_not_measured_tests {
    use super::*;

    fn trivial_project() -> tempfile::TempDir {
        let temp = tempfile::TempDir::new().expect("temp dir");
        std::fs::create_dir_all(temp.path().join("src")).expect("mkdir");
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"t\"\nversion = \"0.1.0\"\n",
        )
        .expect("write manifest");
        std::fs::write(temp.path().join("src/lib.rs"), "pub fn f() {}\n").expect("write lib");
        temp
    }

    /// #1035: fast mode — the DEFAULT for `pmat rust-project-score` — awarded
    /// 5.0/10 for clippy, 2.5-3.0/5 for rustfmt and 3.5/7 for cargo-audit
    /// without running any of them. `score_audit_by_mode` returned its 3.5
    /// before it could look at the project at all: a check that could not fail,
    /// scoring a security audit that never happened.
    ///
    /// Points for a check that did not run leave the denominator.
    #[test]
    fn fast_mode_excludes_the_three_tool_checks_it_never_runs() {
        let temp = trivial_project();
        let scorer = RustToolingScorer::new();
        let full_max = scorer.max_points();
        let result = scorer
            .score_internal(temp.path(), ScoringMode::Fast, None)
            .expect("fast score");

        assert_eq!(
            result.max,
            full_max - 22.0,
            "clippy (10) + rustfmt (5) + cargo-audit (7) must leave the \
             fast-mode maximum, not be paid at half marks"
        );
        assert!(
            result.earned <= result.max,
            "earned {} exceeds the measured maximum {}",
            result.earned,
            result.max
        );
    }

    /// Counter-test one, and the more important half: the checks fast mode CAN
    /// run must still be scored. A fix that removed the whole category would
    /// pass the test above and measure nothing at all.
    #[test]
    fn fast_mode_still_measures_the_checks_it_can_run() {
        let temp = trivial_project();
        let result = RustToolingScorer::new()
            .score_internal(temp.path(), ScoringMode::Fast, None)
            .expect("fast score");
        assert!(
            result.max >= 100.0,
            "108 of 130 points are still measurable in fast mode; got max {}",
            result.max
        );
    }

    /// Counter-test two: a `rustfmt.toml` nobody ran must not move the score.
    ///
    /// The fast branch used to pay 3.0 for the file's presence and 2.5 for its
    /// absence — half a point for a filesystem trinket, the same shape this
    /// repository already deleted from Code Quality's mutation heuristic.
    #[test]
    fn a_rustfmt_config_file_does_not_move_the_fast_score() {
        let bare = trivial_project();
        let dressed = trivial_project();
        std::fs::write(dressed.path().join("rustfmt.toml"), "edition = \"2021\"\n")
            .expect("write rustfmt.toml");

        let scorer = RustToolingScorer::new();
        let a = scorer
            .score_internal(bare.path(), ScoringMode::Fast, None)
            .expect("bare");
        let b = scorer
            .score_internal(dressed.path(), ScoringMode::Fast, None)
            .expect("dressed");

        assert_eq!(
            (a.earned, a.max),
            (b.earned, b.max),
            "a config file nothing ran moved the score: bare {a:?} vs dressed {b:?}"
        );
    }

    /// And the by-mode wrappers say NOT MEASURED rather than a number, which is
    /// the fact the denominator arithmetic above is derived from.
    #[test]
    fn the_by_mode_wrappers_report_not_measured_in_fast_mode() {
        let temp = trivial_project();
        let scorer = RustToolingScorer::new();
        assert_eq!(
            scorer
                .score_clippy_by_mode(temp.path(), ScoringMode::Fast)
                .expect("clippy"),
            None
        );
        assert_eq!(
            scorer
                .score_rustfmt_by_mode(temp.path(), ScoringMode::Fast)
                .expect("rustfmt"),
            None
        );
        assert_eq!(
            scorer
                .score_audit_by_mode(temp.path(), ScoringMode::Fast)
                .expect("audit"),
            None
        );
    }
}
