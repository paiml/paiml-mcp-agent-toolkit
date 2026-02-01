//! YAML-first configuration for pmat comply checks.
//!
//! Implements COMPLY-044 from improve-pmat-comply.md v2.8:
//! "Every quality check should be configurable via .pmat.yaml without code changes."
//!
//! # Configuration File
//!
//! Create a `.pmat.yaml` file in your project root:
//!
//! ```yaml
//! comply:
//!   checks:
//!     cb-050: { enabled: true, severity: critical }
//!     cb-060: { enabled: true, severity: high }
//!     cb-128: { enabled: true, threshold: 5.0 }
//!   thresholds:
//!     coverage: 95.0
//!     complexity: 20
//!     dead_code_pct: 1.0
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use pmat::models::comply_config::ComplyConfig;
//! use std::path::Path;
//!
//! let config = ComplyConfig::load(Path::new(".")).unwrap_or_default();
//! if config.is_check_enabled("cb-050") {
//!     // Run the check
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root configuration loaded from .pmat.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PmatYamlConfig {
    /// Comply-specific configuration
    #[serde(default)]
    pub comply: ComplyConfig,

    /// Quality gate configuration
    #[serde(default)]
    pub quality: QualityConfig,

    /// Work contract configuration
    #[serde(default)]
    pub work: WorkConfig,
}

/// Configuration for pmat comply checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplyConfig {
    /// Individual check configurations keyed by check ID (e.g., "cb-050")
    #[serde(default)]
    pub checks: HashMap<String, CheckConfig>,

    /// Global thresholds
    #[serde(default)]
    pub thresholds: ComplyThresholds,

    /// Whether to fail on first error or collect all errors
    #[serde(default)]
    pub fail_fast: bool,

    /// Output format preferences
    #[serde(default)]
    pub output: OutputConfig,
}

impl Default for ComplyConfig {
    fn default() -> Self {
        Self {
            checks: default_checks(),
            thresholds: ComplyThresholds::default(),
            fail_fast: false,
            output: OutputConfig::default(),
        }
    }
}

/// Configuration for an individual check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    /// Whether the check is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Severity level for this check
    #[serde(default)]
    pub severity: CheckSeverity,

    /// Optional threshold for numeric checks
    #[serde(default)]
    pub threshold: Option<f64>,

    /// Additional check-specific options
    #[serde(default)]
    pub options: HashMap<String, serde_yaml::Value>,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        }
    }
}

/// Severity levels for checks (matches check_handlers.rs Severity enum)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    /// Informational - logged but doesn't fail
    Info,
    /// Warning - logged, may fail in strict mode
    #[default]
    Warning,
    /// Error - always fails
    Error,
    /// Critical - blocks all further checks
    Critical,
}

/// Global thresholds for comply checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplyThresholds {
    /// Minimum coverage percentage (0-100)
    #[serde(default = "default_coverage")]
    pub coverage: f64,

    /// Minimum per-file coverage percentage (0-100)
    #[serde(default = "default_per_file_coverage")]
    pub per_file_coverage: f64,

    /// Maximum cyclomatic complexity per function
    #[serde(default = "default_complexity")]
    pub complexity: u32,

    /// Maximum allowed dead code percentage
    #[serde(default = "default_dead_code")]
    pub dead_code_pct: f64,

    /// Maximum file size in lines
    #[serde(default = "default_file_size")]
    pub max_file_lines: u32,

    /// Maximum function size in lines
    #[serde(default = "default_function_size")]
    pub max_function_lines: u32,

    /// Slow test threshold in seconds
    #[serde(default = "default_slow_test")]
    pub slow_test_seconds: f64,

    /// Slow coverage threshold in minutes
    #[serde(default = "default_slow_coverage")]
    pub slow_coverage_minutes: f64,
}

impl Default for ComplyThresholds {
    fn default() -> Self {
        Self {
            coverage: default_coverage(),
            per_file_coverage: default_per_file_coverage(),
            complexity: default_complexity(),
            dead_code_pct: default_dead_code(),
            max_file_lines: default_file_size(),
            max_function_lines: default_function_size(),
            slow_test_seconds: default_slow_test(),
            slow_coverage_minutes: default_slow_coverage(),
        }
    }
}

/// Output format configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    /// Show only failures (hide passing checks)
    #[serde(default)]
    pub failures_only: bool,

    /// Use colors in output
    #[serde(default = "default_true")]
    pub colors: bool,

    /// Show verbose details
    #[serde(default)]
    pub verbose: bool,
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityConfig {
    /// Enable TDG scoring
    #[serde(default = "default_true")]
    pub tdg_enabled: bool,

    /// Minimum TDG score (0-100)
    #[serde(default = "default_tdg_score")]
    pub min_tdg_score: f64,

    /// Enable SATD detection
    #[serde(default = "default_true")]
    pub satd_enabled: bool,

    /// Block on new SATD markers
    #[serde(default)]
    pub block_on_new_satd: bool,
}

/// Work contract configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkConfig {
    /// Cache staleness warning threshold in hours
    #[serde(default = "default_cache_warn_hours")]
    pub cache_warn_hours: i64,

    /// Cache staleness block threshold in hours
    #[serde(default = "default_cache_block_hours")]
    pub cache_block_hours: i64,

    /// Skip quality gates on completion (not recommended)
    #[serde(default)]
    pub skip_quality_gates: bool,
}

// Default value functions
fn default_true() -> bool { true }
fn default_coverage() -> f64 { 95.0 }
fn default_per_file_coverage() -> f64 { 95.0 }
fn default_complexity() -> u32 { 20 }
fn default_dead_code() -> f64 { 1.0 }
fn default_file_size() -> u32 { 500 }
fn default_function_size() -> u32 { 50 }
fn default_slow_test() -> f64 { 5.0 }
fn default_slow_coverage() -> f64 { 10.0 }
fn default_tdg_score() -> f64 { 70.0 }
fn default_cache_warn_hours() -> i64 { 1 }
fn default_cache_block_hours() -> i64 { 24 }

/// Create default check configurations for all CB checks
fn default_checks() -> HashMap<String, CheckConfig> {
    let mut checks = HashMap::new();

    // CB-050: Stub detection (Critical - runtime panics)
    checks.insert("cb-050".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Critical,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-060: GPU kernel quality (High - hardware crashes)
    checks.insert("cb-060".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Error,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-070: Critical unwrap detection
    checks.insert("cb-070".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-120: NaN-unsafe comparisons
    checks.insert("cb-120".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-121: Lock poisoning vulnerabilities
    checks.insert("cb-121".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-122: Serde deserialization panics
    checks.insert("cb-122".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-123: Undocumented ignored tests
    checks.insert("cb-123".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Info,
        threshold: None,
        options: HashMap::new(),
    });

    // CB-124: Coverage threshold
    checks.insert("cb-124".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Error,
        threshold: Some(80.0),
        options: HashMap::new(),
    });

    // CB-125: Coverage exclusion gaming
    checks.insert("cb-125".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: Some(20.0), // Max exclusion percentage
        options: HashMap::new(),
    });

    // CB-126: Slow tests
    checks.insert("cb-126".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: Some(5.0), // seconds
        options: HashMap::new(),
    });

    // CB-127: Slow coverage
    checks.insert("cb-127".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: Some(10.0), // minutes
        options: HashMap::new(),
    });

    // CB-128: Dead code detection
    checks.insert("cb-128".to_string(), CheckConfig {
        enabled: true,
        severity: CheckSeverity::Warning,
        threshold: Some(1.0), // Max dead code percentage
        options: HashMap::new(),
    });

    checks
}

impl PmatYamlConfig {
    /// Load configuration from .pmat.yaml in the given directory
    ///
    /// Returns default configuration if file doesn't exist.
    /// Returns error if file exists but is malformed.
    pub fn load(project_path: &Path) -> Result<Self, ConfigError> {
        let config_path = project_path.join(".pmat.yaml");

        if !config_path.exists() {
            // Also check for .pmat.yml
            let alt_path = project_path.join(".pmat.yml");
            if !alt_path.exists() {
                return Ok(Self::default());
            }
            return Self::load_from_path(&alt_path);
        }

        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific file path
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save configuration to .pmat.yaml in the given directory
    pub fn save(&self, project_path: &Path) -> Result<(), ConfigError> {
        let config_path = project_path.join(".pmat.yaml");
        let content = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(config_path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }
}

impl ComplyConfig {
    /// Check if a specific check is enabled
    pub fn is_check_enabled(&self, check_id: &str) -> bool {
        self.checks
            .get(check_id)
            .map(|c| c.enabled)
            .unwrap_or(true) // Default to enabled for unknown checks
    }

    /// Get the severity for a check
    pub fn get_severity(&self, check_id: &str) -> CheckSeverity {
        self.checks
            .get(check_id)
            .map(|c| c.severity)
            .unwrap_or(CheckSeverity::Warning)
    }

    /// Get the threshold for a check
    pub fn get_threshold(&self, check_id: &str) -> Option<f64> {
        self.checks.get(check_id).and_then(|c| c.threshold)
    }

    /// Check if a severity level should cause failure
    pub fn should_fail(&self, severity: CheckSeverity, strict: bool) -> bool {
        match severity {
            CheckSeverity::Critical | CheckSeverity::Error => true,
            CheckSeverity::Warning => strict,
            CheckSeverity::Info => false,
        }
    }
}

/// Configuration loading errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error loading config: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error in .pmat.yaml: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PmatYamlConfig::default();
        assert!(config.comply.is_check_enabled("cb-050"));
        assert!(config.comply.is_check_enabled("cb-060"));
        assert_eq!(config.comply.thresholds.coverage, 95.0);
        assert_eq!(config.comply.thresholds.complexity, 20);
    }

    #[test]
    fn test_severity_should_fail() {
        let config = ComplyConfig::default();

        // Critical always fails
        assert!(config.should_fail(CheckSeverity::Critical, false));
        assert!(config.should_fail(CheckSeverity::Critical, true));

        // Error always fails
        assert!(config.should_fail(CheckSeverity::Error, false));
        assert!(config.should_fail(CheckSeverity::Error, true));

        // Warning fails only in strict mode
        assert!(!config.should_fail(CheckSeverity::Warning, false));
        assert!(config.should_fail(CheckSeverity::Warning, true));

        // Info never fails
        assert!(!config.should_fail(CheckSeverity::Info, false));
        assert!(!config.should_fail(CheckSeverity::Info, true));
    }

    #[test]
    fn test_yaml_parsing() {
        let yaml = r#"
comply:
  checks:
    cb-050:
      enabled: false
      severity: warning
    cb-128:
      enabled: true
      threshold: 2.5
  thresholds:
    coverage: 90.0
    complexity: 15
"#;

        let config: PmatYamlConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.comply.is_check_enabled("cb-050"));
        assert!(config.comply.is_check_enabled("cb-128"));
        assert_eq!(config.comply.get_threshold("cb-128"), Some(2.5));
        assert_eq!(config.comply.thresholds.coverage, 90.0);
        assert_eq!(config.comply.thresholds.complexity, 15);
    }

    #[test]
    fn test_unknown_check_defaults_to_enabled() {
        let config = ComplyConfig::default();
        // Unknown check should default to enabled
        assert!(config.is_check_enabled("cb-999"));
        assert_eq!(config.get_severity("cb-999"), CheckSeverity::Warning);
    }

    #[test]
    fn test_check_config_default() {
        let check = CheckConfig::default();
        assert!(check.enabled);
        assert_eq!(check.severity, CheckSeverity::Warning);
        assert!(check.threshold.is_none());
    }
}
