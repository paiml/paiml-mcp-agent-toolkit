#![cfg_attr(coverage_nightly, coverage(off))]
//! Known Defects Scorer - Production Defect Pattern Detection (20 points)
//!
//! Detects known defect patterns that have caused production incidents.
//!
//! ## Scoring (20 points total)
//!
//! - **Base Score**: 20 points (perfect - zero production defects)
//! - **unwrap() Penalty**: -5 points per 100 unwrap() calls in production code
//! - **Minimum Score**: 0 points (cannot go negative)
//!
//! ## Defect Patterns Detected
//!
//! ### 1. unwrap() in Production Code (Cloudflare Incident 2025-11-18)
//!
//! **Incident**: Cloudflare's worst outage since 2019 caused by uncaught panic from `.unwrap()`
//!
//! **Root Cause**:
//! ```text
//! // Cloudflare's code that caused the outage
//! thread fl2_worker_thread panicked: called Result::unwrap() on an Err value
//! ```
//!
//! **Impact**:
//! - Network unavailable from 11:20-14:30 UTC (3+ hours)
//! - HTTP 5xx errors for all customer traffic
//! - Workers KV, Access, Dashboard, Turnstile all impacted
//!
//! **Fix**:
//! ```ignore
//! // BAD - no error context
//! result.unwrap()
//!
//! // GOOD - descriptive error message
//! result.expect("Bot feature file must be valid and within size limits")
//!
//! // BEST - proper error handling
//! result.map_err(|e| anyhow!("Failed to load bot features: {}", e))?
//! ```
//!
//! **Academic Foundation**:
//! - Post-Mortem Analysis: Cloudflare Blog (2025-11-18)
//! - Rust RFC 1937: Error handling best practices
//! - "Effective Rust" (2024): Prefer expect() with context over unwrap()
//!
//! ## Test Code Exemption
//!
//! `.unwrap()` is allowed in test code (`#[cfg(test)]`, `tests/` directory)
//! as panics are acceptable for test failures.

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use regex::Regex;
use std::path::Path;

/// Known Defects scorer - detects production defect patterns
#[derive(Debug, Clone)]
pub struct KnownDefectsScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl KnownDefectsScorer {
    /// Create a new KnownDefectsScorer
    pub fn new() -> Self {
        Self {
            name: "Known Defects".to_string(),
            max_points: 20.0,
        }
    }
}

// Detection logic: count_unwraps, count_unwraps_in_file, strip_comments, is_test_file
include!("known_defects_scorer_detection.rs");

// Scoring and trait implementations: calculate_unwrap_score, score_internal, Scorer impl
include!("known_defects_scorer_scoring.rs");

// Tests
include!("known_defects_scorer_tests.rs");
