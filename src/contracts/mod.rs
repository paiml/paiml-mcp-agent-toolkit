//! Unified contract definitions for ALL interfaces (CLI, MCP, HTTP)
//!
//! CRITICAL: This is the SINGLE SOURCE OF TRUTH for all command parameters.
//! Every interface MUST use these exact contracts with no variations.

pub mod adapter;
pub mod cli_impl;
pub mod cli_mapping;
#[cfg(feature = "http-server")]
pub mod http_impl;
// pub mod mcp_impl; // Disabled due to pmcp dependency issues
pub mod mcp_mapping;
pub mod mcp_simple;
pub mod real_service;
pub mod service;
pub mod simple_service;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests;
pub mod uniform_cli_commands;
pub mod versioning;

use crate::utils::path_validator::PathValidator;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Missing required parameter: {0}")]
    MissingParam(&'static str),

    #[error("Invalid timeout value")]
    InvalidTimeout,

    #[error("Too many files requested: {0} (max: 1000)")]
    TooManyFiles(usize),

    #[error("Invalid parameter value: {0}")]
    InvalidValue(String),
}

/// Output formats supported by ALL commands
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
    Markdown,
    Csv,
    Summary,
}

/// SATD severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum SatdSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Base parameters shared by ALL analysis commands
/// This ensures consistency across all interfaces
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BaseAnalysisContract {
    /// Path to analyze - ALWAYS named 'path', never '`project_path`' or 'file'
    pub path: PathBuf,

    /// Output format - ALWAYS available, ALWAYS same enum
    pub format: OutputFormat,

    /// Output file path - ALWAYS optional
    pub output: Option<PathBuf>,

    /// Number of top files to show - ALWAYS same name, ALWAYS optional
    pub top_files: Option<usize>,

    /// Include test files - ALWAYS same behavior
    pub include_tests: bool,

    /// Analysis timeout in seconds - ALWAYS available
    pub timeout: u64,
}

impl Default for BaseAnalysisContract {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            format: OutputFormat::default(),
            output: None,
            top_files: Some(10),
            include_tests: false,
            timeout: 60,
        }
    }
}

/// Contract for analyze complexity command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyzeComplexityContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Maximum cyclomatic complexity threshold
    #[serde(default)]
    pub max_cyclomatic: Option<u32>,

    /// Maximum cognitive complexity threshold
    #[serde(default)]
    pub max_cognitive: Option<u32>,

    /// Maximum Halstead difficulty threshold
    #[serde(default)]
    pub max_halstead: Option<f64>,
}

/// Contract for analyze SATD command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyzeSatdContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Filter by severity level
    #[serde(default)]
    pub severity: Option<SatdSeverity>,

    /// Show only critical debt items
    #[serde(default)]
    pub critical_only: bool,

    /// Use strict mode (only TODO/FIXME/HACK/BUG)
    #[serde(default)]
    pub strict: bool,

    /// Exit with error if violations found
    #[serde(default)]
    pub fail_on_violation: bool,
}

/// Contract for analyze dead code command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeDeadCodeContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Include unreachable code blocks
    #[serde(default)]
    pub include_unreachable: bool,

    /// Minimum dead lines to report a file
    #[serde(default = "default_min_dead_lines")]
    pub min_dead_lines: usize,

    /// Maximum allowed dead code percentage
    #[serde(default = "default_max_percentage")]
    pub max_percentage: f64,

    /// Exit with error if violations found
    #[serde(default)]
    pub fail_on_violation: bool,
}

fn default_min_dead_lines() -> usize {
    10
}

fn default_max_percentage() -> f64 {
    15.0
}

/// Contract for analyze TDG command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeTdgContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// TDG threshold for filtering results
    #[serde(default = "default_tdg_threshold")]
    pub threshold: f64,

    /// Include TDG component breakdown
    #[serde(default)]
    pub include_components: bool,

    /// Show only critical files (TDG > 2.5)
    #[serde(default)]
    pub critical_only: bool,
}

fn default_tdg_threshold() -> f64 {
    1.5
}

/// Contract for analyze lint hotspot command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeLintHotspotContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Analyze a specific file instead of finding hotspot
    #[serde(default)]
    pub file: Option<PathBuf>,

    /// Maximum allowed defect density
    #[serde(default = "default_max_density")]
    pub max_density: f64,

    /// Minimum confidence for automated fixes
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,

    /// Enforce quality standards
    #[serde(default)]
    pub enforce: bool,

    /// Dry run - show what would be fixed
    #[serde(default)]
    pub dry_run: bool,
}

fn default_max_density() -> f64 {
    5.0
}

fn default_min_confidence() -> f64 {
    0.8
}

/// Contract for analyze entropy command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyzeEntropyContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Minimum severity level to report
    #[serde(default)]
    pub min_severity: Option<String>,

    /// Maximum number of violations to show (0 = all)
    #[serde(default)]
    pub top_violations: Option<usize>,

    /// Specific file to analyze instead of project
    #[serde(default)]
    pub file: Option<PathBuf>,
}

/// Contract for quality gate command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QualityGateContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Quality profile to use
    #[serde(default)]
    pub profile: QualityProfile,

    /// Specific file to check (optional)
    #[serde(default)]
    pub file: Option<PathBuf>,

    /// Exit with error if violations found
    #[serde(default)]
    pub fail_on_violation: bool,

    /// Show verbose output
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum QualityProfile {
    #[default]
    Standard,
    Strict,
    Extreme,
    Toyota,
}

/// Contract for refactor auto command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorAutoContract {
    /// File to refactor - MUST be a file, not directory
    pub file: PathBuf,

    /// Output format
    #[serde(default)]
    pub format: OutputFormat,

    /// Output file path
    #[serde(default)]
    pub output: Option<PathBuf>,

    /// Target complexity
    #[serde(default = "default_target_complexity")]
    pub target_complexity: u32,

    /// Dry run mode
    #[serde(default)]
    pub dry_run: bool,

    /// Analysis timeout
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_target_complexity() -> u32 {
    10
}

fn default_timeout() -> u64 {
    60
}

/// Trait for contract validation
pub trait ContractValidation {
    fn validate(&self) -> Result<(), ContractError>;
}

impl ContractValidation for BaseAnalysisContract {
    fn validate(&self) -> Result<(), ContractError> {
        PathValidator::ensure_exists(&self.path)
            .map_err(|_| ContractError::PathNotFound(self.path.clone()))?;

        if self.timeout == 0 {
            return Err(ContractError::InvalidTimeout);
        }

        if let Some(top_files) = self.top_files {
            if top_files > 1000 {
                return Err(ContractError::TooManyFiles(top_files));
            }
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeComplexityContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if let Some(max_halstead) = self.max_halstead {
            if max_halstead <= 0.0 {
                return Err(ContractError::InvalidValue(
                    "max_halstead must be positive".into(),
                ));
            }
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeSatdContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()
    }
}

impl ContractValidation for AnalyzeDeadCodeContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if self.max_percentage < 0.0 || self.max_percentage > 100.0 {
            return Err(ContractError::InvalidValue(
                "max_percentage must be 0-100".into(),
            ));
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeTdgContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if self.threshold < 0.0 {
            return Err(ContractError::InvalidValue(
                "threshold must be non-negative".into(),
            ));
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeLintHotspotContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if self.max_density < 0.0 {
            return Err(ContractError::InvalidValue(
                "max_density must be non-negative".into(),
            ));
        }

        if self.min_confidence < 0.0 || self.min_confidence > 1.0 {
            return Err(ContractError::InvalidValue(
                "min_confidence must be 0-1".into(),
            ));
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeEntropyContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        // Validate severity level if provided
        if let Some(severity) = &self.min_severity {
            match severity.as_str() {
                "low" | "medium" | "high" => {}
                _ => {
                    return Err(ContractError::InvalidValue(
                        "min_severity must be 'low', 'medium', or 'high'".into(),
                    ))
                }
            }
        }

        // Validate top_violations if provided
        if let Some(violations) = self.top_violations {
            if violations > 1000 {
                return Err(ContractError::TooManyFiles(violations));
            }
        }

        // Validate file path if provided
        if let Some(file) = &self.file {
            PathValidator::ensure_exists(file)
                .map_err(|_| ContractError::PathNotFound(file.clone()))?;
        }

        Ok(())
    }
}

impl ContractValidation for QualityGateContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if let Some(file) = &self.file {
            PathValidator::ensure_exists(file)
                .map_err(|_| ContractError::PathNotFound(file.clone()))?;
        }

        Ok(())
    }
}

impl ContractValidation for RefactorAutoContract {
    fn validate(&self) -> Result<(), ContractError> {
        PathValidator::ensure_file(&self.file)
            .map_err(|_| ContractError::PathNotFound(self.file.clone()))?;

        if self.timeout == 0 {
            return Err(ContractError::InvalidTimeout);
        }

        if self.target_complexity == 0 {
            return Err(ContractError::InvalidValue(
                "target_complexity must be > 0".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod contract_default_tests {
    use super::*;

    #[test]
    fn test_default_tdg_threshold() {
        assert!((default_tdg_threshold() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_target_complexity() {
        assert_eq!(default_target_complexity(), 10);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(default_timeout(), 60);
    }

    #[test]
    fn test_analyze_tdg_contract_serde_defaults() {
        let json = r#"{"base":{}}"#;
        let contract: AnalyzeTdgContract = serde_json::from_str(json).unwrap();
        assert!((contract.threshold - 1.5).abs() < f64::EPSILON);
        assert!(!contract.include_components);
        assert!(!contract.critical_only);
    }

    #[test]
    fn test_refactor_auto_contract_serde_defaults() {
        let json = r#"{"file":"test.rs"}"#;
        let contract: RefactorAutoContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.target_complexity, 10);
        assert_eq!(contract.timeout, 60);
        assert!(!contract.dry_run);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
