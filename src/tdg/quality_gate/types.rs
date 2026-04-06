//! Types, enums, and structs for the Quality Gate system.

use crate::tdg::Grade;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::tdg::TdgBaseline;
use anyhow::Result;

/// Result of running a quality gate check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Whether the gate passed
    pub passed: bool,
    /// Gate that was executed
    pub gate_name: String,
    /// Violations found (empty if passed)
    pub violations: Vec<Violation>,
    /// Summary message
    pub message: String,
}

/// A single quality gate violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// File that violated the gate
    pub path: PathBuf,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Severity of violation
    pub severity: Severity,
    /// Detailed message
    pub message: String,
    /// Old score (for regressions)
    pub old_score: Option<f32>,
    /// New score
    pub new_score: f32,
    /// Old grade (for regressions)
    pub old_grade: Option<Grade>,
    /// New grade
    pub new_grade: Grade,
}

/// Type of quality gate violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Quality regression detected
    Regression,
    /// File below minimum grade
    BelowMinimum,
    /// New file below threshold
    NewFileBelowThreshold,
}

/// Severity level for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational (doesn't fail gate)
    Info,
    /// Warning (logs but doesn't fail)
    Warning,
    /// Error (fails the gate)
    Error,
    /// Critical (fails gate with high priority)
    Critical,
}

/// Configuration for quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    /// Maximum score drop allowed before flagging regression
    pub max_score_drop: f32,
    /// Whether to allow grade drops (e.g., A -> B)
    pub allow_grade_drop: bool,
    /// Minimum grades by language
    pub min_grades: HashMap<String, Grade>,
    /// Default minimum grade if language not specified
    pub default_min_grade: Grade,
    /// Whether to enforce quality on new files
    pub enforce_new_files: bool,
    /// Minimum grade for new files
    pub new_file_min_grade: Grade,
}

impl Default for GateConfig {
    fn default() -> Self {
        let mut min_grades = HashMap::new();
        min_grades.insert("rust".to_string(), Grade::BPlus);
        min_grades.insert("typescript".to_string(), Grade::BPlus);
        min_grades.insert("python".to_string(), Grade::B);
        min_grades.insert("javascript".to_string(), Grade::B);

        Self {
            max_score_drop: 5.0,
            allow_grade_drop: false,
            min_grades,
            default_min_grade: Grade::B,
            enforce_new_files: true,
            new_file_min_grade: Grade::B,
        }
    }
}

/// Trait for quality gates
pub trait QualityGate {
    /// Name of this gate
    fn name(&self) -> &str;

    /// Run the gate check
    fn check(&self, baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult>;
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    debug_assert!(true, "contract: name");
    use super::*;
    include!("types_tests.rs");
}
