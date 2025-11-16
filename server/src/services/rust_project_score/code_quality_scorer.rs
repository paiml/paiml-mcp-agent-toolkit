//! CodeQualityScorer - Code Quality Category (26 points)
//!
//! Analyzes Rust project code quality metrics:
//! - Cyclomatic Complexity (3pts): All functions ≤20 complexity
//! - Unsafe Code (9pts): Proper unsafe usage with safety comments
//! - Mutation Testing (8pts): ≥80% mutation score
//! - Build Time (4pts): Fast incremental builds
//! - Dead Code (2pts): No unused code
//!
//! Evidence-based refinement (arXiv 2024): Complexity weight reduced from 8→3pts
//! due to low correlation with bugs. Unsafe and mutation weights increased.

use super::models::CategoryScore;
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Code Quality scorer
#[derive(Debug, Clone)]
pub struct CodeQualityScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl CodeQualityScorer {
    /// Create a new CodeQualityScorer
    pub fn new() -> Self {
        Self {
            name: "Code Quality".to_string(),
            max_points: 26.0,
        }
    }

    /// Score cyclomatic complexity (3pts)
    /// All functions must be ≤20 complexity
    fn score_complexity(&self, project_path: &Path) -> ScorerResult<f64> {
        // Try to use pmat binary if available (not cargo run to avoid recursion)
        let output = Command::new("pmat")
            .arg("analyze")
            .arg("complexity")
            .arg("--path")
            .arg(project_path)
            .arg("--threshold")
            .arg("20")
            .current_dir(project_path)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // All functions ≤20 complexity
                    Ok(3.0)
                } else {
                    // Some functions exceed threshold
                    let stderr = String::from_utf8_lossy(&result.stderr);

                    // Count violations (simplified heuristic)
                    let violations = stderr.matches("exceeds").count();

                    if violations == 0 {
                        Ok(3.0)
                    } else if violations <= 3 {
                        Ok(2.0)
                    } else if violations <= 10 {
                        Ok(1.0)
                    } else {
                        Ok(0.0)
                    }
                }
            }
            Err(_) => {
                // If pmat binary not available, use simpler heuristic
                // (avoids recursive cargo run execution)
                self.score_complexity_simple(project_path)
            }
        }
    }

    /// Simple complexity heuristic when pmat not available
    fn score_complexity_simple(&self, project_path: &Path) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(3.0); // No code = no complexity issues
        }

        // Walk source files and count deep nesting
        let mut deep_nesting_count = 0;

        if let Ok(entries) = std::fs::read_dir(&src_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            // Simple heuristic: count deeply nested blocks
                            for line in content.lines() {
                                let indent = line.chars().take_while(|c| c.is_whitespace()).count();
                                if indent > 32 {
                                    // More than 8 levels of nesting (4 spaces each)
                                    deep_nesting_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if deep_nesting_count == 0 {
            Ok(3.0)
        } else if deep_nesting_count <= 5 {
            Ok(2.0)
        } else if deep_nesting_count <= 20 {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }

    /// Score unsafe code usage (9pts)
    /// Checks for SAFETY comments and unsafe ratio
    fn score_unsafe(&self, project_path: &Path) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(9.0); // No code = no unsafe
        }

        let mut _total_lines = 0;
        let mut unsafe_blocks = 0;
        let mut documented_unsafe = 0;

        if let Ok(entries) = std::fs::read_dir(&src_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            let lines: Vec<&str> = content.lines().collect();
                            _total_lines += lines.len();

                            for (i, line) in lines.iter().enumerate() {
                                if line.contains("unsafe") && line.contains("{") {
                                    unsafe_blocks += 1;

                                    // Check for SAFETY comment in previous 5 lines
                                    let start = i.saturating_sub(5);
                                    let has_safety = lines[start..=i]
                                        .iter()
                                        .any(|l| l.contains("SAFETY:") || l.contains("Safety:"));

                                    if has_safety {
                                        documented_unsafe += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if unsafe_blocks == 0 {
            // No unsafe code = full points
            Ok(9.0)
        } else {
            // Score based on documentation ratio
            let doc_ratio = documented_unsafe as f64 / unsafe_blocks as f64;

            if doc_ratio >= 0.9 {
                Ok(9.0) // ≥90% documented
            } else if doc_ratio >= 0.7 {
                Ok(7.0) // ≥70% documented
            } else if doc_ratio >= 0.5 {
                Ok(5.0) // ≥50% documented
            } else if doc_ratio >= 0.3 {
                Ok(3.0) // ≥30% documented
            } else {
                Ok(1.0) // <30% documented
            }
        }
    }

    /// Score mutation testing (8pts)
    /// Requires ≥80% mutation score
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
                // Format: "X caught, Y missed, Z timeout"
                if let Some(caught_line) = stdout.lines().find(|l| l.contains("caught")) {
                    // Simplified parsing - real implementation would be more robust
                    let caught = caught_line
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    let total = stdout.lines().filter(|l| l.contains("mutant")).count() as f64;

                    if total > 0.0 {
                        let ratio = caught / total;

                        if ratio >= 0.80 {
                            Ok(8.0) // ≥80% mutation score
                        } else if ratio >= 0.70 {
                            Ok(6.0)
                        } else if ratio >= 0.60 {
                            Ok(4.0)
                        } else if ratio >= 0.50 {
                            Ok(2.0)
                        } else {
                            Ok(0.0)
                        }
                    } else {
                        Ok(4.0) // No mutants = moderate score
                    }
                } else {
                    Ok(4.0) // Can't parse = moderate score
                }
            }
            Err(_) => {
                // cargo-mutants not installed - give moderate credit
                Ok(4.0)
            }
        }
    }

    /// Score build time (4pts)
    /// Fast builds (<30s) get full points
    fn score_build_time(&self, project_path: &Path) -> ScorerResult<f64> {
        // Measure clean build time
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

                    if seconds < 30 {
                        Ok(4.0) // Very fast
                    } else if seconds < 60 {
                        Ok(3.0) // Fast
                    } else if seconds < 120 {
                        Ok(2.0) // Moderate
                    } else if seconds < 300 {
                        Ok(1.0) // Slow
                    } else {
                        Ok(0.0) // Very slow
                    }
                } else {
                    Ok(0.0) // Build failed
                }
            }
            Err(_) => Ok(2.0), // Can't measure = moderate score
        }
    }

    /// Score dead code detection (2pts)
    fn score_dead_code(&self, project_path: &Path) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(2.0);
        }

        let mut dead_code_count = 0;

        if let Ok(entries) = std::fs::read_dir(&src_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            // Count allow(dead_code) attributes
                            dead_code_count += content.matches("#[allow(dead_code)]").count();
                            dead_code_count += content.matches("#![allow(dead_code)]").count();
                        }
                    }
                }
            }
        }

        if dead_code_count == 0 {
            Ok(2.0)
        } else if dead_code_count <= 3 {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
}

impl Default for CodeQualityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for CodeQualityScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score_with_mode(&self, project_path: &Path, full: bool) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score complexity (3pts)
        match self.score_complexity(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score unsafe code (9pts)
        match self.score_unsafe(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score mutation testing (8pts) - ONLY in full mode
        if full {
            match self.score_mutation(project_path) {
                Ok(score) => total_earned += score,
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode: Skip mutation testing (too slow)
            // Give moderate credit (4/8 points) to avoid penalizing fast mode too much
            total_earned += 4.0;
        }

        // Score build time (4pts) - ONLY in full mode (cargo build is very slow)
        if full && cfg!(not(test)) {
            match self.score_build_time(project_path) {
                Ok(score) => total_earned += score,
                Err(e) => return Err(e),
            }
        } else {
            // Fast mode or test mode: Skip build time measurement (too slow)
            // Give moderate credit (2/4 points)
            total_earned += 2.0;
        }

        // Score dead code (2pts)
        match self.score_dead_code(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check complexity
        if let Ok(score) = self.score_complexity(project_path) {
            if score < 3.0 {
                recommendations.push(
                    "Reduce cyclomatic complexity: refactor functions with >20 complexity into smaller units".to_string(),
                );
            }
        }

        // Check unsafe
        if let Ok(score) = self.score_unsafe(project_path) {
            if score < 9.0 {
                recommendations.push(
                    "Add SAFETY comments for all unsafe blocks explaining invariants".to_string(),
                );
            }
        }

        // Check mutation
        if let Ok(score) = self.score_mutation(project_path) {
            if score < 8.0 {
                recommendations.push(
                    "Improve test quality: install cargo-mutants and aim for ≥80% mutation score"
                        .to_string(),
                );
            }
        }

        // Check dead code
        if let Ok(score) = self.score_dead_code(project_path) {
            if score < 2.0 {
                recommendations.push(
                    "Remove dead code: delete or document unused functions with #[allow(dead_code)]".to_string(),
                );
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for CodeQualityScorer {}
unsafe impl Sync for CodeQualityScorer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = CodeQualityScorer::new();
        assert_eq!(scorer.name(), "Code Quality");
        assert_eq!(scorer.max_points(), 26.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = CodeQualityScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }
}
