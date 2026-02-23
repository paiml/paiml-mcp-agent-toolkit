#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality report types for command output validation.
//!
//! Contains types for quality checks, violations, and severity levels.

/// Quality report for command output
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Whether all quality checks passed
    pub passed: bool,

    /// Individual quality checks
    pub checks: Vec<QualityCheck>,

    /// Quality violations found
    pub violations: Vec<QualityViolation>,
}

/// Individual quality check
#[derive(Debug, Clone)]
pub struct QualityCheck {
    /// Check name
    pub name: String,

    /// Whether check passed
    pub passed: bool,

    /// Check message
    pub message: String,
}

/// Quality violation
#[derive(Debug, Clone)]
pub struct QualityViolation {
    /// Violation type
    pub violation_type: ViolationType,

    /// Violation message
    pub message: String,

    /// Severity
    pub severity: Severity,
}

/// Types of violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationType {
    Error,
    Warning,
    Timeout,
    ResourceLimit,
    SecurityRisk,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}
