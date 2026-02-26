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
