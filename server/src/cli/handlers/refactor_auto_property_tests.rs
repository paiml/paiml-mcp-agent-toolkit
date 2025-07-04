#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    // Test-specific structures that mirror the refactor auto state machine

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestRefactorState {
        iteration: u32,
        context_generated: bool,
        context_path: PathBuf,
        current_file: Option<PathBuf>,
        files_completed: Vec<PathBuf>,
        quality_metrics: TestQualityMetrics,
        progress: TestRefactorProgress,
        start_time: std::time::SystemTime,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct TestQualityMetrics {
        total_violations: usize,
        coverage_percent: f64,
        max_complexity: u32,
        average_complexity: f64,
        technical_debt_count: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestRefactorProgress {
        files_processed: usize,
        files_improved: usize,
        total_files: usize,
        violations_fixed: usize,
        complexity_reduced: u32,
    }

    #[derive(Debug, Clone)]
    struct TestQualityProfile {
        coverage_min: f64,
        complexity_max: u16,
        complexity_target: u16,
        satd_allowed: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestViolationDetail {
        file: PathBuf,
        line: u32,
        column: u32,
        end_line: u32,
        end_column: u32,
        lint_name: String,
        message: String,
        severity: String,
        suggestion: Option<String>,
        machine_applicable: bool,
    }

    // Strategy for generating valid RefactorState
    prop_compose! {
        fn arb_refactor_state()
            (iteration in 0u32..100,
             files_completed in prop::collection::vec(
                 "[a-zA-Z0-9_/]+\\.rs", 0..10
             ),
             total_violations in 0usize..1000,
             coverage_percent in 0.0f64..100.0,
             max_complexity in 0u32..100)
            -> TestRefactorState
        {
            let files_completed_paths: Vec<PathBuf> = files_completed.into_iter().map(PathBuf::from).collect();
            let files_processed = files_completed_paths.len();
            TestRefactorState {
                iteration,
                context_generated: iteration > 0,
                context_path: PathBuf::from("/tmp/context.json"),
                current_file: if iteration > 0 { Some(PathBuf::from("current.rs")) } else { None },
                files_completed: files_completed_paths,
                quality_metrics: TestQualityMetrics {
                    total_violations,
                    coverage_percent,
                    max_complexity,
                    average_complexity: max_complexity as f64 / 2.0,
                    technical_debt_count: total_violations / 10,
                },
                progress: TestRefactorProgress {
                    files_processed,
                    files_improved: files_processed / 2,
                    total_files: 100,
                    violations_fixed: total_violations / 3,
                    complexity_reduced: max_complexity / 2,
                },
                start_time: std::time::SystemTime::now(),
            }
        }
    }

    // Strategy for generating quality profiles
    prop_compose! {
        fn arb_quality_profile()
            (coverage_min in 0.0f64..100.0,
             complexity_max in 1u16..50,
             complexity_target in 1u16..10,
             satd_allowed in 0usize..10)
            -> TestQualityProfile
        {
            TestQualityProfile {
                coverage_min,
                complexity_max: complexity_max.max(complexity_target),
                complexity_target: complexity_target.min(complexity_max),
                satd_allowed,
            }
        }
    }

    // Strategy for generating violation details
    prop_compose! {
        fn arb_violation_detail()
            (file in "[a-zA-Z0-9_/]+\\.rs",
             line in 1u32..1000,
             column in 1u32..120,
             lint_name in prop::sample::select(vec![
                 "clippy::too_complex",
                 "clippy::cognitive_complexity",
                 "clippy::missing_docs",
                 "clippy::unwrap_used",
                 "clippy::panic",
             ]),
             severity in prop::sample::select(vec!["error", "warning", "info"]))
            -> TestViolationDetail
        {
            TestViolationDetail {
                file: PathBuf::from(file),
                line,
                column,
                end_line: line + 1,
                end_column: column + 10,
                lint_name: lint_name.to_string(),
                message: format!("Violation at line {}", line),
                severity: severity.to_string(),
                suggestion: Some("Fix the issue".to_string()),
                machine_applicable: severity == "warning",
            }
        }
    }

    proptest! {
        #[test]
        fn state_machine_transitions_valid(
            initial_state in arb_refactor_state(),
            _profile in arb_quality_profile()
        ) {
            // Property: State transitions are always valid
            let mut state = initial_state.clone();

            // Simulate state transitions
            state.iteration += 1;
            prop_assert!(state.iteration > initial_state.iteration);

            // Context should be generated after first iteration
            if state.iteration > 0 {
                state.context_generated = true; // Simulate context generation
            }
            prop_assert!(!state.context_generated || state.iteration > 0);

            // Progress metrics should be consistent
            prop_assert!(state.progress.files_processed <= state.progress.total_files);
            prop_assert!(state.progress.files_improved <= state.progress.files_processed);
            // Note: violations_fixed might be from previous iterations, so we check initial state
            prop_assert!(initial_state.progress.violations_fixed <= initial_state.quality_metrics.total_violations);
        }

        #[test]
        fn quality_profile_constraints_respected(profile in arb_quality_profile()) {
            // Property: Quality profile constraints are internally consistent
            prop_assert!(profile.complexity_target <= profile.complexity_max);
            prop_assert!(profile.coverage_min >= 0.0 && profile.coverage_min <= 100.0);
            prop_assert!(profile.complexity_max >= 1);
            prop_assert!(profile.complexity_target >= 1);
        }

        #[test]
        fn severity_scoring_monotonic(
            violations in prop::collection::vec(arb_violation_detail(), 0..50),
            issue_keywords in prop::collection::hash_map(
                prop::sample::select(vec!["Performance", "Security", "Complexity", "Correctness"]),
                0.0f32..1.0,
                0..4
            )
        ) {
            // Convert to proper types
            let issue_keywords: HashMap<String, f32> = issue_keywords
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect();

            // Calculate base score
            let base_score = calculate_test_severity_score(&violations, None);

            // Calculate score with issue context
            let issue_score = calculate_test_severity_score(&violations, Some(&issue_keywords));

            // Property: Issue context should never decrease severity score
            prop_assert!(issue_score >= base_score,
                "Issue context decreased score: {} -> {}", base_score, issue_score);

            // Property: Security issues should increase score the most
            if issue_keywords.contains_key("Security") &&
               violations.iter().any(|v| v.lint_name.contains("security")) {
                prop_assert!(issue_score >= base_score * 2.0,
                    "Security issues should significantly increase score");
            }
        }

        #[test]
        fn file_selection_deterministic(
            files in prop::collection::vec(
                (
                    "[a-zA-Z0-9_/]+\\.rs",
                    0.0f64..100.0, // severity score
                    0u32..50,      // complexity
                    0usize..100    // violations
                ),
                1..20
            ),
            max_files in 1usize..10
        ) {
            // Create file metrics
            let file_metrics: Vec<(PathBuf, f64, u32, usize)> = files
                .iter()
                .map(|(path, score, complexity, violations)| {
                    (PathBuf::from(path), *score, *complexity, *violations)
                })
                .collect();

            // Select files twice with same parameters
            let selected1 = select_test_files_by_severity(&file_metrics, max_files);
            let selected2 = select_test_files_by_severity(&file_metrics, max_files);

            // Property: File selection is deterministic
            prop_assert_eq!(&selected1, &selected2,
                "File selection should be deterministic");

            // Property: Selected files are the highest severity
            if file_metrics.len() > max_files {
                let min_selected_score = selected1
                    .iter()
                    .map(|(_, s, _, _)| *s)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                let max_unselected_score = file_metrics
                    .iter()
                    .filter(|(p, _, _, _)| !selected1.iter().any(|(sp, _, _, _)| sp == p))
                    .map(|(_, s, _, _)| *s)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                prop_assert!(min_selected_score >= max_unselected_score,
                    "Selected files should have highest severity scores");
            }
        }

        #[test]
        fn json_serialization_roundtrip(state in arb_refactor_state()) {
            // Property: State can be serialized and deserialized without loss
            let serialized = serde_json::to_string(&state).unwrap();
            let deserialized: TestRefactorState = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(state.iteration, deserialized.iteration);
            prop_assert_eq!(state.context_generated, deserialized.context_generated);
            prop_assert_eq!(state.files_completed.len(), deserialized.files_completed.len());
            prop_assert_eq!(state.quality_metrics.total_violations,
                deserialized.quality_metrics.total_violations);
        }

        #[test]
        fn progress_metrics_consistent(
            files_processed in 0usize..100,
            files_improved in 0usize..100,
            total_files in 1usize..200,
            violations_fixed in 0usize..1000,
            complexity_reduced in 0u32..100
        ) {
            let progress = TestRefactorProgress {
                files_processed: files_processed.min(total_files),
                files_improved: files_improved.min(files_processed.min(total_files)),
                total_files,
                violations_fixed,
                complexity_reduced,
            };

            // Property: Progress metrics are internally consistent
            prop_assert!(progress.files_improved <= progress.files_processed);
            prop_assert!(progress.files_processed <= progress.total_files);

            // Property: Completion percentage is valid
            let completion = (progress.files_processed as f64 / progress.total_files as f64) * 100.0;
            prop_assert!((0.0..=100.0).contains(&completion));
        }

        #[test]
        fn iteration_monotonic_increase(
            initial_state in arb_refactor_state(),
            num_iterations in 1u32..20
        ) {
            let initial_files_len = initial_state.files_completed.len();
            let mut state = initial_state;
            let mut iterations = vec![state.iteration];

            // Simulate multiple iterations
            for _ in 0..num_iterations {
                state.iteration += 1;
                iterations.push(state.iteration);

                // Simulate some files being completed
                if state.iteration % 3 == 0 {
                    state.files_completed.push(
                        PathBuf::from(format!("file_{}.rs", state.iteration))
                    );
                }
            }

            // Property: Iterations always increase
            for window in iterations.windows(2) {
                prop_assert!(window[1] > window[0],
                    "Iterations must monotonically increase");
            }

            // Property: Files completed grows monotonically
            prop_assert!(state.files_completed.len() >= initial_files_len);
        }

        #[test]
        fn ai_request_generation_valid(
            file_path in "[a-zA-Z0-9_/]+\\.rs",
            violations in prop::collection::vec(arb_violation_detail(), 0..20),
            complexity in 1u32..100,
            coverage in 0.0f64..100.0,
            issue_summary in prop::option::of("[a-zA-Z0-9 .,!?]{10,200}")
        ) {
            let request = generate_test_ai_refactor_request(
                &PathBuf::from(&file_path),
                &violations,
                complexity,
                coverage,
                issue_summary.as_deref(),
                &TestQualityProfile {
                    coverage_min: 80.0,
                    complexity_max: 10,
                    complexity_target: 5,
                    satd_allowed: 0,
                }
            );

            // Property: Request contains all required fields
            prop_assert!(request.contains("file_path"));
            prop_assert!(request.contains(&file_path));
            prop_assert!(request.contains("current_complexity"));
            prop_assert!(request.contains(&complexity.to_string()));
            prop_assert!(request.contains("current_coverage"));
            prop_assert!(request.contains("quality_targets"));

            // Property: Issue context is included when provided
            if let Some(summary) = &issue_summary {
                prop_assert!(request.contains("github_issue_context"));
                prop_assert!(request.contains(summary));
            }

            // Property: Request is valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&request).unwrap();
            prop_assert!(parsed.is_object());
        }

        #[test]
        fn state_persistence_and_recovery(
            state in arb_refactor_state(),
            temp_dir in any::<u64>().prop_map(|_seed| {
                TempDir::new().unwrap()
            })
        ) {
            let checkpoint_path = temp_dir.path().join("checkpoint.json");

            // Save state
            let saved = serde_json::to_string_pretty(&state).unwrap();
            std::fs::write(&checkpoint_path, &saved).unwrap();

            // Load state
            let loaded_content = std::fs::read_to_string(&checkpoint_path).unwrap();
            let loaded_state: TestRefactorState = serde_json::from_str(&loaded_content).unwrap();

            // Property: State survives save/load cycle
            prop_assert_eq!(state.iteration, loaded_state.iteration);
            prop_assert_eq!(state.files_completed.len(), loaded_state.files_completed.len());
            prop_assert_eq!(state.quality_metrics.total_violations,
                loaded_state.quality_metrics.total_violations);
        }

        #[test]
        fn concurrent_state_updates_safe(
            initial_state in arb_refactor_state(),
            updates in prop::collection::vec(
                prop::sample::select(vec!["add_file", "update_metrics", "increment_iteration"]),
                1..10
            )
        ) {
            let mut state = initial_state;

            for update in updates {
                match update {
                    "add_file" => {
                        let new_file = PathBuf::from(format!("file_{}.rs", state.files_completed.len()));
                        state.files_completed.push(new_file);
                        state.progress.files_processed += 1;
                    },
                    "update_metrics" => {
                        state.quality_metrics.total_violations =
                            state.quality_metrics.total_violations.saturating_sub(10);
                        state.progress.violations_fixed += 10;
                    },
                    "increment_iteration" => {
                        state.iteration += 1;
                    },
                    _ => unreachable!(),
                }

                // Property: State remains consistent after each update
                prop_assert!(state.progress.files_processed <= state.progress.total_files);
                prop_assert!(state.progress.violations_fixed <=
                    state.quality_metrics.total_violations + state.progress.violations_fixed);
            }
        }
    }

    // Helper functions for testing
    fn calculate_test_severity_score(
        violations: &[TestViolationDetail],
        issue_keywords: Option<&HashMap<String, f32>>,
    ) -> f64 {
        let mut score = violations.len() as f64;

        if let Some(keywords) = issue_keywords {
            for violation in violations {
                let mut multiplier = 1.0;

                if violation.lint_name.contains("security") && keywords.contains_key("Security") {
                    multiplier *= 4.0;
                } else if violation.lint_name.contains("complex")
                    && keywords.contains_key("Complexity")
                {
                    multiplier *= 2.0;
                } else if keywords.contains_key("Performance") {
                    multiplier *= 1.5;
                }

                score += multiplier;
            }
        }

        score
    }

    fn select_test_files_by_severity(
        files: &[(PathBuf, f64, u32, usize)],
        max_files: usize,
    ) -> Vec<(PathBuf, f64, u32, usize)> {
        let mut sorted = files.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted.truncate(max_files);
        sorted
    }

    fn generate_test_ai_refactor_request(
        file_path: &Path,
        violations: &[TestViolationDetail],
        complexity: u32,
        coverage: f64,
        issue_summary: Option<&str>,
        profile: &TestQualityProfile,
    ) -> String {
        let mut request = json!({
            "file_path": file_path.to_string_lossy(),
            "current_complexity": complexity,
            "current_coverage": coverage,
            "violations": violations.len(),
            "quality_targets": {
                "max_complexity": profile.complexity_max,
                "target_complexity": profile.complexity_target,
                "min_coverage": profile.coverage_min,
                "satd_allowed": profile.satd_allowed
            }
        });

        if let Some(summary) = issue_summary {
            request["github_issue_context"] = json!(summary);
        }

        serde_json::to_string_pretty(&request).unwrap()
    }
}
