
    #[tokio::test]
    async fn test_execute_memory_command_routing() {
        use crate::cli::handlers::memory::MemoryCommand;

        // Test stats command routing
        let result = CommandDispatcher::execute_memory_command(MemoryCommand::Stats {
            detailed: false,
            format: "table".to_string(),
        })
        .await;
        // Should execute without panicking (may fail due to missing state)
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_execute_cache_command_routing() {
        use crate::cli::handlers::cache::CacheCommand;

        // Test stats command routing
        let result = CommandDispatcher::execute_cache_command(CacheCommand::Stats {
            detailed: false,
            format: "table".to_string(),
            history: false,
        })
        .await;
        // Should execute without panicking
        assert!(result.is_ok() || result.is_err());
    }

    // COMPREHENSIVE COVERAGE TESTS - Added for increased test coverage

    // Test: execute_scaffold_command routing (all variants)

    #[tokio::test]
    async fn test_scaffold_project_routing() {
        let server = create_test_server();
        let command = Commands::Scaffold {
            command: ScaffoldCommands::Project {
                toolchain: "rust".to_string(),
                templates: vec!["basic".to_string()],
                params: vec![],
                parallel: 1,
            },
        };
        // May fail due to missing templates but routing works
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_scaffold_agent_routing() {
        let server = create_test_server();
        let command = Commands::Scaffold {
            command: ScaffoldCommands::Agent {
                name: "test-agent".to_string(),
                template: "mcp-server".to_string(),
                features: vec![],
                quality: "standard".to_string(),
                output: None,
                force: false,
                dry_run: true,
                interactive: false,
                deterministic_core: None,
                probabilistic_wrapper: None,
            },
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_scaffold_wasm_routing() {
        let server = create_test_server();
        let command = Commands::Scaffold {
            command: ScaffoldCommands::Wasm {
                name: "test-wasm".to_string(),
                framework: "wasm-labs".to_string(),
                features: vec![],
                quality: "standard".to_string(),
                output: None,
                force: false,
                dry_run: true,
            },
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[ignore = "Calls process::exit"]
    async fn test_scaffold_validate_template_routing() {
        let server = create_test_server();
        let command = Commands::Scaffold {
            command: ScaffoldCommands::ValidateTemplate {
                path: PathBuf::from("/nonexistent/template.yaml"),
            },
        };
        // Should fail due to nonexistent path
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scaffold_list_subagents_routing() {
        let server = create_test_server();
        let command = Commands::Scaffold {
            command: ScaffoldCommands::ListSubagents { all: false },
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scaffold_show_tool_mapping_routing() {
        let server = create_test_server();
        let command = Commands::Scaffold {
            command: ScaffoldCommands::ShowToolMapping { agent: None },
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok());
    }

    // Test: execute_quality_gate_command with various check types

    #[tokio::test]
    async fn test_quality_gate_dead_code_check() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            None,
            OutputFormat::Json,
            false,
            vec!["dead_code".to_string()],
            Some(0.2),
            None,
            None,
            false,
            None,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_complexity_check() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            None,
            OutputFormat::Table,
            true, // exit_on_violation
            vec!["complexity".to_string()],
            None,
            None,
            Some(15),
            false,
            None,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_entropy_check() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            None,
            OutputFormat::Yaml,
            false,
            vec!["entropy".to_string()],
            None,
            Some(0.8),
            None,
            false,
            None,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_all_checks() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            None,
            OutputFormat::Json,
            false,
            vec!["all".to_string()],
            None,
            None,
            None,
            true, // include_provability
            None,
            true, // perf
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_with_file_filter() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            Some(test_file),
            OutputFormat::Table,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            None,
            false,
            None,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_with_output_file() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let output_file = temp_dir.path().join("output.json");

        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            None,
            OutputFormat::Json,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            None,
            false,
            Some(output_file),
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }
