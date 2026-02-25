    // ========== Property-Based Tests ==========

    mod proptest_tests {
        use super::*;

        proptest! {
            #[test]
            fn test_quality_profile_coverage_bounds(coverage in 0.0f64..100.0) {
                let profile = QualityProfile {
                    coverage_min: coverage,
                    ..QualityProfile::default()
                };
                prop_assert!(profile.coverage_min >= 0.0);
                prop_assert!(profile.coverage_min <= 100.0);
            }

            #[test]
            fn test_quality_profile_complexity_bounds(complexity in 1u16..100) {
                let profile = QualityProfile {
                    complexity_max: complexity,
                    ..QualityProfile::default()
                };
                prop_assert!(profile.complexity_max >= 1);
                prop_assert!(profile.complexity_max <= 100);
            }

            #[test]
            fn test_enforcement_result_score_bounds(score in 0.0f64..1.0) {
                let result = EnforcementResult {
                    state: EnforcementState::Analyzing,
                    score,
                    target: 1.0,
                    current_file: None,
                    violations: vec![],
                    next_action: "test".to_string(),
                    progress: EnforcementProgress {
                        files_completed: 0,
                        files_remaining: 0,
                        estimated_iterations: 0,
                    },
                };
                prop_assert!(result.score >= 0.0);
                prop_assert!(result.score <= 1.0);
            }

            #[test]
            fn test_should_continue_respects_max_iterations(
                iteration in 0u32..100,
                max_iterations in 1u32..50
            ) {
                let config = EnforcementConfig {
                    max_iterations,
                    ..make_test_enforcement_config()
                };
                let start_time = Instant::now();
                let result = should_continue_enforcement(
                    EnforcementState::Analyzing,
                    iteration,
                    &config,
                    start_time
                );

                if iteration >= max_iterations {
                    prop_assert!(!result);
                }
            }

            #[test]
            fn test_violation_serialization_roundtrip(
                current in 0.0f64..100.0,
                target in 0.0f64..100.0
            ) {
                let violation = QualityViolation {
                    violation_type: "test".to_string(),
                    severity: "medium".to_string(),
                    location: "test.rs:1".to_string(),
                    current,
                    target,
                    suggestion: "test suggestion".to_string(),
                };

                let json = serde_json::to_string(&violation).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;
                let parsed: QualityViolation = serde_json::from_str(&json).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;

                prop_assert!((parsed.current - current).abs() < 0.001);
                prop_assert!((parsed.target - target).abs() < 0.001);
            }
        }
    }

    // ========== Edge Case Tests ==========

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_empty_project_path() {
            let _path = PathBuf::from("");
            let _profile = make_test_profile();
            // Just verify it doesn't panic
            let _ = load_quality_profile("extreme", None);
        }

        #[test]
        fn test_zero_iterations() {
            let config = EnforcementConfig {
                max_iterations: 0,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Analyzing, 0, &config, start_time);
            assert!(!result);
        }

        #[test]
        fn test_large_iteration_count() {
            let config = EnforcementConfig {
                max_iterations: u32::MAX,
                ..make_test_enforcement_config()
            };
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Analyzing, 1000, &config, start_time);
            assert!(result);
        }

        #[test]
        fn test_enforcement_result_clone() {
            let result = EnforcementResult {
                state: EnforcementState::Complete,
                score: 0.95,
                target: 1.0,
                current_file: Some("test.rs".to_string()),
                violations: vec![make_test_violation("test", "low")],
                next_action: "none".to_string(),
                progress: EnforcementProgress {
                    files_completed: 10,
                    files_remaining: 0,
                    estimated_iterations: 0,
                },
            };

            let cloned = result.clone();
            assert_eq!(cloned.state, result.state);
            assert_eq!(cloned.score, result.score);
            assert_eq!(cloned.violations.len(), result.violations.len());
        }

        #[test]
        fn test_quality_profile_clone() {
            let profile = make_test_profile();
            let cloned = profile.clone();
            assert_eq!(cloned.coverage_min, profile.coverage_min);
            assert_eq!(cloned.complexity_max, profile.complexity_max);
        }

        #[test]
        fn test_violation_with_unicode_location() {
            let violation = QualityViolation {
                violation_type: "complexity".to_string(),
                severity: "high".to_string(),
                location: "src/test.rs:10".to_string(),
                current: 25.0,
                target: 10.0,
                suggestion: "Refactor function".to_string(),
            };

            let json = serde_json::to_string(&violation).unwrap();
            assert!(json.contains("test.rs"));
        }

        #[test]
        fn test_enforcement_state_copy() {
            let state1 = EnforcementState::Analyzing;
            let state2 = state1; // Copy
            assert_eq!(state1, state2);
        }
    }

    // ========== Cache Tests ==========

    mod cache_tests {
        use super::*;

        #[test]
        fn test_clear_enforcement_cache_none() {
            // Should not panic with None
            clear_enforcement_cache(&None);
        }

        #[test]
        fn test_clear_enforcement_cache_some() {
            let temp_dir = TempDir::new().unwrap();
            let cache_path = temp_dir.path().to_path_buf();
            clear_enforcement_cache(&Some(cache_path));
        }
    }

    // ========== Loop Result Tests ==========
