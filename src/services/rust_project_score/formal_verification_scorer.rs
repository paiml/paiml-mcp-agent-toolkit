#![allow(unused)]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            name: "Formal Verification".to_string(),
            max_points: MAX_POINTS,
        }
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
        let mut recs = Vec::new();
        self.recommend_miri(project_path, &mut recs);
        self.recommend_kani(project_path, &mut recs);
        self.recommend_verus(project_path, &mut recs);
        self.recommend_lean(project_path, &mut recs);
        recs
    }
}

// SAFETY: FormalVerificationScorer holds only a PathBuf (owned, Send+Sync) and no interior
// mutability, making it safe to send between and share across threads for parallel scoring.
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

// --- Include files for impl blocks ---
include!("formal_verification_counting.rs");
include!("formal_verification_lean.rs");
include!("formal_verification_scoring.rs");
include!("formal_verification_scorer_tests.rs");
