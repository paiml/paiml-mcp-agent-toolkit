//! ML-Based Quality Scoring - GH-97 Implementation
//!
//! EXTREME TDD: RED PHASE - Replace heuristic calculations with ML-driven models
//!
//! ## Problem Statement (from GH-97)
//!
//! Current PMAT uses heuristic-based formulas for quality metrics:
//! - Arbitrary constants (why LOC/50? why nesting*2?)
//! - No language-specific adjustments
//! - Cannot learn from actual project outcomes
//!
//! ## Solution
//!
//! Replace with data-driven ML models using `aprender`:
//! - LinearRegression for continuous quality scores
//! - Train on real-world codebases with known outcomes
//! - Support `--ml` flag for opt-in ML-enhanced scoring
//!
//! ## Architecture
//!
//! ```text
//! MLQualityScorer
//! ├── ComplexityModel (LinearRegression)
//! │   └── Features: LOC, nesting, control_flow, loops, language
//! ├── TDGModel (LinearRegression)
//! │   └── Features: complexity, churn, coupling, domain_risk
//! └── HealthScoreModel (LinearRegression)
//!     └── Features: coverage, docs, ci_cd, community
//! ```

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use aprender::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Types: ComplexityFeatures, TDGFeatures, QualityTrainingSample, MLQualityScorer, QualityPrediction
include!("ml_quality_scorer_types.rs");

// Training: train_complexity_model, train_tdg_model, calculate_feature_importance, correlation
include!("ml_quality_scorer_training.rs");

// Prediction: new, predict_complexity, predict_tdg, heuristic_*, is_trained, save, load
include!("ml_quality_scorer_prediction.rs");

#[cfg(test)]
include!("ml_quality_scorer_tests.rs");
