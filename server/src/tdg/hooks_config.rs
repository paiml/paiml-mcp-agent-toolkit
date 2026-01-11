//! TDG Hooks Configuration (Sprint 66 Phase 3)
//!
//! Configuration for TDG git hooks enforcement system.
//! Loaded from `.pmat/tdg-rules.toml`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// TDG hooks configuration loaded from .pmat/tdg-rules.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TdgHooksConfig {
    #[serde(default)]
    pub quality_gates: QualityGatesConfig,

    #[serde(default)]
    pub baseline: BaselineConfig,

    #[serde(default)]
    pub ci_cd: CiCdConfig,
}

impl TdgHooksConfig {
    /// Load configuration from .pmat/tdg-rules.toml
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join(".pmat").join("tdg-rules.toml");

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .context(format!("Failed to read config file: {:?}", config_path))?;

        let config: TdgHooksConfig = toml::from_str(&contents)
            .context(format!("Failed to parse config file: {:?}", config_path))?;

        Ok(config)
    }

    /// Create default configuration file at .pmat/tdg-rules.toml
    pub fn create_default(project_root: &Path) -> Result<()> {
        let pmat_dir = project_root.join(".pmat");
        let config_path = pmat_dir.join("tdg-rules.toml");

        // Create .pmat directory if it doesn't exist
        if !pmat_dir.exists() {
            fs::create_dir_all(&pmat_dir)?;
        }

        // Don't overwrite existing config
        if config_path.exists() {
            return Ok(());
        }

        let default_config = Self::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        fs::write(&config_path, toml_string)
            .context(format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }
}

/// Quality gates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesConfig {
    /// Minimum grades by language
    #[serde(default)]
    pub min_grades: HashMap<String, String>,

    /// Deprecated: Use min_grades instead
    #[serde(default)]
    pub rust_min_grade: Option<String>,

    /// Deprecated: Use min_grades instead
    #[serde(default)]
    pub typescript_min_grade: Option<String>,

    /// Deprecated: Use min_grades instead
    #[serde(default)]
    pub python_min_grade: Option<String>,

    /// Maximum score drop allowed
    #[serde(default = "default_max_score_drop")]
    pub max_score_drop: f32,

    /// Allow grade drops (but enforce max_score_drop)
    #[serde(default)]
    pub allow_grade_drop: bool,

    /// Enforcement mode: strict, warning, disabled
    #[serde(default = "default_mode")]
    pub mode: EnforcementMode,

    /// Block commits on quality regression
    #[serde(default = "default_true")]
    pub block_on_regression: bool,

    /// Block commits when new files below threshold
    #[serde(default = "default_true")]
    pub block_on_new_files_below_threshold: bool,
}

impl Default for QualityGatesConfig {
    fn default() -> Self {
        let mut min_grades = HashMap::new();
        min_grades.insert("rust".to_string(), "B+".to_string());
        min_grades.insert("typescript".to_string(), "B+".to_string());
        min_grades.insert("python".to_string(), "B".to_string());

        Self {
            min_grades,
            rust_min_grade: Some("B+".to_string()),
            typescript_min_grade: Some("B+".to_string()),
            python_min_grade: Some("B".to_string()),
            max_score_drop: 5.0,
            allow_grade_drop: false,
            mode: EnforcementMode::Strict,
            block_on_regression: true,
            block_on_new_files_below_threshold: true,
        }
    }
}

impl QualityGatesConfig {
    /// Get minimum grade for a language (with fallback to deprecated fields)
    pub fn get_min_grade(&self, language: &str) -> Option<&str> {
        // Try new min_grades map first
        if let Some(grade) = self.min_grades.get(language) {
            return Some(grade.as_str());
        }

        // Fallback to deprecated fields for backward compatibility
        match language.to_lowercase().as_str() {
            "rust" => self.rust_min_grade.as_deref(),
            "typescript" | "javascript" => self.typescript_min_grade.as_deref(),
            "python" => self.python_min_grade.as_deref(),
            _ => None,
        }
    }

    /// Get default minimum grade (B+ for most languages)
    pub fn get_default_min_grade(&self) -> &str {
        "B+"
    }
}

/// Baseline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    /// Auto-update baseline on successful commit
    #[serde(default = "default_true")]
    pub auto_update_on_commit: bool,

    /// Auto-update baseline on merge commits
    #[serde(default = "default_true")]
    pub auto_update_on_merge: bool,

    /// Path to baseline file (relative to project root)
    #[serde(default = "default_baseline_path")]
    pub baseline_path: String,

    /// Track baseline file in git
    #[serde(default = "default_true")]
    pub store_in_git: bool,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            auto_update_on_commit: true,
            auto_update_on_merge: true,
            baseline_path: ".pmat/baseline.json".to_string(),
            store_in_git: true,
        }
    }
}

/// CI/CD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCdConfig {
    /// Fail fast on first quality gate failure
    #[serde(default)]
    pub fail_fast: bool,

    /// Generate HTML/JSON reports
    #[serde(default = "default_true")]
    pub generate_reports: bool,

    /// Comment quality results on PRs (if CI supports it)
    #[serde(default = "default_true")]
    pub comment_on_pr: bool,
}

impl Default for CiCdConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            generate_reports: true,
            comment_on_pr: true,
        }
    }
}

/// Enforcement mode for quality gates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum EnforcementMode {
    /// Strict mode: block commits on violations
    #[default]
    Strict,
    /// Warning mode: show warnings but allow commits
    Warning,
    /// Disabled: no enforcement
    Disabled,
}

impl std::fmt::Display for EnforcementMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Warning => write!(f, "warning"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

// Default value functions for serde
fn default_max_score_drop() -> f32 {
    5.0
}

fn default_mode() -> EnforcementMode {
    EnforcementMode::Strict
}

fn default_true() -> bool {
    true
}

fn default_baseline_path() -> String {
    ".pmat/baseline.json".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tdg_hooks_config_default() {
        let config = TdgHooksConfig::default();
        assert_eq!(config.quality_gates.max_score_drop, 5.0);
        assert_eq!(config.quality_gates.mode, EnforcementMode::Strict);
        assert!(config.baseline.auto_update_on_commit);
        assert!(config.ci_cd.generate_reports);
    }

    #[test]
    fn test_tdg_hooks_config_load_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let config = TdgHooksConfig::load(temp_dir.path()).unwrap();

        // Should return default when file doesn't exist
        assert_eq!(config.quality_gates.max_score_drop, 5.0);
    }

    #[test]
    fn test_tdg_hooks_config_create_default() {
        let temp_dir = tempdir().unwrap();

        // Create default config
        TdgHooksConfig::create_default(temp_dir.path()).unwrap();

        // Verify file was created
        let config_path = temp_dir.path().join(".pmat").join("tdg-rules.toml");
        assert!(config_path.exists());

        // Verify config can be loaded
        let loaded = TdgHooksConfig::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.quality_gates.max_score_drop, 5.0);
    }

    #[test]
    fn test_tdg_hooks_config_create_default_idempotent() {
        let temp_dir = tempdir().unwrap();

        // Create once
        TdgHooksConfig::create_default(temp_dir.path()).unwrap();

        // Create again - should not error
        TdgHooksConfig::create_default(temp_dir.path()).unwrap();
    }

    #[test]
    fn test_quality_gates_config_default() {
        let config = QualityGatesConfig::default();

        assert_eq!(config.min_grades.get("rust"), Some(&"B+".to_string()));
        assert_eq!(config.min_grades.get("typescript"), Some(&"B+".to_string()));
        assert_eq!(config.min_grades.get("python"), Some(&"B".to_string()));
        assert_eq!(config.max_score_drop, 5.0);
        assert!(!config.allow_grade_drop);
        assert_eq!(config.mode, EnforcementMode::Strict);
        assert!(config.block_on_regression);
        assert!(config.block_on_new_files_below_threshold);
    }

    #[test]
    fn test_quality_gates_get_min_grade() {
        let config = QualityGatesConfig::default();

        assert_eq!(config.get_min_grade("rust"), Some("B+"));
        assert_eq!(config.get_min_grade("typescript"), Some("B+"));
        assert_eq!(config.get_min_grade("python"), Some("B"));
        assert_eq!(config.get_min_grade("unknown"), None);
    }

    #[test]
    fn test_quality_gates_get_min_grade_fallback() {
        let mut config = QualityGatesConfig::default();
        config.min_grades.clear(); // Clear the new format

        // Should fallback to deprecated fields
        assert_eq!(config.get_min_grade("rust"), Some("B+"));
        assert_eq!(config.get_min_grade("javascript"), Some("B+")); // Uses typescript fallback
    }

    #[test]
    fn test_quality_gates_get_default_min_grade() {
        let config = QualityGatesConfig::default();
        assert_eq!(config.get_default_min_grade(), "B+");
    }

    #[test]
    fn test_baseline_config_default() {
        let config = BaselineConfig::default();

        assert!(config.auto_update_on_commit);
        assert!(config.auto_update_on_merge);
        assert_eq!(config.baseline_path, ".pmat/baseline.json");
        assert!(config.store_in_git);
    }

    #[test]
    fn test_cicd_config_default() {
        let config = CiCdConfig::default();

        assert!(!config.fail_fast);
        assert!(config.generate_reports);
        assert!(config.comment_on_pr);
    }

    #[test]
    fn test_enforcement_mode_variants() {
        assert!(matches!(EnforcementMode::Strict, EnforcementMode::Strict));
        assert!(matches!(EnforcementMode::Warning, EnforcementMode::Warning));
        assert!(matches!(EnforcementMode::Disabled, EnforcementMode::Disabled));
    }

    #[test]
    fn test_enforcement_mode_display() {
        assert_eq!(EnforcementMode::Strict.to_string(), "strict");
        assert_eq!(EnforcementMode::Warning.to_string(), "warning");
        assert_eq!(EnforcementMode::Disabled.to_string(), "disabled");
    }

    #[test]
    fn test_enforcement_mode_default() {
        let mode = EnforcementMode::default();
        assert_eq!(mode, EnforcementMode::Strict);
    }

    #[test]
    fn test_enforcement_mode_equality() {
        assert_eq!(EnforcementMode::Strict, EnforcementMode::Strict);
        assert_ne!(EnforcementMode::Strict, EnforcementMode::Warning);
    }

    #[test]
    fn test_enforcement_mode_clone() {
        let mode = EnforcementMode::Warning;
        let cloned = mode.clone();
        assert_eq!(cloned, EnforcementMode::Warning);
    }

    #[test]
    fn test_default_helper_functions() {
        assert_eq!(default_max_score_drop(), 5.0);
        assert_eq!(default_mode(), EnforcementMode::Strict);
        assert!(default_true());
        assert_eq!(default_baseline_path(), ".pmat/baseline.json");
    }

    #[test]
    fn test_config_serialization() {
        let config = TdgHooksConfig::default();
        let toml_str = toml::to_string(&config).unwrap();

        assert!(toml_str.contains("max_score_drop"));
        assert!(toml_str.contains("baseline_path"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[quality_gates]
max_score_drop = 10.0
mode = "warning"

[baseline]
auto_update_on_commit = false

[ci_cd]
fail_fast = true
"#;
        let config: TdgHooksConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.quality_gates.max_score_drop, 10.0);
        assert_eq!(config.quality_gates.mode, EnforcementMode::Warning);
        assert!(!config.baseline.auto_update_on_commit);
        assert!(config.ci_cd.fail_fast);
    }

    #[test]
    fn test_tdg_hooks_config_clone() {
        let config = TdgHooksConfig::default();
        let cloned = config.clone();

        assert_eq!(cloned.quality_gates.max_score_drop, config.quality_gates.max_score_drop);
    }

    #[test]
    fn test_quality_gates_config_clone() {
        let config = QualityGatesConfig::default();
        let cloned = config.clone();

        assert_eq!(cloned.max_score_drop, config.max_score_drop);
    }

    #[test]
    fn test_baseline_config_clone() {
        let config = BaselineConfig::default();
        let cloned = config.clone();

        assert_eq!(cloned.baseline_path, config.baseline_path);
    }

    #[test]
    fn test_cicd_config_clone() {
        let config = CiCdConfig::default();
        let cloned = config.clone();

        assert_eq!(cloned.fail_fast, config.fail_fast);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn config_serialization_roundtrip(max_score in 0.0f32..100.0) {
            let mut config = TdgHooksConfig::default();
            config.quality_gates.max_score_drop = max_score;

            let toml_str = toml::to_string(&config).unwrap();
            let deserialized: TdgHooksConfig = toml::from_str(&toml_str).unwrap();

            prop_assert!((deserialized.quality_gates.max_score_drop - max_score).abs() < 0.01);
        }

        #[test]
        fn enforcement_mode_string_conversion(mode_val in 0u8..3) {
            let mode = match mode_val {
                0 => EnforcementMode::Strict,
                1 => EnforcementMode::Warning,
                _ => EnforcementMode::Disabled,
            };

            let mode_str = mode.to_string();
            prop_assert!(mode_str == "strict" || mode_str == "warning" || mode_str == "disabled");
        }
    }
}
