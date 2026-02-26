    // ========== EXECUTE FUNCTION TESTS ==========

    #[tokio::test]
    async fn test_execute_init_command() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        let cmd = RoadmapCommand {
            command: RoadmapSubcommand::Init {
                version: "v1.0.0".to_string(),
                title: "Test Sprint".to_string(),
                duration_days: 14,
                priority: "P0".to_string(),
            },
        };

        let result = execute(cmd, config.clone()).await;

        assert!(result.is_ok());
        assert!(config.path.exists());
    }

    #[tokio::test]
    async fn test_execute_start_command() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let cmd = RoadmapCommand {
            command: RoadmapSubcommand::Start {
                task_id: "PMAT-0001".to_string(),
                create_branch: false,
            },
        };

        let result = execute(cmd, config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_complete_command() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let cmd = RoadmapCommand {
            command: RoadmapSubcommand::Complete {
                task_id: "PMAT-0001".to_string(),
                skip_quality_check: true,
            },
        };

        let result = execute(cmd, config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_status_command() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let cmd = RoadmapCommand {
            command: RoadmapSubcommand::Status {
                sprint: Some("v1.0.0".to_string()),
                task: None,
                format: OutputFormat::Json,
            },
        };

        let result = execute(cmd, config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_todos_command() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let output_path = temp_dir.path().join("generated_todos.md");

        let cmd = RoadmapCommand {
            command: RoadmapSubcommand::Todos {
                sprint: Some("v1.0.0".to_string()),
                output: output_path.clone(),
                include_quality_gates: false,
            },
        };

        let result = execute(cmd, config).await;

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_execute_validate_command() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        create_sample_roadmap(&config.path);

        let cmd = RoadmapCommand {
            command: RoadmapSubcommand::Validate {
                sprint: "v1.0.0".to_string(),
                strict: false,
            },
        };

        let result = execute(cmd, config).await;

        assert!(result.is_ok());
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
