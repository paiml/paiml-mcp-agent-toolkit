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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for TdgHooksConfig {
    fn default() -> Self {
        Self {
            quality_gates: QualityGatesConfig::default(),
            baseline: BaselineConfig::default(),
            ci_cd: CiCdConfig::default(),
        }
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
pub enum EnforcementMode {
    /// Strict mode: block commits on violations
    Strict,
    /// Warning mode: show warnings but allow commits
    Warning,
    /// Disabled: no enforcement
    Disabled,
}

impl Default for EnforcementMode {
    fn default() -> Self {
        Self::Strict
    }
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
