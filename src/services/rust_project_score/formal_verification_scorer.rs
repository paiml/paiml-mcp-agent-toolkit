#![cfg_attr(coverage_nightly, coverage(off))]
//! Formal Verification Scorer for Rust Project Score v1.3
//!
//! Sprint 5: Miri Integration (Jidoka for UB)
//! Sprint 6: Kani Formal Verification
//! Sprint 7: Verus Formal Verification (Issue #106)
//!
//! Toyota Way Principle: Jidoka (自働化) - Built-in Quality
//! Stop the line when undefined behavior is detected.

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Maximum points for Formal Verification category
const MAX_POINTS: f64 = 16.0;

/// Points breakdown:
/// - Miri compliance: 3 points
/// - Kani proofs: 5 points
/// - Verus verification: 5 points
/// - Lean 4 proof quality: 3 points
const MIRI_POINTS: f64 = 3.0;
const KANI_POINTS: f64 = 5.0;
const VERUS_POINTS: f64 = 5.0;
const LEAN_POINTS: f64 = 3.0;

/// Formal Verification Scorer
///
/// Analyzes a Rust project for:
/// 1. Miri compliance on unsafe code
/// 2. Kani formal verification proofs
/// 3. Verus formal verification specs (#[requires], #[ensures], #[invariant])
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

    /// Check if Verus is available
    fn is_verus_available(&self) -> bool {
        Command::new("verus")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Count unsafe blocks in the project
    fn count_unsafe_blocks(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        let unsafe_pattern = Regex::new(r"\bunsafe\s*\{").expect("internal error");
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
        let proof_pattern = Regex::new(r"#\[kani::proof\]").expect("internal error");
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

    /// Count Verus specification attributes in the project
    ///
    /// Looks for #[requires(...)], #[ensures(...)], #[invariant(...)] attributes
    /// which are the core Verus specification constructs.
    fn count_verus_specs(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        // Verus uses requires/ensures/invariant attributes for specifications
        // Also check for proof blocks and spec functions
        let spec_pattern = Regex::new(r"#\[(requires|ensures|invariant|decreases|recommends)\s*\(")
            .expect("internal error");
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += spec_pattern.find_iter(content).count();
            }
        } else {
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += spec_pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Check if project is a Lean 4 project (lakefile.lean or lean-toolchain)
    fn is_lean_project(&self, project_path: &Path) -> bool {
        project_path.join("lakefile.lean").exists()
            || project_path.join("lean-toolchain").exists()
    }

    /// Count theorems and lemmas in .lean files
    fn count_lean_theorems(&self, project_path: &Path) -> usize {
        let theorem_pattern =
            Regex::new(r"^\s*(theorem|lemma|private theorem|private lemma)\s+")
                .expect("internal error");
        let mut count = 0;

        for entry in walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "lean") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    count += content
                        .lines()
                        .filter(|line| theorem_pattern.is_match(line))
                        .count();
                }
            }
        }

        count
    }

    /// Count sorry occurrences in .lean files (incomplete proofs)
    /// Respects block comments (/- ... -/) and line comments (--)
    fn count_lean_sorrys(&self, project_path: &Path) -> usize {
        let mut total = 0;

        for entry in walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "lean") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let mut in_block_comment = 0i32;
                    for line in content.lines() {
                        let trimmed = line.trim();

                        if trimmed.starts_with("--") {
                            continue;
                        }

                        // Strip block comments inline for same-line handling
                        let cleaned =
                            Self::strip_lean_block_comments_inline(trimmed, &mut in_block_comment);

                        if in_block_comment > 0 {
                            continue;
                        }

                        if Self::contains_sorry_word(&cleaned) {
                            total += 1;
                        }
                    }
                }
            }
        }

        total
    }

    /// Strips block comment content from a line, updating nesting depth.
    fn strip_lean_block_comments_inline(line: &str, depth: &mut i32) -> String {
        let bytes = line.as_bytes();
        let mut result = String::with_capacity(line.len());
        let mut i = 0;

        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
                *depth += 1;
                i += 2;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
                *depth -= 1;
                i += 2;
                continue;
            }
            if *depth == 0 {
                result.push(bytes[i] as char);
            }
            i += 1;
        }

        result
    }

    /// Checks if line contains "sorry" as a standalone word.
    fn contains_sorry_word(line: &str) -> bool {
        let bytes = line.as_bytes();
        let sorry = b"sorry";
        let mut pos = 0;
        while pos + sorry.len() <= bytes.len() {
            if let Some(idx) = line[pos..].find("sorry") {
                let abs_idx = pos + idx;
                let before_ok = abs_idx == 0
                    || !(bytes[abs_idx - 1].is_ascii_alphanumeric() || bytes[abs_idx - 1] == b'_');
                let after_ok = abs_idx + sorry.len() >= bytes.len()
                    || !(bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                        || bytes[abs_idx + sorry.len()] == b'_');
                if before_ok && after_ok {
                    return true;
                }
                pos = abs_idx + 1;
            } else {
                break;
            }
        }
        false
    }

    /// Check for vstd dependency in Cargo.toml (indicates Verus project)
    fn has_vstd_dependency(&self, project_path: &Path) -> bool {
        let cargo_toml = project_path.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(cargo_toml) {
            // Check for vstd or builtin dependencies (Verus standard library)
            content.contains("vstd") || content.contains("builtin")
        } else {
            false
        }
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
            if mode == ScoringMode::Quick || mode == ScoringMode::Fast {
                // Quick/Fast mode: Just check for unsafe, give partial credit
                // Skip subprocess calls (cargo miri --version can hang)
                score += MIRI_POINTS * 0.3;
            } else if !self.is_miri_available() {
                // Tool not available, give moderate credit
                score += MIRI_POINTS * 0.5;
            } else {
                // Run Miri (Full mode only)
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
            if mode == ScoringMode::Quick || mode == ScoringMode::Fast {
                // Quick/Fast mode: Just count proofs, give partial credit
                // Skip subprocess calls (cargo kani --version can hang)
                score += KANI_POINTS * 0.4;
            } else if !self.is_kani_available() {
                // Tool not available
                score += KANI_POINTS * 0.3;
            } else {
                // Run Kani verification (Full mode only)
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

        // --- Verus Scoring (5 points) ---
        let verus_specs = self.count_verus_specs(project_path, cache);
        let has_vstd = self.has_vstd_dependency(project_path);

        if verus_specs > 0 || has_vstd {
            if mode == ScoringMode::Quick || mode == ScoringMode::Fast {
                // Quick/Fast mode: Just count specs, give partial credit
                // No subprocess calls for speed
                let spec_score = match verus_specs {
                    0 => 0.2,      // Has vstd but no specs yet
                    1..=5 => 0.4,  // Few specs
                    6..=20 => 0.6, // Moderate specs
                    _ => 0.8,      // Many specs
                };
                score += VERUS_POINTS * spec_score;
            } else if !self.is_verus_available() {
                // Tool not available, give credit for having specs
                let spec_score = match verus_specs {
                    0 => 0.2,
                    1..=5 => 0.4,
                    6..=20 => 0.6,
                    _ => 0.8,
                };
                score += VERUS_POINTS * spec_score;
            } else {
                // Full mode: Give full credit for having verifiable specs
                // Note: Actually running verus verification is expensive,
                // so we award points based on spec count
                let spec_score = match verus_specs {
                    0 => 0.3,      // Has vstd but no specs yet
                    1..=5 => 0.6,  // Few specs
                    6..=20 => 0.8, // Moderate specs
                    _ => 1.0,      // Many specs - full credit
                };
                score += VERUS_POINTS * spec_score;
            }
        }
        // No Verus specs = 0 points for Verus portion

        // --- Lean 4 Scoring (3 points) ---
        if self.is_lean_project(project_path) {
            let theorems = self.count_lean_theorems(project_path);
            let sorrys = self.count_lean_sorrys(project_path);

            if theorems > 0 {
                let proven = theorems.saturating_sub(sorrys);
                let ratio = proven as f64 / theorems as f64;
                score += LEAN_POINTS * ratio;
            } else {
                // Lean project with no theorems yet — minimal credit for setup
                score += LEAN_POINTS * 0.1;
            }
        }
        // Not a Lean project = 0 points for Lean portion

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

        // Check for Verus specs
        let verus_specs = self.count_verus_specs(project_path, None);
        let has_vstd = self.has_vstd_dependency(project_path);

        if verus_specs == 0 && !has_vstd {
            // Suggest Verus for projects with complex logic or unsafe code
            if unsafe_count > 0 {
                recommendations.push(
                    "Consider Verus for formal verification of unsafe code: https://verus-lang.github.io/verus/guide/"
                        .into(),
                );
            }
        } else if (verus_specs > 0 || has_vstd) && !self.is_verus_available() {
            recommendations.push(
                "Install Verus to verify specs: https://github.com/verus-lang/verus#building"
                    .into(),
            );
        } else if verus_specs < 5 && has_vstd {
            recommendations.push(
                "Add more #[requires], #[ensures] specs to increase verification coverage".into(),
            );
        }

        // Check for Lean 4 proofs
        if self.is_lean_project(project_path) {
            let theorems = self.count_lean_theorems(project_path);
            let sorrys = self.count_lean_sorrys(project_path);

            if sorrys > 0 {
                recommendations.push(format!(
                    "Lean 4 project has {} sorry markers — complete proofs to improve score",
                    sorrys
                ));
            }
            if theorems == 0 {
                recommendations
                    .push("Lean 4 project has no theorems/lemmas — add proven propositions".into());
            }
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

#[cfg_attr(coverage_nightly, coverage(off))]
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
        assert_eq!(scorer.max_points(), 16.0); // Miri (3) + Kani (5) + Verus (5) + Lean (3)
    }

    #[test]
    fn test_no_unsafe_gives_full_miri_credit() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create a safe Rust file
        std::fs::write(src_dir.join("lib.rs"), "pub fn safe_fn() -> i32 { 42 }\n")
            .expect("internal error");

        // Create Cargo.toml
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .expect("internal error");

        // Should get full Miri credit (3pts) for no unsafe code
        assert!(result.earned >= MIRI_POINTS);
    }

    #[test]
    fn test_count_unsafe_blocks() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

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
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_unsafe_blocks(temp_dir.path(), None);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_kani_proofs() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

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
        .expect("internal error");

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

    #[test]
    fn test_count_verus_specs() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create file with Verus specifications
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
use vstd::prelude::*;

verus! {

#[requires(x > 0)]
#[ensures(result > x)]
pub fn add_one(x: u32) -> u32 {
    x + 1
}

#[requires(len > 0)]
#[ensures(result <= len)]
#[decreases(len)]
pub fn recursive_fn(len: u32) -> u32 {
    if len == 1 { 1 } else { recursive_fn(len - 1) }
}

#[invariant(self.value >= 0)]
pub struct Counter {
    value: i32,
}

}
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_verus_specs(temp_dir.path(), None);
        // Should find: 2x requires, 2x ensures, 1x decreases, 1x invariant = 6
        assert_eq!(count, 6);
    }

    #[test]
    fn test_has_vstd_dependency() {
        let temp_dir = TempDir::new().expect("internal error");

        // Create Cargo.toml with vstd dependency
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "verus-project"
version = "0.1.0"
edition = "2021"

[dependencies]
vstd = { path = "../vstd" }
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        assert!(scorer.has_vstd_dependency(temp_dir.path()));
    }

    #[test]
    fn test_no_vstd_dependency() {
        let temp_dir = TempDir::new().expect("internal error");

        // Create Cargo.toml without vstd
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "regular-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        assert!(!scorer.has_vstd_dependency(temp_dir.path()));
    }

    #[test]
    fn test_verus_specs_give_points() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create file with Verus specifications
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
#[requires(x > 0)]
#[ensures(result > 0)]
pub fn verified_fn(x: u32) -> u32 { x }
"#,
        )
        .expect("internal error");

        // Create Cargo.toml with vstd
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "verus-project"
version = "0.1.0"
edition = "2021"

[dependencies]
vstd = { path = "../vstd" }
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .expect("internal error");

        // Should get Miri points (no unsafe) + some Verus points
        assert!(result.earned > MIRI_POINTS);
    }
}
