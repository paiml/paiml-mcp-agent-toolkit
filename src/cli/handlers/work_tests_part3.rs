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
