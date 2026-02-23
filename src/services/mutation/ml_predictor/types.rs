#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the ML-based mutant survivability predictor.

use crate::services::mutation::{Mutant, MutationOperatorType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Features extracted from a mutant for ML prediction
/// Enhanced feature set (v2) - expanded from 10 to 18 features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutantFeatures {
    /// Type of mutation operator
    pub operator_type: MutationOperatorType,

    /// Cyclomatic complexity at mutation point
    pub cyclomatic_complexity: u32,

    /// Cognitive complexity at mutation point
    pub cognitive_complexity: u32,

    /// Source line number
    pub source_line: u32,

    /// Nesting depth at mutation point
    pub nesting_depth: u32,

    /// Number of control flow constructs nearby
    pub control_flow_count: u32,

    /// Has loops nearby
    pub has_loops: bool,

    /// Has conditionals nearby
    pub has_conditionals: bool,

    /// Function size (LOC)
    pub function_size: u32,

    /// Number of parameters
    pub parameter_count: u32,

    // NEW ENHANCED FEATURES (v2)
    /// Has error handling (try/catch/Result)
    pub has_error_handling: bool,

    /// Has assertions or tests
    pub has_assertions: bool,

    /// Token count (code density)
    pub token_count: u32,

    /// Unique variable count
    pub unique_variables: u32,

    /// Has arithmetic operations
    pub has_arithmetic: bool,

    /// Has comparison operations
    pub has_comparisons: bool,

    /// Has logical operations (&&, ||, !)
    pub has_logical_ops: bool,

    /// Mutation depth (how nested in control flow)
    pub mutation_depth: u32,
}

/// Training data for the ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub mutant: Mutant,
    pub was_killed: bool,
    pub test_failures: Vec<String>,
    pub execution_time_ms: u64,
}

/// Prediction result from the ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Probability that this mutant will be killed (0.0 - 1.0)
    pub kill_probability: f64,

    /// Confidence in the prediction (0.0 - 1.0)
    pub confidence: f64,

    /// Feature importance for this prediction
    pub feature_contributions: HashMap<String, f64>,
}
