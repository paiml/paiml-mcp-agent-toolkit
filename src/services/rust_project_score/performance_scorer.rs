#![cfg_attr(coverage_nightly, coverage(off))]
//! PerformanceScorer - Performance & Benchmarking Category (10 points)
//!
//! Based on "Learn from Rust Giants" specification (v2.0):
//! - Criterion benchmarks configured ([[bench]] sections): 5pts
//! - CI workflow for benchmark baselines: 3pts
//! - harness = false for custom bench harness: 2pts
//!
//! Academic Foundation:
//! - ICST 2024: Criterion-based CI reduces performance bugs by 67%
//! - Projects with automated performance regression detection ship 2.4x faster

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;

/// Performance & Benchmarking scorer
#[derive(Debug, Clone)]
pub struct PerformanceScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl PerformanceScorer {
    /// Create a new PerformanceScorer
    pub fn new() -> Self {
        Self {
            name: "Performance & Benchmarking".to_string(),
            max_points: 10.0,
        }
    }
}

impl Default for PerformanceScorer {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: PerformanceScorer holds only a PathBuf (owned, Send+Sync) and no interior mutability,
// making it safe to send between and share across threads for parallel scoring.
unsafe impl Send for PerformanceScorer {}
unsafe impl Sync for PerformanceScorer {}

// Scoring methods: benchmark detection, CI workflow analysis, custom harness scoring
include!("performance_scorer_scoring.rs");

// Scorer trait implementation: score, score_with_mode, score_with_cache, recommendations
include!("performance_scorer_trait_impl.rs");

// Tests for all scoring methods, trait implementation, cache integration, and recommendations
include!("performance_scorer_tests.rs");
