#![cfg_attr(coverage_nightly, coverage(off))]
//! ML-Based Mutant Survivability Predictor - Phase 4.3 GREEN PHASE
//!
//! EXTREME TDD: GREEN PHASE - Aprender LinearRegression Migration (2025-11-18)
//!
//! ## Migration from linfa to aprender
//!
//! **Rationale**: Migrated from linfa to aprender (PAIML's next-gen ML library)
//! - **Before**: linfa + ndarray (50+ transitive dependencies)
//! - **After**: aprender v0.1.0 (0 transitive dependencies, TDG 94.1/100)
//! - **Benefit**: Zero dependency bloat, pure Rust, reproducible builds
//!
//! ## Current Implementation
//!
//! Uses **LinearRegression** for binary classification via regression with 0.5 threshold:
//! - Predicts kill probability as continuous value [0.0, 1.0]
//! - Falls back to statistical baseline when n_samples < n_features (underdetermined system)
//! - Graceful degradation: Warns on training failure, continues with operator kill rates
//!
//! ## Known Limitations (aprender v0.1.0)
//!
//! 1. **Small Sample Sizes**: LinearRegression requires n_samples >= n_features (18)
//!    - GitHub Issue: https://github.com/paiml/aprender/issues/4
//!    - Workaround: Statistical baseline fallback when training fails
//!
//! 2. **DecisionTree Not Available**: Original implementation used linfa DecisionTree
//!    - GitHub Issue: https://github.com/paiml/aprender/issues/3
//!    - Future: Will migrate back to DecisionTree when available
//!
//! ## Test Results
//!
//! - All 12 RED phase tests passing (100%)
//! - Graceful fallback handles underdetermined systems
//! - Feature extraction works correctly (18-dimensional vectors)
//! - Statistical baseline provides reasonable predictions
//!
//! ## Future Work
//!
//! When aprender implements missing features:
//! - Migrate from LinearRegression to DecisionTree (better for classification)
//! - Add Ridge regression support for small samples (L2 regularization)
//! - Remove linfa/ndarray dependencies completely

mod features;
mod helpers;
mod predictor;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_predictor;

pub use predictor::SurvivabilityPredictor;
pub use types::{MutantFeatures, PredictionResult, TrainingData};
