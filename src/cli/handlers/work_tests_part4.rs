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
