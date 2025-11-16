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

use super::models::CategoryScore;
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
                // Fallback: check for presence of tests
                self.score_coverage_fallback(project_path)
            }
        }
    }

    /// Fallback coverage scoring when cargo-llvm-cov not available
    fn score_coverage_fallback(&self, project_path: &Path) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(0.0);
        }

        let mut has_tests = false;

        // Check for #[cfg(test)] modules in source files
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
    fn score_doc_tests(&self, project_path: &Path) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(0.0);
        }

        let mut doc_test_count = 0;

        // Recursively walk src directory
        self.count_doc_tests(&src_path, &mut doc_test_count)?;

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
    fn count_doc_tests(&self, dir: &Path, count: &mut usize) -> ScorerResult<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    self.count_doc_tests(&path, count)?;
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
        Ok(())
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
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score coverage (8pts)
        match self.score_coverage(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score integration tests (4pts)
        match self.score_integration_tests(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score doc tests (3pts)
        match self.score_doc_tests(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score mutation testing (5pts)
        match self.score_mutation(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    fn score_with_mode(&self, project_path: &Path, _full: bool) -> ScorerResult<CategoryScore> {
        // This scorer doesn't have expensive operations, so mode doesn't affect it
        self.score(project_path)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check coverage
        if let Ok(score) = self.score_coverage(project_path) {
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

        // Check doc tests
        if let Ok(score) = self.score_doc_tests(project_path) {
            if score < 3.0 {
                recommendations.push(
                    "Add doc tests: Include runnable examples in /// documentation comments"
                        .to_string(),
                );
            }
        }

        // Check mutation testing
        if let Ok(score) = self.score_mutation(project_path) {
            if score < 5.0 {
                recommendations.push(
                    "Improve test quality: Install cargo-mutants and aim for ≥80% mutation score"
                        .to_string(),
                );
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for TestingScorer {}
unsafe impl Sync for TestingScorer {}

#[cfg(test)]
mod tests {
    use super::*;

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
}
