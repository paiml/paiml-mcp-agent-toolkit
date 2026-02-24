#![cfg_attr(coverage_nightly, coverage(off))]
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

use super::models::{CategoryScore, FileCache, ScoringMode};
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

// Scoring methods: mutation, build time, score_internal orchestration
include!("code_quality_scoring_methods.rs");

// Heuristic scoring: complexity, unsafe, dead code (cache-aware)
include!("code_quality_scoring_heuristics.rs");

// Trait implementations: Default, Scorer, Send, Sync
include!("code_quality_scorer_trait_impl.rs");

// Tests
include!("code_quality_scorer_tests.rs");
