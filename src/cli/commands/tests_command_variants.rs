    #[test]
    fn test_scaffold_commands_variants() {
        let project = ScaffoldCommands::Project {
            toolchain: "rust".to_string(),
            templates: vec!["cli".to_string(), "lib".to_string()],
            params: vec![("name".to_string(), Value::String("test".to_string()))],
            parallel: 4,
        };

        let agent = ScaffoldCommands::Agent {
            name: "test-agent".to_string(),
            template: "basic".to_string(),
            features: vec!["logging".to_string()],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: false,
            interactive: false,
            deterministic_core: Some("state-machine".to_string()),
            probabilistic_wrapper: None,
        };

        match project {
            ScaffoldCommands::Project {
                toolchain,
                templates,
                params,
                parallel,
            } => {
                assert_eq!(toolchain, "rust");
                assert_eq!(templates, vec!["cli", "lib"]);
                assert_eq!(params.len(), 1);
                assert_eq!(parallel, 4);
            }
            _ => panic!("Expected Project variant"),
        }

        match agent {
            ScaffoldCommands::Agent {
                name,
                template,
                features,
                quality,
                output,
                force,
                dry_run,
                interactive,
                deterministic_core,
                probabilistic_wrapper,
            } => {
                assert_eq!(name, "test-agent");
                assert_eq!(template, "basic");
                assert_eq!(features, vec!["logging"]);
                assert_eq!(quality, "strict");
                assert!(output.is_none());
                assert!(!force);
                assert!(!dry_run);
                assert!(!interactive);
                assert_eq!(deterministic_core, Some("state-machine".to_string()));
                assert!(probabilistic_wrapper.is_none());
            }
            _ => panic!("Expected Agent variant"),
        }
    }

    #[test]
    fn test_roadmap_commands_variants() {
        let init = RoadmapCommands::Init {
            version: "v2.6.0".to_string(),
            title: "Test Sprint".to_string(),
            duration_days: 14,
            priority: "P0".to_string(),
        };

        let start = RoadmapCommands::Start {
            task_id: "task-123".to_string(),
            create_branch: false,
        };

        let complete = RoadmapCommands::Complete {
            task_id: "task-123".to_string(),
            skip_quality_check: true,
        };

        match init {
            RoadmapCommands::Init {
                version,
                title,
                duration_days,
                priority,
            } => {
                assert_eq!(version, "v2.6.0");
                assert_eq!(title, "Test Sprint");
                assert_eq!(duration_days, 14);
                assert_eq!(priority, "P0");
            }
            _ => panic!("Expected Init variant"),
        }

        match start {
            RoadmapCommands::Start {
                task_id,
                create_branch,
            } => {
                assert_eq!(task_id, "task-123");
                assert!(!create_branch);
            }
            _ => panic!("Expected Start variant"),
        }

        match complete {
            RoadmapCommands::Complete {
                task_id,
                skip_quality_check,
            } => {
                assert_eq!(task_id, "task-123");
                assert!(skip_quality_check);
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    #[test]
    fn test_test_suite_variants() {
        let performance = TestSuite::Performance;
        let integration = TestSuite::Integration;
        let property = TestSuite::Property;
        let memory = TestSuite::Memory;

        assert_eq!(performance, TestSuite::Performance);
        assert_eq!(integration, TestSuite::Integration);
        assert_eq!(property, TestSuite::Property);
        assert_eq!(memory, TestSuite::Memory);
    }

    #[test]
    fn test_serve_transport_variants() {
        let http = ServeTransport::Http;
        let websocket = ServeTransport::WebSocket;

        assert_eq!(http, ServeTransport::Http);
        assert_eq!(websocket, ServeTransport::WebSocket);
    }

    #[test]
    fn test_agent_commands_variants() {
        let status = AgentCommands::Status {
            pid_file: None,
            format: OutputFormat::Json,
        };

        let stop = AgentCommands::Stop {
            pid_file: None,
            force: false,
            timeout: 10,
        };

        match status {
            AgentCommands::Status { format, .. } => {
                assert_eq!(format, OutputFormat::Json);
            }
            _ => panic!("Expected Status variant"),
        }

        match stop {
            AgentCommands::Stop { force, timeout, .. } => {
                assert!(!force);
                assert_eq!(timeout, 10);
            }
            _ => panic!("Expected Stop variant"),
        }
    }

    #[test]
    fn test_commands_generate_variant() {
        let generate = Commands::Generate {
            category: "makefile".to_string(),
            template: "rust/cli".to_string(),
            params: vec![("name".to_string(), Value::String("test".to_string()))],
            output: Some(PathBuf::from("Makefile")),
            create_dirs: true,
        };

        match generate {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                assert_eq!(category, "makefile");
                assert_eq!(template, "rust/cli");
                assert_eq!(params.len(), 1);
                assert_eq!(output, Some(PathBuf::from("Makefile")));
                assert!(create_dirs);
            }
            _ => panic!("Expected Generate variant"),
        }
    }

    #[test]
    fn test_commands_list_variant() {
        let list = Commands::List {
            toolchain: Some("rust".to_string()),
            category: Some("cli".to_string()),
            format: OutputFormat::Json,
        };

        match list {
            Commands::List {
                toolchain,
                category,
                format,
            } => {
                assert_eq!(toolchain, Some("rust".to_string()));
                assert_eq!(category, Some("cli".to_string()));
                assert_eq!(format, OutputFormat::Json);
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_commands_search_variant() {
        let search = Commands::Search {
            query: "rust cli".to_string(),
            toolchain: Some("rust".to_string()),
            limit: 10,
        };

        match search {
            Commands::Search {
                query,
                toolchain,
                limit,
            } => {
                assert_eq!(query, "rust cli");
                assert_eq!(toolchain, Some("rust".to_string()));
                assert_eq!(limit, 10);
            }
            _ => panic!("Expected Search variant"),
        }
    }

    #[test]
    fn test_commands_validate_variant() {
        let validate = Commands::Validate {
            uri: "template://rust/cli".to_string(),
            params: vec![("name".to_string(), Value::String("test".to_string()))],
        };

        match validate {
            Commands::Validate { uri, params } => {
                assert_eq!(uri, "template://rust/cli");
                assert_eq!(params.len(), 1);
            }
            _ => panic!("Expected Validate variant"),
        }
    }

    #[test]
    fn test_commands_context_variant() {
        let context = Commands::Context {
            toolchain: Some("rust".to_string()),
            project_path: PathBuf::from("."),
            output: Some(PathBuf::from("context.md")),
            format: ContextFormat::Markdown,
            include_large_files: false,
            skip_expensive_metrics: true,
            language: None,
            languages: None,
        };

        match context {
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
                include_large_files,
                skip_expensive_metrics,
                language,
                languages,
            } => {
                assert_eq!(toolchain, Some("rust".to_string()));
                assert_eq!(project_path, PathBuf::from("."));
                assert_eq!(output, Some(PathBuf::from("context.md")));
                assert_eq!(format, ContextFormat::Markdown);
                assert!(!include_large_files);
                assert!(skip_expensive_metrics);
                assert_eq!(language, None);
                assert_eq!(languages, None);
            }
            _ => panic!("Expected Context variant"),
        }
    }

    #[test]
    fn test_commands_serve_variant() {
        let serve = Commands::Serve {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors: true,
            transport: ServeTransport::Http,
        };

        match serve {
            Commands::Serve {
                host, port, cors, ..
            } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 3000);
                assert!(cors);
            }
            _ => panic!("Expected Serve variant"),
        }
    }
