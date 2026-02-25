#![cfg_attr(coverage_nightly, coverage(off))]
//! Defect probability prediction using multi-factor analysis.
//!
//! This module implements a defect probability calculator that combines multiple
//! software metrics to predict the likelihood of defects in code files. It uses
//! an empirically-derived weighted ensemble approach based on research in defect
//! prediction and software quality metrics.
//!
//! # Prediction Model
//!
//! The model uses four primary factors:
//! - **Code Churn** (35%): Frequency and magnitude of changes
//! - **Complexity** (30%): Cyclomatic and cognitive complexity
//! - **Duplication** (25%): Amount of duplicated code
//! - **Coupling** (10%): Inter-module dependencies
//!
//! # Risk Levels
//!
//! Files are classified into risk levels based on probability:
//! - **Critical** (>0.8): Immediate attention required
//! - **High** (0.6-0.8): Should be refactored soon
//! - **Medium** (0.4-0.6): Monitor and plan refactoring
//! - **Low** (<0.4): Acceptable risk level
//!
//! # Example
//!
//! ```
//! use pmat::services::defect_probability::{DefectProbabilityCalculator, FileMetrics};
//!
//! let calculator = DefectProbabilityCalculator::new();
//!
//! let metrics = FileMetrics {
//!     file_path: "src/complex_module.rs".to_string(),
//!     churn_score: 0.7,
//!     complexity: 45.0,
//!     duplicate_ratio: 0.2,
//!     afferent_coupling: 5.0,
//!     efferent_coupling: 12.0,
//!     lines_of_code: 500,
//!     cyclomatic_complexity: 25,
//!     cognitive_complexity: 35,
//! };
//!
//! let score = calculator.calculate(&metrics);
//!
//! println!("Defect probability: {:.2}%", score.probability * 100.0);
//! println!("Risk level: {:?}", score.risk_level);
//!
//! // Show contributing factors
//! for (factor, contribution) in &score.contributing_factors {
//!     println!("{}: {:.2}%", factor, contribution * 100.0);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defect probability calculator using weighted ensemble approach
#[derive(Debug, Clone)]
pub struct DefectProbabilityCalculator {
    weights: DefectWeights,
}

/// Weights for different factors in defect probability calculation
#[derive(Debug, Clone)]
pub struct DefectWeights {
    pub churn: f32,       // α = 0.35
    pub complexity: f32,  // β = 0.30
    pub duplication: f32, // γ = 0.25
    pub coupling: f32,    // δ = 0.10
}

impl Default for DefectWeights {
    fn default() -> Self {
        Self {
            churn: 0.35,
            complexity: 0.30,
            duplication: 0.25,
            coupling: 0.10,
        }
    }
}

/// Input metrics for defect probability calculation
#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub file_path: String,
    pub churn_score: f32,       // 0.0 to 1.0
    pub complexity: f32,        // Raw complexity score
    pub duplicate_ratio: f32,   // 0.0 to 1.0
    pub afferent_coupling: f32, // Number of incoming dependencies
    pub efferent_coupling: f32, // Number of outgoing dependencies
    pub lines_of_code: usize,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
}

/// Defect probability score with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectScore {
    pub probability: f32,                         // 0.0 to 1.0
    pub contributing_factors: Vec<(String, f32)>, // Factor name and weighted contribution
    pub confidence: f32,                          // 0.0 to 1.0
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,    // 0.0 - 0.3
    Medium, // 0.3 - 0.7
    High,   // 0.7 - 1.0
}

// --- Implementation split into include files ---

include!("defect_probability_calculation.rs");
include!("defect_probability_analysis.rs");
include!("defect_probability_tests.rs");
