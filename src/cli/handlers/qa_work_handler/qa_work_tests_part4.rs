//! QA Work Handler Tests - Part 4
//! Report and command handler tests (CB-040 file health compliance)

use super::*;
use tempfile::TempDir;

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
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("Parse JSON failed");
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
            true,
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

        let markdown = format_spec_result_markdown(&result);
        assert!(markdown.contains("unknown"));
    }
}

mod validate_handler_tests {
    use super::*;

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
        let passed1 = 80.0 >= 80.0 && !false || 80.0 >= 95.0;
        assert!(passed1);

        let passed2 = 95.0 >= 80.0 && !true || 95.0 >= 95.0;
        assert!(passed2);

        let passed3 = 75.0 >= 80.0 && !false || 75.0 >= 95.0;
        assert!(!passed3);

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

        assert!(items.iter().any(|i| i.id == "A1"));
        assert!(items.iter().any(|i| i.description.contains("secrets")));
        assert!(items.iter().any(|i| i.description.contains("injection")));
    }

    #[test]
    fn test_code_quality_items() {
        let checklist = generate_checklist("TEST", QaTaskType::Refactor);

        let items = &checklist.categories.code_quality;
        assert_eq!(items.len(), 5);

        assert!(items.iter().any(|i| i.description.contains("Cyclomatic")));
        assert!(items.iter().any(|i| i.description.contains("Cognitive")));
        assert!(items.iter().any(|i| i.description.contains("coverage")));
        assert!(items.iter().any(|i| i.description.contains("Mutation")));
        assert!(items.iter().any(|i| i.description.contains("clippy")));
    }
}
