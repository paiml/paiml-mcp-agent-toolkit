//! EXTREME TDD Tests for PMAT Oracle
//!
//! RED-GREEN-REFACTOR methodology following Toyota Way principles.

use super::*;
use std::time::Duration;

mod defect_category_tests {
    use super::*;

    #[test]
    fn test_from_rustc_error_type_errors() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0308"),
            Some(DefectCategory::TypeErrors)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0412"),
            Some(DefectCategory::TypeErrors)
        );
    }

    #[test]
    fn test_from_rustc_error_ownership_borrow() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0382"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0502"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0503"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0505"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0499"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0597"),
            Some(DefectCategory::OwnershipBorrow)
        );
    }

    #[test]
    fn test_from_rustc_error_memory_safety() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0507"),
            Some(DefectCategory::MemorySafety)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0133"),
            Some(DefectCategory::MemorySafety)
        );
    }

    #[test]
    fn test_from_rustc_error_trait_bounds() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0277"),
            Some(DefectCategory::TraitBounds)
        );
    }

    #[test]
    fn test_from_rustc_error_unknown() {
        assert_eq!(DefectCategory::from_rustc_error("E9999"), None);
        assert_eq!(DefectCategory::from_rustc_error("unknown"), None);
    }

    #[test]
    fn test_rustc_confidence_high_confidence() {
        assert!(DefectCategory::TypeErrors.rustc_confidence() >= 0.9);
        assert!(DefectCategory::TraitBounds.rustc_confidence() >= 0.9);
        assert!(DefectCategory::OwnershipBorrow.rustc_confidence() >= 0.9);
    }

    #[test]
    fn test_rustc_confidence_medium_confidence() {
        assert!(DefectCategory::StdlibMapping.rustc_confidence() >= 0.8);
        assert!(DefectCategory::ASTTransform.rustc_confidence() >= 0.8);
    }

    #[test]
    fn test_rustc_confidence_lower_confidence() {
        assert!(DefectCategory::Configuration.rustc_confidence() >= 0.7);
        assert!(DefectCategory::Configuration.rustc_confidence() < 0.8);
    }
}

mod severity_tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }
}

mod signal_source_tests {
    use super::*;

    #[test]
    fn test_signal_source_variants() {
        // Ensure all signal sources are defined
        let sources = vec![
            SignalSource::Rustc,
            SignalSource::Clippy,
            SignalSource::CargoTest,
            SignalSource::CargoBuild,
            SignalSource::LlvmCov,
            SignalSource::CargoMutants,
            SignalSource::PmatTdg,
            SignalSource::PmatComplexity,
            SignalSource::PmatSatd,
            SignalSource::PmatDeadCode,
            SignalSource::PmatRustProjectScore,
            SignalSource::PmatFiveWhys,
            SignalSource::PmatChurn,
        ];
        assert_eq!(sources.len(), 13);
    }
}

mod defect_report_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_defect_report() {
        let location = CodeLocation {
            file_path: PathBuf::from("src/main.rs"),
            line: 42,
            column: Some(10),
            span_end_line: None,
        };

        let report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        assert!(!report.id.is_empty());
        assert_eq!(report.category, DefectCategory::TypeErrors);
        assert_eq!(report.severity, Severity::High);
        assert_eq!(report.confidence, 0.0);
        assert!(report.signals.is_empty());
        assert!(report.suggested_fixes.is_empty());
        assert_eq!(report.decision, OracleDecision::Skip);
    }

    #[test]
    fn test_add_signal_updates_confidence() {
        let location = CodeLocation {
            file_path: PathBuf::from("src/main.rs"),
            line: 42,
            column: None,
            span_end_line: None,
        };

        let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "mismatched types".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        });

        assert!(report.confidence > 0.0);
    }

    #[test]
    fn test_update_decision_auto_apply() {
        let location = CodeLocation {
            file_path: PathBuf::from("src/main.rs"),
            line: 42,
            column: None,
            span_end_line: None,
        };

        let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        // Add high-weight signal
        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        });

        report.update_decision(0.9, 0.7);

        // High confidence TypeErrors should be auto-apply
        assert_eq!(report.decision, OracleDecision::AutoApply);
    }

    #[test]
    fn test_update_decision_skip() {
        let location = CodeLocation {
            file_path: PathBuf::from("src/main.rs"),
            line: 42,
            column: None,
            span_end_line: None,
        };

        let mut report = DefectReport::new(DefectCategory::Configuration, Severity::Low, location);

        // Low confidence category with low weight
        report.add_signal(SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.3,
        });

        report.update_decision(0.9, 0.7);

        assert_eq!(report.decision, OracleDecision::Skip);
    }
}

mod convergence_targets_tests {
    use super::*;

    #[test]
    fn test_default_targets() {
        let targets = ConvergenceTargets::default();

        assert_eq!(targets.test_coverage, 0.95);
        assert_eq!(targets.mutation_score, 0.85);
        assert_eq!(targets.max_compiler_errors, 0);
        assert_eq!(targets.max_clippy_warnings, 0);
        assert_eq!(targets.max_test_failures, 0);
        assert_eq!(targets.min_tdg_score, 95.0);
        assert_eq!(targets.min_rust_project_score, 90);
        assert_eq!(targets.max_satd_markers, 0);
        assert_eq!(targets.max_dead_code, 0);
        assert_eq!(targets.max_cyclomatic_complexity, 15);
        assert_eq!(targets.max_cognitive_complexity, 25);
        assert_eq!(targets.max_build_time, Duration::from_secs(60));
    }

    #[test]
    fn test_check_converged() {
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.96,
            mutation_score: 0.86,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 92,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        let status = targets.check(&metrics);
        assert!(matches!(status, ConvergenceStatus::Converged));
    }

    #[test]
    fn test_check_not_converged_coverage() {
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.80, // Below 0.95 target
            ..Default::default()
        };

        let status = targets.check(&metrics);
        if let ConvergenceStatus::NotConverged { remaining } = status {
            assert!(remaining.iter().any(|r| r.contains("Coverage")));
        } else {
            panic!("Expected NotConverged");
        }
    }

    #[test]
    fn test_check_not_converged_compiler_errors() {
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            compiler_errors: 5, // Above 0 target
            ..Default::default()
        };

        let status = targets.check(&metrics);
        if let ConvergenceStatus::NotConverged { remaining } = status {
            assert!(remaining.iter().any(|r| r.contains("Compiler errors")));
        } else {
            panic!("Expected NotConverged");
        }
    }

    #[test]
    fn test_check_multiple_failures() {
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.50,
            compiler_errors: 10,
            clippy_warnings: 20,
            test_failures: 5,
            ..Default::default()
        };

        let status = targets.check(&metrics);
        if let ConvergenceStatus::NotConverged { remaining } = status {
            assert!(remaining.len() >= 4);
        } else {
            panic!("Expected NotConverged");
        }
    }
}

mod oracle_config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OracleConfig::default();

        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.min_progress_per_iteration, 0.001);
        assert_eq!(config.stagnation_threshold, 5);
        assert!(config.andon_enabled);
        assert_eq!(config.require_human_approval_above, Some(10));
        assert_eq!(config.auto_apply_threshold, 0.9);
        assert_eq!(config.review_threshold, 0.7);
        assert_eq!(config.batch_size, 10);
    }
}

mod convergence_tracker_tests {
    use super::*;

    #[test]
    fn test_new_tracker() {
        let tracker = ConvergenceTracker::new();

        assert_eq!(tracker.iterations, 0);
        assert!(tracker.history.is_empty());
        assert!(tracker.best_metrics.is_none());
        assert!(tracker.current_status.is_none());
    }

    #[test]
    fn test_record_iteration() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.80,
            compiler_errors: 5,
            ..Default::default()
        };

        tracker.record(metrics.clone(), 5, &targets);

        assert_eq!(tracker.iterations, 1);
        assert_eq!(tracker.history.len(), 1);
        assert!(tracker.best_metrics.is_some());
        assert!(tracker.current_status.is_some());
    }

    #[test]
    fn test_convergence_percentage() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.95,
            mutation_score: 0.85,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 95.0,
            rust_project_score: 90,
            satd_markers: 0,
            dead_code_items: 0,
            ..Default::default()
        };

        tracker.record(metrics, 0, &targets);

        let percentage = tracker.convergence_percentage(&targets);
        assert!(percentage > 0.9);
    }

    #[test]
    fn test_is_converged() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Perfect metrics
        let metrics = ProjectMetrics {
            test_coverage: 0.96,
            mutation_score: 0.86,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 92,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        tracker.record(metrics, 0, &targets);

        assert!(tracker.is_converged());
    }

    #[test]
    fn test_remaining_failures() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.50, // Below target
            ..Default::default()
        };

        tracker.record(metrics, 5, &targets);

        let failures = tracker.remaining_failures();
        assert!(!failures.is_empty());
        assert!(failures.iter().any(|f| f.contains("Coverage")));
    }

    #[test]
    fn test_trend_improving() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // First iteration: 10 defects
        tracker.record(ProjectMetrics::default(), 10, &targets);

        // Second iteration: 5 defects (improving)
        tracker.record(ProjectMetrics::default(), 5, &targets);

        let trend = tracker.trend();
        assert!(trend > 0.0);
    }

    #[test]
    fn test_trend_stable() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Same defects across iterations
        tracker.record(ProjectMetrics::default(), 5, &targets);
        tracker.record(ProjectMetrics::default(), 5, &targets);

        let trend = tracker.trend();
        assert_eq!(trend, 0.0);
    }
}

mod aggregated_collector_tests {
    use super::*;

    #[test]
    fn test_new_aggregated_collector() {
        let collector = AggregatedCollector::new();
        // Should have 3 default collectors
        assert!(collector.collector_count() > 0);
    }

    #[test]
    fn test_signals_to_defects_empty() {
        let collector = AggregatedCollector::new();
        let defects = collector.signals_to_defects(vec![]);
        assert!(defects.is_empty());
    }

    #[test]
    fn test_signals_to_defects_rustc() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "mismatched types".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);

        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].category, DefectCategory::TypeErrors);
        assert_eq!(defects[0].severity, Severity::Critical);
    }

    #[test]
    fn test_signals_to_defects_clippy() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: unused variable".to_string(),
            error_code: Some("clippy::unused".to_string()),
            weight: 0.5,
        }];

        let defects = collector.signals_to_defects(signals);

        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].severity, Severity::Medium);
    }

    #[test]
    fn test_signals_to_defects_test_failure() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::CargoTest,
            raw_message: "test failed: test_something".to_string(),
            error_code: None,
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);

        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].severity, Severity::High);
    }
}

mod pdca_loop_tests {
    use super::*;

    #[test]
    fn test_new_pdca_loop() {
        let pdca = PdcaLoop::new();
        // Just verify it creates successfully with default 100 iterations
        assert_eq!(pdca.max_iterations(), 100);
    }

    #[test]
    fn test_with_config() {
        let config = OracleConfig {
            max_iterations: 50,
            ..Default::default()
        };
        let targets = ConvergenceTargets::default();

        let pdca = PdcaLoop::with_config(config, targets);
        assert_eq!(pdca.max_iterations(), 50);
    }
}

mod integration_tests {
    use super::*;
    use std::path::PathBuf;

    // These tests require actual project paths and are ignored by default
    #[ignore = "requires oracle service setup"]
    #[tokio::test]
    async fn test_run_on_real_project() {
        let pdca = PdcaLoop::new();
        let project_path = PathBuf::from(".");

        let results = pdca.run(&project_path).await.unwrap();
        assert!(!results.is_empty());
    }

    #[ignore = "requires oracle service setup"]
    #[tokio::test]
    async fn test_single_iteration() {
        let pdca = PdcaLoop::new();
        let project_path = PathBuf::from(".");

        let result = pdca.run_single(&project_path).await.unwrap();
        assert_eq!(result.iteration, 1);
    }
}
