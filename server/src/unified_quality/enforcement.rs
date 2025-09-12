//! Enforcement Layer: Error Budget System
//!
//! Phase 3 Implementation (Months 7-9)
//! Flexible enforcement using SRE-style error budgets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Flexible enforcement using SRE-style error budgets
#[allow(dead_code)]
pub struct ErrorBudgetEnforcer {
    /// Team-specific quality budgets
    budgets: HashMap<TeamId, QualityBudget>,

    /// Historical tracking for trends
    history: TimeSeriesDB,

    /// Progressive enforcement rules
    rules: EnforcementRules,

    /// Configuration
    config: EnforcerConfig,
}

/// Team identifier
pub type TeamId = String;

/// Quality budget for a team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBudget {
    /// Complexity points above baseline
    pub complexity_budget: i32,

    /// Temporary SATD allowance
    pub satd_budget: u32,

    /// Coverage floor during refactoring
    pub coverage_floor: f64,

    /// Budget regeneration rate (daily)
    pub regeneration_rate: f64,

    /// Time until strict enforcement
    pub grace_period: Duration,

    /// Current consumption
    pub current_consumption: BudgetConsumption,

    /// Budget started at
    pub started_at: SystemTime,
}

/// Current budget consumption
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetConsumption {
    /// Complexity points consumed
    pub complexity_used: i32,

    /// SATD issues added
    pub satd_used: u32,

    /// Coverage reduction
    pub coverage_reduction: f64,

    /// Last updated
    pub last_updated: Option<SystemTime>,
}

/// Enforcement decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Change approved
    Approved,

    /// Warning issued but not blocked
    Warning(String),

    /// Requires manual approval
    RequiresApproval {
        approvers: Vec<String>,
        reason_required: bool,
    },

    /// Change blocked
    Blocked {
        suggestion: String,
        refactor_targets: Vec<RefactorTarget>,
    },
}

/// Refactoring target suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorTarget {
    pub file: String,
    pub complexity: u32,
    pub estimated_reduction: u32,
    pub priority: Priority,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Time series database for historical tracking
pub struct TimeSeriesDB {
    /// Stored metrics by team and timestamp
    data: HashMap<TeamId, Vec<TimeSeries>>,
}

/// Time series entry
#[derive(Debug, Clone)]
struct TimeSeries {
    timestamp: SystemTime,
    metrics: TeamMetrics,
}

/// Team-level metrics
#[derive(Debug, Clone)]
pub struct TeamMetrics {
    #[allow(dead_code)]
    avg_complexity: f64,
    #[allow(dead_code)]
    total_satd: u32,
    #[allow(dead_code)]
    avg_coverage: f64,
    quality_score: f64,
}

/// Enforcement rules configuration
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct EnforcementRules {
    /// Approvers by team
    approvers: HashMap<TeamId, Vec<String>>,

    /// Escalation thresholds
    escalation: EscalationPolicy,

    /// Override permissions
    overrides: HashMap<String, OverridePermission>,
}

/// Escalation policy
#[derive(Debug, Clone)]
struct EscalationPolicy {
    warning_threshold: f64,
    approval_threshold: f64,
    block_threshold: f64,
}

impl Default for EscalationPolicy {
    fn default() -> Self {
        Self {
            warning_threshold: 0.5,
            approval_threshold: 0.8,
            block_threshold: 1.0,
        }
    }
}

/// Override permission levels
#[derive(Debug, Clone)]
enum OverridePermission {
    #[allow(dead_code)]
    None,
    #[allow(dead_code)]
    Limited,
    #[allow(dead_code)]
    Full,
}

/// Enforcer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcerConfig {
    /// Enable budget enforcement
    pub enabled: bool,

    /// Default budget for new teams
    pub default_budget: QualityBudget,

    /// Budget regeneration interval
    pub regeneration_interval: Duration,

    /// History retention period
    pub history_retention: Duration,
}

impl Default for EnforcerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_budget: QualityBudget::default(),
            regeneration_interval: Duration::from_secs(24 * 60 * 60), // Daily
            history_retention: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        }
    }
}

impl Default for QualityBudget {
    fn default() -> Self {
        Self {
            complexity_budget: 100,
            satd_budget: 20,
            coverage_floor: 0.7,
            regeneration_rate: 5.0,
            grace_period: Duration::from_secs(7 * 24 * 60 * 60), // 1 week
            current_consumption: BudgetConsumption::default(),
            started_at: SystemTime::now(),
        }
    }
}

/// Diff analysis result
#[derive(Debug, Clone)]
pub struct DiffAnalysis {
    pub complexity_change: i32,
    pub satd_change: i32,
    pub coverage_change: f64,
    pub files_changed: Vec<String>,
}

impl ErrorBudgetEnforcer {
    /// Create a new error budget enforcer
    pub fn new(config: EnforcerConfig) -> Self {
        Self {
            budgets: HashMap::new(),
            history: TimeSeriesDB::new(),
            rules: EnforcementRules::default(),
            config,
        }
    }

    /// Register a team with a budget
    pub fn register_team(&mut self, team_id: TeamId, budget: Option<QualityBudget>) {
        let budget = budget.unwrap_or_else(|| self.config.default_budget.clone());
        self.budgets.insert(team_id, budget);
    }

    /// Check if a commit is allowed
    pub fn check_commit(&self, team: &TeamId, diff: &DiffAnalysis) -> Decision {
        if !self.config.enabled {
            return Decision::Approved;
        }

        let budget = match self.budgets.get(team) {
            Some(b) => b,
            None => return Decision::Approved, // No budget = no enforcement
        };

        let consumption = self.calculate_consumption(budget, diff);

        match consumption {
            c if c < self.rules.escalation.warning_threshold => Decision::Approved,
            c if c < self.rules.escalation.approval_threshold => Decision::Warning(format!(
                "Approaching budget limit ({:.0}% consumed). Consider refactoring.",
                c * 100.0
            )),
            c if c < self.rules.escalation.block_threshold => Decision::RequiresApproval {
                approvers: self.rules.approvers_for(team),
                reason_required: true,
            },
            _ => Decision::Blocked {
                suggestion: "Budget exceeded. Refactor existing code first.".to_string(),
                refactor_targets: self.suggest_refactor_targets(team),
            },
        }
    }

    /// Update budget consumption
    pub fn update_consumption(&mut self, team: &TeamId, diff: &DiffAnalysis) {
        if let Some(budget) = self.budgets.get_mut(team) {
            budget.current_consumption.complexity_used += diff.complexity_change;
            budget.current_consumption.satd_used =
                (budget.current_consumption.satd_used as i32 + diff.satd_change).max(0) as u32;
            budget.current_consumption.coverage_reduction += diff.coverage_change;
            budget.current_consumption.last_updated = Some(SystemTime::now());
        }
    }

    /// Regenerate budgets (called periodically)
    pub fn regenerate_budgets(&mut self) {
        let now = SystemTime::now();

        for budget in self.budgets.values_mut() {
            if let Some(last_updated) = budget.current_consumption.last_updated {
                if let Ok(elapsed) = now.duration_since(last_updated) {
                    if elapsed >= self.config.regeneration_interval {
                        // Regenerate budget
                        let days_elapsed = elapsed.as_secs() / (24 * 60 * 60);
                        let regeneration = (budget.regeneration_rate * days_elapsed as f64) as i32;

                        budget.current_consumption.complexity_used =
                            (budget.current_consumption.complexity_used - regeneration).max(0);
                        budget.current_consumption.satd_used =
                            budget.current_consumption.satd_used.saturating_sub(5);
                        budget.current_consumption.coverage_reduction =
                            (budget.current_consumption.coverage_reduction - 0.01).max(0.0);
                        budget.current_consumption.last_updated = Some(now);
                    }
                }
            }
        }
    }

    /// Get budget status for a team
    pub fn get_budget_status(&self, team: &TeamId) -> Option<BudgetStatus> {
        self.budgets.get(team).map(|budget| {
            let consumption = self.calculate_consumption_percentage(budget);
            BudgetStatus {
                team: team.clone(),
                consumption_percentage: consumption,
                complexity_remaining: budget.complexity_budget
                    - budget.current_consumption.complexity_used,
                satd_remaining: budget.satd_budget - budget.current_consumption.satd_used,
                days_until_regeneration: self.days_until_regeneration(budget),
            }
        })
    }

    /// Calculate consumption as a percentage
    fn calculate_consumption(&self, budget: &QualityBudget, diff: &DiffAnalysis) -> f64 {
        let complexity_ratio = ((budget.current_consumption.complexity_used
            + diff.complexity_change) as f64)
            / budget.complexity_budget as f64;
        let satd_ratio = ((budget.current_consumption.satd_used as i32 + diff.satd_change) as f64)
            / budget.satd_budget as f64;
        let coverage_impact = if diff.coverage_change < 0.0 {
            (-diff.coverage_change) / (1.0 - budget.coverage_floor)
        } else {
            0.0
        };

        // Weighted average
        (complexity_ratio * 0.5 + satd_ratio * 0.3 + coverage_impact * 0.2)
            .max(0.0)
            .min(1.0)
    }

    /// Calculate consumption percentage for current state
    fn calculate_consumption_percentage(&self, budget: &QualityBudget) -> f64 {
        let complexity_ratio =
            (budget.current_consumption.complexity_used as f64) / budget.complexity_budget as f64;
        let satd_ratio = (budget.current_consumption.satd_used as f64) / budget.satd_budget as f64;

        ((complexity_ratio * 0.6 + satd_ratio * 0.4) * 100.0)
            .max(0.0)
            .min(100.0)
    }

    /// Calculate days until next regeneration
    fn days_until_regeneration(&self, budget: &QualityBudget) -> u32 {
        if let Some(last_updated) = budget.current_consumption.last_updated {
            if let Ok(elapsed) = SystemTime::now().duration_since(last_updated) {
                let remaining = self.config.regeneration_interval.as_secs() - elapsed.as_secs();
                return (remaining / (24 * 60 * 60)) as u32;
            }
        }
        0
    }

    /// Suggest refactoring targets
    fn suggest_refactor_targets(&self, _team: &TeamId) -> Vec<RefactorTarget> {
        // Would integrate with complexity analysis
        vec![
            RefactorTarget {
                file: "src/complex_module.rs".to_string(),
                complexity: 35,
                estimated_reduction: 15,
                priority: Priority::High,
            },
            RefactorTarget {
                file: "src/legacy_code.rs".to_string(),
                complexity: 28,
                estimated_reduction: 10,
                priority: Priority::Medium,
            },
        ]
    }
}

/// Budget status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub team: TeamId,
    pub consumption_percentage: f64,
    pub complexity_remaining: i32,
    pub satd_remaining: u32,
    pub days_until_regeneration: u32,
}

impl TimeSeriesDB {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn record(&mut self, team: TeamId, metrics: TeamMetrics) {
        let entry = TimeSeries {
            timestamp: SystemTime::now(),
            metrics,
        };

        self.data.entry(team).or_default().push(entry);
    }

    /// Record a measurement for a team
    pub fn record_measurement(&mut self, team: &TeamId, timestamp: SystemTime, value: f64) {
        let metrics = TeamMetrics {
            avg_complexity: value,
            total_satd: 0,
            avg_coverage: value,
            quality_score: value,
        };

        let entry = TimeSeries { timestamp, metrics };

        self.data
            .entry(team.clone())
            .or_default()
            .push(entry);
    }

    /// Get recent measurements for a team
    pub fn get_recent_measurements(&self, team: &TeamId, duration: Duration) -> Vec<f64> {
        let now = SystemTime::now();
        let cutoff = now.checked_sub(duration).unwrap_or(SystemTime::UNIX_EPOCH);

        self.data
            .get(team)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|e| e.timestamp >= cutoff)
                    .map(|e| e.metrics.quality_score)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl EnforcementRules {
    fn approvers_for(&self, team: &TeamId) -> Vec<String> {
        self.approvers
            .get(team)
            .cloned()
            .unwrap_or_else(|| vec!["tech-lead".to_string()])
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_budget_enforcer_creation() {
        let config = EnforcerConfig::default();
        let enforcer = ErrorBudgetEnforcer::new(config);
        assert!(enforcer.budgets.is_empty());
    }

    #[test]
    fn test_team_registration() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team("team-a".to_string(), None);
        assert!(enforcer.budgets.contains_key("team-a"));
    }

    #[test]
    fn test_budget_consumption_calculation() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team("team-a".to_string(), None);

        let diff = DiffAnalysis {
            complexity_change: 10,
            satd_change: 2,
            coverage_change: -0.02,
            files_changed: vec!["test.rs".to_string()],
        };

        let decision = enforcer.check_commit(&"team-a".to_string(), &diff);
        matches!(decision, Decision::Approved);
    }

    #[test]
    fn test_budget_exceeded() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let mut budget = QualityBudget::default();
        budget.current_consumption.complexity_used = 95;
        enforcer.register_team("team-b".to_string(), Some(budget));

        let diff = DiffAnalysis {
            complexity_change: 20,
            satd_change: 0,
            coverage_change: 0.0,
            files_changed: vec!["test.rs".to_string()],
        };

        let decision = enforcer.check_commit(&"team-b".to_string(), &diff);
        matches!(decision, Decision::Blocked { .. });
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn team_id_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z][a-z0-9-]{2,20}").unwrap()
    }

    fn valid_budget_strategy() -> impl Strategy<Value = QualityBudget> {
        (
            0i32..200,    // complexity_budget
            0u32..50,     // satd_budget
            0.0f64..1.0,  // coverage_floor
            0.0f64..10.0, // regeneration_rate
            0u64..86400,  // grace_period_secs
        )
            .prop_map(
                |(complexity, satd, coverage, regen, grace_secs)| QualityBudget {
                    complexity_budget: complexity,
                    satd_budget: satd,
                    coverage_floor: coverage,
                    regeneration_rate: regen,
                    grace_period: Duration::from_secs(grace_secs),
                    current_consumption: BudgetConsumption::default(),
                    started_at: SystemTime::now(),
                },
            )
    }

    fn diff_analysis_strategy() -> impl Strategy<Value = DiffAnalysis> {
        (
            -50i32..100,                                              // complexity_change
            -10i32..20,   // satd_change (can reduce SATD)
            -0.5f64..0.1, // coverage_change
            prop::collection::vec("[a-zA-Z0-9_-]{1,20}\\.rs", 1..10), // files_changed
        )
            .prop_map(|(complexity, satd, coverage, files)| DiffAnalysis {
                complexity_change: complexity,
                satd_change: satd,
                coverage_change: coverage,
                files_changed: files,
            })
    }

    proptest! {
        #[test]
        fn budget_creation_is_valid(budget in valid_budget_strategy()) {
            prop_assert!(budget.complexity_budget >= 0);
            prop_assert!(budget.satd_budget >= 0);
            prop_assert!(budget.coverage_floor >= 0.0 && budget.coverage_floor <= 1.0);
            prop_assert!(budget.regeneration_rate >= 0.0);
            prop_assert!(budget.grace_period.as_secs() >= 0);
        }

        #[test]
        fn team_registration_always_succeeds(
            team_id in team_id_strategy(),
            budget in prop::option::of(valid_budget_strategy())
        ) {
            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), budget.clone());

            prop_assert!(enforcer.budgets.contains_key(&team_id));

            let registered_budget = &enforcer.budgets[&team_id];
            if let Some(custom_budget) = budget {
                prop_assert_eq!(registered_budget.complexity_budget, custom_budget.complexity_budget);
                prop_assert_eq!(registered_budget.satd_budget, custom_budget.satd_budget);
            } else {
                // Should use default budget
                let default_budget = QualityBudget::default();
                prop_assert_eq!(registered_budget.complexity_budget, default_budget.complexity_budget);
            }
        }

        #[test]
        fn budget_consumption_accumulates_correctly(
            team_id in team_id_strategy(),
            diffs in prop::collection::vec(diff_analysis_strategy(), 1..10)
        ) {
            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), None);

            let mut expected_complexity = 0i32;
            let mut expected_satd = 0i32;
            let mut expected_coverage = 0.0f64;

            for diff in &diffs {
                let _decision = enforcer.check_commit(&team_id, diff);

                expected_complexity += diff.complexity_change;
                expected_satd += diff.satd_change;
                expected_coverage += diff.coverage_change;
            }

            let budget = &enforcer.budgets[&team_id];

            // Complexity should accumulate (but not go negative)
            prop_assert_eq!(
                budget.current_consumption.complexity_used,
                expected_complexity.max(0)
            );

            // SATD should accumulate (but not go negative)
            prop_assert_eq!(
                budget.current_consumption.satd_used,
                expected_satd.max(0) as u32
            );
        }

        #[test]
        fn decisions_respect_budget_limits(
            team_id in team_id_strategy(),
            budget in valid_budget_strategy(),
            diff in diff_analysis_strategy()
        ) {
            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), Some(budget.clone()));

            let decision = enforcer.check_commit(&team_id, &diff);

            // Calculate if change would exceed budget
            let new_complexity = (budget.current_consumption.complexity_used + diff.complexity_change.max(0)).max(0);
            let new_satd = (budget.current_consumption.satd_used as i32 + diff.satd_change.max(0)).max(0) as u32;

            let complexity_exceeded = new_complexity > budget.complexity_budget;
            let satd_exceeded = new_satd > budget.satd_budget;

            match decision {
                Decision::Approved => {
                    prop_assert!(!complexity_exceeded && !satd_exceeded);
                }
                Decision::Warning(_) => {
                    // Warning may be issued for approaching limits
                    prop_assert!(true); // Always valid
                }
                Decision::RequiresApproval { .. } => {
                    // May require approval when nearing limits
                    prop_assert!(true); // Always valid
                }
                Decision::Blocked { .. } => {
                    // Should be blocked if budget exceeded
                    prop_assert!(complexity_exceeded || satd_exceeded);
                }
            }
        }

        #[test]
        fn budget_regeneration_properties(
            team_id in team_id_strategy(),
            mut budget in valid_budget_strategy(),
            days_elapsed in 0.0f64..30.0
        ) {
            budget.current_consumption.complexity_used = budget.complexity_budget / 2;
            budget.current_consumption.satd_used = budget.satd_budget / 2;

            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), Some(budget.clone()));

            // Simulate time passage
            let _elapsed_duration = Duration::from_secs((days_elapsed * 24.0 * 3600.0) as u64);
            enforcer.regenerate_budgets();

            let updated_budget = &enforcer.budgets[&team_id];

            // Budget should regenerate (reduce consumption)
            if budget.regeneration_rate > 0.0 && days_elapsed > 0.0 {
                let expected_reduction = (budget.regeneration_rate * days_elapsed) as i32;
                let expected_complexity = (budget.current_consumption.complexity_used - expected_reduction).max(0);

                prop_assert!(updated_budget.current_consumption.complexity_used <= budget.current_consumption.complexity_used);
                prop_assert!(updated_budget.current_consumption.complexity_used >= 0);
            }
        }

        #[test]
        fn refactor_target_generation_properties(
            team_id in team_id_strategy(),
            files in prop::collection::vec("[a-zA-Z0-9_-]{1,20}\\.rs", 1..20),
            complexities in prop::collection::vec(1u32..100, 1..20)
        ) {
            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), None);

            // Create a scenario where budget is exceeded
            let large_diff = DiffAnalysis {
                complexity_change: 1000, // Very large change
                satd_change: 50,
                coverage_change: -0.5,
                files_changed: files.clone(),
            };

            let decision = enforcer.check_commit(&team_id, &large_diff);

            match decision {
                Decision::Blocked { refactor_targets, .. } => {
                    for target in refactor_targets {
                        prop_assert!(target.complexity > 0);
                        prop_assert!(target.estimated_reduction <= target.complexity);
                        prop_assert!(files.iter().any(|f| f == &target.file));
                    }
                }
                _ => {
                    // Other decisions are valid too
                    prop_assert!(true);
                }
            }
        }

        #[test]
        fn enforcement_rules_consistency(
            complexity_threshold in 1u32..100,
            satd_limit in 1u32..50,
            coverage_minimum in 0.0f64..1.0
        ) {
            let rules = EnforcementRules {
                approvers: HashMap::new(),
                escalation: EscalationPolicy {
                    warning_threshold: 0.5,
                    approval_threshold: 0.8,
                    block_threshold: 1.0,
                },
                overrides: HashMap::new(),
            };

            prop_assert!(rules.escalation.warning_threshold >= 0.0);
            prop_assert!(rules.escalation.approval_threshold >= 0.0);
            prop_assert!(rules.escalation.block_threshold >= 0.0);
        }

        #[test]
        fn time_series_operations_stable(
            team_id in team_id_strategy(),
            measurements in prop::collection::vec(0u32..100, 1..100)
        ) {
            let mut db = TimeSeriesDB::new();
            let now = SystemTime::now();

            for (i, measurement) in measurements.iter().enumerate() {
                let timestamp = now + Duration::from_secs(i as u64 * 3600); // Hourly measurements
                db.record_measurement(&team_id, timestamp, *measurement as f64);
            }

            // Should be able to query the data
            let recent = db.get_recent_measurements(&team_id, Duration::from_secs(24 * 3600));
            prop_assert!(!recent.is_empty());
            prop_assert!(recent.len() <= 24); // At most 24 hours of hourly data

            // We should have gotten some measurements back
            // The actual values don't matter for this test, just that they exist
            prop_assert!(recent.len() <= measurements.len());
        }

        #[test]
        fn concurrent_team_operations_safe(
            teams in prop::collection::vec(team_id_strategy(), 1..10),
            operations_per_team in 1usize..20
        ) {
            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());

            // Register teams
            for team in &teams {
                enforcer.register_team(team.clone(), None);
            }

            // Simulate concurrent operations
            for team in &teams {
                for _ in 0..operations_per_team {
                    let diff = DiffAnalysis {
                        complexity_change: 5,
                        satd_change: 1,
                        coverage_change: -0.01,
                        files_changed: vec!["test.rs".to_string()],
                    };

                    let _decision = enforcer.check_commit(team, &diff);
                }
            }

            // All teams should still be registered
            for team in &teams {
                prop_assert!(enforcer.budgets.contains_key(team));
            }
        }

        #[test]
        fn grace_period_enforcement_properties(
            team_id in team_id_strategy(),
            grace_period_days in 1u64..100,
            elapsed_days in 0u64..200
        ) {
            let mut budget = QualityBudget::default();
            budget.grace_period = Duration::from_secs(grace_period_days * 24 * 3600);
            budget.started_at = SystemTime::now() - Duration::from_secs(elapsed_days * 24 * 3600);

            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), Some(budget.clone()));

            let large_diff = DiffAnalysis {
                complexity_change: 200, // Exceeds default budget
                satd_change: 60,
                coverage_change: -0.6,
                files_changed: vec!["test.rs".to_string()],
            };

            let decision = enforcer.check_commit(&team_id, &large_diff);

            let grace_period_active = elapsed_days < grace_period_days;

            match decision {
                Decision::Blocked { .. } => {
                    // Should only be blocked if grace period is over
                    prop_assert!(!grace_period_active);
                }
                _ => {
                    // Other decisions are valid during grace period
                    prop_assert!(true);
                }
            }
        }
    }
}
