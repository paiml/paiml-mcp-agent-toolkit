// Code quality scoring: mutation testing, build time, and orchestration

impl CodeQualityScorer {
    /// Create a new CodeQualityScorer
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            name: "Code Quality".to_string(),
            max_points: 26.0,
        }
    }

    /// Score mutation testing (8pts)
    fn score_mutation(&self, project_path: &Path) -> ScorerResult<f64> {
        let output = Command::new("cargo")
            .arg("mutants")
            .arg("--no-times")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);

                if let Some(caught_line) = stdout.lines().find(|l| l.contains("caught")) {
                    let caught = caught_line
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let total = stdout.lines().filter(|l| l.contains("mutant")).count() as f64;

                    if total > 0.0 {
                        let ratio = caught / total;
                        if ratio >= 0.80 { Ok(8.0) }
                        else if ratio >= 0.70 { Ok(6.0) }
                        else if ratio >= 0.60 { Ok(4.0) }
                        else if ratio >= 0.50 { Ok(2.0) }
                        else { Ok(0.0) }
                    } else {
                        Ok(4.0)
                    }
                } else {
                    Ok(4.0)
                }
            }
            Err(_) => Ok(4.0),
        }
    }

    /// Score build time (4pts)
    fn score_build_time(&self, project_path: &Path) -> ScorerResult<f64> {
        let start = Instant::now();

        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(project_path)
            .output();

        let duration = start.elapsed();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let seconds = duration.as_secs();
                    if seconds < 30 { Ok(4.0) }
                    else if seconds < 60 { Ok(3.0) }
                    else if seconds < 120 { Ok(2.0) }
                    else if seconds < 300 { Ok(1.0) }
                    else { Ok(0.0) }
                } else {
                    Ok(0.0)
                }
            }
            Err(_) => Ok(2.0),
        }
    }

    /// Points that only a `--full` run can measure: Mutation Testing (8) and
    /// Build Time (4). In fast mode they are subtracted from the category
    /// maximum rather than awarded a heuristic.
    const UNMEASURED_IN_FAST_MODE: f64 = 12.0;

    /// Points the Complexity check is worth when it could be measured. When no
    /// function could be measured at all they leave the denominator, the same
    /// treatment Mutation Testing and Build Time get in fast mode.
    const COMPLEXITY_POINTS: f64 = 3.0;

    /// Internal scoring logic that accepts optional cache
    fn score_internal(
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

        // Complexity (3pts), measured with the AST visitor `analyze complexity`
        // and `quality-gate` use. A tree with no measurable function scores
        // nothing and costs nothing (#937).
        let unmeasured_complexity = match self.measure_cyclomatic(project_path, cache) {
            Some(profile) => {
                total_earned += score_from_cyclomatic(profile);
                0.0
            }
            None => Self::COMPLEXITY_POINTS,
        };

        match self.score_unsafe(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Mutation Testing (8pts) and Build Time (4pts) are only scored when
        // they were actually run.
        //
        // Fast mode used to award `estimate_mutation_fast` (a flat 3/4/5 of 8
        // decided by whether a mutants.toml or a Makefile mutation target
        // exists) and `estimate_build_time_fast` (1.0 plus 0.5 per config
        // heuristic, capped at 3 of 4). Neither measures anything about the
        // code, and the invented partial credit put the two modes on the same
        // 279-point scale while disagreeing about it: the same copy of the same
        // fixture scored Code Quality 18.0/26.0 (69.2%) fast and 26.0/26.0
        // (100.0%) under `--full`. A check that did not run is not a check that
        // scored 3 out of 8 — it is N/A, exactly as GPU/SIMD Quality already
        // reports when there is no GPU code to measure, so its points leave the
        // denominator too.
        let max_points = if mode.is_full() {
            match self.score_mutation(project_path) {
                Ok(score) => total_earned += score,
                Err(_) => total_earned += 4.0,
            }
            match self.score_build_time(project_path) {
                Ok(score) => total_earned += score,
                Err(_) => total_earned += 2.0,
            }
            self.max_points - unmeasured_complexity
        } else {
            self.max_points - Self::UNMEASURED_IN_FAST_MODE - unmeasured_complexity
        };

        match self.score_dead_code(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, max_points))
    }
}

#[cfg(test)]
mod fast_mode_not_measured_tests {
    use super::*;
    use tempfile::TempDir;

    fn trivial_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        std::fs::write(temp.path().join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        temp
    }

    /// Fast mode scored two checks it never ran — Mutation Testing (8pts) and
    /// Build Time (4pts) — with heuristics that measure no code at all, so the
    /// same fixture reported Code Quality 69.2% fast and 100.0% under `--full`.
    /// A check that did not run leaves the denominator.
    #[test]
    fn fast_mode_excludes_the_checks_it_never_ran() {
        let temp = trivial_project();
        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_internal(temp.path(), ScoringMode::Fast, None)
            .expect("fast score");

        assert_eq!(
            result.max, 14.0,
            "Mutation Testing (8) + Build Time (4) must leave the fast-mode maximum"
        );
        assert!(
            result.earned <= result.max,
            "earned {} exceeds the measured maximum {}",
            result.earned,
            result.max
        );
        // A project with nothing wrong in the three checks fast mode CAN run
        // must read 100%, not the 69.2% the invented partial credit produced.
        assert!(
            (result.percentage() - 100.0).abs() < 0.01,
            "expected 100% of the measured checks, got {}% ({} / {})",
            result.percentage(),
            result.earned,
            result.max
        );
    }

    /// The heuristics are gone, so no filesystem trinket can move the score:
    /// a mutants.toml and a Makefile mutation target used to be worth 2 points
    /// each way without a single mutant ever being run.
    #[test]
    fn mutation_infrastructure_files_do_not_move_the_fast_score() {
        let bare = trivial_project();
        let dressed = trivial_project();
        std::fs::write(dressed.path().join("mutants.toml"), "exclude = []\n").unwrap();
        std::fs::write(dressed.path().join("Makefile"), "mutation:\n\tcargo mutants\n").unwrap();

        let scorer = CodeQualityScorer::new();
        let bare_score = scorer
            .score_internal(bare.path(), ScoringMode::Fast, None)
            .expect("bare");
        let dressed_score = scorer
            .score_internal(dressed.path(), ScoringMode::Fast, None)
            .expect("dressed");

        assert_eq!(bare_score.earned, dressed_score.earned);
        assert_eq!(bare_score.max, dressed_score.max);
    }
}
