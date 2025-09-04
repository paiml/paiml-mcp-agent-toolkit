//! Quality gate models for the PMAT system

use serde::{Deserialize, Serialize};

/// Result of quality gate execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub violations: Option<u32>,
    pub passed: bool,
}

/// Comprehensive quality gate results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResults {
    pub passed: bool,
    pub total_violations: usize,
    pub complexity_violations: usize,
    pub dead_code_violations: usize,
    pub satd_violations: usize,
    pub entropy_violations: usize,
    pub security_violations: usize,
    pub duplicate_violations: usize,
    pub coverage_violations: usize,
    pub section_violations: usize,
    pub violations: Vec<QualityViolation>,
}

/// Individual quality violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub file: String,
    pub line: Option<usize>,
    pub violation_type: String,
    pub message: String,
    pub severity: ViolationSeverity,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

impl Default for QualityGateResults {
    fn default() -> Self {
        Self {
            passed: true,
            total_violations: 0,
            complexity_violations: 0,
            dead_code_violations: 0,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            violations: Vec::new(),
        }
    }
}