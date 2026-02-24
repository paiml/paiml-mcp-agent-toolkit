// Coverage scoring methods for TestingScorer
// Included from testing_scorer.rs - shares parent module scope

impl TestingScorer {
    /// Score test coverage (8pts)
    /// >=85% line coverage = full points
    fn score_coverage(&self, project_path: &Path) -> ScorerResult<f64> {
        let output = Command::new("cargo")
            .arg("llvm-cov")
            .arg("--all-targets")
            .arg("--no-report")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                Ok(self
                    .parse_coverage(&stdout)
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
