// Analysis methods for TestingScorer: integration tests, doc tests, mutation, config warnings
// Included from testing_scorer.rs - shares parent module scope

impl TestingScorer {
    /// Score integration tests (4pts)
    /// Checks for tests/ directory with integration test files
    fn score_integration_tests(&self, project_path: &Path) -> ScorerResult<f64> {
        let tests_dir = project_path.join("tests");

        if !tests_dir.exists() {
            return Ok(0.0);
        }

        // Count integration test files
        let mut test_count = 0;

        if let Ok(entries) = std::fs::read_dir(&tests_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        test_count += 1;
                    }
                }
            }
        }

        // Scoring based on number of integration test files
        if test_count >= 3 {
            Ok(4.0) // >=3 integration test files
        } else if test_count >= 1 {
            Ok(3.0) // >=1 integration test file
        } else {
            Ok(0.0) // Empty tests/ directory
        }
    }

    /// Score doc tests (3pts)
    /// Checks for rustdoc examples in source files
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for src/*.rs
    fn score_doc_tests(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(0.0);
        }

        let mut doc_test_count = 0;

        // Recursively walk src directory
        self.count_doc_tests(&src_path, &mut doc_test_count, cache)?;

        // Scoring based on number of doc tests
        if doc_test_count >= 5 {
            Ok(3.0) // >=5 doc tests
        } else if doc_test_count >= 3 {
            Ok(2.0) // >=3 doc tests
        } else if doc_test_count >= 1 {
            Ok(1.0) // >=1 doc test
        } else {
            Ok(0.0) // No doc tests
        }
    }

    /// Count doc tests in directory (recursive)
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available
    fn count_doc_tests(
        &self,
        dir: &Path,
        count: &mut usize,
        cache: Option<&FileCache>,
    ) -> ScorerResult<()> {
        if let Some(cache) = cache {
            for (_path, content) in cache.get_rust_files_in_dir(dir) {
                *count += Self::count_doc_test_markers(content);
            }
        } else {
            self.count_doc_tests_from_fs(dir, count)?;
        }
        Ok(())
    }

    fn count_doc_tests_from_fs(&self, dir: &Path, count: &mut usize) -> ScorerResult<()> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.count_doc_tests_from_fs(&path, count)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    *count += Self::count_doc_test_markers(&content);
                }
            }
        }
        Ok(())
    }

    fn count_doc_test_markers(content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let t = line.trim();
                (t.starts_with("///") || t.starts_with("//!")) && t.contains("```")
            })
            .count()
    }

    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    fn score_internal(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score coverage (8pts)
        // FAST MODE: Skip expensive cargo llvm-cov, use heuristic estimation (#243)
        if mode.is_full() {
            {
                let score = self.score_coverage(project_path)?;
                total_earned += score
            }
        } else {
            // Fast mode: Estimate from project metadata (#243)
            total_earned += self.estimate_coverage_fast(project_path, cache);
        }

        // Score integration tests (4pts) - Fast (filesystem check, no cache benefit)
        {
            let score = self.score_integration_tests(project_path)?;
            total_earned += score
        }

        // Score doc tests (3pts) - Fast (filesystem check with cache)
        {
            let score = self.score_doc_tests(project_path, cache)?;
            total_earned += score
        }

        // Score mutation testing (5pts)
        // FAST MODE: Skip expensive cargo mutants, estimate from metadata (#243)
        if mode.is_full() {
            {
                let score = self.score_mutation(project_path)?;
                total_earned += score
            }
        } else {
            // Fast mode: Estimate from project metadata (#243)
            total_earned += self.estimate_mutation_fast(project_path);
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    /// Check for nextest coverage anti-pattern (Five Whys discovery)
    ///
    /// Problem: Using `cargo llvm-cov nextest` generates 1 profraw file per test,
    /// leading to O(n^2) memory usage in llvm-profdata merge (14GB+ RAM, 90+ min).
    ///
    /// Solution: Use `cargo llvm-cov test` which generates 1 profraw per binary (~5 files).
    ///
    /// Returns a warning message if the anti-pattern is detected.
    fn check_coverage_config_warning(&self, project_path: &Path) -> Option<String> {
        // Check Makefile for coverage configuration
        let makefile_path = project_path.join("Makefile");
        if let Ok(content) = std::fs::read_to_string(&makefile_path) {
            // Look for nextest in coverage context
            let has_nextest_coverage = content.lines().any(|line| {
                let line_lower = line.to_lowercase();
                (line_lower.contains("llvm-cov") || line_lower.contains("coverage"))
                    && line_lower.contains("nextest")
            });

            // Check if there's a profraw cleanup guard
            let has_profraw_guard = content.contains("profraw")
                && (content.contains("-delete") || content.contains("clean"));

            if has_nextest_coverage && !has_profraw_guard {
                return Some(
                    "\u{26a0}\u{fe0f}  Coverage config: Uses nextest (1 profraw/test = slow merge). \
                     Consider `cargo llvm-cov test` (1 profraw/binary) or add profraw cleanup guard."
                        .to_string(),
                );
            }
        }

        // Also check .config/nextest.toml for coverage profile without timeout
        let nextest_config = project_path.join(".config/nextest.toml");
        if nextest_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&nextest_config) {
                if content.contains("[profile.coverage]") && !content.contains("terminate-after") {
                    return Some(
                        "\u{26a0}\u{fe0f}  nextest coverage profile missing timeout. Add `terminate-after = 1` \
                         to prevent hanging tests from blocking coverage."
                            .to_string(),
                    );
                }
            }
        }

        None
    }

    /// Score mutation testing (5pts)
    /// >=80% mutation score = full points
    fn score_mutation(&self, project_path: &Path) -> ScorerResult<f64> {
        let output = Command::new("cargo")
            .arg("mutants")
            .arg("--no-times")
            .current_dir(project_path)
            .output();

        let Ok(result) = output else { return Ok(2.5) }; // not installed
        let stdout = String::from_utf8_lossy(&result.stdout);
        Ok(Self::parse_mutation_score(&stdout))
    }

    fn parse_mutation_score(stdout: &str) -> f64 {
        let Some(caught_line) = stdout.lines().find(|l| l.contains("caught")) else {
            return 2.5;
        };
        let caught = caught_line
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let total = stdout.lines().filter(|l| l.contains("mutant")).count() as f64;
        if total <= 0.0 {
            return 2.5;
        }
        Self::mutation_ratio_to_score(caught / total)
    }

    fn mutation_ratio_to_score(ratio: f64) -> f64 {
        match () {
            _ if ratio >= 0.80 => 5.0,
            _ if ratio >= 0.70 => 4.0,
            _ if ratio >= 0.60 => 3.0,
            _ if ratio >= 0.50 => 2.0,
            _ => 1.0,
        }
    }
}
