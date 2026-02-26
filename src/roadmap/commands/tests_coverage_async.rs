    // ========== ASYNC FUNCTION TESTS ==========

    #[tokio::test]
    async fn test_init_sprint_creates_new_roadmap() {
        let temp_dir = TempDir::new().unwrap();
        let roadmap_path = temp_dir.path().join("new_roadmap.md");

        let result = init_sprint(&roadmap_path, "v1.0.0", "Initial Sprint", 14, "P0").await;

        assert!(result.is_ok());
        assert!(roadmap_path.exists());

        // Verify the roadmap content
        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();
        assert_eq!(roadmap.current_sprint, Some("v1.0.0".to_string()));
        assert!(roadmap.sprints.contains_key("v1.0.0"));
    }

    #[tokio::test]
    async fn test_init_sprint_adds_to_existing_roadmap() {
        let temp_dir = TempDir::new().unwrap();
        let roadmap_path = temp_dir.path().join("existing_roadmap.md");

        // Create initial roadmap
        create_sample_roadmap(&roadmap_path);

        // Add new sprint
        let result = init_sprint(&roadmap_path, "v2.0.0", "Second Sprint", 7, "P1").await;

        assert!(result.is_ok());

        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();
        assert!(roadmap.sprints.contains_key("v1.0.0"));
        assert!(roadmap.sprints.contains_key("v2.0.0"));
        // Current sprint should remain v1.0.0 (first one set)
        assert_eq!(roadmap.current_sprint, Some("v1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_init_sprint_with_invalid_priority() {
        let temp_dir = TempDir::new().unwrap();
        let roadmap_path = temp_dir.path().join("roadmap.md");

        // Invalid priority should be handled gracefully (defaults to P0)
        let result = init_sprint(&roadmap_path, "v1.0.0", "Test", 14, "INVALID").await;

        assert!(result.is_ok());
        let roadmap = Roadmap::from_file(&roadmap_path).unwrap();
        let sprint = roadmap.get_sprint("v1.0.0").unwrap();
        assert_eq!(sprint.priority, Priority::P0); // Should default
    }

    #[tokio::test]
    async fn test_start_task_updates_status() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let result = start_task(&config.path, "PMAT-0001", false, &config).await;

        assert!(result.is_ok());

        let roadmap = Roadmap::from_file(&config.path).unwrap();
        let task = roadmap.get_task("PMAT-0001").unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_start_task_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let result = start_task(&config.path, "PMAT-9999", false, &config).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_task_with_skip_quality_check() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        // First start the task
        start_task(&config.path, "PMAT-0001", false, &config)
            .await
            .unwrap();

        // Complete with skip quality check
        let result = complete_task(&config.path, "PMAT-0001", true, &config).await;

        assert!(result.is_ok());

        let roadmap = Roadmap::from_file(&config.path).unwrap();
        let task = roadmap.get_task("PMAT-0001").unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_show_status_with_sprint() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let result = show_status(&config.path, Some("v1.0.0"), None, OutputFormat::Json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_status_with_task() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let result = show_status(&config.path, None, Some("PMAT-0001"), OutputFormat::Json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_status_uses_current_sprint() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        // Should use current sprint when none specified
        let result = show_status(&config.path, None, None, OutputFormat::Json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_status_nonexistent_sprint() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let result = show_status(&config.path, Some("v99.0.0"), None, OutputFormat::Json).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_sprint_all_completed() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let roadmap_path = &config.path;

        // Create roadmap with completed task
        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Completed,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
        };

        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Complete Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: vec![task],
            definition_of_done: vec!["All tests pass".to_string()],
            quality_gates: vec!["Coverage > 80%".to_string()],
        };

        let mut roadmap = Roadmap {
            current_sprint: Some("v1.0.0".to_string()),
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };
        roadmap.sprints.insert("v1.0.0".to_string(), sprint);
        roadmap.to_file(roadmap_path).unwrap();

        let result = validate_sprint(roadmap_path, "v1.0.0", false, &config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_sprint_incomplete_strict() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        // Task is in Planned status, so validation should fail in strict mode
        let result = validate_sprint(&config.path, "v1.0.0", true, &config).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_sprint_incomplete_non_strict() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        // Non-strict mode should succeed even with incomplete tasks
        let result = validate_sprint(&config.path, "v1.0.0", false, &config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_todos_with_quality_gates() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let output_path = temp_dir.path().join("todos_output.md");

        let result = generate_todos(
            &config.path,
            Some("v1.0.0"),
            &output_path,
            true, // Include quality gates
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        // Should contain quality requirements
        assert!(content.contains("Quality") || content.contains("Max Complexity"));
    }

    #[tokio::test]
    async fn test_generate_todos_without_quality_gates() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let output_path = temp_dir.path().join("simple_todos.md");

        let result = generate_todos(
            &config.path,
            Some("v1.0.0"),
            &output_path,
            false, // No quality gates
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_generate_todos_uses_current_sprint() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let output_path = temp_dir.path().join("todos.md");

        // No sprint specified, should use current
        let result = generate_todos(&config.path, None, &output_path, false, &config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_todos_nonexistent_sprint() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let output_path = temp_dir.path().join("todos.md");

        let result =
            generate_todos(&config.path, Some("v99.0.0"), &output_path, false, &config).await;

        assert!(result.is_err());
    }
