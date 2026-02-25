#![cfg_attr(coverage_nightly, coverage(off))]
//! Core data models for Rust Project Score v1.1
//!
//! This module defines the core types for the 106-point scoring system
//! with 6 categories following the evidence-based specification.
//!
//! All scores are normalized to 0-100 for display (PMAT-454).
//!
//! Evidence-based design from 15 peer-reviewed papers (2022-2025)

use crate::services::normalized_score::NormalizedScore;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

// ScoringMode + RUST_PROJECT_MAX_POINTS constant
include!("models_scoring_mode.rs");

// FileCache - in-memory file cache for project scoring
include!("models_file_cache.rs");

// RustProjectScore + Grade
include!("models_score.rs");

// CategoryScores + CategoryScore
include!("models_categories.rs");

// ScoreVelocity + Recommendation + RecommendationPriority + ScoreMetadata
include!("models_velocity.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // All test functions extracted to models_tests.rs
    include!("models_tests.rs");
}
