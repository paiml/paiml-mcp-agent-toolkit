//! QA Work Handler Tests - Part 1
//! Core tests and data structure tests (CB-040 file health compliance)

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
            ("TASK-1".to_string(), 20, 25),
            ("TASK-2".to_string(), 25, 25),
            ("TASK-3".to_string(), 15, 25),
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

mod data_structure_tests {
    use super::*;

    #[test]
    fn test_qa_checklist_serialization() {
        let checklist = generate_checklist("TASK-123", QaTaskType::Feature);

        let yaml = serde_yaml::to_string(&checklist).expect("YAML serialization failed");
        assert!(yaml.contains("task_id: TASK-123"));
        assert!(yaml.contains("task_type: feature"));

        let json = serde_json::to_string(&checklist).expect("JSON serialization failed");
        assert!(json.contains("\"task_id\":\"TASK-123\""));

        let parsed: QaChecklist = serde_yaml::from_str(&yaml).expect("YAML deserialization failed");
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
            task_scores: vec![("TASK-1".into(), 90.0), ("TASK-2".into(), 70.0)],
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

        for status in &statuses {
            assert_eq!(status, status);
        }

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
        let tasks = vec![("TASK-1".to_string(), 0, 25), ("TASK-2".to_string(), 0, 25)];
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
            ("TASK-1".to_string(), 10, 20),
            ("TASK-2".to_string(), 15, 20),
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
