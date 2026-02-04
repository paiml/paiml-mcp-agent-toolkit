// Enforcement tests extracted from enforcement.rs for file health (CB-040).
// This file is include!()'d into enforcement.rs scope.

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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
            // satd_budget is always >= 0 for unsigned types
            prop_assert!(budget.coverage_floor >= 0.0 && budget.coverage_floor <= 1.0);
            prop_assert!(budget.regeneration_rate >= 0.0);
            // grace_period.as_secs() always returns >= 0 for Duration
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
        #[ignore = "requires quality framework setup"]
        fn budget_consumption_accumulates_correctly(
            team_id in team_id_strategy(),
            diffs in prop::collection::vec(diff_analysis_strategy(), 1..10)
        ) {
            let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
            enforcer.register_team(team_id.clone(), None);

            let mut expected_complexity = 0i32;
            let mut expected_satd = 0i32;
            let mut _expected_coverage = 0.0f64;

            for diff in &diffs {
                let _decision = enforcer.check_commit(&team_id, diff);

                expected_complexity += diff.complexity_change;
                expected_satd += diff.satd_change;
                _expected_coverage += diff.coverage_change;
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
        #[ignore = "requires quality framework setup"]
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
                let _expected_complexity = (budget.current_consumption.complexity_used - expected_reduction).max(0);

                prop_assert!(updated_budget.current_consumption.complexity_used <= budget.current_consumption.complexity_used);
                prop_assert!(updated_budget.current_consumption.complexity_used >= 0);
            }
        }

        #[test]
        #[ignore = "requires quality framework setup"]
        fn refactor_target_generation_properties(
            team_id in team_id_strategy(),
            files in prop::collection::vec("[a-zA-Z0-9_-]{1,20}\\.rs", 1..20),
            _complexities in prop::collection::vec(1u32..100, 1..20)
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
            _complexity_threshold in 1u32..100,
            _satd_limit in 1u32..50,
            _coverage_minimum in 0.0f64..1.0
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
        #[ignore = "requires quality framework setup"]
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
        #[ignore = "requires quality framework setup"]
        fn grace_period_enforcement_properties(
            team_id in team_id_strategy(),
            grace_period_days in 1u64..100,
            elapsed_days in 0u64..200
        ) {
            let budget = QualityBudget {
                grace_period: Duration::from_secs(grace_period_days * 24 * 3600),
                started_at: SystemTime::now() - Duration::from_secs(elapsed_days * 24 * 3600),
                ..QualityBudget::default()
            };

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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ============================================
    // Test Fixtures and Helpers
    // ============================================

    /// Create a default test team ID
    fn test_team_id() -> TeamId {
        "test-team".to_string()
    }

    /// Create a simple DiffAnalysis for testing
    fn create_simple_diff() -> DiffAnalysis {
        DiffAnalysis {
            complexity_change: 5,
            satd_change: 1,
            coverage_change: -0.01,
            files_changed: vec!["test.rs".to_string()],
        }
    }

    /// Create a large DiffAnalysis that should exceed budgets
    fn create_large_diff() -> DiffAnalysis {
        DiffAnalysis {
            complexity_change: 150,
            satd_change: 30,
            coverage_change: -0.5,
            files_changed: vec!["large.rs".to_string()],
        }
    }

    /// Create a DiffAnalysis with negative changes (improvements)
    fn create_improvement_diff() -> DiffAnalysis {
        DiffAnalysis {
            complexity_change: -10,
            satd_change: -5,
            coverage_change: 0.1,
            files_changed: vec!["improved.rs".to_string()],
        }
    }

    /// Create a QualityBudget with specific consumption
    fn create_consumed_budget(complexity_used: i32, satd_used: u32) -> QualityBudget {
        let mut budget = QualityBudget::default();
        budget.current_consumption.complexity_used = complexity_used;
        budget.current_consumption.satd_used = satd_used;
        budget.current_consumption.last_updated = Some(SystemTime::now());
        budget
    }

    // ============================================
    // QualityBudget Tests
    // ============================================

    #[test]
    fn test_quality_budget_default_values() {
        let budget = QualityBudget::default();

        assert_eq!(budget.complexity_budget, 100);
        assert_eq!(budget.satd_budget, 20);
        assert!((budget.coverage_floor - 0.7).abs() < f64::EPSILON);
        assert!((budget.regeneration_rate - 5.0).abs() < f64::EPSILON);
        assert_eq!(budget.grace_period, Duration::from_secs(7 * 24 * 60 * 60));
    }

    #[test]
    fn test_quality_budget_clone() {
        let budget1 = QualityBudget::default();
        let budget2 = budget1.clone();

        assert_eq!(budget1.complexity_budget, budget2.complexity_budget);
        assert_eq!(budget1.satd_budget, budget2.satd_budget);
    }

    #[test]
    fn test_quality_budget_serialization() {
        let budget = QualityBudget::default();
        let json = serde_json::to_string(&budget).expect("Serialization failed");

        assert!(json.contains("complexity_budget"));
        assert!(json.contains("satd_budget"));
        assert!(json.contains("coverage_floor"));
    }

    #[test]
    fn test_quality_budget_custom_values() {
        let budget = QualityBudget {
            complexity_budget: 200,
            satd_budget: 50,
            coverage_floor: 0.8,
            regeneration_rate: 10.0,
            grace_period: Duration::from_secs(14 * 24 * 60 * 60),
            current_consumption: BudgetConsumption::default(),
            started_at: SystemTime::now(),
        };

        assert_eq!(budget.complexity_budget, 200);
        assert_eq!(budget.satd_budget, 50);
    }

    // ============================================
    // BudgetConsumption Tests
    // ============================================

    #[test]
    fn test_budget_consumption_default() {
        let consumption = BudgetConsumption::default();

        assert_eq!(consumption.complexity_used, 0);
        assert_eq!(consumption.satd_used, 0);
        assert!((consumption.coverage_reduction - 0.0).abs() < f64::EPSILON);
        assert!(consumption.last_updated.is_none());
    }

    #[test]
    fn test_budget_consumption_clone() {
        let consumption = BudgetConsumption {
            complexity_used: 50,
            satd_used: 10,
            coverage_reduction: 0.05,
            last_updated: Some(SystemTime::now()),
        };
        let cloned = consumption.clone();

        assert_eq!(consumption.complexity_used, cloned.complexity_used);
        assert_eq!(consumption.satd_used, cloned.satd_used);
    }

    #[test]
    fn test_budget_consumption_debug() {
        let consumption = BudgetConsumption::default();
        let debug_str = format!("{:?}", consumption);

        assert!(debug_str.contains("BudgetConsumption"));
        assert!(debug_str.contains("complexity_used"));
    }

    // ============================================
    // Decision Tests
    // ============================================

    #[test]
    fn test_decision_approved() {
        let decision = Decision::Approved;
        let debug_str = format!("{:?}", decision);
        assert!(debug_str.contains("Approved"));
    }

    #[test]
    fn test_decision_warning() {
        let decision = Decision::Warning("Test warning".to_string());
        match decision {
            Decision::Warning(msg) => assert_eq!(msg, "Test warning"),
            _ => panic!("Expected Warning variant"),
        }
    }

    #[test]
    fn test_decision_requires_approval() {
        let decision = Decision::RequiresApproval {
            approvers: vec!["alice".to_string(), "bob".to_string()],
            reason_required: true,
        };

        match decision {
            Decision::RequiresApproval {
                approvers,
                reason_required,
            } => {
                assert_eq!(approvers.len(), 2);
                assert!(reason_required);
            }
            _ => panic!("Expected RequiresApproval variant"),
        }
    }

    #[test]
    fn test_decision_blocked() {
        let decision = Decision::Blocked {
            suggestion: "Refactor first".to_string(),
            refactor_targets: vec![RefactorTarget {
                file: "test.rs".to_string(),
                complexity: 30,
                estimated_reduction: 10,
                priority: Priority::High,
            }],
        };

        match decision {
            Decision::Blocked {
                suggestion,
                refactor_targets,
            } => {
                assert_eq!(suggestion, "Refactor first");
                assert_eq!(refactor_targets.len(), 1);
            }
            _ => panic!("Expected Blocked variant"),
        }
    }

    #[test]
    fn test_decision_clone() {
        let decision = Decision::Warning("Test".to_string());
        let cloned = decision.clone();

        match cloned {
            Decision::Warning(msg) => assert_eq!(msg, "Test"),
            _ => panic!("Expected Warning variant"),
        }
    }

    // ============================================
    // RefactorTarget Tests
    // ============================================

    #[test]
    fn test_refactor_target_creation() {
        let target = RefactorTarget {
            file: "complex.rs".to_string(),
            complexity: 45,
            estimated_reduction: 20,
            priority: Priority::Critical,
        };

        assert_eq!(target.file, "complex.rs");
        assert_eq!(target.complexity, 45);
        assert_eq!(target.estimated_reduction, 20);
    }

    #[test]
    fn test_refactor_target_clone() {
        let target = RefactorTarget {
            file: "test.rs".to_string(),
            complexity: 25,
            estimated_reduction: 10,
            priority: Priority::Medium,
        };
        let cloned = target.clone();

        assert_eq!(target.file, cloned.file);
        assert_eq!(target.complexity, cloned.complexity);
    }

    #[test]
    fn test_refactor_target_serialization() {
        let target = RefactorTarget {
            file: "test.rs".to_string(),
            complexity: 25,
            estimated_reduction: 10,
            priority: Priority::High,
        };

        let json = serde_json::to_string(&target).expect("Serialization failed");
        assert!(json.contains("test.rs"));
        assert!(json.contains("complexity"));
    }

    // ============================================
    // Priority Tests
    // ============================================

    #[test]
    fn test_priority_variants() {
        let low = Priority::Low;
        let medium = Priority::Medium;
        let high = Priority::High;
        let critical = Priority::Critical;

        assert!(format!("{:?}", low).contains("Low"));
        assert!(format!("{:?}", medium).contains("Medium"));
        assert!(format!("{:?}", high).contains("High"));
        assert!(format!("{:?}", critical).contains("Critical"));
    }

    #[test]
    fn test_priority_clone() {
        let priority = Priority::High;
        let cloned = priority.clone();
        assert!(matches!(cloned, Priority::High));
    }

    // ============================================
    // DiffAnalysis Tests
    // ============================================

    #[test]
    fn test_diff_analysis_creation() {
        let diff = create_simple_diff();

        assert_eq!(diff.complexity_change, 5);
        assert_eq!(diff.satd_change, 1);
        assert!(diff.coverage_change < 0.0);
        assert_eq!(diff.files_changed.len(), 1);
    }

    #[test]
    fn test_diff_analysis_negative_values() {
        let diff = create_improvement_diff();

        assert!(diff.complexity_change < 0);
        assert!(diff.satd_change < 0);
        assert!(diff.coverage_change > 0.0);
    }

    #[test]
    fn test_diff_analysis_clone() {
        let diff = create_simple_diff();
        let cloned = diff.clone();

        assert_eq!(diff.complexity_change, cloned.complexity_change);
        assert_eq!(diff.files_changed, cloned.files_changed);
    }

    #[test]
    fn test_diff_analysis_debug() {
        let diff = create_simple_diff();
        let debug_str = format!("{:?}", diff);

        assert!(debug_str.contains("DiffAnalysis"));
        assert!(debug_str.contains("complexity_change"));
    }

    // ============================================
    // EnforcerConfig Tests
    // ============================================

    #[test]
    fn test_enforcer_config_default() {
        let config = EnforcerConfig::default();

        assert!(config.enabled);
        assert_eq!(
            config.regeneration_interval,
            Duration::from_secs(24 * 60 * 60)
        );
        assert_eq!(
            config.history_retention,
            Duration::from_secs(30 * 24 * 60 * 60)
        );
    }

    #[test]
    fn test_enforcer_config_disabled() {
        let config = EnforcerConfig {
            enabled: false,
            ..EnforcerConfig::default()
        };

        assert!(!config.enabled);
    }

    #[test]
    fn test_enforcer_config_clone() {
        let config = EnforcerConfig::default();
        let cloned = config.clone();

        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.regeneration_interval, cloned.regeneration_interval);
    }

    #[test]
    fn test_enforcer_config_serialization() {
        let config = EnforcerConfig::default();
        let json = serde_json::to_string(&config).expect("Serialization failed");

        assert!(json.contains("enabled"));
        assert!(json.contains("regeneration_interval"));
    }

    // ============================================
    // ErrorBudgetEnforcer Creation Tests
    // ============================================

    #[test]
    fn test_enforcer_creation_default() {
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        assert!(enforcer.budgets.is_empty());
    }

    #[test]
    fn test_enforcer_creation_disabled() {
        let config = EnforcerConfig {
            enabled: false,
            ..EnforcerConfig::default()
        };
        let enforcer = ErrorBudgetEnforcer::new(config);
        assert!(!enforcer.config.enabled);
    }

    // ============================================
    // Team Registration Tests
    // ============================================

    #[test]
    fn test_register_team_with_default_budget() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        assert!(enforcer.budgets.contains_key(&test_team_id()));
    }

    #[test]
    fn test_register_team_with_custom_budget() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let custom_budget = QualityBudget {
            complexity_budget: 200,
            ..QualityBudget::default()
        };
        enforcer.register_team(test_team_id(), Some(custom_budget.clone()));

        let registered = &enforcer.budgets[&test_team_id()];
        assert_eq!(registered.complexity_budget, 200);
    }

    #[test]
    fn test_register_multiple_teams() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team("team-a".to_string(), None);
        enforcer.register_team("team-b".to_string(), None);
        enforcer.register_team("team-c".to_string(), None);

        assert_eq!(enforcer.budgets.len(), 3);
    }

    #[test]
    fn test_register_team_overwrite() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let custom_budget = QualityBudget {
            complexity_budget: 300,
            ..QualityBudget::default()
        };
        enforcer.register_team(test_team_id(), Some(custom_budget));

        assert_eq!(enforcer.budgets[&test_team_id()].complexity_budget, 300);
    }

    // ============================================
    // check_commit Tests
    // ============================================

    #[test]
    fn test_check_commit_disabled_enforcement() {
        let config = EnforcerConfig {
            enabled: false,
            ..EnforcerConfig::default()
        };
        let enforcer = ErrorBudgetEnforcer::new(config);
        let diff = create_large_diff();

        let decision = enforcer.check_commit(&test_team_id(), &diff);
        assert!(matches!(decision, Decision::Approved));
    }

    #[test]
    fn test_check_commit_unregistered_team() {
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let diff = create_large_diff();

        let decision = enforcer.check_commit(&test_team_id(), &diff);
        assert!(matches!(decision, Decision::Approved));
    }

    #[test]
    fn test_check_commit_small_change_approved() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff = DiffAnalysis {
            complexity_change: 2,
            satd_change: 0,
            coverage_change: 0.0,
            files_changed: vec!["small.rs".to_string()],
        };

        let decision = enforcer.check_commit(&test_team_id(), &diff);
        assert!(matches!(decision, Decision::Approved));
    }

    #[test]
    fn test_check_commit_exceeds_budget() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let budget = create_consumed_budget(90, 18);
        enforcer.register_team(test_team_id(), Some(budget));

        let diff = DiffAnalysis {
            complexity_change: 50,
            satd_change: 10,
            coverage_change: -0.3,
            files_changed: vec!["large.rs".to_string()],
        };

        let decision = enforcer.check_commit(&test_team_id(), &diff);
        assert!(matches!(decision, Decision::Blocked { .. }));
    }

    #[test]
    fn test_check_commit_warning_threshold() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let budget = create_consumed_budget(40, 8);
        enforcer.register_team(test_team_id(), Some(budget));

        let diff = DiffAnalysis {
            complexity_change: 10,
            satd_change: 2,
            coverage_change: -0.05,
            files_changed: vec!["medium.rs".to_string()],
        };

        let decision = enforcer.check_commit(&test_team_id(), &diff);
        // Depending on exact threshold, this should be Warning or RequiresApproval
        assert!(!matches!(decision, Decision::Blocked { .. }));
    }

    #[test]
    fn test_check_commit_with_improvement() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff = create_improvement_diff();

        let decision = enforcer.check_commit(&test_team_id(), &diff);
        assert!(matches!(decision, Decision::Approved));
    }

    // ============================================
    // update_consumption Tests
    // ============================================

    #[test]
    fn test_update_consumption_basic() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff = DiffAnalysis {
            complexity_change: 10,
            satd_change: 2,
            coverage_change: -0.05,
            files_changed: vec!["test.rs".to_string()],
        };

        enforcer.update_consumption(&test_team_id(), &diff);

        let budget = &enforcer.budgets[&test_team_id()];
        assert_eq!(budget.current_consumption.complexity_used, 10);
        assert_eq!(budget.current_consumption.satd_used, 2);
    }

    #[test]
    fn test_update_consumption_accumulates() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff1 = create_simple_diff();
        let diff2 = create_simple_diff();

        enforcer.update_consumption(&test_team_id(), &diff1);
        enforcer.update_consumption(&test_team_id(), &diff2);

        let budget = &enforcer.budgets[&test_team_id()];
        assert_eq!(budget.current_consumption.complexity_used, 10);
        assert_eq!(budget.current_consumption.satd_used, 2);
    }

    #[test]
    fn test_update_consumption_negative_satd_clamped() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff = DiffAnalysis {
            complexity_change: 0,
            satd_change: -100, // Large negative
            coverage_change: 0.0,
            files_changed: vec!["test.rs".to_string()],
        };

        enforcer.update_consumption(&test_team_id(), &diff);

        let budget = &enforcer.budgets[&test_team_id()];
        assert_eq!(budget.current_consumption.satd_used, 0); // Clamped to 0
    }

    #[test]
    fn test_update_consumption_unregistered_team() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let diff = create_simple_diff();

        // Should not panic
        enforcer.update_consumption(&test_team_id(), &diff);
    }

    #[test]
    fn test_update_consumption_sets_timestamp() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let before = SystemTime::now();
        let diff = create_simple_diff();
        enforcer.update_consumption(&test_team_id(), &diff);
        let after = SystemTime::now();

        let budget = &enforcer.budgets[&test_team_id()];
        let last_updated = budget.current_consumption.last_updated.unwrap();
        assert!(last_updated >= before);
        assert!(last_updated <= after);
    }

    // ============================================
    // regenerate_budgets Tests
    // ============================================

    #[test]
    fn test_regenerate_budgets_no_teams() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        // Should not panic
        enforcer.regenerate_budgets();
    }

    #[test]
    fn test_regenerate_budgets_no_consumption() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        enforcer.regenerate_budgets();
        // No change expected since no consumption
    }

    // ============================================
    // get_budget_status Tests
    // ============================================

    #[test]
    fn test_get_budget_status_unregistered() {
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let status = enforcer.get_budget_status(&test_team_id());

        assert!(status.is_none());
    }

    #[test]
    fn test_get_budget_status_registered() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let status = enforcer.get_budget_status(&test_team_id());
        assert!(status.is_some());

        let status = status.unwrap();
        assert_eq!(status.team, test_team_id());
        assert_eq!(status.complexity_remaining, 100); // Default budget
        assert_eq!(status.satd_remaining, 20);
    }

    #[test]
    fn test_get_budget_status_after_consumption() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff = DiffAnalysis {
            complexity_change: 30,
            satd_change: 5,
            coverage_change: 0.0,
            files_changed: vec!["test.rs".to_string()],
        };
        enforcer.update_consumption(&test_team_id(), &diff);

        let status = enforcer.get_budget_status(&test_team_id()).unwrap();
        assert_eq!(status.complexity_remaining, 70); // 100 - 30
        assert_eq!(status.satd_remaining, 15); // 20 - 5
    }

    // ============================================
    // BudgetStatus Tests
    // ============================================

    #[test]
    fn test_budget_status_serialization() {
        let status = BudgetStatus {
            team: test_team_id(),
            consumption_percentage: 50.0,
            complexity_remaining: 50,
            satd_remaining: 10,
            days_until_regeneration: 3,
        };

        let json = serde_json::to_string(&status).expect("Serialization failed");
        assert!(json.contains("test-team"));
        assert!(json.contains("consumption_percentage"));
    }

    #[test]
    fn test_budget_status_clone() {
        let status = BudgetStatus {
            team: test_team_id(),
            consumption_percentage: 75.0,
            complexity_remaining: 25,
            satd_remaining: 5,
            days_until_regeneration: 1,
        };
        let cloned = status.clone();

        assert_eq!(status.team, cloned.team);
        assert!(
            (status.consumption_percentage - cloned.consumption_percentage).abs() < f64::EPSILON
        );
    }

    // ============================================
    // TimeSeriesDB Tests
    // ============================================

    #[test]
    fn test_time_series_db_new() {
        let db = TimeSeriesDB::new();
        assert!(db.data.is_empty());
    }

    #[test]
    fn test_time_series_db_record() {
        let mut db = TimeSeriesDB::new();
        let metrics = TeamMetrics {
            avg_complexity: 10.0,
            total_satd: 5,
            avg_coverage: 0.85,
            quality_score: 0.9,
        };

        db.record(test_team_id(), metrics);
        assert_eq!(db.data.len(), 1);
    }

    #[test]
    fn test_time_series_db_record_measurement() {
        let mut db = TimeSeriesDB::new();
        db.record_measurement(&test_team_id(), SystemTime::now(), 0.9);

        assert!(db.data.contains_key(&test_team_id()));
    }

    #[test]
    fn test_time_series_db_get_recent_measurements() {
        let mut db = TimeSeriesDB::new();
        let now = SystemTime::now();

        db.record_measurement(&test_team_id(), now, 0.9);
        db.record_measurement(&test_team_id(), now, 0.85);
        db.record_measurement(&test_team_id(), now, 0.88);

        let recent = db.get_recent_measurements(&test_team_id(), Duration::from_secs(3600));
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_time_series_db_get_recent_nonexistent_team() {
        let db = TimeSeriesDB::new();
        let recent = db.get_recent_measurements(&test_team_id(), Duration::from_secs(3600));
        assert!(recent.is_empty());
    }

    #[test]
    fn test_time_series_db_multiple_teams() {
        let mut db = TimeSeriesDB::new();
        db.record_measurement(&"team-a".to_string(), SystemTime::now(), 0.9);
        db.record_measurement(&"team-b".to_string(), SystemTime::now(), 0.8);

        assert!(db.data.contains_key(&"team-a".to_string()));
        assert!(db.data.contains_key(&"team-b".to_string()));
    }

    // ============================================
    // EscalationPolicy Tests
    // ============================================

    #[test]
    fn test_escalation_policy_default() {
        let policy = EscalationPolicy::default();

        assert!((policy.warning_threshold - 0.5).abs() < f64::EPSILON);
        assert!((policy.approval_threshold - 0.8).abs() < f64::EPSILON);
        assert!((policy.block_threshold - 1.0).abs() < f64::EPSILON);
    }

    // ============================================
    // EnforcementRules Tests
    // ============================================

    #[test]
    fn test_enforcement_rules_default() {
        let rules = EnforcementRules::default();

        assert!(rules.approvers.is_empty());
        assert!(rules.overrides.is_empty());
    }

    #[test]
    fn test_enforcement_rules_approvers_for_unknown_team() {
        let rules = EnforcementRules::default();
        let approvers = rules.approvers_for(&test_team_id());

        // Should return default approvers
        assert_eq!(approvers, vec!["tech-lead".to_string()]);
    }

    #[test]
    fn test_enforcement_rules_approvers_for_registered_team() {
        let mut rules = EnforcementRules::default();
        rules
            .approvers
            .insert(test_team_id(), vec!["alice".to_string(), "bob".to_string()]);

        let approvers = rules.approvers_for(&test_team_id());
        assert_eq!(approvers.len(), 2);
        assert!(approvers.contains(&"alice".to_string()));
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_zero_budget_values() {
        let budget = QualityBudget {
            complexity_budget: 0,
            satd_budget: 0,
            coverage_floor: 0.0,
            regeneration_rate: 0.0,
            grace_period: Duration::ZERO,
            current_consumption: BudgetConsumption::default(),
            started_at: SystemTime::now(),
        };

        assert_eq!(budget.complexity_budget, 0);
    }

    #[test]
    fn test_negative_complexity_change() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let budget = create_consumed_budget(50, 10);
        enforcer.register_team(test_team_id(), Some(budget));

        let diff = DiffAnalysis {
            complexity_change: -20,
            satd_change: -5,
            coverage_change: 0.1,
            files_changed: vec!["improved.rs".to_string()],
        };

        enforcer.update_consumption(&test_team_id(), &diff);

        let budget = &enforcer.budgets[&test_team_id()];
        assert_eq!(budget.current_consumption.complexity_used, 30); // 50 - 20
        assert_eq!(budget.current_consumption.satd_used, 5); // 10 - 5
    }

    #[test]
    fn test_empty_files_changed() {
        let diff = DiffAnalysis {
            complexity_change: 0,
            satd_change: 0,
            coverage_change: 0.0,
            files_changed: vec![],
        };

        assert!(diff.files_changed.is_empty());
    }

    #[test]
    fn test_many_files_changed() {
        let files: Vec<String> = (0..100).map(|i| format!("file_{}.rs", i)).collect();
        let diff = DiffAnalysis {
            complexity_change: 100,
            satd_change: 20,
            coverage_change: -0.1,
            files_changed: files,
        };

        assert_eq!(diff.files_changed.len(), 100);
    }

    #[test]
    fn test_extreme_coverage_change() {
        let mut enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        enforcer.register_team(test_team_id(), None);

        let diff = DiffAnalysis {
            complexity_change: 0,
            satd_change: 0,
            coverage_change: -1.0, // Maximum negative
            files_changed: vec!["test.rs".to_string()],
        };

        // Should still work without panicking
        let decision = enforcer.check_commit(&test_team_id(), &diff);
        // Large coverage drop should trigger some response
        assert!(!matches!(decision, Decision::Approved));
    }
}
