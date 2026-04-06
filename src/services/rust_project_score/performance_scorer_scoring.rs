// Scoring methods for PerformanceScorer: benchmark detection, CI workflow analysis,
// and custom harness scoring against Cargo.toml and GitHub Actions configurations.

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext == "yml" || ext == "yaml")
        .unwrap_or(false)
}

fn has_benchmark_workflow(path: &Path) -> bool {
    if !is_yaml_file(path) {
        return false;
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    content.contains("cargo bench")
        || content.contains("benchmark")
        || content.contains("bench-baseline")
}

impl PerformanceScorer {
    /// Score Criterion benchmarks configured in [[bench]] sections (5pts)
    /// Based on "Learn from Rust Giants" specification
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for Cargo.toml
    /// **v2.0**: Simplified to match spec - checks for [[bench]] sections only
    fn score_benchmarks(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content_result = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).map(|s| s.to_string()).ok_or(())
        } else {
            std::fs::read_to_string(&cargo_toml_path).map_err(|_| ())
        };

        if let Ok(content) = content_result {
            // Check for [[bench]] sections in Cargo.toml
            if content.contains("[[bench]]") {
                return Ok(5.0);
            }
        }

        Ok(0.0)
    }

    /// Score CI workflow for benchmark baselines (3pts)
    /// Checks for .github/workflows with benchmark automation
    fn score_benchmark_ci(&self, project_path: &Path) -> ScorerResult<f64> {
        let workflows_dir = project_path.join(".github/workflows");
        if !workflows_dir.exists() {
            return Ok(0.0);
        }

        let entries = match std::fs::read_dir(&workflows_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(0.0),
        };

        for entry in entries.flatten() {
            if has_benchmark_workflow(&entry.path()) {
                return Ok(3.0);
            }
        }

        Ok(0.0)
    }

    /// Score harness = false for custom bench harness (2pts)
    /// Checks [[bench]] sections for harness = false
    fn score_custom_harness(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content_result = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).map(|s| s.to_string()).ok_or(())
        } else {
            std::fs::read_to_string(&cargo_toml_path).map_err(|_| ())
        };

        if let Ok(content) = content_result {
            // Check for harness = false in [[bench]] sections
            if content.contains("[[bench]]") && content.contains("harness = false") {
                return Ok(2.0);
            }
        }

        Ok(0.0)
    }

    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    /// **v2.0**: Aligned with "Learn from Rust Giants" specification
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

        // Score benchmarks - [[bench]] sections configured (5pts)
        match self.score_benchmarks(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score benchmark CI workflow (3pts)
        match self.score_benchmark_ci(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score custom harness (2pts)
        match self.score_custom_harness(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }
}
