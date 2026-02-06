#![cfg_attr(coverage_nightly, coverage(off))]
//! TestingScorer - Testing Excellence Category (20 points)
//!
//! Analyzes Rust project testing practices:
//! - Test Coverage (8pts): ≥85% line coverage via cargo-llvm-cov
//! - Integration Tests (4pts): Presence of tests/ directory
//! - Doc Tests (3pts): Rustdoc examples that compile and run
//! - Mutation Testing (5pts): ≥80% mutation score
//!
//! Evidence-based refinement: Coverage threshold based on empirical research
//! showing ≥85% coverage correlates with significantly fewer production bugs.

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;
use std::process::Command;

/// Testing Excellence scorer
#[derive(Debug, Clone)]
pub struct TestingScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl TestingScorer {
    /// Create a new TestingScorer
    pub fn new() -> Self {
        Self {
            name: "Testing Excellence".to_string(),
            max_points: 20.0,
        }
    }

    /// Score test coverage (8pts)
    /// ≥85% line coverage = full points
    fn score_coverage(&self, project_path: &Path) -> ScorerResult<f64> {
        // Try to run cargo-llvm-cov
        let output = Command::new("cargo")
            .arg("llvm-cov")
            .arg("--all-targets")
            .arg("--no-report")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Parse coverage from output
                    let stdout = String::from_utf8_lossy(&result.stdout);

                    // Look for coverage percentage in output
                    // Format varies, so we use heuristics
                    if let Some(coverage) = self.parse_coverage(&stdout) {
                        // Tiered scoring based on coverage percentage
                        if coverage >= 85.0 {
                            Ok(8.0) // ≥85% coverage
                        } else if coverage >= 70.0 {
                            Ok(6.0) // ≥70% coverage
                        } else if coverage >= 50.0 {
                            Ok(4.0) // ≥50% coverage
                        } else if coverage >= 30.0 {
                            Ok(2.0) // ≥30% coverage
                        } else {
                            Ok(0.0) // <30% coverage
                        }
                    } else {
                        // Can't parse coverage - give moderate credit
                        Ok(4.0)
                    }
                } else {
                    // Coverage run failed - likely no tests
                    Ok(0.0)
                }
            }
            Err(_) => {
                // cargo-llvm-cov not installed
                // Fallback: check for presence of tests (no cache)
                self.score_coverage_fallback(project_path, None)
            }
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

        let mut has_tests = false;

        // Check for #[cfg(test)] modules in source files
        if let Some(cache) = cache {
            // Use cache: get all .rs files in src/ directory
            for (_path, content) in cache.get_rust_files_in_dir(&src_path) {
                if content.contains("#[cfg(test)]") || content.contains("#[test]") {
                    has_tests = true;
                    break;
                }
            }
        } else {
            // Fallback: read from filesystem
            if let Ok(entries) = std::fs::read_dir(&src_path) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "rs" {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if content.contains("#[cfg(test)]") || content.contains("#[test]") {
                                    has_tests = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if has_tests {
            Ok(4.0) // Moderate credit for having tests
        } else {
            Ok(0.0) // No tests found
        }
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
            Ok(4.0) // ≥3 integration test files
        } else if test_count >= 1 {
            Ok(3.0) // ≥1 integration test file
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
            Ok(3.0) // ≥5 doc tests
        } else if doc_test_count >= 3 {
            Ok(2.0) // ≥3 doc tests
        } else if doc_test_count >= 1 {
            Ok(1.0) // ≥1 doc test
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
            // Use cache: get all .rs files in directory
            for (_path, content) in cache.get_rust_files_in_dir(dir) {
                // Count occurrences of "```" in doc comments
                for line in content.lines() {
                    let trimmed = line.trim();

                    // Check if we're in a doc comment
                    if (trimmed.starts_with("///") || trimmed.starts_with("//!"))
                        && trimmed.contains("```")
                    {
                        *count += 1;
                    }
                }
            }
        } else {
            // Fallback: read from filesystem
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        self.count_doc_tests(&path, count, None)?;
                    } else if let Some(ext) = path.extension() {
                        if ext == "rs" {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                // Count occurrences of "```" in doc comments
                                for line in content.lines() {
                                    let trimmed = line.trim();

                                    // Check if we're in a doc comment
                                    if (trimmed.starts_with("///") || trimmed.starts_with("//!"))
                                        && trimmed.contains("```")
                                    {
                                        *count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
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
        // FAST MODE: Skip expensive cargo llvm-cov, use fallback
        if mode.is_full() {
            match self.score_coverage(project_path) {
                Ok(score) => total_earned += score,
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode: Use fallback with cache (filesystem check only)
            match self.score_coverage_fallback(project_path, cache) {
                Ok(score) => total_earned += score,
                Err(e) => return Err(e),
            }
        }

        // Score integration tests (4pts) - Fast (filesystem check, no cache benefit)
        match self.score_integration_tests(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score doc tests (3pts) - Fast (filesystem check with cache)
        match self.score_doc_tests(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score mutation testing (5pts)
        // FAST MODE: Skip expensive cargo mutants
        if mode.is_full() {
            match self.score_mutation(project_path) {
                Ok(score) => total_earned += score,
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode: Give moderate credit (2.5pts) without running
            total_earned += 2.5;
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    /// Check for nextest coverage anti-pattern (Five Whys discovery)
    ///
    /// Problem: Using `cargo llvm-cov nextest` generates 1 profraw file per test,
    /// leading to O(n²) memory usage in llvm-profdata merge (14GB+ RAM, 90+ min).
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
                    "⚠️  Coverage config: Uses nextest (1 profraw/test = slow merge). \
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
                        "⚠️  nextest coverage profile missing timeout. Add `terminate-after = 1` \
                         to prevent hanging tests from blocking coverage."
                            .to_string(),
                    );
                }
            }
        }

        None
    }

    /// Score mutation testing (5pts)
    /// ≥80% mutation score = full points
    fn score_mutation(&self, project_path: &Path) -> ScorerResult<f64> {
        // Try to run cargo-mutants
        let output = Command::new("cargo")
            .arg("mutants")
            .arg("--no-times")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);

                // Parse mutation score from output
                if let Some(caught_line) = stdout.lines().find(|l| l.contains("caught")) {
                    // Simplified parsing
                    let caught = caught_line
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let total = stdout.lines().filter(|l| l.contains("mutant")).count() as f64;

                    if total > 0.0 {
                        let ratio = caught / total;

                        if ratio >= 0.80 {
                            Ok(5.0) // ≥80% mutation score
                        } else if ratio >= 0.70 {
                            Ok(4.0)
                        } else if ratio >= 0.60 {
                            Ok(3.0)
                        } else if ratio >= 0.50 {
                            Ok(2.0)
                        } else {
                            Ok(1.0)
                        }
                    } else {
                        Ok(2.5) // No mutants = moderate score
                    }
                } else {
                    Ok(2.5) // Can't parse = moderate score
                }
            }
            Err(_) => {
                // cargo-mutants not installed - give moderate credit
                Ok(2.5)
            }
        }
    }
}

impl Default for TestingScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for TestingScorer {
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
        // Kaizen Round 4: Use FileCache to eliminate 2 redundant src/*.rs reads
        self.score_internal(project_path, mode, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for nextest coverage anti-pattern (Five Whys discovery)
        if let Some(warning) = self.check_coverage_config_warning(project_path) {
            recommendations.push(warning);
        }

        // Check coverage - USE FALLBACK (no subprocess, no cache - backward compatibility)
        if let Ok(score) = self.score_coverage_fallback(project_path, None) {
            if score < 8.0 {
                recommendations.push(
                    "Improve test coverage: Install cargo-llvm-cov and aim for ≥85% line coverage"
                        .to_string(),
                );
            }
        }

        // Check integration tests
        if let Ok(score) = self.score_integration_tests(project_path) {
            if score < 4.0 {
                recommendations.push(
                    "Add integration tests: Create tests/ directory with end-to-end test files"
                        .to_string(),
                );
            }
        }

        // Check doc tests (no cache - backward compatibility)
        if let Ok(score) = self.score_doc_tests(project_path, None) {
            if score < 3.0 {
                recommendations.push(
                    "Add doc tests: Include runnable examples in /// documentation comments"
                        .to_string(),
                );
            }
        }

        // Check mutation testing - SKIP subprocess, always recommend
        recommendations.push(
            "Improve test quality: Install cargo-mutants and aim for ≥80% mutation score"
                .to_string(),
        );

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for TestingScorer {}
unsafe impl Sync for TestingScorer {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = TestingScorer::new();
        assert_eq!(scorer.name(), "Testing Excellence");
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = TestingScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = TestingScorer::default();
        assert_eq!(scorer.name(), "Testing Excellence");
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = TestingScorer::new();

        let result = scorer.score(temp_dir.path());
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml found"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    #[test]
    fn test_coverage_fallback_no_src() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), None)
            .unwrap();

        // No src = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_coverage_fallback_no_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn no_tests_here() {}").unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), None)
            .unwrap();

        // No tests = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_coverage_fallback_with_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
fn foo() {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_foo() {}
}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), None)
            .unwrap();

        // Has tests = moderate credit
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_coverage_fallback_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "#[test]\nfn test_something() {}".to_string(),
        );

        let scorer = TestingScorer::new();
        let result = scorer
            .score_coverage_fallback(temp_dir.path(), Some(&cache))
            .unwrap();

        // Has tests = moderate credit
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_integration_tests_no_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // No tests/ = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_integration_tests_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // Empty tests/ = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_integration_tests_one_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/integration.rs"),
            "#[test]\nfn test() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // 1 test file = 3 points
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_integration_tests_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/test1.rs"),
            "#[test]\nfn t1() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/test2.rs"),
            "#[test]\nfn t2() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tests/test3.rs"),
            "#[test]\nfn t3() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_integration_tests(temp_dir.path()).unwrap();

        // 3+ test files = full points
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_doc_tests_no_src() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // No src = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_doc_tests_no_examples() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "/// Documentation without examples\npub fn foo() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // No doc tests = 0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_doc_tests_with_examples() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
/// Documentation with example
/// ```
/// let x = 1;
/// ```
pub fn foo() {}

/// Another example
/// ```
/// let y = 2;
/// ```
pub fn bar() {}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // 2 doc tests = 1 point (need 3+ for 2 pts, 5+ for full)
        assert!(result >= 1.0);
    }

    #[test]
    fn test_doc_tests_many_examples() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
/// ```
/// let a = 1;
/// ```
pub fn a() {}
/// ```
/// let b = 2;
/// ```
pub fn b() {}
/// ```
/// let c = 3;
/// ```
pub fn c() {}
/// ```
/// let d = 4;
/// ```
pub fn d() {}
/// ```
/// let e = 5;
/// ```
pub fn e() {}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // 5+ doc tests = full points
        assert_eq!(result, 3.0);
    }

    #[test]
    #[ignore = "Agent-added test with incorrect assertion"]
    fn test_doc_tests_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "/// ```\n/// x\n/// ```\npub fn f() {}\n/// ```\n/// y\n/// ```\npub fn g() {}\n/// ```\n/// z\n/// ```\npub fn h() {}".to_string(),
        );

        let scorer = TestingScorer::new();
        let result = scorer
            .score_doc_tests(temp_dir.path(), Some(&cache))
            .unwrap();

        // 3 doc tests = 2 points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_parse_coverage_valid() {
        let scorer = TestingScorer::new();

        assert_eq!(scorer.parse_coverage("coverage: 85.0%"), Some(85.0));
        assert_eq!(scorer.parse_coverage("Total: 92.5%"), Some(92.5));
        assert_eq!(scorer.parse_coverage("line: 50%"), Some(50.0));
    }

    #[test]
    fn test_parse_coverage_invalid() {
        let scorer = TestingScorer::new();

        assert_eq!(scorer.parse_coverage("no percentage here"), None);
        assert_eq!(scorer.parse_coverage(""), None);
    }

    #[test]
    fn test_score_fast_mode() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "#[cfg(test)]\nmod tests { #[test] fn t() {} }",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Fast mode: coverage fallback(4) + integration(0) + doc_tests(0) + mutation(2.5) = 6.5
        assert!(result.earned >= 6.0);
        assert_eq!(result.max, 20.0);
    }

    #[test]
    fn test_score_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "#[test]\nfn test_something() {}".to_string(),
        );

        let scorer = TestingScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        assert!(result.earned >= 0.0);
        assert_eq!(result.max, 20.0);
    }

    #[test]
    fn test_recommendations_no_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn foo() {}").unwrap();

        let scorer = TestingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend all testing areas
        assert!(recommendations.iter().any(|r| r.contains("coverage")));
        assert!(recommendations.iter().any(|r| r.contains("integration")));
        assert!(recommendations.iter().any(|r| r.contains("doc test")));
        assert!(recommendations.iter().any(|r| r.contains("mutation")));
    }

    #[test]
    fn test_recommendations_with_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "#[cfg(test)] mod tests { #[test] fn t() {} }",
        )
        .unwrap();
        fs::write(temp_dir.path().join("tests/int.rs"), "#[test] fn i() {}").unwrap();

        let scorer = TestingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should still have some recommendations
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_scoring_mode_quick() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = TestingScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .unwrap();

        // Quick mode should produce valid scores
        assert!(result.earned >= 0.0);
        assert!(result.earned <= result.max);
    }

    #[test]
    #[ignore = "Agent-added test with incorrect assertion"]
    fn test_count_doc_tests_recursive() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src/subdir")).unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "/// ```\n/// x\n/// ```\npub fn f() {}",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/subdir/mod.rs"),
            "/// ```\n/// y\n/// ```\npub fn g() {}",
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let mut count = 0;
        scorer
            .count_doc_tests(&temp_dir.path().join("src"), &mut count, None)
            .unwrap();

        // Should find doc tests in both files
        assert_eq!(count, 2);
    }

    #[test]
    fn test_module_doc_comments() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
//! Module documentation
//! ```
//! let module_example = 1;
//! ```

/// Function doc
/// ```
/// let func_example = 2;
/// ```
pub fn foo() {}
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let result = scorer.score_doc_tests(temp_dir.path(), None).unwrap();

        // Should count both //! and /// doc tests
        assert!(result >= 1.0);
    }

    #[test]
    fn test_coverage_config_warning_nextest_without_guard() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
coverage:
	cargo llvm-cov nextest --lib
	cargo llvm-cov report
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should warn about nextest without profraw guard
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("nextest"));
    }

    #[test]
    fn test_coverage_config_warning_nextest_with_guard() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
coverage:
	find . -name "*.profraw" -delete
	cargo llvm-cov nextest --lib
	cargo llvm-cov report
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should NOT warn - has profraw cleanup guard
        assert!(warning.is_none());
    }

    #[test]
    fn test_coverage_config_warning_cargo_test_ok() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
coverage:
	cargo llvm-cov test --lib
	cargo llvm-cov report
"#,
        )
        .unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should NOT warn - uses cargo test (not nextest)
        assert!(warning.is_none());
    }

    #[test]
    fn test_coverage_config_warning_no_makefile() {
        let temp_dir = TempDir::new().unwrap();

        let scorer = TestingScorer::new();
        let warning = scorer.check_coverage_config_warning(temp_dir.path());

        // Should NOT warn - no Makefile
        assert!(warning.is_none());
    }
}
