// Coverage scoring methods for TestingScorer
// Included from testing_scorer.rs - shares parent module scope

impl TestingScorer {
    /// Score test coverage (8pts)
    /// >=85% line coverage = full points
    fn score_coverage(&self, project_path: &Path) -> ScorerResult<f64> {
        // First try reading cached coverage results (much faster than running llvm-cov)
        // Check .pmat-metrics/coverage.result (written by `make coverage`)
        let metrics_path = project_path.join(".pmat-metrics/coverage.result");
        if let Ok(content) = std::fs::read_to_string(&metrics_path) {
            if let Some(coverage) = self.parse_coverage_json(&content) {
                return Ok(Self::coverage_to_score(coverage));
            }
        }
        // Check .pmat/coverage-cache.json (written by `pmat query --coverage`)
        let cache_path = project_path.join(".pmat/coverage-cache.json");
        if let Ok(cache_content) = std::fs::read_to_string(&cache_path) {
            if let Some(coverage) = self.parse_coverage_json(&cache_content) {
                return Ok(Self::coverage_to_score(coverage));
            }
        }

        // Fall back to running cargo llvm-cov with text summary
        let output = Command::new("cargo")
            .arg("llvm-cov")
            .arg("--lib")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                // Parse from stdout (text report) or stderr (summary line)
                let combined = format!("{}\n{}", stdout, stderr);
                Ok(self
                    .parse_coverage(&combined)
                    .map(Self::coverage_to_score)
                    .unwrap_or(4.0))
            }
            Ok(_) => Ok(0.0), // Coverage run failed
            Err(_) => self.score_coverage_fallback(project_path, None),
        }
    }

    fn coverage_to_score(coverage: f64) -> f64 {
        match () {
            _ if coverage >= 85.0 => 8.0,
            _ if coverage >= 70.0 => 6.0,
            _ if coverage >= 50.0 => 4.0,
            _ if coverage >= 30.0 => 2.0,
            _ => 0.0,
        }
    }

    /// Fallback coverage scoring when cargo-llvm-cov not available
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for src/*.rs
    fn score_coverage_fallback(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(0.0);
        }
        let has_tests = Self::any_file_contains_tests(&src_path, cache);
        Ok(if has_tests { 4.0 } else { 0.0 })
    }

    fn any_file_contains_tests(src_path: &Path, cache: Option<&FileCache>) -> bool {
        if let Some(cache) = cache {
            return cache
                .get_rust_files_in_dir(src_path)
                .iter()
                .any(|(_p, content)| {
                    content.contains("#[cfg(test)]") || content.contains("#[test]")
                });
        }
        let Ok(entries) = std::fs::read_dir(src_path) else {
            return false;
        };
        entries.flatten().any(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "rs")
                && std::fs::read_to_string(entry.path())
                    .map(|c| c.contains("#[cfg(test)]") || c.contains("#[test]"))
                    .unwrap_or(false)
        })
    }

    /// Fast-mode coverage estimation from project metadata (#243)
    ///
    /// Checks for coverage infrastructure (tools, config, artifacts) and gives
    /// proportional credit without running the expensive coverage tool.
    fn estimate_coverage_fast(&self, project_path: &Path, cache: Option<&FileCache>) -> f64 {
        let mut score: f64 = 0.0;

        // Check if cargo-llvm-cov is configured in Cargo.toml or Makefile
        let cargo_content = cache
            .and_then(|c| c.get(&project_path.join("Cargo.toml")))
            .cloned()
            .or_else(|| std::fs::read_to_string(project_path.join("Cargo.toml")).ok())
            .unwrap_or_default();
        let makefile_content =
            std::fs::read_to_string(project_path.join("Makefile")).unwrap_or_default();

        // Coverage tool configured in build system
        if makefile_content.contains("llvm-cov") || makefile_content.contains("coverage") {
            score += 3.0;
        }

        // Coverage artifacts exist (lcov.info, .pmat/coverage-cache.json)
        if project_path.join("lcov.info").exists()
            || project_path.join(".pmat/coverage-cache.json").exists()
        {
            score += 2.0;
        }

        // Has inline tests (basic check)
        if cargo_content.contains("#[cfg(test)]")
            || Self::any_file_contains_tests(&project_path.join("src"), cache)
        {
            score += 1.0;
        }

        // Check for cached coverage results (strong signal)
        let has_metrics = project_path.join(".pmat-metrics/coverage.result").exists();
        if has_metrics {
            score += 1.0; // Cached coverage data from `make coverage`
        }

        // Cap at 7.0 — reserve top point for verified >85% coverage in full mode
        score.min(7.0)
    }

    /// Fast-mode mutation testing estimation from project metadata (#243)
    fn estimate_mutation_fast(&self, project_path: &Path) -> f64 {
        // Check if cargo-mutants is configured
        let makefile_content =
            std::fs::read_to_string(project_path.join("Makefile")).unwrap_or_default();
        let has_mutants_config = project_path.join("mutants.toml").exists()
            || project_path.join(".config/mutants.toml").exists()
            || makefile_content.contains("cargo mutants")
            || makefile_content.contains("mutant");

        if has_mutants_config {
            3.5 // Good credit for having mutation testing infrastructure
        } else {
            2.0 // Minimal credit
        }
    }

    /// Parse coverage from cached JSON (e.g., .pmat-metrics/coverage.result)
    fn parse_coverage_json(&self, json_str: &str) -> Option<f64> {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            // Format from `make coverage`: { "coverage_pct": 45.02, ... }
            if let Some(pct) = json.get("coverage_pct").and_then(|v| v.as_f64()) {
                return Some(pct);
            }
            // Format: { "line_coverage_percent": 94.2, ... }
            if let Some(pct) = json.get("line_coverage_percent").and_then(|v| v.as_f64()) {
                return Some(pct);
            }
            // Alternative format: { "coverage": 94.2, ... }
            if let Some(pct) = json.get("coverage").and_then(|v| v.as_f64()) {
                return Some(pct);
            }
        }
        None
    }

    /// Parse coverage percentage from cargo-llvm-cov output
    fn parse_coverage(&self, output: &str) -> Option<f64> {
        // Try to find coverage percentage in various formats
        for line in output.lines() {
            // Look for patterns like "85.0%" or "coverage: 85%"
            if let Some(pct_idx) = line.find('%') {
                let before = &line[..pct_idx];
                if let Some(num_start) = before.rfind(|c: char| !c.is_ascii_digit() && c != '.') {
                    let num_str = &before[num_start + 1..];
                    if let Ok(coverage) = num_str.parse::<f64>() {
                        return Some(coverage);
                    }
                }
            }
        }
        None
    }
}
