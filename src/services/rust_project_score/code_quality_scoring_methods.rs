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

    /// Fast-mode mutation testing estimation
    ///
    /// Checks for mutation testing infrastructure (mutants.toml, cargo-mutants in Makefile)
    /// and gives proportional credit without running the expensive tool.
    fn estimate_mutation_fast(&self, project_path: &Path) -> f64 {
        let has_config = project_path.join("mutants.toml").exists()
            || project_path.join(".config/mutants.toml").exists();
        let makefile_content =
            std::fs::read_to_string(project_path.join("Makefile")).unwrap_or_default();
        let has_makefile_target = makefile_content.contains("cargo mutants")
            || makefile_content.contains("mutation");

        if has_config && has_makefile_target {
            5.0 // Infrastructure + build target = good credit
        } else if has_config || has_makefile_target {
            4.0 // Partial infrastructure
        } else {
            3.0 // Minimal credit (tests exist but no mutation setup)
        }
    }

    /// Fast-mode build time estimation
    ///
    /// Checks build configuration quality (LTO, profiles, .cargo/config.toml)
    /// and gives proportional credit without running a full build.
    fn estimate_build_time_fast(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> f64 {
        let cargo_content = cache
            .and_then(|c| c.get(&project_path.join("Cargo.toml")))
            .cloned()
            .or_else(|| std::fs::read_to_string(project_path.join("Cargo.toml")).ok())
            .unwrap_or_default();

        let mut score: f64 = 1.0; // Base credit for having a Rust project

        // Release profile optimization
        if cargo_content.contains("[profile.release]") {
            score += 0.5;
        }
        // LTO configured (build optimization)
        if cargo_content.contains("lto = ") {
            score += 0.5;
        }
        // .cargo/config.toml (build settings)
        if project_path.join(".cargo/config.toml").exists() {
            score += 0.5;
        }
        // Makefile or justfile (build automation)
        if project_path.join("Makefile").exists() || project_path.join("justfile").exists() {
            score += 0.5;
        }

        score.min(3.0) // Cap at 3.0 — reserve 4.0 for verified <30s builds
    }

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

        match self.score_complexity_simple(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        match self.score_unsafe(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        if mode.is_full() {
            match self.score_mutation(project_path) {
                Ok(score) => total_earned += score,
                Err(_) => total_earned += 4.0,
            }
        } else {
            total_earned += self.estimate_mutation_fast(project_path);
        }

        if mode.is_full() {
            match self.score_build_time(project_path) {
                Ok(score) => total_earned += score,
                Err(_) => total_earned += 2.0,
            }
        } else {
            total_earned += self.estimate_build_time_fast(project_path, cache);
        }

        match self.score_dead_code(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}
