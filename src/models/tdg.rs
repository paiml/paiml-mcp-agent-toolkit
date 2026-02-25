#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Technical Debt Gradient (TDG) - Primary code quality metric
/// Replaces defect probability throughout the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TDGScore {
    /// The calculated TDG value (typically 0.0 - 5.0)
    pub value: f64,

    /// Component breakdown for transparency
    pub components: TDGComponents,

    /// Severity classification based on thresholds
    pub severity: TDGSeverity,

    /// Percentile ranking within the codebase
    pub percentile: f64,

    /// Confidence level of the calculation (0.0 - 1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct TDGComponents {
    /// Complexity contribution (cognitive + cyclomatic)
    pub complexity: f64,

    /// Code churn velocity contribution
    pub churn: f64,

    /// Coupling score contribution
    pub coupling: f64,

    /// Domain-specific risk factors
    pub domain_risk: f64,

    /// Code duplication factor
    pub duplication: f64,

    /// Dead code percentage (CB-128: 6th TDG dimension)
    /// 0.0 = no dead code, 5.0 = severe dead code burden
    #[serde(default)]
    pub dead_code: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TDGSeverity {
    /// TDG < 1.5 - Normal technical debt levels
    Normal,

    /// TDG 1.5-2.5 - Elevated technical debt requiring attention
    Warning,

    /// TDG > 2.5 - Critical technical debt requiring immediate action
    Critical,
}

/// Configuration for TDG calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGConfig {
    /// Weight for complexity component (default: 0.25, was 0.30 pre-CB-128)
    pub complexity_weight: f64,

    /// Weight for churn component (default: 0.20, was 0.35 pre-CB-128)
    pub churn_weight: f64,

    /// Weight for coupling component (default: 0.15)
    pub coupling_weight: f64,

    /// Weight for domain risk component (default: 0.10)
    pub domain_risk_weight: f64,

    /// Weight for duplication component (default: 0.10)
    pub duplication_weight: f64,

    /// Weight for dead code component (default: 0.20, CB-128 6th dimension)
    /// High weight because dead code is pure waste
    #[serde(default = "default_dead_code_weight")]
    pub dead_code_weight: f64,

    /// Threshold for critical severity (default: 2.5)
    pub critical_threshold: f64,

    /// Threshold for warning severity (default: 1.5)
    pub warning_threshold: f64,
}

// --- Impl blocks (TDGSeverity, TDGConfig Default) ---
include!("tdg_impls.rs");

// --- Extended types (Summary, Hotspot, Analysis, Recommendation, Distribution, SATD) ---
include!("tdg_types.rs");

// --- Tests ---
include!("tdg_tests.rs");
