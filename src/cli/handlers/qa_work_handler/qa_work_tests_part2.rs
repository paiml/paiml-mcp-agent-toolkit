//! QA Work Handler Tests - Part 2
//! Format output and example script tests (CB-040 file health compliance)

use super::*;

mod format_output_tests {
    use super::*;

    #[test]
    fn test_format_checklist_text_complete_coverage() {
        let mut checklist = generate_checklist("FORMAT-TEST", QaTaskType::Security);

        checklist.categories.safety_ethics[0].checked = true;
        checklist.categories.code_quality[0].checked = true;

        let text = format_checklist_text(&checklist);

        assert!(text.contains("# QA Checklist for FORMAT-TEST"));
        assert!(text.contains("Task Type: security"));
        assert!(text.contains("Generated:"));
        assert!(text.contains("## Safety & Ethics"));
        assert!(text.contains("## Code Quality"));
        assert!(text.contains("## Testing"));
        assert!(text.contains("## Documentation"));
        assert!(text.contains("## Process"));

        assert!(text.contains("[x]"));
        assert!(text.contains("[ ]"));
        assert!(text.contains("(auto)"));
    }

    #[test]
    fn test_print_validation_text_format() {
        let mut categories = HashMap::new();

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
            assert!(
                example.content.starts_with("#!/bin/bash"),
                "Script should have shebang"
            );

            assert!(
                example.content.contains("set -euo pipefail"),
                "Script should use strict mode"
            );

            assert!(
                example.content.contains("TASK-123"),
                "Script should reference task ID"
            );

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
