#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the enforce quality enforcement system

use serde::{Deserialize, Serialize};

/// Quality enforcement state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnforcementState {
    /// Initial analysis of codebase
    Analyzing,
    /// Quality violations detected
    Violating,
    /// Applying improvements
    Refactoring,
    /// Checking if improvements meet standards
    Validating,
    /// All quality standards met
    Complete,
}

/// Quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub violation_type: String,
    pub severity: String,
    pub location: String,
    pub current: f64,
    pub target: f64,
    pub suggestion: String,
}

/// Enforcement progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementProgress {
    pub files_completed: usize,
    pub files_remaining: usize,
    pub estimated_iterations: u32,
}

/// Main enforcement result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementResult {
    pub state: EnforcementState,
    pub score: f64,
    pub target: f64,
    pub current_file: Option<String>,
    pub violations: Vec<QualityViolation>,
    pub next_action: String,
    pub progress: EnforcementProgress,
}

/// Quality profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProfile {
    pub coverage_min: f64,
    pub complexity_max: u16,
    pub complexity_target: u16,
    pub tdg_max: f64,
    pub satd_allowed: usize,
    pub duplication_max_lines: usize,
    pub big_o_max: String,
    pub provability_min: f64,
}

impl Default for QualityProfile {
    fn default() -> Self {
        // RIGID extreme quality profile - the highest standards
        Self {
            coverage_min: 80.0,            // Minimum 80% test coverage
            complexity_max: 20, // Toyota Way standard: maximum cyclomatic complexity of 20
            complexity_target: 10, // Target complexity of 10 for good readability
            tdg_max: 1.0,       // Technical Debt Gradient must be under 1.0
            satd_allowed: 0,    // Zero self-admitted technical debt
            duplication_max_lines: 0, // Zero duplicate code allowed
            big_o_max: "O(n)".to_string(), // Linear complexity or better (was O(n log n))
            provability_min: 0.9, // 90% provability score (was 0.8)
        }
    }
}

/// Structure to hold enforcement iteration result
pub struct EnforcementIterationResult {
    pub iteration: u32,
    pub state: EnforcementState,
    pub score: f64,
}

/// Structure to hold complete loop execution result
pub struct EnforcementLoopResult {
    pub final_iteration: u32,
    pub final_state: EnforcementState,
    pub final_score: f64,
}
