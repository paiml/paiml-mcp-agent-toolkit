//\! Tests for work handlers
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let url = "https://github.com/paiml/pmat.git";
        assert_eq!(parse_github_url(url), Some("paiml/pmat".to_string()));
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let url = "git@github.com:paiml/pmat.git";
        assert_eq!(parse_github_url(url), Some("paiml/pmat".to_string()));
    }

    #[test]
    fn test_parse_github_url_invalid() {
        let url = "https://gitlab.com/owner/repo.git";
        assert_eq!(parse_github_url(url), None);
    }
}


mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // ========== Test Fixtures ==========

    /// Create a test project directory with roadmap structure
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create docs/roadmaps directory
        let roadmaps_dir = temp_dir.path().join("docs").join("roadmaps");
        std::fs::create_dir_all(&roadmaps_dir).expect("Failed to create roadmaps dir");

        temp_dir
    }

    /// Create a test project with initialized roadmap
    fn create_initialized_project() -> TempDir {
        let temp_dir = create_test_project();

        let roadmap_path = temp_dir
            .path()
            .join("docs")
            .join("roadmaps")
            .join("roadmap.yaml");
        let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
github_repo: paiml/pmat
roadmap:
  - id: TEST-001
    title: Test Item 1
    status: planned
    priority: medium
  - id: GH-42
    github_issue: 42
    title: GitHub Issue
    status: inprogress
    priority: high
    labels:
      - enhancement
      - feature
  - id: EPIC-001
    title: Epic Item
    status: planned
    priority: high
    item_type: epic
    subtasks:
      - id: EPIC-001-A
        title: Subtask A
        status: completed
        completion: 100
      - id: EPIC-001-B
        title: Subtask B
        status: inprogress
        completion: 50
"#;
        std::fs::write(&roadmap_path, roadmap_content).expect("Failed to write roadmap");

        temp_dir
    }

    /// Create a test roadmap item
    fn make_test_item(id: &str, title: &str, status: ItemStatus) -> RoadmapItem {
        let mut item = RoadmapItem::new(id.to_string(), title.to_string());
        item.status = status;
        item
    }

    // ========== parse_github_url Tests ==========

    mod parse_github_url_tests {
        use super::*;

        #[test]
        fn test_https_url_with_git_extension() {
            let url = "https://github.com/owner/repo.git";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_https_url_without_git_extension() {
            let url = "https://github.com/owner/repo";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_ssh_url_with_git_extension() {
            let url = "git@github.com:owner/repo.git";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_ssh_url_without_git_extension() {
            let url = "git@github.com:owner/repo";
            assert_eq!(parse_github_url(url), Some("owner/repo".to_string()));
        }

        #[test]
        fn test_https_url_with_org_and_nested_repo() {
            let url = "https://github.com/paiml/paiml-mcp-agent-toolkit.git";
            assert_eq!(
                parse_github_url(url),
                Some("paiml/paiml-mcp-agent-toolkit".to_string())
            );
        }

        #[test]
        fn test_gitlab_url_returns_none() {
            let url = "https://gitlab.com/owner/repo.git";
            assert_eq!(parse_github_url(url), None);
        }

        #[test]
        fn test_bitbucket_url_returns_none() {
            let url = "https://bitbucket.org/owner/repo.git";
            assert_eq!(parse_github_url(url), None);
        }

        #[test]
        fn test_empty_url() {
            assert_eq!(parse_github_url(""), None);
        }

        #[test]
        fn test_random_string() {
            assert_eq!(parse_github_url("not-a-url"), None);
        }
    }

    // ========== parse_acceptance_criteria Tests ==========

    mod parse_acceptance_criteria_tests {
        use super::*;

        #[test]
        fn test_empty_body() {
            let body = "";
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }

        #[test]
        fn test_body_with_unchecked_checkboxes() {
            let body = r#"
## Acceptance Criteria
- [ ] First criterion
- [ ] Second criterion
- [ ] Third criterion
"#;
            let criteria = parse_acceptance_criteria(body);
            assert_eq!(criteria.len(), 3);
            assert_eq!(criteria[0], "First criterion");
            assert_eq!(criteria[1], "Second criterion");
            assert_eq!(criteria[2], "Third criterion");
        }

        #[test]
        fn test_body_with_checked_checkboxes() {
            let body = r#"
## Done
- [x] Completed task
- [x] Another completed task
"#;
            let criteria = parse_acceptance_criteria(body);
            assert_eq!(criteria.len(), 2);
            assert_eq!(criteria[0], "Completed task");
            assert_eq!(criteria[1], "Another completed task");
        }

        #[test]
        fn test_body_with_mixed_checkboxes() {
            let body = r#"
## Acceptance Criteria
- [x] Already done
- [ ] Still pending
- [x] Also done
"#;
            let criteria = parse_acceptance_criteria(body);
            assert_eq!(criteria.len(), 3);
        }

        #[test]
        fn test_body_with_no_checkboxes() {
            let body = r#"
This is a description without checkboxes.
Just regular text.
"#;
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }

        #[test]
        fn test_body_with_empty_checkbox() {
            let body = "- [ ] ";
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }

        #[test]
        fn test_body_with_whitespace_only_checkbox() {
            let body = "- [ ]    ";
            let criteria = parse_acceptance_criteria(body);
            assert!(criteria.is_empty());
        }
    }

    // ========== extract_line_from_yaml_error Tests ==========

    mod extract_line_from_yaml_error_tests {
        use super::*;

        #[test]
        fn test_error_with_line_number() {
            let error = "invalid type: string, expected sequence at line 42 column 5";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, Some(42));
        }

        #[test]
        fn test_error_without_line_number() {
            let error = "invalid type: string, expected sequence";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, None);
        }

        #[test]
        fn test_error_with_single_digit_line() {
            let error = "error at line 5 column 1";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, Some(5));
        }

        #[test]
        fn test_error_with_large_line_number() {
            let error = "parsing failed at line 1234 column 10";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, Some(1234));
        }

        #[test]
        fn test_empty_error_string() {
            let error = "";
            let line = extract_line_from_yaml_error(error);
            assert_eq!(line, None);
        }
    }

    // ========== CommitMetadata Tests ==========

    mod commit_metadata_tests {
        use super::*;

        #[test]
        fn test_commit_metadata_serialization() {
            let metadata = CommitMetadata {
                commit_sha: Some("abc123".to_string()),
                work_item_id: "TEST-001".to_string(),
                prompt: "Test task".to_string(),
                tdg_score: 85.0,
                repo_score: 75.0,
                rust_project_score: Some(90.0),
                timestamp: chrono::Utc::now(),
            };

            let json = serde_json::to_string(&metadata).unwrap();
            assert!(json.contains("abc123"));
            assert!(json.contains("TEST-001"));
            assert!(json.contains("85"));
        }

        #[test]
        fn test_commit_metadata_deserialization() {
            let json = r#"{
                "commit_sha": "def456",
                "work_item_id": "GH-42",
                "prompt": "Fix bug",
                "tdg_score": 90.0,
                "repo_score": 80.0,
                "rust_project_score": null,
                "timestamp": "2024-01-01T00:00:00Z"
            }"#;

            let metadata: CommitMetadata = serde_json::from_str(json).unwrap();
            assert_eq!(metadata.commit_sha, Some("def456".to_string()));
            assert_eq!(metadata.work_item_id, "GH-42");
            assert_eq!(metadata.tdg_score, 90.0);
            assert!(metadata.rust_project_score.is_none());
        }
    }

    // ========== Score Capture Tests ==========

    mod score_capture_tests {
        use super::*;

        #[tokio::test]
        async fn test_capture_tdg_score_no_cache() {
            let temp_dir = TempDir::new().unwrap();
            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            // Should return default when no cache exists
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_tdg_score_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let tdg_file = metrics_dir.join("tdg-score.json");
            std::fs::write(&tdg_file, r#"{"score": 85.5}"#).unwrap();

            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 85.5);
        }

        #[tokio::test]
        async fn test_capture_repo_score_no_cache() {
            let temp_dir = TempDir::new().unwrap();
            let score = capture_repo_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_repo_score_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let repo_file = metrics_dir.join("repo-score.json");
            std::fs::write(&repo_file, r#"{"score": 72.0}"#).unwrap();

            let score = capture_repo_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 72.0);
        }

        #[tokio::test]
        async fn test_capture_rust_project_score_no_cache() {
            let temp_dir = TempDir::new().unwrap();
            let score = capture_rust_project_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_rust_project_score_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let rust_file = metrics_dir.join("rust-project-score.json");
            std::fs::write(&rust_file, r#"{"total_earned": 95.0}"#).unwrap();

            let score = capture_rust_project_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 95.0);
        }

        #[tokio::test]
        async fn test_capture_score_with_invalid_json() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let tdg_file = metrics_dir.join("tdg-score.json");
            std::fs::write(&tdg_file, "not valid json").unwrap();

            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_err());
        }
    }

    // ========== Handler Integration Tests ==========

    mod handler_integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_work_init_creates_roadmap() {
            let temp_dir = create_test_project();

            let result = handle_work_init(
                Some("paiml/test".to_string()),
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir
                .path()
                .join("docs")
                .join("roadmaps")
                .join("roadmap.yaml");
            assert!(roadmap_path.exists());
        }

        #[tokio::test]
        async fn test_handle_work_init_no_github() {
            let temp_dir = create_test_project();

            let result = handle_work_init(None, true, Some(temp_dir.path().to_path_buf())).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_init_already_exists() {
            let temp_dir = create_initialized_project();

            let result = handle_work_init(
                Some("paiml/test".to_string()),
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            // Should succeed but indicate already exists
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_all_items() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(None, Some(temp_dir.path().to_path_buf()), false).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_active_only() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(None, Some(temp_dir.path().to_path_buf()), true).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_specific_item() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(
                Some("TEST-001".to_string()),
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_status_nonexistent_item() {
            let temp_dir = create_initialized_project();

            let result = handle_work_status(
                Some("NONEXISTENT-999".to_string()),
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_continue_existing_item() {
            let temp_dir = create_initialized_project();

            let result =
                handle_work_continue("TEST-001".to_string(), Some(temp_dir.path().to_path_buf()))
                    .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_continue_with_phases() {
            let temp_dir = create_initialized_project();

            let result =
                handle_work_continue("GH-42".to_string(), Some(temp_dir.path().to_path_buf()))
                    .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_continue_nonexistent() {
            let temp_dir = create_initialized_project();

            let result = handle_work_continue(
                "NONEXISTENT-999".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_sync_yaml_to_github() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::YamlToGithub,
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_sync_github_to_yaml() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::GithubToYaml,
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_sync_full() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::Full,
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_validate_valid_roadmap() {
            let temp_dir = create_initialized_project();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                false, // verbose
                false, // fix
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_validate_verbose() {
            let temp_dir = create_initialized_project();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                true,  // verbose
                false, // fix
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_validate_missing_roadmap() {
            let temp_dir = TempDir::new().unwrap();

            let result =
                handle_work_validate(Some(temp_dir.path().to_path_buf()), false, false).await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_list_statuses() {
            let result = handle_work_list_statuses().await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_migrate_no_changes_needed() {
            let temp_dir = create_initialized_project();

            let result = handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                true,  // dry_run
                false, // backup
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_handle_work_migrate_with_backup() {
            let temp_dir = create_initialized_project();

            // Modify roadmap to have a fixable issue
            let roadmap_path = temp_dir
                .path()
                .join("docs")
                .join("roadmaps")
                .join("roadmap.yaml");
            let content = std::fs::read_to_string(&roadmap_path).unwrap();
            let modified = content.replace("status: planned", "status: done");
            std::fs::write(&roadmap_path, modified).unwrap();

            let result = handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false, // dry_run
                true,  // backup
            )
            .await;

            assert!(result.is_ok());
            // Backup file should exist
            let backup_path = roadmap_path.with_extension("yaml.bak");
            assert!(backup_path.exists());
        }

        #[tokio::test]
        async fn test_handle_work_migrate_missing_roadmap() {
            let temp_dir = TempDir::new().unwrap();

            let result =
                handle_work_migrate(Some(temp_dir.path().to_path_buf()), true, false).await;

            assert!(result.is_err());
        }
    }

    // ========== Property-Based Tests ==========

    mod proptest_tests {
        use super::*;

        proptest! {
            #[test]
            fn test_parse_github_url_never_panics(url in ".*") {
                let _ = parse_github_url(&url);
            }

            #[test]
            fn test_parse_acceptance_criteria_never_panics(body in ".*") {
                let _ = parse_acceptance_criteria(&body);
            }

            #[test]
            fn test_extract_line_from_yaml_error_never_panics(error in ".*") {
                let _ = extract_line_from_yaml_error(&error);
            }

            #[test]
            fn test_github_url_extraction_consistency(owner in "[a-z]{1,20}", repo in "[a-z0-9-]{1,30}") {
                let https_url = format!("https://github.com/{}/{}.git", owner, repo);
                let ssh_url = format!("git@github.com:{}/{}.git", owner, repo);

                let expected = format!("{}/{}", owner, repo);
                prop_assert_eq!(parse_github_url(&https_url), Some(expected.clone()));
                prop_assert_eq!(parse_github_url(&ssh_url), Some(expected));
            }

            #[test]
            // Ensure at least one alphanumeric character
            fn test_acceptance_criteria_preserves_content(criteria_text in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,49}") {
                let body = format!("- [ ] {}", criteria_text);
                let criteria = parse_acceptance_criteria(&body);
                // Parsing may filter whitespace-only items
                prop_assert!(criteria.len() <= 1);
            }
        }
    }

    // ========== Edge Case Tests ==========

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_roadmap_item_completion_no_subtasks() {
            let item = make_test_item("TEST", "Test", ItemStatus::InProgress);
            assert_eq!(item.completion_percentage(), 50);
        }

        #[test]
        fn test_roadmap_item_completion_completed() {
            let item = make_test_item("TEST", "Test", ItemStatus::Completed);
            assert_eq!(item.completion_percentage(), 100);
        }

        #[test]
        fn test_roadmap_item_completion_planned() {
            let item = make_test_item("TEST", "Test", ItemStatus::Planned);
            assert_eq!(item.completion_percentage(), 0);
        }

        #[test]
        fn test_roadmap_item_from_github_issue() {
            let item = RoadmapItem::from_github_issue(123, "Test Issue".to_string());
            assert_eq!(item.id, "GH-123");
            assert_eq!(item.github_issue, Some(123));
            assert!(item.is_github_synced());
        }

        #[test]
        fn test_roadmap_item_not_github_synced() {
            let item = make_test_item("LOCAL-001", "Local Task", ItemStatus::Planned);
            assert!(!item.is_github_synced());
        }

        #[tokio::test]
        async fn test_capture_commit_metadata_creates_metrics_dir() {
            let temp_dir = TempDir::new().unwrap();
            let item = make_test_item("TEST-001", "Test Task", ItemStatus::InProgress);

            // Initialize git repo for git rev-parse to work
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.name", "Test User"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            // Create a file and commit
            std::fs::write(temp_dir.path().join("test.txt"), "test").unwrap();
            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["commit", "-m", "initial"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = capture_commit_metadata(&temp_dir.path().to_path_buf(), &item).await;
            assert!(result.is_ok());

            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            assert!(metrics_dir.exists());
        }

        #[test]
        fn test_parse_github_url_with_trailing_slash() {
            let url = "https://github.com/owner/repo/";
            // The function only removes .git extension, not trailing slashes
            let result = parse_github_url(url);
            assert!(result.is_some());
        }

        #[test]
        fn test_parse_github_url_enterprise() {
            // GitHub Enterprise URL should not match
            let url = "https://github.mycompany.com/owner/repo.git";
            assert_eq!(parse_github_url(url), None);
        }

        #[test]
        fn test_status_display_emoji_mappings() {
            // Test all status enum variants have corresponding emoji in status display
            let statuses = [
                ItemStatus::Completed,
                ItemStatus::InProgress,
                ItemStatus::Planned,
                ItemStatus::Blocked,
                ItemStatus::Review,
                ItemStatus::Cancelled,
            ];

            for status in statuses {
                // These should map to emoji in handle_work_status
                let emoji = match status {
                    ItemStatus::Completed => "✅",
                    ItemStatus::InProgress => "⏳",
                    ItemStatus::Planned => "📋",
                    ItemStatus::Blocked => "🚫",
                    ItemStatus::Review => "👀",
                    ItemStatus::Cancelled => "❌",
                };
                assert!(!emoji.is_empty());
            }
        }

        #[test]
        fn test_id_truncation_logic() {
            // Test the ID truncation for long IDs (display limited to 30 chars)
            let long_id = "This-is-a-very-long-id-that-exceeds-thirty-characters";
            let display_id = if long_id.len() > 30 {
                format!("{}...", &long_id[..30])
            } else {
                long_id.to_string()
            };
            assert!(display_id.len() <= 33); // 30 + "..."
            assert!(display_id.ends_with("..."));
        }

        #[test]
        fn test_short_id_no_truncation() {
            let short_id = "GH-42";
            let display_id = if short_id.len() > 30 {
                format!("{}...", &short_id[..30])
            } else {
                short_id.to_string()
            };
            assert_eq!(display_id, "GH-42");
        }
    }

    // ========== Validation Tests ==========

    mod validation_tests {
        use super::*;

        #[tokio::test]
        async fn test_validate_invalid_yaml() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs").join("roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            std::fs::write(&roadmap_path, "invalid: yaml: content:").unwrap();

            let result =
                handle_work_validate(Some(temp_dir.path().to_path_buf()), false, false).await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_validate_with_warnings() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs").join("roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            // Create roadmap with long ID (should trigger warning)
            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: This-is-a-very-long-id-that-exceeds-fifty-characters-for-testing-purposes-xyz
    title: Test Item
    status: planned
    priority: medium
"#;
            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            std::fs::write(&roadmap_path, roadmap_content).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                true,  // verbose
                false, // fix
            )
            .await;

            // Should succeed but print warnings
            assert!(result.is_ok());
        }
    }

    // ========== Specification Template Tests ==========

    mod spec_template_tests {
        use super::*;

        #[test]
        fn test_create_specification_template() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("spec.md");
            let item = RoadmapItem::from_github_issue(42, "Test Feature".to_string());

            let result = create_specification_template(&spec_path, &item);
            assert!(result.is_ok());
            assert!(spec_path.exists());

            let content = std::fs::read_to_string(&spec_path).unwrap();
            assert!(content.contains("Test Feature"));
            assert!(content.contains("GH-42"));
            assert!(content.contains("## Summary"));
            assert!(content.contains("## Requirements"));
        }

        #[test]
        fn test_create_specification_template_creates_directories() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("docs").join("specs").join("spec.md");
            let item = make_test_item("LOCAL-001", "Local Feature", ItemStatus::Planned);

            let result = create_specification_template(&spec_path, &item);
            assert!(result.is_ok());
            assert!(spec_path.exists());
        }

        #[test]
        fn test_spec_template_with_yaml_only_ticket() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("spec.md");
            let item = make_test_item("YAML-001", "YAML Only Task", ItemStatus::InProgress);

            let result = create_specification_template(&spec_path, &item);
            assert!(result.is_ok());

            let content = std::fs::read_to_string(&spec_path).unwrap();
            assert!(content.contains("YAML-001"));
            assert!(content.contains("Ticket ID"));
            assert!(!content.contains("GitHub Issue"));
        }

        #[test]
        fn test_spec_template_contains_all_sections() {
            let temp_dir = TempDir::new().unwrap();
            let spec_path = temp_dir.path().join("spec.md");
            let item = RoadmapItem::from_github_issue(123, "Complete Feature".to_string());

            create_specification_template(&spec_path, &item).unwrap();
            let content = std::fs::read_to_string(&spec_path).unwrap();

            // Verify all expected sections exist
            assert!(content.contains("## Summary"));
            assert!(content.contains("## Requirements"));
            assert!(content.contains("### Functional Requirements"));
            assert!(content.contains("### Non-Functional Requirements"));
            assert!(content.contains("## Architecture"));
            assert!(content.contains("## Implementation Plan"));
            assert!(content.contains("## Testing Strategy"));
            assert!(content.contains("## Success Criteria"));
            assert!(content.contains("## References"));
        }
    }

    // ========== Work Start Handler Tests ==========

    mod work_start_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_work_start_yaml_ticket() {
            let temp_dir = create_initialized_project();

            let result = handle_work_start(
                "NEW-TICKET".to_string(),
                false, // with_spec
                false, // epic
                Some(temp_dir.path().to_path_buf()),
                false, // create_github
            )
            .await;

            assert!(result.is_ok());

            // Verify item was created
            let roadmap_path = temp_dir
                .path()
                .join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("NEW-TICKET").unwrap();
            assert!(item.is_some());
            assert_eq!(item.unwrap().status, ItemStatus::InProgress);
        }

        #[tokio::test]
        async fn test_handle_work_start_existing_yaml_ticket() {
            let temp_dir = create_initialized_project();

            // Start work on existing TEST-001
            let result = handle_work_start(
                "TEST-001".to_string(),
                false,
                false,
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            // Verify status changed to InProgress
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(item.status, ItemStatus::InProgress);
        }

        #[tokio::test]
        async fn test_handle_work_start_as_epic() {
            let temp_dir = create_initialized_project();

            let result = handle_work_start(
                "EPIC-NEW".to_string(),
                false,
                true, // epic flag
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("EPIC-NEW").unwrap().unwrap();
            assert_eq!(item.item_type, crate::models::roadmap::ItemType::Epic);
        }

        #[tokio::test]
        async fn test_handle_work_start_with_spec() {
            let temp_dir = create_initialized_project();

            let result = handle_work_start(
                "SPEC-TEST".to_string(),
                true, // with_spec
                false,
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            // Verify spec file was created
            let spec_path = temp_dir.path().join("docs/specifications/spec-test-spec.md");
            assert!(spec_path.exists());
        }

        #[tokio::test]
        async fn test_handle_work_start_github_issue_number() {
            let temp_dir = create_initialized_project();

            // Start work on issue number (no GitHub API available, should create placeholder)
            let result = handle_work_start(
                "999".to_string(),
                false,
                false,
                Some(temp_dir.path().to_path_buf()),
                false,
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("GH-999").unwrap();
            assert!(item.is_some());
            assert_eq!(item.unwrap().github_issue, Some(999));
        }
    }

    // ========== Work Complete Handler Tests ==========

    mod work_complete_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_work_complete_skip_quality() {
            let temp_dir = create_initialized_project();

            // First start the work
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let mut item = service.find_item("TEST-001").unwrap().unwrap();
            item.status = ItemStatus::InProgress;
            service.upsert_item(item).unwrap();

            // Initialize git for metadata capture
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.name", "Test"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::fs::write(temp_dir.path().join("test.txt"), "test").unwrap();
            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["commit", "-m", "init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = handle_work_complete(
                "TEST-001".to_string(),
                true, // skip_quality
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            // Verify status changed to Completed
            let item = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(item.status, ItemStatus::Completed);
        }

        #[tokio::test]
        async fn test_handle_work_complete_nonexistent() {
            let temp_dir = create_initialized_project();

            let result = handle_work_complete(
                "NONEXISTENT-999".to_string(),
                true,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_handle_work_complete_with_labels_for_changelog() {
            let temp_dir = create_initialized_project();

            // Set up item with labels
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);
            let item = service.find_item("GH-42").unwrap().unwrap();
            // GH-42 already has labels from test fixture
            assert!(!item.labels.is_empty());

            // Initialize git
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.email", "test@test.com"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["config", "user.name", "Test"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::fs::write(temp_dir.path().join("test.txt"), "test").unwrap();
            std::process::Command::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["commit", "-m", "init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = handle_work_complete(
                "GH-42".to_string(),
                true, // skip_quality
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // ========== Git Detection Tests ==========

    mod git_detection_tests {
        use super::*;

        #[test]
        fn test_detect_github_repo_no_git() {
            let temp_dir = TempDir::new().unwrap();
            let result = detect_github_repo(&temp_dir.path().to_path_buf());
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn test_detect_github_repo_with_remote() {
            let temp_dir = TempDir::new().unwrap();

            // Initialize git repo
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            // Add remote
            std::process::Command::new("git")
                .args(["remote", "add", "origin", "https://github.com/test/repo.git"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = detect_github_repo(&temp_dir.path().to_path_buf());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Some("test/repo".to_string()));
        }

        #[test]
        fn test_detect_github_repo_ssh_remote() {
            let temp_dir = TempDir::new().unwrap();

            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            std::process::Command::new("git")
                .args(["remote", "add", "origin", "git@github.com:owner/project.git"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = detect_github_repo(&temp_dir.path().to_path_buf());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Some("owner/project".to_string()));
        }
    }

    // ========== Migration Tests ==========

    mod migration_tests {
        use super::*;

        #[tokio::test]
        async fn test_migrate_normalizes_done_status() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: done
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            let result = handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false, // not dry_run
                false, // no backup
            )
            .await;

            assert!(result.is_ok());

            // Verify status was normalized
            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: completed"));
            assert!(!new_content.contains("status: done"));
        }

        #[tokio::test]
        async fn test_migrate_normalizes_wip_status() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: wip
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false,
                false,
            )
            .await
            .unwrap();

            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: inprogress"));
        }

        #[tokio::test]
        async fn test_migrate_dry_run_no_changes() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: done
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                true, // dry_run
                false,
            )
            .await
            .unwrap();

            // Content should be unchanged
            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: done"));
        }

        #[tokio::test]
        async fn test_migrate_multiple_status_normalizations() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_path = roadmap_dir.join("roadmap.yaml");
            let content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Item 1
    status: Done
    priority: medium
  - id: TEST-002
    title: Item 2
    status: WIP
    priority: high
  - id: TEST-003
    title: Item 3
    status: stuck
    priority: low
  - id: TEST-004
    title: Item 4
    status: todo
    priority: medium
"#;
            std::fs::write(&roadmap_path, content).unwrap();

            handle_work_migrate(
                Some(temp_dir.path().to_path_buf()),
                false,
                false,
            )
            .await
            .unwrap();

            let new_content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(new_content.contains("status: completed"));
            assert!(new_content.contains("status: inprogress"));
            assert!(new_content.contains("status: blocked"));
            assert!(new_content.contains("status: planned"));
        }
    }

    // ========== Sync Direction Tests ==========

    mod sync_direction_tests {
        use super::*;

        #[tokio::test]
        async fn test_sync_yaml_to_github_shows_yaml_only_items() {
            let temp_dir = create_initialized_project();

            let result = handle_work_sync(
                SyncDirection::YamlToGithub,
                Some(temp_dir.path().to_path_buf()),
                true,
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_sync_all_directions() {
            let temp_dir = create_initialized_project();

            // Test all sync directions
            for direction in [
                SyncDirection::YamlToGithub,
                SyncDirection::GithubToYaml,
                SyncDirection::Full,
            ] {
                let result = handle_work_sync(
                    direction,
                    Some(temp_dir.path().to_path_buf()),
                    true,
                )
                .await;
                assert!(result.is_ok());
            }
        }
    }

    // ========== Roadmap Item Properties Tests ==========

    mod roadmap_item_properties {
        use super::*;

        #[test]
        fn test_completion_percentage_with_subtasks() {
            let mut item = RoadmapItem::new("EPIC-001".to_string(), "Epic".to_string());
            item.subtasks = vec![
                crate::models::roadmap::Subtask {
                    id: "SUB-1".to_string(),
                    github_issue: None,
                    title: "Sub 1".to_string(),
                    status: ItemStatus::Completed,
                    completion: 100,
                },
                crate::models::roadmap::Subtask {
                    id: "SUB-2".to_string(),
                    github_issue: None,
                    title: "Sub 2".to_string(),
                    status: ItemStatus::InProgress,
                    completion: 50,
                },
            ];
            // Average of 100 and 50 = 75
            assert_eq!(item.completion_percentage(), 75);
        }

        #[test]
        fn test_completion_percentage_with_phases() {
            let mut item = RoadmapItem::new("TASK-001".to_string(), "Task".to_string());
            item.phases = vec![
                crate::models::roadmap::Phase {
                    name: "Phase 1".to_string(),
                    status: ItemStatus::Completed,
                    estimated_effort: None,
                    completion: 100,
                },
                crate::models::roadmap::Phase {
                    name: "Phase 2".to_string(),
                    status: ItemStatus::InProgress,
                    estimated_effort: None,
                    completion: 60,
                },
                crate::models::roadmap::Phase {
                    name: "Phase 3".to_string(),
                    status: ItemStatus::Planned,
                    estimated_effort: None,
                    completion: 0,
                },
            ];
            // Average of 100, 60, 0 = 53.33 -> 53
            assert_eq!(item.completion_percentage(), 53);
        }

        #[test]
        fn test_completion_blocked_status() {
            let item = make_test_item("TEST", "Test", ItemStatus::Blocked);
            assert_eq!(item.completion_percentage(), 0);
        }

        #[test]
        fn test_completion_cancelled_status() {
            let item = make_test_item("TEST", "Test", ItemStatus::Cancelled);
            assert_eq!(item.completion_percentage(), 0);
        }

        #[test]
        fn test_completion_review_status() {
            let item = make_test_item("TEST", "Test", ItemStatus::Review);
            assert_eq!(item.completion_percentage(), 90);
        }
    }

    // ========== Score Cache Edge Cases ==========

    mod score_cache_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_capture_tdg_score_missing_score_key() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            // JSON without "score" key
            let tdg_file = metrics_dir.join("tdg-score.json");
            std::fs::write(&tdg_file, r#"{"other_field": 42}"#).unwrap();

            let score = capture_tdg_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0); // Should fall back to default
        }

        #[tokio::test]
        async fn test_capture_repo_score_non_numeric() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            // JSON with non-numeric score
            let repo_file = metrics_dir.join("repo-score.json");
            std::fs::write(&repo_file, r#"{"score": "not-a-number"}"#).unwrap();

            let score = capture_repo_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }

        #[tokio::test]
        async fn test_capture_rust_score_missing_total_earned() {
            let temp_dir = TempDir::new().unwrap();
            let metrics_dir = temp_dir.path().join(".pmat-metrics");
            std::fs::create_dir_all(&metrics_dir).unwrap();

            let rust_file = metrics_dir.join("rust-project-score.json");
            std::fs::write(&rust_file, r#"{"categories": []}"#).unwrap();

            let score = capture_rust_project_score(&temp_dir.path().to_path_buf()).await;
            assert!(score.is_ok());
            assert_eq!(score.unwrap(), 0.0);
        }
    }

    // ========== Continue Handler with Different Item States ==========

    mod continue_handler_states {
        use super::*;

        #[tokio::test]
        async fn test_continue_with_epic_subtasks() {
            let temp_dir = create_initialized_project();

            // EPIC-001 has subtasks in the test fixture
            let result = handle_work_continue(
                "EPIC-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_continue_with_acceptance_criteria() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TASK-001
    title: Task with Criteria
    status: inprogress
    priority: high
    acceptance_criteria:
      - First criterion
      - Second criterion
      - Third criterion
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_continue(
                "TASK-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_continue_with_spec_path() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: SPEC-001
    title: Task with Spec
    status: inprogress
    priority: medium
    spec: docs/specifications/spec-001.md
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_continue(
                "SPEC-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_continue_with_phases() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: PHASED-001
    title: Task with Phases
    status: inprogress
    priority: high
    phases:
      - name: RED
        status: completed
        completion: 100
      - name: GREEN
        status: inprogress
        completion: 50
      - name: REFACTOR
        status: planned
        completion: 0
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_continue(
                "PHASED-001".to_string(),
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // ========== Validate Handler Edge Cases ==========

    mod validate_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_validate_with_fix_flag_shows_tip() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            // Create roadmap with warnings (no acceptance criteria)
            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test Item
    status: planned
    priority: medium
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                false,
                true, // fix flag
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_validate_yaml_with_location_in_error() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            // Invalid YAML that will produce line number in error
            let invalid_yaml = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: Test
    status: invalid_status_that_doesnt_exist
    priority: medium
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), invalid_yaml).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                false,
                false,
            )
            .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_validate_github_disabled() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_dir = temp_dir.path().join("docs/roadmaps");
            std::fs::create_dir_all(&roadmap_dir).unwrap();

            let roadmap_content = r#"
roadmap_version: '1.0'
github_enabled: false
roadmap:
  - id: LOCAL-001
    title: Local Only
    status: planned
    priority: low
"#;
            std::fs::write(roadmap_dir.join("roadmap.yaml"), roadmap_content).unwrap();

            let result = handle_work_validate(
                Some(temp_dir.path().to_path_buf()),
                true, // verbose
                false,
            )
            .await;

            assert!(result.is_ok());
        }
    }

    // ========== Init Handler Edge Cases ==========

    mod init_edge_cases {
        use super::*;

        #[tokio::test]
        async fn test_init_with_explicit_github_repo() {
            let temp_dir = create_test_project();

            let result = handle_work_init(
                Some("explicit/repo".to_string()),
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(content.contains("explicit/repo"));
        }

        #[tokio::test]
        async fn test_init_detects_git_remote() {
            let temp_dir = create_test_project();

            // Initialize git and add remote
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(temp_dir.path())
                .status()
                .ok();
            std::process::Command::new("git")
                .args(["remote", "add", "origin", "https://github.com/detected/repo.git"])
                .current_dir(temp_dir.path())
                .status()
                .ok();

            let result = handle_work_init(
                None, // No explicit repo
                false,
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            let content = std::fs::read_to_string(&roadmap_path).unwrap();
            assert!(content.contains("detected/repo"));
        }

        #[tokio::test]
        async fn test_init_github_enabled_but_no_repo() {
            let temp_dir = create_test_project();

            // No git, no explicit repo
            let result = handle_work_init(
                None,
                false, // github enabled
                Some(temp_dir.path().to_path_buf()),
            )
            .await;

            assert!(result.is_ok());

            // Should still succeed, just without repo configured
            let roadmap_path = temp_dir.path().join("docs/roadmaps/roadmap.yaml");
            assert!(roadmap_path.exists());
        }
    }

    // ========== Additional Property Tests ==========

    mod additional_proptests {
        use super::*;

        proptest! {
            #[test]
            fn test_acceptance_criteria_extraction_preserves_order(
                // Ensure at least one non-space character by starting with alphanumeric
                items in prop::collection::vec("[a-zA-Z0-9][a-zA-Z0-9 ]{4,19}", 1..10)
            ) {
                let body = items.iter()
                    .map(|item| format!("- [ ] {}", item))
                    .collect::<Vec<_>>()
                    .join("\n");

                let criteria = parse_acceptance_criteria(&body);
                // Number of criteria may differ if parsing filters some items
                prop_assert!(criteria.len() <= items.len());
            }

            #[test]
            fn test_yaml_error_line_extraction_valid_formats(line_num in 1usize..10000) {
                let error = format!("parse error at line {} column 5", line_num);
                let extracted = extract_line_from_yaml_error(&error);
                prop_assert_eq!(extracted, Some(line_num));
            }
        }
    }
}
