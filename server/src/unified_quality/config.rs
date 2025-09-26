//! Configuration for the unified quality system

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

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            mode: QualityMode::Observe,
            auto_progress: true,
            progress_after_days: 30,
            budget: BudgetConfig::default(),
            monitoring: MonitoringConfig::default(),
            automation: AutomationConfig::default(),
            research: ResearchConfig::default(),
        }
    }
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            complexity_points: 100,
            satd_allowance: 20,
            coverage_floor: 70.0,
            regeneration_daily: 5.0,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            incremental: true,
            cache_ast: true,
            dashboard_port: 8080,
            update_interval: 5,
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
            ],
        }
    }
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_review: true,
            safe_only: true,
            create_branches: true,
        }
    }
}

impl UnifiedConfig {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get default configuration path
    #[must_use]
    pub fn default_path() -> PathBuf {
        PathBuf::from(".pmat/config.toml")
    }

    /// Check if should auto-progress
    #[must_use]
    pub fn should_progress(&self, days_in_mode: u32) -> bool {
        self.auto_progress && days_in_mode >= self.progress_after_days
    }

    /// Get next quality mode
    #[must_use]
    pub fn next_mode(&self) -> Option<QualityMode> {
        match self.mode {
            QualityMode::Observe => Some(QualityMode::Advise),
            QualityMode::Advise => Some(QualityMode::Guide),
            QualityMode::Guide => Some(QualityMode::Enforce),
            QualityMode::Enforce => Some(QualityMode::Extreme),
            QualityMode::Extreme => None,
        }
    }
}

/// Team adoption playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionPlaybook {
    /// Current week in adoption process
    pub current_week: u32,

    /// Adoption phases
    pub phases: Vec<AdoptionPhase>,
}

/// An adoption phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionPhase {
    /// Week range (e.g., 1-2)
    pub weeks: (u32, u32),

    /// Phase name
    pub name: String,

    /// Activities in this phase
    pub activities: Vec<String>,

    /// Success criteria
    pub success_criteria: Vec<String>,
}

impl Default for AdoptionPlaybook {
    fn default() -> Self {
        Self {
            current_week: 1,
            phases: vec![
                AdoptionPhase {
                    weeks: (1, 2),
                    name: "Baseline Establishment".to_string(),
                    activities: vec![
                        "Install PMAT in observe mode".to_string(),
                        "Gather baseline metrics".to_string(),
                        "No enforcement, just visibility".to_string(),
                    ],
                    success_criteria: vec![
                        "PMAT installed and running".to_string(),
                        "Baseline metrics collected".to_string(),
                    ],
                },
                AdoptionPhase {
                    weeks: (3, 4),
                    name: "Team Education".to_string(),
                    activities: vec![
                        "Review complexity hotspots together".to_string(),
                        "Discuss SATD findings".to_string(),
                        "Set initial quality goals".to_string(),
                    ],
                    success_criteria: vec![
                        "Team understands metrics".to_string(),
                        "Quality goals agreed upon".to_string(),
                    ],
                },
                AdoptionPhase {
                    weeks: (5, 8),
                    name: "Assisted Improvement".to_string(),
                    activities: vec![
                        "Enable suggestions".to_string(),
                        "Track suggestion success rate".to_string(),
                        "Celebrate improvements".to_string(),
                    ],
                    success_criteria: vec![
                        "60% suggestion acceptance rate".to_string(),
                        "Measurable quality improvement".to_string(),
                    ],
                },
                AdoptionPhase {
                    weeks: (9, 12),
                    name: "Gradual Enforcement".to_string(),
                    activities: vec![
                        "Enable warnings on PR".to_string(),
                        "Set generous error budgets".to_string(),
                        "Allow overrides with justification".to_string(),
                    ],
                    success_criteria: vec![
                        "80% PRs pass quality gates".to_string(),
                        "Error budget not exceeded".to_string(),
                    ],
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_config_default() {
        let config = UnifiedConfig::default();
        assert_eq!(config.mode, QualityMode::Observe);
        assert!(config.auto_progress);
        assert!(config.monitoring.enabled);
        assert!(!config.automation.enabled);
    }

    #[test]
    fn test_should_progress() {
        let config = UnifiedConfig::default();
        assert!(!config.should_progress(15));
        assert!(config.should_progress(30));
        assert!(config.should_progress(45));
    }

    #[test]
    fn test_next_mode() {
        let mut config = UnifiedConfig {
            mode: QualityMode::Observe,
            ..UnifiedConfig::default()
        };
        assert_eq!(config.next_mode(), Some(QualityMode::Advise));

        config.mode = QualityMode::Extreme;
        assert_eq!(config.next_mode(), None);
    }

    #[test]
    fn test_adoption_playbook() {
        let playbook = AdoptionPlaybook::default();
        assert_eq!(playbook.current_week, 1);
        assert_eq!(playbook.phases.len(), 4);
        assert_eq!(playbook.phases[0].name, "Baseline Establishment");
    }
}
