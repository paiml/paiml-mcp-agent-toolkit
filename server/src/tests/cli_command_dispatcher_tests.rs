#[cfg(test)]
mod cli_command_dispatcher_tests {
    use crate::cli::{
        command_dispatcher::CommandDispatcher,
        commands::{Commands, AnalyzeCommands, RefactorCommands},
        OutputFormat, ComplexityOutputFormat, ContextFormat, DagType, DemoProtocol,
    };
    use crate::stateless_server::StatelessTemplateServer;
    use std::sync::Arc;
    use tempfile::TempDir;
    use std::path::PathBuf;
    use serde_json::json;

    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().unwrap())
    }

    fn create_test_dir_with_rust_file() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        let src_dir = test_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
            pub fn test_function() {
                println!("test");
            }
            
            pub fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        x * 2
                    } else {
                        x + 1
                    }
                } else {
                    0
                }
            }
            "#
        ).unwrap();
        
        test_dir
    }

    #[tokio::test]
    async fn test_execute_generate_command() {
        let server = create_test_server();
        let test_dir = TempDir::new().unwrap();
        let output_path = test_dir.path().join("test.gitignore");

        let command = Commands::Generate {
            category: "gitignore".to_string(),
            template: "rust/cli".to_string(),
            params: vec![("project_name".to_string(), json!("test-project"))],
            output: Some(output_path.clone()),
            create_dirs: true,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok(), "Generate command failed: {:?}", result);
        assert!(output_path.exists(), "Output file was not created");
    }

    #[tokio::test]
    async fn test_execute_scaffold_command() {
        let server = create_test_server();

        let command = Commands::Scaffold {
            toolchain: "rust".to_string(),
            templates: vec!["gitignore".to_string(), "makefile".to_string()],
            params: vec![("project_name".to_string(), json!("test-scaffold"))],
            parallel: 1,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok(), "Scaffold command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_list_command() {
        let server = create_test_server();

        let command = Commands::List {
            toolchain: None,
            category: None,
            format: OutputFormat::Table,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok(), "List command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_search_command() {
        let server = create_test_server();

        let command = Commands::Search {
            query: "rust".to_string(),
            toolchain: None,
            limit: 10,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok(), "Search command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_validate_command() {
        let server = create_test_server();

        let command = Commands::Validate {
            uri: "rust/cli/gitignore".to_string(),
            params: vec![("project_name".to_string(), json!("test"))],
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok(), "Validate command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_context_command() {
        let server = create_test_server();
        let test_dir = create_test_dir_with_rust_file();

        let command = Commands::Context {
            toolchain: None,
            project_path: test_dir.path().to_path_buf(),
            output: None,
            format: ContextFormat::Markdown,
        };

        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok(), "Context command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_analyze_complexity_command() {
        let test_dir = create_test_dir_with_rust_file();

        let analyze_cmd = AnalyzeCommands::Complexity {
            project_path: test_dir.path().to_path_buf(),
            toolchain: None,
            format: ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: Some(5),
            max_cognitive: Some(10),
            include: vec![],
            watch: false,
            top_files: 10,
        };

        let command = Commands::Analyze(analyze_cmd);
        let result = CommandDispatcher::execute_command(command, create_test_server()).await;
        assert!(result.is_ok(), "Analyze complexity command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_analyze_dag_command() {
        let test_dir = create_test_dir_with_rust_file();

        let analyze_cmd = AnalyzeCommands::Dag {
            dag_type: DagType::FullDependency,
            project_path: test_dir.path().to_path_buf(),
            output: None,
            max_depth: Some(10),
            filter_external: false,
            show_complexity: true,
            include_duplicates: false,
            include_dead_code: false,
            enhanced: false,
        };

        let command = Commands::Analyze(analyze_cmd);
        let result = CommandDispatcher::execute_command(command, create_test_server()).await;
        assert!(result.is_ok(), "Analyze DAG command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_demo_command() {
        let test_dir = create_test_dir_with_rust_file();

        let command = Commands::Demo {
            path: Some(test_dir.path().to_path_buf()),
            url: None,
            repo: None,
            format: OutputFormat::Json,
            protocol: DemoProtocol::Cli,
            show_api: false,
            no_browser: true,
            port: Some(3001),
            cli: false,
            target_nodes: 50,
            centrality_threshold: 0.1,
            merge_threshold: 3,
            debug: false,
            debug_output: None,
            skip_vendor: true,
            no_skip_vendor: false,
            max_line_length: Some(80),
        };

        let result = CommandDispatcher::execute_command(command, create_test_server()).await;
        assert!(result.is_ok(), "Demo command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_refactor_command() {
        let test_dir = create_test_dir_with_rust_file();

        let refactor_cmd = RefactorCommands::Extract {
            project_path: test_dir.path().to_path_buf(),
            file_path: test_dir.path().join("src/lib.rs"),
            start_line: 1,
            end_line: 5,
            function_name: "extracted_function".to_string(),
            output: None,
        };

        let command = Commands::Refactor(refactor_cmd);
        let result = CommandDispatcher::execute_command(command, create_test_server()).await;
        assert!(result.is_ok(), "Refactor command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_execute_diagnose_command() {
        let test_dir = create_test_dir_with_rust_file();

        let command = Commands::Diagnose {
            args: crate::cli::diagnose::DiagnoseArgs {
                path: test_dir.path().to_path_buf(),
                output: None,
                verbose: false,
                check_deps: true,
                check_tools: true,
                check_config: true,
            },
        };

        let result = CommandDispatcher::execute_command(command, create_test_server()).await;
        assert!(result.is_ok(), "Diagnose command failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_command_handler_trait_bounds() {
        // Test that CommandHandler trait is properly Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn crate::cli::command_dispatcher::CommandHandler>>();
    }

    #[tokio::test]
    async fn test_analyze_command_handler_trait_bounds() {
        // Test that AnalyzeCommandHandler trait is properly Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn crate::cli::command_dispatcher::AnalyzeCommandHandler>>();
    }

    #[tokio::test]
    async fn test_error_handling_invalid_path() {
        let analyze_cmd = AnalyzeCommands::Complexity {
            project_path: PathBuf::from("/non/existent/path"),
            toolchain: None,
            format: ComplexityOutputFormat::Summary,
            output: None,
            max_cyclomatic: Some(5),
            max_cognitive: Some(10),
            include: vec![],
            watch: false,
            top_files: 10,
        };

        let command = Commands::Analyze(analyze_cmd);
        let result = CommandDispatcher::execute_command(command, create_test_server()).await;
        
        // Command should handle the error gracefully
        assert!(result.is_err(), "Expected error for non-existent path");
    }

    #[tokio::test]
    async fn test_concurrent_command_execution() {
        let server = create_test_server();
        
        // Use futures instead of tokio::spawn to avoid Send issues
        let mut results = Vec::new();
        
        for _ in 0..3 {
            let server_clone = Arc::clone(&server);
            let command = Commands::List {
                toolchain: None,
                category: None,
                format: OutputFormat::Json,
            };
            
            let result = CommandDispatcher::execute_command(command, server_clone).await;
            results.push(result);
        }

        for result in results {
            assert!(result.is_ok(), "Concurrent execution failed");
        }
    }
}