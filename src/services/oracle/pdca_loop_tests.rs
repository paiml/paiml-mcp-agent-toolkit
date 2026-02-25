// Tests for PDCA loop: construction, iteration results, regression checks,
// progress calculation, run execution, config, convergence, and metrics.

mod coverage_tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    // ==================== PdcaLoop Construction Tests ====================

    #[test]
    fn test_pdca_loop_new_uses_default_config() {
        let pdca = PdcaLoop::new();
        assert_eq!(pdca.max_iterations(), 100);
    }

    #[test]
    fn test_pdca_loop_default_trait() {
        let pdca = PdcaLoop::default();
        assert_eq!(pdca.max_iterations(), 100);
    }

    #[test]
    fn test_pdca_loop_with_custom_config() {
        let config = OracleConfig {
            max_iterations: 50,
            min_progress_per_iteration: 0.01,
            stagnation_threshold: 3,
            andon_enabled: false,
            require_human_approval_above: None,
            auto_apply_threshold: 0.85,
            review_threshold: 0.6,
            batch_size: 5,
        };
        let targets = ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.80,
            ..Default::default()
        };
        let pdca = PdcaLoop::with_config(config, targets);
        assert_eq!(pdca.max_iterations(), 50);
    }

    // ==================== PdcaIterationResult Tests ====================

    #[test]
    fn test_pdca_iteration_result_fields() {
        let result = PdcaIterationResult {
            iteration: 1,
            defects_found: 10,
            defects_fixed: 5,
            defects_skipped: 3,
            metrics_before: ProjectMetrics::default(),
            metrics_after: ProjectMetrics::default(),
            converged: false,
        };

        assert_eq!(result.iteration, 1);
        assert_eq!(result.defects_found, 10);
        assert_eq!(result.defects_fixed, 5);
        assert_eq!(result.defects_skipped, 3);
        assert!(!result.converged);
    }

    #[test]
    fn test_pdca_iteration_result_converged() {
        let result = PdcaIterationResult {
            iteration: 5,
            defects_found: 0,
            defects_fixed: 0,
            defects_skipped: 0,
            metrics_before: ProjectMetrics::default(),
            metrics_after: ProjectMetrics {
                test_coverage: 0.95,
                mutation_score: 0.85,
                ..Default::default()
            },
            converged: true,
        };

        assert!(result.converged);
        assert_eq!(result.iteration, 5);
    }

    // ==================== Regression Check Tests ====================

    #[test]
    fn test_check_regression_no_regression() {
        let pdca = PdcaLoop::new();
        let before = ProjectMetrics {
            test_coverage: 0.80,
            compiler_errors: 5,
            test_failures: 2,
            ..Default::default()
        };
        let after = ProjectMetrics {
            test_coverage: 0.82,
            compiler_errors: 3,
            test_failures: 1,
            ..Default::default()
        };

        let result = pdca.check_regression(&before, &after);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_regression_coverage_decreased() {
        let pdca = PdcaLoop::new();
        let before = ProjectMetrics {
            test_coverage: 0.80,
            ..Default::default()
        };
        let after = ProjectMetrics {
            test_coverage: 0.75, // Decreased by more than 1%
            ..Default::default()
        };

        let result = pdca.check_regression(&before, &after);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ANDON: Coverage decreased"));
    }

    #[test]
    fn test_check_regression_compiler_errors_increased() {
        let pdca = PdcaLoop::new();
        let before = ProjectMetrics {
            compiler_errors: 5,
            ..Default::default()
        };
        let after = ProjectMetrics {
            compiler_errors: 8,
            ..Default::default()
        };

        let result = pdca.check_regression(&before, &after);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ANDON: Compiler errors increased"));
    }

    #[test]
    fn test_check_regression_test_failures_increased() {
        let pdca = PdcaLoop::new();
        let before = ProjectMetrics {
            test_failures: 2,
            ..Default::default()
        };
        let after = ProjectMetrics {
            test_failures: 5,
            ..Default::default()
        };

        let result = pdca.check_regression(&before, &after);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ANDON: Test failures increased"));
    }

    #[test]
    fn test_check_regression_minor_coverage_decrease_ok() {
        let pdca = PdcaLoop::new();
        let before = ProjectMetrics {
            test_coverage: 0.80,
            ..Default::default()
        };
        let after = ProjectMetrics {
            test_coverage: 0.795, // Decreased by less than 1%
            ..Default::default()
        };

        let result = pdca.check_regression(&before, &after);
        assert!(result.is_ok());
    }

    // ==================== Progress Calculation Tests ====================

    #[test]
    fn test_calculate_progress_single_iteration() {
        let pdca = PdcaLoop::new();
        let results = vec![PdcaIterationResult {
            iteration: 1,
            defects_found: 10,
            defects_fixed: 5,
            defects_skipped: 3,
            metrics_before: ProjectMetrics::default(),
            metrics_after: ProjectMetrics::default(),
            converged: false,
        }];

        // Single iteration should return 1.0
        let progress = pdca.calculate_progress(&results);
        assert!((progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_progress_multiple_iterations() {
        let pdca = PdcaLoop::new();
        let results = vec![
            PdcaIterationResult {
                iteration: 1,
                defects_found: 10,
                defects_fixed: 3,
                defects_skipped: 2,
                metrics_before: ProjectMetrics::default(),
                metrics_after: ProjectMetrics::default(),
                converged: false,
            },
            PdcaIterationResult {
                iteration: 2,
                defects_found: 5, // Reduced from 10
                defects_fixed: 2,
                defects_skipped: 1,
                metrics_before: ProjectMetrics::default(),
                metrics_after: ProjectMetrics::default(),
                converged: false,
            },
        ];

        // Progress = (10 - 5) / 10 = 0.5
        let progress = pdca.calculate_progress(&results);
        assert!((progress - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_progress_zero_initial_defects() {
        let pdca = PdcaLoop::new();
        let results = vec![
            PdcaIterationResult {
                iteration: 1,
                defects_found: 0,
                defects_fixed: 0,
                defects_skipped: 0,
                metrics_before: ProjectMetrics::default(),
                metrics_after: ProjectMetrics::default(),
                converged: true,
            },
            PdcaIterationResult {
                iteration: 2,
                defects_found: 0,
                defects_fixed: 0,
                defects_skipped: 0,
                metrics_before: ProjectMetrics::default(),
                metrics_after: ProjectMetrics::default(),
                converged: true,
            },
        ];

        // Zero initial defects should return 1.0
        let progress = pdca.calculate_progress(&results);
        assert!((progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_calculate_progress_no_improvement() {
        let pdca = PdcaLoop::new();
        let results = vec![
            PdcaIterationResult {
                iteration: 1,
                defects_found: 10,
                defects_fixed: 0,
                defects_skipped: 10,
                metrics_before: ProjectMetrics::default(),
                metrics_after: ProjectMetrics::default(),
                converged: false,
            },
            PdcaIterationResult {
                iteration: 2,
                defects_found: 10, // No change
                defects_fixed: 0,
                defects_skipped: 10,
                metrics_before: ProjectMetrics::default(),
                metrics_after: ProjectMetrics::default(),
                converged: false,
            },
        ];

        // No progress = 0.0
        let progress = pdca.calculate_progress(&results);
        assert!((progress - 0.0).abs() < f32::EPSILON);
    }

    // ==================== Run Tests (with temp directory) ====================

    #[tokio::test]
    async fn test_run_with_nonexistent_path() {
        let pdca = PdcaLoop::new();
        let result = pdca.run(Path::new("/nonexistent/path/xyz123")).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Project path does not exist"));
    }

    #[tokio::test]
    async fn test_run_single_with_nonexistent_path() {
        let pdca = PdcaLoop::new();
        let result = pdca.run_single(Path::new("/nonexistent/path/xyz123")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_iterations_with_nonexistent_path() {
        let pdca = PdcaLoop::new();
        let result = pdca
            .run_iterations(Path::new("/nonexistent/path/xyz123"), 5)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_with_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pdca = PdcaLoop::new();

        // Running on empty directory should work but find no signals
        let result = pdca.run_iterations(temp_dir.path(), 1).await;
        // May succeed or fail depending on whether cargo commands work
        // Just verify it doesn't panic
        let _ = result;
    }

    // ==================== OracleConfig Tests ====================

    #[test]
    fn test_oracle_config_default() {
        let config = OracleConfig::default();
        assert_eq!(config.max_iterations, 100);
        assert!((config.min_progress_per_iteration - 0.001).abs() < f32::EPSILON);
        assert_eq!(config.stagnation_threshold, 5);
        assert!(config.andon_enabled);
        assert_eq!(config.require_human_approval_above, Some(10));
        assert!((config.auto_apply_threshold - 0.9).abs() < f32::EPSILON);
        assert!((config.review_threshold - 0.7).abs() < f32::EPSILON);
        assert_eq!(config.batch_size, 10);
    }

    #[test]
    fn test_oracle_config_custom() {
        let config = OracleConfig {
            max_iterations: 200,
            min_progress_per_iteration: 0.005,
            stagnation_threshold: 10,
            andon_enabled: false,
            require_human_approval_above: Some(50),
            auto_apply_threshold: 0.95,
            review_threshold: 0.75,
            batch_size: 20,
        };

        assert_eq!(config.max_iterations, 200);
        assert_eq!(config.stagnation_threshold, 10);
        assert!(!config.andon_enabled);
        assert_eq!(config.batch_size, 20);
    }

    // ==================== ConvergenceTargets Tests ====================

    #[test]
    fn test_convergence_targets_default() {
        let targets = ConvergenceTargets::default();
        assert!((targets.test_coverage - 0.95).abs() < f32::EPSILON);
        assert!((targets.mutation_score - 0.85).abs() < f32::EPSILON);
        assert_eq!(targets.max_compiler_errors, 0);
        assert_eq!(targets.max_clippy_warnings, 0);
        assert_eq!(targets.max_test_failures, 0);
        assert!((targets.min_tdg_score - 95.0).abs() < f32::EPSILON);
        assert_eq!(targets.min_rust_project_score, 90);
        assert_eq!(targets.max_satd_markers, 0);
        assert_eq!(targets.max_dead_code, 0);
        assert_eq!(targets.max_cyclomatic_complexity, 15);
        assert_eq!(targets.max_cognitive_complexity, 25);
        assert_eq!(targets.max_build_time, Duration::from_secs(60));
    }

    #[test]
    fn test_convergence_targets_check_converged() {
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 0.96,
            mutation_score: 0.90,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 95,
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
    fn test_convergence_targets_check_not_converged_coverage() {
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 0.80, // Below 0.95 target
            mutation_score: 0.90,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 95,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        let status = targets.check(&metrics);
        if let ConvergenceStatus::NotConverged { remaining } = status {
            assert!(!remaining.is_empty());
            assert!(remaining.iter().any(|s| s.contains("Coverage")));
        } else {
            panic!("Expected NotConverged status");
        }
    }

    #[test]
    fn test_convergence_targets_check_multiple_failures() {
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 0.80,           // Below target
            mutation_score: 0.70,          // Below target
            compiler_errors: 5,            // Above max
            clippy_warnings: 10,           // Above max
            test_failures: 3,              // Above max
            tdg_score: 80.0,               // Below min
            rust_project_score: 70,        // Below min
            satd_markers: 5,               // Above max
            dead_code_items: 10,           // Above max
            max_cyclomatic_complexity: 25, // Above max
            max_cognitive_complexity: 50,  // Above max
            build_time: Duration::from_secs(30),
        };

        let status = targets.check(&metrics);
        if let ConvergenceStatus::NotConverged { remaining } = status {
            assert!(remaining.len() >= 5);
        } else {
            panic!("Expected NotConverged status");
        }
    }

    // ==================== ProjectMetrics Tests ====================

    #[test]
    fn test_project_metrics_default() {
        let metrics = ProjectMetrics::default();
        assert!((metrics.test_coverage - 0.0).abs() < f32::EPSILON);
        assert!((metrics.mutation_score - 0.0).abs() < f32::EPSILON);
        assert_eq!(metrics.compiler_errors, 0);
        assert_eq!(metrics.clippy_warnings, 0);
        assert_eq!(metrics.test_failures, 0);
        assert!((metrics.tdg_score - 0.0).abs() < f32::EPSILON);
        assert_eq!(metrics.rust_project_score, 0);
        assert_eq!(metrics.satd_markers, 0);
        assert_eq!(metrics.dead_code_items, 0);
        assert_eq!(metrics.max_cyclomatic_complexity, 0);
        assert_eq!(metrics.max_cognitive_complexity, 0);
        assert_eq!(metrics.build_time, Duration::ZERO);
    }

    #[test]
    fn test_project_metrics_custom() {
        let metrics = ProjectMetrics {
            test_coverage: 0.85,
            mutation_score: 0.75,
            compiler_errors: 2,
            clippy_warnings: 5,
            test_failures: 1,
            tdg_score: 88.5,
            rust_project_score: 82,
            satd_markers: 3,
            dead_code_items: 7,
            max_cyclomatic_complexity: 12,
            max_cognitive_complexity: 18,
            build_time: Duration::from_secs(45),
        };

        assert!((metrics.test_coverage - 0.85).abs() < f32::EPSILON);
        assert_eq!(metrics.compiler_errors, 2);
        assert_eq!(metrics.build_time, Duration::from_secs(45));
    }
}
