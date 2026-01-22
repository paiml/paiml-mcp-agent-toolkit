//! QA Work Handler Tests - Part 3
//! Filesystem and validation tests (CB-040 file health compliance)

use super::*;
use tempfile::TempDir;

mod filesystem_tests {
    use super::*;

    #[test]
    fn test_handle_generate_checklist_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result =
                handle_generate_checklist("FS-TEST-001", QaTaskType::Feature, project_path, None)
                    .await;

            assert!(result.is_ok());

            let qa_dir = project_path.join(".pmat-qa").join("FS-TEST-001");
            assert!(qa_dir.exists(), "QA directory should be created");

            let checklist_path = qa_dir.join("checklist.yaml");
            assert!(checklist_path.exists(), "Checklist file should be created");

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
            let result =
                handle_generate_examples("EX-TEST-001", "my-feature", project_path, None).await;

            assert!(result.is_ok());

            let examples_dir = project_path.join("examples").join("my-feature");
            assert!(examples_dir.exists(), "Examples directory should exist");

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

        let result = print_task_status("NO-CHECKLIST", task_dir);
        assert!(result.is_ok());
    }
}

mod epic_summary_tests {
    use super::*;

    #[test]
    fn test_handle_epic_summary() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let qa_dir = temp_dir.path();

        for (task_id, checked_count) in [("EPIC-TASK-1", 20), ("EPIC-TASK-2", 25)] {
            let task_dir = qa_dir.join(task_id);
            fs::create_dir_all(&task_dir).expect("Failed to create dir");

            let mut checklist = generate_checklist(task_id, QaTaskType::Feature);

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

mod validation_check_tests {
    use super::*;

    #[tokio::test]
    async fn test_run_code_quality_checks() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = run_code_quality_checks(temp_dir.path()).await;

        assert_eq!(result.name, "Code Quality");
        assert!(result.total > 0);
        assert!(!result.items.is_empty());

        let ids: Vec<&str> = result.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"B1"));
        assert!(ids.contains(&"B2"));
        assert!(ids.contains(&"B3"));
        assert!(ids.contains(&"B4"));
        assert!(ids.contains(&"B5"));
    }

    #[tokio::test]
    async fn test_run_testing_checks() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = run_testing_checks(temp_dir.path()).await;

        assert_eq!(result.name, "Testing");
        assert!(result.total > 0);

        let ids: Vec<&str> = result.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"C1"));
        assert!(ids.contains(&"C2"));
    }

    #[tokio::test]
    async fn test_run_documentation_checks() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            "# Changelog\n## Unreleased\n- DOC-TEST feature",
        )
        .expect("Write failed");

        let result = run_documentation_checks(temp_dir.path(), "DOC-TEST").await;

        assert_eq!(result.name, "Documentation");
        assert!(result.total > 0);

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

        let commit_item = result.items.iter().find(|i| i.id == "E2");
        assert!(commit_item.is_some());
        assert_eq!(commit_item.unwrap().status, ValidationStatus::Skipped);
    }
}
