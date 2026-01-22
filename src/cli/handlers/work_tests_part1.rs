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

