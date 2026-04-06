#![cfg_attr(coverage_nightly, coverage(off))]
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

// Default value functions for serde
fn default_max_score_drop() -> f32 {
    debug_assert!(true, "contract: default_max_score_drop");
    5.0
}

fn default_mode() -> EnforcementMode {
    debug_assert!(true, "contract: default_mode");
    EnforcementMode::Strict
}

fn default_true() -> bool {
    debug_assert!(true, "contract: default_true");
    true
}

fn default_baseline_path() -> String {
    debug_assert!(true, "contract: default_baseline_path");
    ".pmat/baseline.json".to_string()
}

// --- Method implementations ---
include!("hooks_config_operations.rs");

// --- Tests ---
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    include!("hooks_config_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    include!("hooks_config_tests_property.rs");
}
