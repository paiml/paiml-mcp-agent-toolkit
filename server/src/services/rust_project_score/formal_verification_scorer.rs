//! Formal Verification Scorer for Rust Project Score v1.2
//!
//! Sprint 5: Miri Integration (Jidoka for UB)
//! Sprint 6: Kani Formal Verification
//!
//! Toyota Way Principle: Jidoka (自働化) - Built-in Quality
//! Stop the line when undefined behavior is detected.

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Maximum points for Formal Verification category
const MAX_POINTS: f64 = 8.0;

/// Points breakdown:
/// - Miri compliance: 3 points
/// - Kani proofs: 5 points
const MIRI_POINTS: f64 = 3.0;
const KANI_POINTS: f64 = 5.0;

/// Formal Verification Scorer
///
/// Analyzes a Rust project for:
/// 1. Miri compliance on unsafe code
/// 2. Kani formal verification proofs
#[derive(Debug, Clone)]
pub struct FormalVerificationScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl FormalVerificationScorer {
    /// Create a new FormalVerificationScorer
    pub fn new() -> Self {
        Self {
            name: "Formal Verification".to_string(),
            max_points: MAX_POINTS,
        }
    }

    /// Check if Miri is available
    fn is_miri_available(&self) -> bool {
        Command::new("cargo")
            .args(["miri", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if Kani is available
    fn is_kani_available(&self) -> bool {
        Command::new("cargo")
            .args(["kani", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Count unsafe blocks in the project
    fn count_unsafe_blocks(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        let unsafe_pattern = Regex::new(r"\bunsafe\s*\{").unwrap();
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            // Use cache for efficiency (Kaizen Round 4)
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += unsafe_pattern.find_iter(content).count();
            }
        } else {
            // Fallback: walk directory
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += unsafe_pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Run Miri tests and return pass/fail status
    fn run_miri_tests(&self, project_path: &Path) -> ScorerResult<MiriResult> {
        let output = Command::new("cargo")
            .args(["miri", "test", "--", "--test-threads=1"])
            .current_dir(project_path)
            .output()
            .map_err(|e| ScorerError::CommandError(e.to_string()))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for Miri errors
        let has_ub_errors = stderr.contains("Undefined Behavior")
            || stderr.contains("error: Miri evaluation error");

        // Parse test results
        let passed_tests = parse_test_count(&stdout, "passed");
        let failed_tests = parse_test_count(&stdout, "failed");

        Ok(MiriResult {
            passed: output.status.success() && !has_ub_errors,
            _passed_tests: passed_tests,
            _failed_tests: failed_tests,
            has_ub_errors,
        })
    }

    /// Check for Kani proofs in the project
    fn count_kani_proofs(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        // Look for #[kani::proof] attributes
        let proof_pattern = Regex::new(r"#\[kani::proof\]").unwrap();
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += proof_pattern.find_iter(content).count();
            }
        } else {
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += proof_pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Run Kani verification and return results
    fn run_kani_verification(&self, project_path: &Path) -> ScorerResult<KaniResult> {
        let output = Command::new("cargo")
            .args(["kani", "--only-codegen"])
            .current_dir(project_path)
            .output()
            .map_err(|e| ScorerError::CommandError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse Kani results
        let verified = stdout.contains("VERIFICATION:- SUCCESSFUL")
            || stdout.contains("Verification succeeded");
        let has_failures =
            stdout.contains("VERIFICATION:- FAILED") || stderr.contains("VERIFICATION FAILED");

        Ok(KaniResult {
            all_verified: verified && !has_failures,
            _has_proofs: true,
        })
    }

    /// Internal scoring logic with cache support
    fn score_internal(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        let mut score = 0.0;

        // Count unsafe blocks to determine if Miri is relevant
        let unsafe_count = self.count_unsafe_blocks(project_path, cache);
        let has_unsafe = unsafe_count > 0;

        // --- Miri Scoring (3 points) ---
        if has_unsafe {
            if mode == ScoringMode::Quick {
                // Quick mode: Just check for unsafe, give partial credit
                score += MIRI_POINTS * 0.3;
            } else if !self.is_miri_available() {
                // Tool not available, give moderate credit
                score += MIRI_POINTS * 0.5;
            } else {
                // Run Miri
                match self.run_miri_tests(project_path) {
                    Ok(result) => {
                        if result.passed {
                            score += MIRI_POINTS;
                        } else if result.has_ub_errors {
                            // UB detected - Andon Cord! No points
                        } else {
                            // Some tests failed but no UB
                            score += MIRI_POINTS * 0.5;
                        }
                    }
                    Err(_) => {
                        score += MIRI_POINTS * 0.3;
                    }
                }
            }
        } else {
            // No unsafe code - full Miri points (nothing to check)
            score += MIRI_POINTS;
        }

        // --- Kani Scoring (5 points) ---
        let kani_proofs = self.count_kani_proofs(project_path, cache);

        if kani_proofs > 0 {
            if mode == ScoringMode::Quick {
                // Quick mode: Just count proofs
                score += KANI_POINTS * 0.4;
            } else if !self.is_kani_available() {
                // Tool not available
                score += KANI_POINTS * 0.3;
            } else {
                // Run Kani verification
                match self.run_kani_verification(project_path) {
                    Ok(result) => {
                        if result.all_verified {
                            score += KANI_POINTS;
                        } else {
                            score += KANI_POINTS * 0.5;
                        }
                    }
                    Err(_) => {
                        score += KANI_POINTS * 0.2;
                    }
                }
            }
        }
        // No Kani proofs = 0 points for Kani portion

        Ok(CategoryScore::new(score.min(MAX_POINTS), self.max_points))
    }
}

impl Default for FormalVerificationScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for FormalVerificationScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, ScoringMode::default(), None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, mode, None)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        self.score_internal(project_path, mode, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for unsafe blocks
        let unsafe_count = self.count_unsafe_blocks(project_path, None);

        if unsafe_count > 0 {
            if !self.is_miri_available() {
                recommendations.push("Install Miri: rustup +nightly component add miri".into());
            } else {
                recommendations.push(format!(
                    "Run Miri on {} unsafe blocks: cargo +nightly miri test",
                    unsafe_count
                ));
            }
        }

        // Check for Kani proofs
        let kani_proofs = self.count_kani_proofs(project_path, None);

        if kani_proofs == 0 && unsafe_count > 0 {
            recommendations.push(
                "Consider adding Kani proofs for unsafe code: https://model-checking.github.io/kani/"
                    .into(),
            );
        } else if kani_proofs > 0 && !self.is_kani_available() {
            recommendations.push("Install Kani: cargo install --locked kani-verifier".into());
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for FormalVerificationScorer {}
unsafe impl Sync for FormalVerificationScorer {}

/// Result of Miri test run
struct MiriResult {
    passed: bool,
    _passed_tests: usize,
    _failed_tests: usize,
    has_ub_errors: bool,
}

/// Result of Kani verification
struct KaniResult {
    all_verified: bool,
    _has_proofs: bool,
}

/// Parse test count from cargo test output
fn parse_test_count(output: &str, status: &str) -> usize {
    let pattern = format!(r"(\d+) {}", status);
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(output))
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_name() {
        let scorer = FormalVerificationScorer::new();
        assert_eq!(scorer.name(), "Formal Verification");
    }

    #[test]
    fn test_max_points() {
        let scorer = FormalVerificationScorer::new();
        assert_eq!(scorer.max_points(), 8.0);
    }

    #[test]
    fn test_no_unsafe_gives_full_miri_credit() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create a safe Rust file
        std::fs::write(src_dir.join("lib.rs"), "pub fn safe_fn() -> i32 { 42 }\n").unwrap();

        // Create Cargo.toml
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let scorer = FormalVerificationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .unwrap();

        // Should get full Miri credit (3pts) for no unsafe code
        assert!(result.earned >= MIRI_POINTS);
    }

    #[test]
    fn test_count_unsafe_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe blocks
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
pub fn with_unsafe() {
    unsafe {
        // do something
    }

    unsafe { std::ptr::null::<i32>().read() }
}
"#,
        )
        .unwrap();

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_unsafe_blocks(temp_dir.path(), None);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_kani_proofs() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Kani proofs
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
#[kani::proof]
fn check_addition() {
    let a: u8 = kani::any();
    let b: u8 = kani::any();
    kani::assume(a.checked_add(b).is_some());
    assert!(a + b >= a);
}

#[kani::proof]
fn check_subtraction() {
    let a: u8 = kani::any();
    let b: u8 = kani::any();
    kani::assume(a >= b);
    assert!(a - b <= a);
}
"#,
        )
        .unwrap();

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_kani_proofs(temp_dir.path(), None);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_test_count() {
        let output = "test result: ok. 5 passed; 2 failed; 0 ignored;";
        assert_eq!(parse_test_count(output, "passed"), 5);
        assert_eq!(parse_test_count(output, "failed"), 2);
        assert_eq!(parse_test_count(output, "ignored"), 0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = FormalVerificationScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }
}
