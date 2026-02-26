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

    // ========== CLI PARSING TESTS ==========

    #[test]
    fn test_status_subcommand_parsing_with_sprint() {
        let cmd = RoadmapCommand::try_parse_from([
            "roadmap", "status", "--sprint", "v1.0.0", "--format", "json",
        ]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Status {
                    sprint,
                    task,
                    format,
                } => {
                    assert_eq!(sprint, Some("v1.0.0".to_string()));
                    assert!(task.is_none());
                    assert_eq!(format, OutputFormat::Json);
                }
                _ => panic!("Expected Status subcommand"),
            }
        }
    }

    #[test]
    fn test_status_subcommand_parsing_with_task() {
        let cmd = RoadmapCommand::try_parse_from([
            "roadmap",
            "status",
            "--task",
            "PMAT-0001",
            "--format",
            "json",
        ]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Status {
                    sprint,
                    task,
                    format,
                } => {
                    assert!(sprint.is_none());
                    assert_eq!(task, Some("PMAT-0001".to_string()));
                    assert_eq!(format, OutputFormat::Json);
                }
                _ => panic!("Expected Status subcommand"),
            }
        }
    }

    #[test]
    fn test_validate_subcommand_parsing() {
        let cmd = RoadmapCommand::try_parse_from([
            "roadmap", "validate", "--sprint", "v1.0.0", "--strict",
        ]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Validate { sprint, strict } => {
                    assert_eq!(sprint, "v1.0.0");
                    assert!(strict);
                }
                _ => panic!("Expected Validate subcommand"),
            }
        }
    }

    #[test]
    fn test_validate_subcommand_without_strict() {
        let cmd = RoadmapCommand::try_parse_from(["roadmap", "validate", "--sprint", "v2.0.0"]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Validate { sprint, strict } => {
                    assert_eq!(sprint, "v2.0.0");
                    assert!(!strict);
                }
                _ => panic!("Expected Validate subcommand"),
            }
        }
    }

    #[test]
    fn test_quality_check_subcommand_parsing() {
        let cmd =
            RoadmapCommand::try_parse_from(["roadmap", "quality-check", "--task-id", "PMAT-1234"]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::QualityCheck { task_id } => {
                    assert_eq!(task_id, "PMAT-1234");
                }
                _ => panic!("Expected QualityCheck subcommand"),
            }
        }
    }

    #[test]
    fn test_init_subcommand_with_defaults() {
        let cmd = RoadmapCommand::try_parse_from([
            "roadmap",
            "init",
            "--version",
            "v3.0.0",
            "--title",
            "New Sprint",
        ]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Init {
                    version,
                    title,
                    duration_days,
                    priority,
                } => {
                    assert_eq!(version, "v3.0.0");
                    assert_eq!(title, "New Sprint");
                    assert_eq!(duration_days, 14); // Default
                    assert_eq!(priority, "P0"); // Default
                }
                _ => panic!("Expected Init subcommand"),
            }
        }
    }

    #[test]
    fn test_todos_subcommand_defaults() {
        let cmd = RoadmapCommand::try_parse_from(["roadmap", "todos"]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Todos {
                    sprint,
                    output,
                    include_quality_gates,
                } => {
                    assert!(sprint.is_none());
                    assert_eq!(output, PathBuf::from("todos.md")); // Default
                    assert!(!include_quality_gates); // Default
                }
                _ => panic!("Expected Todos subcommand"),
            }
        }
    }

    #[test]
    fn test_start_subcommand_defaults() {
        let cmd = RoadmapCommand::try_parse_from(["roadmap", "start", "PMAT-5001"]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Start {
                    task_id,
                    create_branch,
                } => {
                    assert_eq!(task_id, "PMAT-5001");
                    assert!(!create_branch); // Default
                }
                _ => panic!("Expected Start subcommand"),
            }
        }
    }

    #[test]
    fn test_complete_subcommand_defaults() {
        let cmd = RoadmapCommand::try_parse_from(["roadmap", "complete", "PMAT-6001"]);

        assert!(cmd.is_ok());
        if let Ok(parsed) = cmd {
            match parsed.command {
                RoadmapSubcommand::Complete {
                    task_id,
                    skip_quality_check,
                } => {
                    assert_eq!(task_id, "PMAT-6001");
                    assert!(!skip_quality_check); // Default
                }
                _ => panic!("Expected Complete subcommand"),
            }
        }
    }
