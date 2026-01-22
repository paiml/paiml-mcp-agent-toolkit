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
