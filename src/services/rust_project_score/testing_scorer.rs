#![cfg_attr(coverage_nightly, coverage(off))]
//! TestingScorer - Testing Excellence Category (20 points)
//!
//! Analyzes Rust project testing practices:
//! - Test Coverage (8pts): >=85% line coverage via cargo-llvm-cov
//! - Integration Tests (4pts): Presence of tests/ directory
//! - Doc Tests (3pts): Rustdoc examples that compile and run
//! - Mutation Testing (5pts): >=80% mutation score
//!
//! Evidence-based refinement: Coverage threshold based on empirical research
//! showing >=85% coverage correlates with significantly fewer production bugs.

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
}

impl Default for TestingScorer {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for TestingScorer {}
unsafe impl Sync for TestingScorer {}

// --- Include files for semantic method groups ---

// Coverage scoring methods (score_coverage, parse_coverage, fallback)
include!("testing_scorer_coverage.rs");

// Analysis methods (integration tests, doc tests, mutation, config warnings, score_internal)
include!("testing_scorer_analysis.rs");

// Scorer trait implementation
include!("testing_scorer_trait_impl.rs");

// Tests - Part 1: Creation, coverage, integration, and doc tests
include!("testing_scorer_tests.rs");

// Tests - Part 2: Scoring modes, recommendations, and config warnings
include!("testing_scorer_tests_part2.rs");
