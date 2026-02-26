//! DocumentationScorer - Documentation Category (15 points)
//!
//! Analyzes Rust project documentation quality:
//! - Rustdoc Coverage (7pts): Public API documentation with examples
//! - README Quality (5pts): Comprehensive project README
//! - Changelog Presence (3pts): CHANGELOG.md with version history
//!
//! Evidence-based design: Well-documented projects have 30-40% fewer
//! support issues and faster onboarding (GitHub State of the Octoverse 2024).

#![cfg_attr(coverage_nightly, coverage(off))]
use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;

/// Documentation scorer
#[derive(Debug, Clone)]
pub struct DocumentationScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl DocumentationScorer {
    /// Create a new DocumentationScorer
    pub fn new() -> Self {
        Self {
            name: "Documentation".to_string(),
            max_points: 15.0,
        }
    }
}

impl Default for DocumentationScorer {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for DocumentationScorer {}
unsafe impl Sync for DocumentationScorer {}

// --- Include files for semantic method groups ---

// Rustdoc coverage scoring methods
include!("documentation_scorer_rustdoc.rs");

// README quality scoring methods
include!("documentation_scorer_readme.rs");

// Changelog presence scoring (free function + methods)
include!("documentation_scorer_changelog.rs");

// Scorer trait implementation and core scoring logic
include!("documentation_scorer_trait_impl.rs");

// Tests
include!("documentation_scorer_tests.rs");
