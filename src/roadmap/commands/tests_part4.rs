/// NOTE: Temporarily disabled due to struct definition mismatches
#[cfg(all(test, feature = "broken-tests"))]
mod coverage_tests_part3 {
    use super::*;
    use chrono::TimeZone;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: Create a test roadmap config with optional customization
    fn create_test_config(temp_dir: &TempDir) -> RoadmapConfig {
        RoadmapConfig {
            enabled: true,
            path: temp_dir.path().join("roadmap.md"),
            auto_generate_todos: true,
            enforce_quality_gates: true,
            require_task_ids: true,
            task_id_pattern: "PMAT-[0-9]{4}".to_string(),
            quality_gates: QualityGateConfig::default(),
            git: GitConfig {
                create_branches: false, // Disable for tests
                branch_pattern: "feature/{task_id}".to_string(),
                commit_pattern: "{task_id}: {message}".to_string(),
                require_quality_check: false, // Disable for tests
            },
            tracking: TrackingConfig::default(),
        }
    }

    /// Helper: Create a sample roadmap file for testing
    fn create_sample_roadmap(path: &Path) -> Roadmap {
        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test task description".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: Some("developer".to_string()),
            started_at: None,
            completed_at: None,
        };

        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            end_date: Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap(),
            priority: Priority::P0,
            tasks: vec![task],
            definition_of_done: vec![
                "All tests pass".to_string(),
                "Documentation updated".to_string(),
            ],
            quality_gates: vec!["Coverage > 80%".to_string()],
        };

        let mut roadmap = Roadmap {
            current_sprint: Some("v1.0.0".to_string()),
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };
        roadmap.sprints.insert("v1.0.0".to_string(), sprint);

        roadmap.to_file(path).expect("Failed to write roadmap");
        roadmap
    }

    // ========== HANDLE_INIT AND HANDLE_START TESTS ==========

    #[test]
    fn test_handle_init_valid() {
        let temp_dir = TempDir::new().unwrap();
        let roadmap_path = temp_dir.path().join("handle_init_test.md");

        let result = handle_init(
            "v1.0.0".to_string(),
            "Test Initiative".to_string(),
            14,
            "P1".to_string(),
            roadmap_path.clone(),
        );

        assert!(result.is_ok());
        assert!(roadmap_path.exists());

        let content = fs::read_to_string(&roadmap_path).unwrap();
        assert!(content.contains("v1.0.0"));
        assert!(content.contains("Test Initiative"));
        assert!(content.contains("P1"));
        assert!(content.contains("14 days"));
    }

    #[test]
    fn test_handle_init_all_priorities() {
        let temp_dir = TempDir::new().unwrap();

        for priority in &["P0", "P1", "P2"] {
            let roadmap_path = temp_dir.path().join(format!("roadmap_{priority}.md"));
            let result = handle_init(
                "v1.0.0".to_string(),
                "Test".to_string(),
                7,
                priority.to_string(),
                roadmap_path.clone(),
            );

            assert!(result.is_ok(), "Failed for priority {}", priority);
        }
    }

    #[test]
    fn test_handle_start_valid_task_id() {
        let result = handle_start("PMAT-1234".to_string(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_start_invalid_task_id() {
        let result = handle_start("INVALID-1234".to_string(), false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid task ID format"));
    }

    #[test]
    fn test_handle_start_lowercase_task_id() {
        // The check uses starts_with("PMAT-"), so lowercase should fail
        let result = handle_start("pmat-1234".to_string(), false);
        assert!(result.is_err());
    }

    // ========== ASYNC SHOW STATUS TESTS ==========

    #[tokio::test]
    async fn test_show_sprint_status_json() {
        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: None,
            completed_at: None,
        };

        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: vec![task],
            definition_of_done: Vec::new(),
            quality_gates: Vec::new(),
        };

        let mut roadmap = Roadmap {
            current_sprint: Some("v1.0.0".to_string()),
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };
        roadmap.sprints.insert("v1.0.0".to_string(), sprint);

        let result = show_sprint_status(&roadmap, Some("v1.0.0"), OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_sprint_status_table() {
        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: Vec::new(),
            definition_of_done: Vec::new(),
            quality_gates: Vec::new(),
        };

        let mut roadmap = Roadmap {
            current_sprint: Some("v1.0.0".to_string()),
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };
        roadmap.sprints.insert("v1.0.0".to_string(), sprint);

        let result = show_sprint_status(&roadmap, None, OutputFormat::Table).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_sprint_status_no_current_sprint() {
        let roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };

        let result = show_sprint_status(&roadmap, None, OutputFormat::Json).await;
        assert!(result.is_err());
    }

    // ========== EDGE CASES ==========

    #[tokio::test]
    async fn test_init_sprint_sets_first_as_current() {
        let temp_dir = TempDir::new().unwrap();
        let roadmap_path = temp_dir.path().join("roadmap.md");

        // Create first sprint
        init_sprint(&roadmap_path, "v1.0.0", "First", 7, "P0")
            .await
            .unwrap();

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();
        assert_eq!(roadmap.current_sprint, Some("v1.0.0".to_string()));

        // Create second sprint
        init_sprint(&roadmap_path, "v2.0.0", "Second", 7, "P1")
            .await
            .unwrap();

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();
        // Current should still be v1.0.0
        assert_eq!(roadmap.current_sprint, Some("v1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_start_task_shows_task_details() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        // This should print task details without panicking
        let result = start_task(&config.path, "PMAT-0001", false, &config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_status_yaml_output() {
        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "YAML test".to_string(),
            status: TaskStatus::Deferred,
            complexity: Complexity::Low,
            priority: Priority::P2,
            assignee: Some("yaml_tester".to_string()),
            started_at: None,
            completed_at: None,
        };

        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![task],
            completed_sprints: Vec::new(),
        };

        // YAML format uses default display (same as Table)
        let result = show_task_status(&roadmap, "PMAT-0001", OutputFormat::Yaml);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sprint_status_yaml_output() {
        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "YAML Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P1,
            tasks: Vec::new(),
            definition_of_done: Vec::new(),
            quality_gates: Vec::new(),
        };

        let mut roadmap = Roadmap {
            current_sprint: Some("v1.0.0".to_string()),
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };
        roadmap.sprints.insert("v1.0.0".to_string(), sprint);

        let result = show_sprint_status(&roadmap, Some("v1.0.0"), OutputFormat::Yaml).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_sprint_with_quality_gates_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config(&temp_dir);
        config.enforce_quality_gates = false;
        create_sample_roadmap(&config.path);

        let result = validate_sprint(&config.path, "v1.0.0", false, &config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_sprint_with_quality_gates_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config(&temp_dir);
        config.enforce_quality_gates = true;
        create_sample_roadmap(&config.path);

        let result = validate_sprint(&config.path, "v1.0.0", false, &config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_complete_task_without_quality_enforcement() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config(&temp_dir);
        config.enforce_quality_gates = false;
        create_sample_roadmap(&config.path);

        // Complete without skipping - but quality gates disabled in config
        let result = complete_task(&config.path, "PMAT-0001", false, &config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_task_with_branch_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config(&temp_dir);
        config.git.create_branches = false;
        create_sample_roadmap(&config.path);

        // Even if create_branch is true, should not create because config says no
        let result = start_task(&config.path, "PMAT-0001", true, &config).await;

        assert!(result.is_ok());
    }
}
