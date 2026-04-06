// Code quality scoring: mutation testing, build time, and orchestration

impl CodeQualityScorer {
    /// Create a new CodeQualityScorer
    pub fn new() -> Self {
        Self {
            name: "Code Quality".to_string(),
            max_points: 26.0,
        }
    }

    /// Score mutation testing (8pts)
    fn score_mutation(&self, project_path: &Path) -> ScorerResult<f64> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
            total_earned += 4.0;
        }

        if mode.is_full() {
            match self.score_build_time(project_path) {
                Ok(score) => total_earned += score,
                Err(_) => total_earned += 2.0,
            }
        } else {
            total_earned += 2.0;
        }

        match self.score_dead_code(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}
