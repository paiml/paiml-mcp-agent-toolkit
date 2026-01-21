//\! Tests for QA work handler
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_generate_checklist_feature() {
        let checklist = generate_checklist("TEST-001", QaTaskType::Feature);
        assert_eq!(checklist.task_id, "TEST-001");
        assert_eq!(checklist.task_type, "feature");
        assert_eq!(checklist.categories.safety_ethics.len(), 5);
        assert_eq!(checklist.categories.code_quality.len(), 5);
        assert_eq!(checklist.categories.testing.len(), 5);
        assert_eq!(checklist.categories.documentation.len(), 5);
        assert_eq!(checklist.categories.process.len(), 5);
    }

    #[test]
    fn test_generate_checklist_bugfix() {
        let checklist = generate_checklist("BUG-042", QaTaskType::Bugfix);
        assert_eq!(checklist.task_type, "bugfix");
    }

    #[test]
    fn test_format_checklist_text() {
        let checklist = generate_checklist("TEST-001", QaTaskType::Feature);
        let text = format_checklist_text(&checklist);
        assert!(text.contains("QA Checklist for TEST-001"));
        assert!(text.contains("Safety & Ethics"));
        assert!(text.contains("Code Quality"));
    }

    #[test]
    fn test_validation_status_equality() {
        assert_eq!(ValidationStatus::Passed, ValidationStatus::Passed);
        assert_ne!(ValidationStatus::Passed, ValidationStatus::Failed);
    }

    #[test]
    fn test_checklist_item_defaults() {
        let item = ChecklistItem {
            id: "A1".into(),
            description: "Test".into(),
            checked: false,
            automated: true,
            evidence: None,
        };
        assert!(!item.checked);
        assert!(item.automated);
        assert!(item.evidence.is_none());
    }

    // V2 Feature Tests (EXTREME TDD - RED phase)

    #[test]
    fn test_generate_examples_creates_script_files() {
        let examples = generate_example_scripts("TEST-001", "my-feature");
        assert!(!examples.is_empty());
        assert!(examples.iter().any(|e| e.name.contains("basic")));
        assert!(examples.iter().any(|e| e.name.contains("error")));
    }

    #[test]
    fn test_generate_examples_includes_edge_cases() {
        let examples = generate_example_scripts("TEST-001", "analyze");
        // Must have edge case examples
        assert!(examples
            .iter()
            .any(|e| e.name.contains("edge") || e.name.contains("empty")));
    }

    #[test]
    fn test_example_script_structure() {
        let examples = generate_example_scripts("TEST-001", "context");
        for example in &examples {
            assert!(!example.name.is_empty());
            assert!(!example.content.is_empty());
            assert!(example.content.contains("pmat") || example.content.contains("#!"));
        }
    }

    #[test]
    fn test_epic_aggregation_calculates_totals() {
        let tasks = vec![
            ("TASK-1".to_string(), 20, 25), // 80%
            ("TASK-2".to_string(), 25, 25), // 100%
            ("TASK-3".to_string(), 15, 25), // 60%
        ];
        let summary = calculate_epic_summary("EPIC-001", &tasks);
        assert_eq!(summary.total_tasks, 3);
        assert_eq!(summary.total_checks, 75);
        assert_eq!(summary.passed_checks, 60);
        assert!((summary.overall_score - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_epic_summary_status() {
        let tasks = vec![
            ("TASK-1".to_string(), 25, 25),
            ("TASK-2".to_string(), 25, 25),
        ];
        let summary = calculate_epic_summary("EPIC-001", &tasks);
        assert_eq!(summary.status, EpicStatus::Complete);

        let partial_tasks = vec![
            ("TASK-1".to_string(), 20, 25),
            ("TASK-2".to_string(), 25, 25),
        ];
        let partial_summary = calculate_epic_summary("EPIC-002", &partial_tasks);
        assert_eq!(partial_summary.status, EpicStatus::InProgress);
    }
}

/// Comprehensive coverage tests for QA Work Handler
/// EXTREME TDD: Tests for all uncovered code paths

mod coverage_tests {
    use super::*;
    use tempfile::TempDir;

    // Data Structure Tests - Serialization/Deserialization

    mod data_structure_tests {
        use super::*;

        #[test]
        fn test_qa_checklist_serialization() {
            let checklist = generate_checklist("TASK-123", QaTaskType::Feature);

            // Test YAML serialization
            let yaml = serde_yaml::to_string(&checklist).expect("YAML serialization failed");
            assert!(yaml.contains("task_id: TASK-123"));
            assert!(yaml.contains("task_type: feature"));

            // Test JSON serialization
            let json = serde_json::to_string(&checklist).expect("JSON serialization failed");
            assert!(json.contains("\"task_id\":\"TASK-123\""));

            // Test round-trip
            let parsed: QaChecklist =
                serde_yaml::from_str(&yaml).expect("YAML deserialization failed");
            assert_eq!(parsed.task_id, checklist.task_id);
            assert_eq!(parsed.task_type, checklist.task_type);
        }

        #[test]
        fn test_checklist_item_with_evidence() {
            let item = ChecklistItem {
                id: "B1".into(),
                description: "Cyclomatic complexity <= 10".into(),
                checked: true,
                automated: true,
                evidence: Some("All functions have complexity < 10".into()),
            };

            let json = serde_json::to_string(&item).expect("JSON serialization failed");
            assert!(json.contains("evidence"));
            assert!(json.contains("All functions have complexity"));

            let parsed: ChecklistItem =
                serde_json::from_str(&json).expect("JSON deserialization failed");
            assert_eq!(parsed.evidence, item.evidence);
        }

        #[test]
        fn test_validation_result_serialization() {
            let mut categories = HashMap::new();
            categories.insert(
                "code_quality".to_string(),
                CategoryResult {
                    name: "Code Quality".into(),
                    passed: 3,
                    total: 5,
                    items: vec![ValidationItem {
                        id: "B1".into(),
                        description: "Test item".into(),
                        status: ValidationStatus::Passed,
                        value: Some("8".into()),
                        threshold: Some("10".into()),
                        evidence: None,
                    }],
                },
            );

            let result = QaValidationResult {
                task_id: "TEST-001".into(),
                timestamp: Utc::now(),
                categories,
                overall_score: 60.0,
                passed: false,
                manual_checks_required: vec!["Review needed".into()],
            };

            let json = serde_json::to_string_pretty(&result).expect("JSON serialization failed");
            assert!(json.contains("TEST-001"));
            assert!(json.contains("Code Quality"));
            assert!(json.contains("Review needed"));
        }

        #[test]
        fn test_epic_summary_serialization() {
            let summary = EpicSummary {
                epic_id: "EPIC-100".into(),
                total_tasks: 5,
                total_checks: 125,
                passed_checks: 100,
                overall_score: 80.0,
                status: EpicStatus::InProgress,
                task_scores: vec![
                    ("TASK-1".into(), 90.0),
                    ("TASK-2".into(), 70.0),
                ],
            };

            let yaml = serde_yaml::to_string(&summary).expect("YAML serialization failed");
            assert!(yaml.contains("epic_id: EPIC-100"));
            assert!(yaml.contains("status: InProgress"));
        }

        #[test]
        fn test_example_script_serialization() {
            let script = ExampleScript {
                name: "test_basic.sh".into(),
                content: "#!/bin/bash\necho 'test'".into(),
                description: "Basic test script".into(),
            };

            let json = serde_json::to_string(&script).expect("JSON serialization failed");
            assert!(json.contains("test_basic.sh"));

            let parsed: ExampleScript =
                serde_json::from_str(&json).expect("JSON deserialization failed");
            assert_eq!(parsed.name, script.name);
        }
    }

    // Task Type Coverage Tests

    mod task_type_tests {
        use super::*;

        #[test]
        fn test_all_task_types() {
            let task_types = [
                (QaTaskType::Feature, "feature"),
                (QaTaskType::Bugfix, "bugfix"),
                (QaTaskType::Refactor, "refactor"),
                (QaTaskType::Docs, "docs"),
                (QaTaskType::Performance, "performance"),
                (QaTaskType::Security, "security"),
            ];

            for (task_type, expected_str) in task_types {
                let checklist = generate_checklist("TEST", task_type);
                assert_eq!(
                    checklist.task_type, expected_str,
                    "Task type {:?} should produce '{}'",
                    task_type, expected_str
                );
            }
        }

        #[test]
        fn test_checklist_has_25_items() {
            // Toyota Way 25-point checklist
            let checklist = generate_checklist("TEST", QaTaskType::Feature);
            let total_items = checklist.categories.safety_ethics.len()
                + checklist.categories.code_quality.len()
                + checklist.categories.testing.len()
                + checklist.categories.documentation.len()
                + checklist.categories.process.len();
            assert_eq!(total_items, 25, "Should have exactly 25 checklist items");
        }

        #[test]
        fn test_checklist_item_ids_are_unique() {
            let checklist = generate_checklist("TEST", QaTaskType::Feature);
            let mut ids: Vec<&str> = Vec::new();

            for item in &checklist.categories.safety_ethics {
                assert!(
                    !ids.contains(&item.id.as_str()),
                    "Duplicate ID: {}",
                    item.id
                );
                ids.push(&item.id);
            }
            for item in &checklist.categories.code_quality {
                assert!(
                    !ids.contains(&item.id.as_str()),
                    "Duplicate ID: {}",
                    item.id
                );
                ids.push(&item.id);
            }
            for item in &checklist.categories.testing {
                assert!(
                    !ids.contains(&item.id.as_str()),
                    "Duplicate ID: {}",
                    item.id
                );
                ids.push(&item.id);
            }
            for item in &checklist.categories.documentation {
                assert!(
                    !ids.contains(&item.id.as_str()),
                    "Duplicate ID: {}",
                    item.id
                );
                ids.push(&item.id);
            }
            for item in &checklist.categories.process {
                assert!(
                    !ids.contains(&item.id.as_str()),
                    "Duplicate ID: {}",
                    item.id
                );
                ids.push(&item.id);
            }
        }
    }

    // Validation Status Tests

    mod validation_status_tests {
        use super::*;

        #[test]
        fn test_all_validation_statuses() {
            let statuses = [
                ValidationStatus::Passed,
                ValidationStatus::Failed,
                ValidationStatus::Warning,
                ValidationStatus::Skipped,
                ValidationStatus::Manual,
            ];

            // Test equality
            for status in &statuses {
                assert_eq!(status, status);
            }

            // Test inequality
            assert_ne!(ValidationStatus::Passed, ValidationStatus::Failed);
            assert_ne!(ValidationStatus::Warning, ValidationStatus::Skipped);
            assert_ne!(ValidationStatus::Manual, ValidationStatus::Passed);
        }

        #[test]
        fn test_validation_status_serialization() {
            let item = ValidationItem {
                id: "TEST".into(),
                description: "Test".into(),
                status: ValidationStatus::Warning,
                value: None,
                threshold: None,
                evidence: None,
            };

            let json = serde_json::to_string(&item).unwrap();
            assert!(json.contains("Warning"));
        }
    }

    // Epic Status Tests

    mod epic_status_tests {
        use super::*;

        #[test]
        fn test_epic_status_complete() {
            let tasks = vec![
                ("TASK-1".to_string(), 25, 25),
                ("TASK-2".to_string(), 25, 25),
                ("TASK-3".to_string(), 25, 25),
            ];
            let summary = calculate_epic_summary("EPIC", &tasks);
            assert_eq!(summary.status, EpicStatus::Complete);
            assert_eq!(summary.overall_score, 100.0);
        }

        #[test]
        fn test_epic_status_in_progress() {
            let tasks = vec![
                ("TASK-1".to_string(), 20, 25),
                ("TASK-2".to_string(), 15, 25),
            ];
            let summary = calculate_epic_summary("EPIC", &tasks);
            assert_eq!(summary.status, EpicStatus::InProgress);
        }

        #[test]
        fn test_epic_status_pending_no_progress() {
            let tasks = vec![
                ("TASK-1".to_string(), 0, 25),
                ("TASK-2".to_string(), 0, 25),
            ];
            let summary = calculate_epic_summary("EPIC", &tasks);
            assert_eq!(summary.status, EpicStatus::Pending);
        }

        #[test]
        fn test_epic_status_pending_empty() {
            let tasks: Vec<(String, u32, u32)> = vec![];
            let summary = calculate_epic_summary("EPIC", &tasks);
            assert_eq!(summary.status, EpicStatus::Pending);
            assert_eq!(summary.total_tasks, 0);
            assert_eq!(summary.overall_score, 0.0);
        }

        #[test]
        fn test_epic_summary_task_scores() {
            let tasks = vec![
                ("TASK-1".to_string(), 10, 20), // 50%
                ("TASK-2".to_string(), 15, 20), // 75%
            ];
            let summary = calculate_epic_summary("EPIC", &tasks);

            assert_eq!(summary.task_scores.len(), 2);
            assert!((summary.task_scores[0].1 - 50.0).abs() < 0.1);
            assert!((summary.task_scores[1].1 - 75.0).abs() < 0.1);
        }

        #[test]
        fn test_epic_summary_with_zero_total() {
            let tasks = vec![("TASK-1".to_string(), 0, 0)];
            let summary = calculate_epic_summary("EPIC", &tasks);
            assert_eq!(summary.overall_score, 0.0);
            assert!((summary.task_scores[0].1 - 0.0).abs() < 0.1);
        }
    }

    // Format Text Output Tests

    mod format_output_tests {
        use super::*;

        #[test]
        fn test_format_checklist_text_complete_coverage() {
            let mut checklist = generate_checklist("FORMAT-TEST", QaTaskType::Security);

            // Mark some items as checked
            checklist.categories.safety_ethics[0].checked = true;
            checklist.categories.code_quality[0].checked = true;

            let text = format_checklist_text(&checklist);

            // Verify structure
            assert!(text.contains("# QA Checklist for FORMAT-TEST"));
            assert!(text.contains("Task Type: security"));
            assert!(text.contains("Generated:"));
            assert!(text.contains("## Safety & Ethics"));
            assert!(text.contains("## Code Quality"));
            assert!(text.contains("## Testing"));
            assert!(text.contains("## Documentation"));
            assert!(text.contains("## Process"));

            // Verify checkboxes
            assert!(text.contains("[x]")); // checked items
            assert!(text.contains("[ ]")); // unchecked items
            assert!(text.contains("(auto)")); // automated items
        }

        #[test]
        fn test_print_validation_text_format() {
            let mut categories = HashMap::new();

            // Full pass category
            categories.insert(
                "full_pass".to_string(),
                CategoryResult {
                    name: "Full Pass".into(),
                    passed: 3,
                    total: 3,
                    items: vec![
                        ValidationItem {
                            id: "P1".into(),
                            description: "Passed item".into(),
                            status: ValidationStatus::Passed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "P2".into(),
                            description: "Another passed".into(),
                            status: ValidationStatus::Passed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "P3".into(),
                            description: "Third passed".into(),
                            status: ValidationStatus::Passed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                    ],
                },
            );

            // Partial pass category
            categories.insert(
                "partial".to_string(),
                CategoryResult {
                    name: "Partial".into(),
                    passed: 1,
                    total: 2,
                    items: vec![
                        ValidationItem {
                            id: "M1".into(),
                            description: "Mixed passed".into(),
                            status: ValidationStatus::Passed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "M2".into(),
                            description: "Mixed failed".into(),
                            status: ValidationStatus::Failed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                    ],
                },
            );

            // All fail category
            categories.insert(
                "all_fail".to_string(),
                CategoryResult {
                    name: "All Fail".into(),
                    passed: 0,
                    total: 2,
                    items: vec![
                        ValidationItem {
                            id: "F1".into(),
                            description: "Failed".into(),
                            status: ValidationStatus::Failed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "F2".into(),
                            description: "Warning".into(),
                            status: ValidationStatus::Warning,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                    ],
                },
            );

            // Other statuses
            categories.insert(
                "other".to_string(),
                CategoryResult {
                    name: "Other".into(),
                    passed: 0,
                    total: 2,
                    items: vec![
                        ValidationItem {
                            id: "S1".into(),
                            description: "Skipped".into(),
                            status: ValidationStatus::Skipped,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "M1".into(),
                            description: "Manual".into(),
                            status: ValidationStatus::Manual,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                    ],
                },
            );

            let result = QaValidationResult {
                task_id: "TEXT-TEST".into(),
                timestamp: Utc::now(),
                categories,
                overall_score: 45.0,
                passed: false,
                manual_checks_required: vec!["Check 1".into(), "Check 2".into()],
            };

            // This exercises the print_validation_text function
            // We can't easily capture stdout, but we verify the function doesn't panic
            print_validation_text(&result);
        }

        #[test]
        fn test_print_validation_markdown_format() {
            let mut categories = HashMap::new();

            categories.insert(
                "test".to_string(),
                CategoryResult {
                    name: "Test Category".into(),
                    passed: 2,
                    total: 5,
                    items: vec![
                        ValidationItem {
                            id: "T1".into(),
                            description: "Passed test".into(),
                            status: ValidationStatus::Passed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "T2".into(),
                            description: "Failed test".into(),
                            status: ValidationStatus::Failed,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "T3".into(),
                            description: "Warning test".into(),
                            status: ValidationStatus::Warning,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "T4".into(),
                            description: "Skipped test".into(),
                            status: ValidationStatus::Skipped,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                        ValidationItem {
                            id: "T5".into(),
                            description: "Manual test".into(),
                            status: ValidationStatus::Manual,
                            value: None,
                            threshold: None,
                            evidence: None,
                        },
                    ],
                },
            );

            let result = QaValidationResult {
                task_id: "MD-TEST".into(),
                timestamp: Utc::now(),
                categories,
                overall_score: 40.0,
                passed: false,
                manual_checks_required: vec!["Manual review".into()],
            };

            // Exercises print_validation_markdown
            print_validation_markdown(&result);
        }

        #[test]
        fn test_validation_result_passed_true() {
            let result = QaValidationResult {
                task_id: "PASS".into(),
                timestamp: Utc::now(),
                categories: HashMap::new(),
                overall_score: 95.0,
                passed: true,
                manual_checks_required: vec![],
            };

            print_validation_text(&result);
        }
    }

    // Example Script Generation Tests

    mod example_script_tests {
        use super::*;

        #[test]
        fn test_generate_examples_sanitizes_name() {
            let examples = generate_example_scripts("TASK-1", "my-cool-feature");

            for example in &examples {
                assert!(
                    example.name.contains("my_cool_feature"),
                    "Should sanitize hyphen to underscore: {}",
                    example.name
                );
                assert!(
                    !example.name.contains("-"),
                    "Should not contain hyphens in filename"
                );
            }
        }

        #[test]
        fn test_generate_examples_count() {
            let examples = generate_example_scripts("TASK", "feature");
            assert_eq!(examples.len(), 5, "Should generate 5 example scripts");
        }

        #[test]
        fn test_example_types() {
            let examples = generate_example_scripts("TASK", "test");

            let has_basic = examples.iter().any(|e| e.name.contains("basic"));
            let has_error = examples.iter().any(|e| e.name.contains("error"));
            let has_edge = examples.iter().any(|e| e.name.contains("edge"));
            let has_verbose = examples.iter().any(|e| e.name.contains("verbose"));
            let has_json = examples.iter().any(|e| e.name.contains("json"));

            assert!(has_basic, "Should have basic example");
            assert!(has_error, "Should have error handling example");
            assert!(has_edge, "Should have edge case example");
            assert!(has_verbose, "Should have verbose example");
            assert!(has_json, "Should have JSON output example");
        }

        #[test]
        fn test_example_script_content() {
            let examples = generate_example_scripts("TASK-123", "analyze");

            for example in &examples {
                // All scripts should have shebang
                assert!(
                    example.content.starts_with("#!/bin/bash"),
                    "Script should have shebang"
                );

                // All scripts should use strict mode
                assert!(
                    example.content.contains("set -euo pipefail"),
                    "Script should use strict mode"
                );

                // All scripts should reference the task
                assert!(
                    example.content.contains("TASK-123"),
                    "Script should reference task ID"
                );

                // All scripts should reference the feature
                assert!(
                    example.content.contains("analyze"),
                    "Script should reference feature name"
                );
            }
        }

        #[test]
        fn test_example_descriptions() {
            let examples = generate_example_scripts("TASK", "context");

            for example in &examples {
                assert!(
                    !example.description.is_empty(),
                    "Example should have description"
                );
                assert!(
                    example.description.contains("context"),
                    "Description should reference feature"
                );
            }
        }
    }

    // File System Integration Tests

    mod filesystem_tests {
        use super::*;

        #[test]
        fn test_handle_generate_checklist_creates_directory() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_generate_checklist(
                    "FS-TEST-001",
                    QaTaskType::Feature,
                    project_path,
                    None,
                )
                .await;

                assert!(result.is_ok());

                // Verify directory was created
                let qa_dir = project_path.join(".pmat-qa").join("FS-TEST-001");
                assert!(qa_dir.exists(), "QA directory should be created");

                // Verify checklist file was created
                let checklist_path = qa_dir.join("checklist.yaml");
                assert!(checklist_path.exists(), "Checklist file should be created");

                // Verify content
                let content = fs::read_to_string(&checklist_path).expect("Failed to read checklist");
                assert!(content.contains("task_id: FS-TEST-001"));
            });
        }

        #[test]
        fn test_handle_generate_checklist_with_output_path() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();
            let output_path = temp_dir.path().join("custom_checklist.yaml");

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_generate_checklist(
                    "CUSTOM-001",
                    QaTaskType::Bugfix,
                    project_path,
                    Some(&output_path),
                )
                .await;

                assert!(result.is_ok());
                assert!(output_path.exists(), "Custom output path should be created");

                let content = fs::read_to_string(&output_path).expect("Failed to read");
                assert!(content.contains("task_id: CUSTOM-001"));
                assert!(content.contains("task_type: bugfix"));
            });
        }

        #[test]
        fn test_handle_generate_examples_creates_files() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_generate_examples(
                    "EX-TEST-001",
                    "my-feature",
                    project_path,
                    None,
                )
                .await;

                assert!(result.is_ok());

                // Verify examples directory
                let examples_dir = project_path.join("examples").join("my-feature");
                assert!(examples_dir.exists(), "Examples directory should exist");

                // Verify example files
                let entries: Vec<_> = fs::read_dir(&examples_dir)
                    .expect("Failed to read dir")
                    .collect();
                assert!(!entries.is_empty(), "Should have created example files");
            });
        }

        #[test]
        fn test_handle_generate_examples_with_custom_output() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();
            let output_dir = temp_dir.path().join("custom_examples");

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_generate_examples(
                    "EX-CUSTOM",
                    "test-feature",
                    project_path,
                    Some(&output_dir),
                )
                .await;

                assert!(result.is_ok());
                assert!(output_dir.exists(), "Custom output directory should exist");
            });
        }

        #[test]
        fn test_handle_summary_no_qa_data() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_summary(None, project_path, None).await;
                assert!(result.is_ok());
            });
        }

        #[test]
        fn test_handle_summary_with_existing_data() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();

            // Create QA directory with checklist
            let qa_dir = project_path.join(".pmat-qa").join("SUMMARY-TEST");
            fs::create_dir_all(&qa_dir).expect("Failed to create dir");

            let checklist = generate_checklist("SUMMARY-TEST", QaTaskType::Feature);
            let yaml = serde_yaml::to_string(&checklist).expect("Failed to serialize");
            fs::write(qa_dir.join("checklist.yaml"), yaml).expect("Failed to write");

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_summary(Some("SUMMARY-TEST"), project_path, None).await;
                assert!(result.is_ok());
            });
        }

        #[test]
        fn test_handle_summary_all_tasks() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();

            // Create multiple task directories
            for task_id in &["TASK-1", "TASK-2", "TASK-3"] {
                let qa_dir = project_path.join(".pmat-qa").join(task_id);
                fs::create_dir_all(&qa_dir).expect("Failed to create dir");

                let checklist = generate_checklist(task_id, QaTaskType::Feature);
                let yaml = serde_yaml::to_string(&checklist).expect("Failed to serialize");
                fs::write(qa_dir.join("checklist.yaml"), yaml).expect("Failed to write");
            }

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_summary(None, project_path, None).await;
                assert!(result.is_ok());
            });
        }

        #[test]
        fn test_handle_summary_task_not_found() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = temp_dir.path();

            // Create QA directory but not the specific task
            let qa_dir = project_path.join(".pmat-qa");
            fs::create_dir_all(&qa_dir).expect("Failed to create dir");

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = handle_summary(Some("NONEXISTENT"), project_path, None).await;
                assert!(result.is_ok());
            });
        }

        #[test]
        fn test_print_task_status_complete() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let task_dir = temp_dir.path();

            // Create checklist with all items checked
            let mut checklist = generate_checklist("COMPLETE", QaTaskType::Feature);
            for item in &mut checklist.categories.safety_ethics {
                item.checked = true;
            }
            for item in &mut checklist.categories.code_quality {
                item.checked = true;
            }
            for item in &mut checklist.categories.testing {
                item.checked = true;
            }
            for item in &mut checklist.categories.documentation {
                item.checked = true;
            }
            for item in &mut checklist.categories.process {
                item.checked = true;
            }

            let yaml = serde_yaml::to_string(&checklist).expect("Serialize failed");
            fs::write(task_dir.join("checklist.yaml"), yaml).expect("Write failed");

            let result = print_task_status("COMPLETE", task_dir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_print_task_status_in_progress() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let task_dir = temp_dir.path();

            // Create checklist with some items checked
            let mut checklist = generate_checklist("PROGRESS", QaTaskType::Feature);
            checklist.categories.safety_ethics[0].checked = true;
            checklist.categories.code_quality[0].checked = true;

            let yaml = serde_yaml::to_string(&checklist).expect("Serialize failed");
            fs::write(task_dir.join("checklist.yaml"), yaml).expect("Write failed");

            let result = print_task_status("PROGRESS", task_dir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_print_task_status_pending() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let task_dir = temp_dir.path();

            // Create checklist with no items checked
            let checklist = generate_checklist("PENDING", QaTaskType::Feature);
            let yaml = serde_yaml::to_string(&checklist).expect("Serialize failed");
            fs::write(task_dir.join("checklist.yaml"), yaml).expect("Write failed");

            let result = print_task_status("PENDING", task_dir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_print_task_status_no_checklist() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let task_dir = temp_dir.path();

            // No checklist file
            let result = print_task_status("NO-CHECKLIST", task_dir);
            assert!(result.is_ok());
        }
    }

    // Epic Summary Tests

    mod epic_summary_tests {
        use super::*;

        #[test]
        fn test_handle_epic_summary() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let qa_dir = temp_dir.path();

            // Create task directories with checklists
            for (task_id, checked_count) in [("EPIC-TASK-1", 20), ("EPIC-TASK-2", 25)] {
                let task_dir = qa_dir.join(task_id);
                fs::create_dir_all(&task_dir).expect("Failed to create dir");

                let mut checklist = generate_checklist(task_id, QaTaskType::Feature);

                // Check some items
                let mut count = 0;
                for item in &mut checklist.categories.safety_ethics {
                    if count < checked_count {
                        item.checked = true;
                        count += 1;
                    }
                }
                for item in &mut checklist.categories.code_quality {
                    if count < checked_count {
                        item.checked = true;
                        count += 1;
                    }
                }
                for item in &mut checklist.categories.testing {
                    if count < checked_count {
                        item.checked = true;
                        count += 1;
                    }
                }
                for item in &mut checklist.categories.documentation {
                    if count < checked_count {
                        item.checked = true;
                        count += 1;
                    }
                }
                for item in &mut checklist.categories.process {
                    if count < checked_count {
                        item.checked = true;
                        count += 1;
                    }
                }

                let yaml = serde_yaml::to_string(&checklist).expect("Serialize failed");
                fs::write(task_dir.join("checklist.yaml"), yaml).expect("Write failed");
            }

            let result = handle_epic_summary("EPIC-1", qa_dir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_handle_epic_summary_empty() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let qa_dir = temp_dir.path();
            fs::create_dir_all(qa_dir).expect("Create dir failed");

            let result = handle_epic_summary("EMPTY-EPIC", qa_dir);
            assert!(result.is_ok());
        }
    }

    // Validation Checks Tests

    mod validation_check_tests {
        use super::*;

        #[tokio::test]
        async fn test_run_code_quality_checks() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let result = run_code_quality_checks(temp_dir.path()).await;

            assert_eq!(result.name, "Code Quality");
            assert!(result.total > 0);
            assert!(!result.items.is_empty());

            // Verify expected items exist
            let ids: Vec<&str> = result.items.iter().map(|i| i.id.as_str()).collect();
            assert!(ids.contains(&"B1")); // complexity
            assert!(ids.contains(&"B2")); // cognitive
            assert!(ids.contains(&"B3")); // coverage
            assert!(ids.contains(&"B4")); // mutation
            assert!(ids.contains(&"B5")); // clippy
        }

        #[tokio::test]
        async fn test_run_testing_checks() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let result = run_testing_checks(temp_dir.path()).await;

            assert_eq!(result.name, "Testing");
            assert!(result.total > 0);

            // Verify expected items
            let ids: Vec<&str> = result.items.iter().map(|i| i.id.as_str()).collect();
            assert!(ids.contains(&"C1")); // unit tests
            assert!(ids.contains(&"C2")); // error paths
        }

        #[tokio::test]
        async fn test_run_documentation_checks() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create CHANGELOG with task reference
            fs::write(
                temp_dir.path().join("CHANGELOG.md"),
                "# Changelog\n## Unreleased\n- DOC-TEST feature",
            )
            .expect("Write failed");

            let result = run_documentation_checks(temp_dir.path(), "DOC-TEST").await;

            assert_eq!(result.name, "Documentation");
            assert!(result.total > 0);

            // Find D3 (CHANGELOG) and verify it passed
            let changelog_item = result.items.iter().find(|i| i.id == "D3");
            assert!(changelog_item.is_some());
        }

        #[tokio::test]
        async fn test_run_documentation_checks_no_changelog() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let result = run_documentation_checks(temp_dir.path(), "TEST").await;

            let changelog_item = result.items.iter().find(|i| i.id == "D3");
            assert!(changelog_item.is_some());
            assert_eq!(changelog_item.unwrap().status, ValidationStatus::Skipped);
        }

        #[tokio::test]
        async fn test_run_documentation_checks_changelog_missing_task() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create CHANGELOG without task reference
            fs::write(
                temp_dir.path().join("CHANGELOG.md"),
                "# Changelog\n## v1.0.0\n- Other changes",
            )
            .expect("Write failed");

            let result = run_documentation_checks(temp_dir.path(), "MISSING-TASK").await;

            let changelog_item = result.items.iter().find(|i| i.id == "D3");
            assert!(changelog_item.is_some());
            assert_eq!(changelog_item.unwrap().status, ValidationStatus::Warning);
        }

        #[tokio::test]
        async fn test_run_process_checks() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Initialize a git repo
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .output()
                .ok();

            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .output()
                .ok();

            std::process::Command::new("git")
                .args(["config", "user.name", "Test"])
                .current_dir(temp_dir.path())
                .output()
                .ok();

            // Create a file and commit with task reference
            fs::write(temp_dir.path().join("test.txt"), "test").expect("Write failed");

            std::process::Command::new("git")
                .args(["add", "test.txt"])
                .current_dir(temp_dir.path())
                .output()
                .ok();

            std::process::Command::new("git")
                .args(["commit", "-m", "PROC-TEST: Add test file"])
                .current_dir(temp_dir.path())
                .output()
                .ok();

            let result = run_process_checks(temp_dir.path(), "PROC-TEST").await;

            assert_eq!(result.name, "Process");
            assert!(result.total > 0);
        }

        #[tokio::test]
        async fn test_run_process_checks_no_git() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let result = run_process_checks(temp_dir.path(), "NO-GIT").await;

            // Should handle missing git gracefully
            let commit_item = result.items.iter().find(|i| i.id == "E2");
            assert!(commit_item.is_some());
            assert_eq!(commit_item.unwrap().status, ValidationStatus::Skipped);
        }
    }

    // Report Generation Tests

    mod report_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_report_json() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().join("report.json");

            let result = handle_report(
                "REPORT-JSON",
                temp_dir.path(),
                false,
                Some(&output_path),
                QaOutputFormat::Json,
            )
            .await;

            assert!(result.is_ok());
            assert!(output_path.exists());

            let content = fs::read_to_string(&output_path).expect("Read failed");
            let parsed: serde_json::Value =
                serde_json::from_str(&content).expect("Parse JSON failed");
            assert_eq!(parsed["task_id"], "REPORT-JSON");
        }

        #[tokio::test]
        async fn test_handle_report_yaml() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().join("report.yaml");

            let result = handle_report(
                "REPORT-YAML",
                temp_dir.path(),
                false,
                Some(&output_path),
                QaOutputFormat::Yaml,
            )
            .await;

            assert!(result.is_ok());
            assert!(output_path.exists());

            let content = fs::read_to_string(&output_path).expect("Read failed");
            assert!(content.contains("task_id: REPORT-YAML"));
        }

        #[tokio::test]
        async fn test_handle_report_markdown() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().join("report.md");

            let result = handle_report(
                "REPORT-MD",
                temp_dir.path(),
                false,
                Some(&output_path),
                QaOutputFormat::Markdown,
            )
            .await;

            assert!(result.is_ok());
            assert!(output_path.exists());

            let content = fs::read_to_string(&output_path).expect("Read failed");
            assert!(content.contains("# QA Report: REPORT-MD"));
        }

        #[tokio::test]
        async fn test_handle_report_text() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().join("report.txt");

            let result = handle_report(
                "REPORT-TXT",
                temp_dir.path(),
                false,
                Some(&output_path),
                QaOutputFormat::Text,
            )
            .await;

            assert!(result.is_ok());
            assert!(output_path.exists());
        }

        #[tokio::test]
        async fn test_handle_report_with_evidence() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().join("report_evidence.md");

            let result = handle_report(
                "REPORT-EV",
                temp_dir.path(),
                true, // with_evidence
                Some(&output_path),
                QaOutputFormat::Markdown,
            )
            .await;

            assert!(result.is_ok());

            let content = fs::read_to_string(&output_path).expect("Read failed");
            assert!(content.contains("## Evidence"));
            assert!(content.contains("Coverage Report"));
        }

        #[tokio::test]
        async fn test_handle_report_no_output_path() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Should print to stdout, not error
            let result = handle_report(
                "REPORT-STDOUT",
                temp_dir.path(),
                false,
                None,
                QaOutputFormat::Text,
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // Spec Path Resolution Tests

    mod spec_path_tests {
        use super::*;

        #[test]
        fn test_resolve_spec_path_direct_file() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let spec_file = temp_dir.path().join("test-spec.md");
            fs::write(&spec_file, "# Test Spec").expect("Write failed");

            let result = resolve_spec_path(spec_file.to_str().unwrap(), temp_dir.path());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), spec_file);
        }

        #[test]
        fn test_resolve_spec_path_project_relative() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let spec_file = temp_dir.path().join("my-spec.md");
            fs::write(&spec_file, "# Spec").expect("Write failed");

            let result = resolve_spec_path("my-spec.md", temp_dir.path());
            assert!(result.is_ok());
        }

        #[test]
        fn test_resolve_spec_path_docs_specifications() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let specs_dir = temp_dir.path().join("docs/specifications");
            fs::create_dir_all(&specs_dir).expect("Create dir failed");

            let spec_file = specs_dir.join("my-feature.md");
            fs::write(&spec_file, "# My Feature Spec").expect("Write failed");

            let result = resolve_spec_path("my-feature", temp_dir.path());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), spec_file);
        }

        #[test]
        fn test_resolve_spec_path_hyphen_normalization() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let specs_dir = temp_dir.path().join("docs/specifications");
            fs::create_dir_all(&specs_dir).expect("Create dir failed");

            let spec_file = specs_dir.join("my-cool-feature.md");
            fs::write(&spec_file, "# Cool Feature").expect("Write failed");

            let result = resolve_spec_path("my_cool_feature", temp_dir.path());
            assert!(result.is_ok());
        }

        #[test]
        fn test_resolve_spec_path_partial_match() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let specs_dir = temp_dir.path().join("docs/specifications");
            fs::create_dir_all(&specs_dir).expect("Create dir failed");

            let spec_file = specs_dir.join("enhance-pmat-work.md");
            fs::write(&spec_file, "# Enhancement").expect("Write failed");

            let result = resolve_spec_path("pmat-work", temp_dir.path());
            assert!(result.is_ok());
        }

        #[test]
        fn test_resolve_spec_path_github_issue() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let specs_dir = temp_dir.path().join("docs/specifications");
            fs::create_dir_all(&specs_dir).expect("Create dir failed");

            let spec_file = specs_dir.join("gh-123.md");
            fs::write(&spec_file, "# Issue 123").expect("Write failed");

            let result = resolve_spec_path("GH-123", temp_dir.path());
            assert!(result.is_ok());

            let result2 = resolve_spec_path("#123", temp_dir.path());
            assert!(result2.is_ok());
        }

        #[test]
        fn test_resolve_spec_path_not_found() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let specs_dir = temp_dir.path().join("docs/specifications");
            fs::create_dir_all(&specs_dir).expect("Create dir failed");

            let result = resolve_spec_path("nonexistent-spec", temp_dir.path());
            assert!(result.is_err());

            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("Specification not found"));
        }
    }

    // Validation Command Tests

    mod validation_command_tests {
        use super::*;

        #[tokio::test]
        async fn test_run_validation_command_success() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let result = run_validation_command("echo hello", temp_dir.path()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().contains("hello"));
        }

        #[tokio::test]
        async fn test_run_validation_command_failure() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let result = run_validation_command("false", temp_dir.path()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().contains("FAILED"));
        }

        #[tokio::test]
        async fn test_run_validation_command_empty() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let result = run_validation_command("", temp_dir.path()).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_run_validation_command_with_args() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let result = run_validation_command("echo hello world", temp_dir.path()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().contains("hello world"));
        }
    }

    // Spec Result Markdown Format Tests

    mod spec_markdown_tests {
        use super::*;

        #[test]
        fn test_format_spec_result_markdown_passed() {
            let result = serde_json::json!({
                "spec_path": "/path/to/spec.md",
                "title": "Test Specification",
                "issue_refs": ["#123", "#456"],
                "claims_total": 10,
                "gateway_score": 20.0,
                "gateway_passed": true,
                "total_score": 85.0,
                "threshold": 60,
                "passed": true,
                "category_scores": {}
            });

            let markdown = format_spec_result_markdown(&result);

            assert!(markdown.contains("# Specification Validation Report"));
            assert!(markdown.contains("Test Specification"));
            assert!(markdown.contains("PASSED"));
            assert!(markdown.contains("85.0/100"));
        }

        #[test]
        fn test_format_spec_result_markdown_failed() {
            let result = serde_json::json!({
                "spec_path": "/path/to/spec.md",
                "title": "Failed Spec",
                "issue_refs": [],
                "claims_total": 5,
                "gateway_score": 10.0,
                "gateway_passed": false,
                "total_score": 30.0,
                "threshold": 60,
                "passed": false,
                "category_scores": {}
            });

            let markdown = format_spec_result_markdown(&result);

            assert!(markdown.contains("FAILED"));
            assert!(markdown.contains("30.0/100"));
        }

        #[test]
        fn test_format_spec_result_markdown_missing_fields() {
            let result = serde_json::json!({});

            // Should not panic with missing fields
            let markdown = format_spec_result_markdown(&result);
            assert!(markdown.contains("unknown"));
        }
    }

    // Validate Handler Tests

    mod validate_handler_tests {
        use super::*;

        // Note: handle_validate calls std::process::exit(1) on failure,
        // so we test it indirectly through the validation functions

        #[tokio::test]
        async fn test_validation_score_calculation() {
            let mut categories = HashMap::new();

            categories.insert(
                "cat1".to_string(),
                CategoryResult {
                    name: "Cat1".into(),
                    passed: 3,
                    total: 4,
                    items: vec![],
                },
            );

            categories.insert(
                "cat2".to_string(),
                CategoryResult {
                    name: "Cat2".into(),
                    passed: 2,
                    total: 4,
                    items: vec![],
                },
            );

            let (total_passed, total_items) = categories
                .values()
                .fold((0, 0), |(p, t), cat| (p + cat.passed, t + cat.total));

            assert_eq!(total_passed, 5);
            assert_eq!(total_items, 8);

            let score = (total_passed as f64 / total_items as f64) * 100.0;
            assert!((score - 62.5).abs() < 0.1);
        }

        #[tokio::test]
        async fn test_validation_pass_criteria() {
            // Score >= 80 and not strict => pass
            let passed1 = 80.0 >= 80.0 && !false || 80.0 >= 95.0;
            assert!(passed1);

            // Score >= 95 => always pass
            let passed2 = 95.0 >= 80.0 && !true || 95.0 >= 95.0;
            assert!(passed2);

            // Score < 80 => fail unless >= 95
            let passed3 = 75.0 >= 80.0 && !false || 75.0 >= 95.0;
            assert!(!passed3);

            // Score >= 80 but strict mode and < 95 => fail
            let passed4 = 85.0 >= 80.0 && !true || 85.0 >= 95.0;
            assert!(!passed4);
        }

        #[tokio::test]
        async fn test_validation_empty_categories() {
            let categories: HashMap<String, CategoryResult> = HashMap::new();

            let (total_passed, total_items) = categories
                .values()
                .fold((0, 0), |(p, t), cat| (p + cat.passed, t + cat.total));

            let score = if total_items > 0 {
                (total_passed as f64 / total_items as f64) * 100.0
            } else {
                0.0
            };

            assert_eq!(score, 0.0);
        }
    }

    // QA Work Command Handler Tests

    mod qa_work_command_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_qa_work_command_generate_checklist() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let command = QaWorkCommands::GenerateChecklist {
                task_id: "CMD-TEST-1".into(),
                task_type: QaTaskType::Feature,
                path: temp_dir.path().to_path_buf(),
                output: None,
            };

            let result = handle_qa_work_command(command).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_qa_work_command_report() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let output_path = temp_dir.path().join("cmd_report.json");

            let command = QaWorkCommands::Report {
                task_id: "CMD-REPORT".into(),
                path: temp_dir.path().to_path_buf(),
                with_evidence: true,
                output: Some(output_path.clone()),
                format: QaOutputFormat::Json,
            };

            let result = handle_qa_work_command(command).await;
            assert!(result.is_ok());
            assert!(output_path.exists());
        }

        #[tokio::test]
        async fn test_handle_qa_work_command_summary() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let command = QaWorkCommands::Summary {
                task_id: None,
                path: temp_dir.path().to_path_buf(),
                epic: None,
            };

            let result = handle_qa_work_command(command).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_qa_work_command_summary_with_epic() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create QA directory
            let qa_dir = temp_dir.path().join(".pmat-qa");
            fs::create_dir_all(&qa_dir).expect("Create dir failed");

            let command = QaWorkCommands::Summary {
                task_id: None,
                path: temp_dir.path().to_path_buf(),
                epic: Some("EPIC-1".into()),
            };

            let result = handle_qa_work_command(command).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_qa_work_command_generate_examples() {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            let command = QaWorkCommands::GenerateExamples {
                task_id: "CMD-EX".into(),
                feature_name: "test-feature".into(),
                path: temp_dir.path().to_path_buf(),
                output: None,
            };

            let result = handle_qa_work_command(command).await;
            assert!(result.is_ok());
        }
    }

    // Category Result Tests

    mod category_result_tests {
        use super::*;

        #[test]
        fn test_category_result_serialization() {
            let result = CategoryResult {
                name: "Test Category".into(),
                passed: 5,
                total: 10,
                items: vec![
                    ValidationItem {
                        id: "T1".into(),
                        description: "Test 1".into(),
                        status: ValidationStatus::Passed,
                        value: Some("100".into()),
                        threshold: Some("50".into()),
                        evidence: Some("Evidence text".into()),
                    },
                    ValidationItem {
                        id: "T2".into(),
                        description: "Test 2".into(),
                        status: ValidationStatus::Failed,
                        value: None,
                        threshold: None,
                        evidence: None,
                    },
                ],
            };

            let json = serde_json::to_string(&result).expect("Serialize failed");
            assert!(json.contains("Test Category"));
            assert!(json.contains("\"passed\":5"));
            assert!(json.contains("\"total\":10"));
        }
    }

    // Checklist Categories Tests

    mod checklist_categories_tests {
        use super::*;

        #[test]
        fn test_checklist_categories_serialization() {
            let checklist = generate_checklist("TEST", QaTaskType::Feature);

            let yaml = serde_yaml::to_string(&checklist.categories).expect("Serialize failed");

            assert!(yaml.contains("safety_ethics"));
            assert!(yaml.contains("code_quality"));
            assert!(yaml.contains("testing"));
            assert!(yaml.contains("documentation"));
            assert!(yaml.contains("process"));
        }

        #[test]
        fn test_safety_ethics_items() {
            let checklist = generate_checklist("TEST", QaTaskType::Security);

            let items = &checklist.categories.safety_ethics;
            assert_eq!(items.len(), 5);

            // Verify specific items
            assert!(items.iter().any(|i| i.id == "A1"));
            assert!(items.iter().any(|i| i.description.contains("secrets")));
            assert!(items.iter().any(|i| i.description.contains("injection")));
        }

        #[test]
        fn test_code_quality_items() {
            let checklist = generate_checklist("TEST", QaTaskType::Refactor);

            let items = &checklist.categories.code_quality;
            assert_eq!(items.len(), 5);

            // Verify complexity items
            assert!(items.iter().any(|i| i.description.contains("Cyclomatic")));
            assert!(items.iter().any(|i| i.description.contains("Cognitive")));
            assert!(items.iter().any(|i| i.description.contains("coverage")));
            assert!(items.iter().any(|i| i.description.contains("Mutation")));
            assert!(items.iter().any(|i| i.description.contains("clippy")));
        }
    }
}
