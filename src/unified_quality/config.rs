//! Configuration for the unified quality system

#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::unified_quality::QualityMode;

/// Complete configuration for the unified quality system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    /// Current quality mode
    pub mode: QualityMode,

    /// Auto-progress through modes
    pub auto_progress: bool,

    /// Days before auto-progression
    pub progress_after_days: u32,

    /// Budget configuration
    pub budget: BudgetConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,

    /// Automation configuration
    pub automation: AutomationConfig,

    /// Research features (disabled by default)
    pub research: ResearchConfig,
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Complexity points allowed
    pub complexity_points: i32,

    /// SATD allowance
    pub satd_allowance: u32,

    /// Coverage floor percentage
    pub coverage_floor: f64,

    /// Daily regeneration rate
    pub regeneration_daily: f64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,

    /// Use incremental parsing
    pub incremental: bool,

    /// Cache ASTs for performance
    pub cache_ast: bool,

    /// Dashboard port
    pub dashboard_port: u16,

    /// Update interval in seconds
    pub update_interval: u64,

    /// File patterns to watch
    pub watch_patterns: Vec<String>,
}

/// Automation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    /// Enable automation
    pub enabled: bool,

    /// Require human review
    pub require_review: bool,

    /// Only apply safe fixes
    pub safe_only: bool,

    /// Create branches for fixes
    pub create_branches: bool,
}

/// Research features configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchConfig {
    /// Enable property synthesis
    pub property_synthesis: bool,

    /// Enable formal verification
    pub formal_verification: bool,

    /// Enable ML suggestions
    pub ml_suggestions: bool,

    /// Enable autonomous mode
    pub autonomous_mode: bool,
}

// Default implementations and UnifiedConfig methods
include!("config_defaults.rs");

// Adoption playbook types and defaults
include!("config_adoption.rs");

// Tests
include!("config_tests.rs");
