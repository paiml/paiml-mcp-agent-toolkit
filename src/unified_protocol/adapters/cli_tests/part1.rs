// Tests for CLI adapter
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use crate::cli::commands::ServeTransport;
    use crate::cli::{
        AnalyzeCommands, Commands, ComplexityOutputFormat, DagType, DeadCodeOutputFormat,
        OutputFormat, SatdOutputFormat,
    };
    use crate::cli::{
        DeepContextCacheStrategy, DeepContextDagType, DeepContextOutputFormat, DemoProtocol,
    };
    use crate::models::churn::ChurnOutputFormat;
    use crate::unified_protocol::{HeaderMap, StatusCode, Uuid};
    use serde_json::{json, Value};
    use std::path::PathBuf;

    #[test]
    fn test_cli_input_creation() {
        let params = vec![
            (
                "project_name".to_string(),
                Value::String("test".to_string()),
            ),
            ("version".to_string(), Value::String("1.0.0".to_string())),
        ];

        let command = Commands::Generate {
            category: "makefile".to_string(),
            template: "rust/cli".to_string(),
            params,
            output: Some(PathBuf::from("Makefile")),
            create_dirs: true,
        };

        let input = CliInput::from_commands(command);
        assert_eq!(input.command_name, "generate");
    }

    #[tokio::test]
    async fn test_cli_adapter_decode_generate() {
        let adapter = CliAdapter::new();
        let params = vec![(
            "project_name".to_string(),
            Value::String("test".to_string()),
        )];

        let command = Commands::Generate {
            category: "makefile".to_string(),
            template: "rust/cli".to_string(),
            params,
            output: None,
            create_dirs: false,
        };

        let input = CliInput::from_commands(command);
        let unified_request = adapter.decode(input).await.unwrap();

        assert_eq!(unified_request.method, Method::POST);
        assert_eq!(unified_request.path, "/api/v1/generate");
        assert_eq!(
            unified_request.get_extension::<Protocol>("protocol"),
            Some(Protocol::Cli)
        );

        let cli_context: CliContext = unified_request.get_extension("cli_context").unwrap();
        assert_eq!(cli_context.command, "generate");
    }

    #[tokio::test]
    async fn test_cli_adapter_decode_list() {
        let adapter = CliAdapter::new();
        let command = Commands::List {
            toolchain: Some("rust".to_string()),
            category: None,
            format: OutputFormat::Json,
        };

        let input = CliInput::from_commands(command);
        let unified_request = adapter.decode(input).await.unwrap();

        assert_eq!(unified_request.method, Method::GET);
        assert!(unified_request.path.starts_with("/api/v1/templates"));
        assert!(unified_request.path.contains("toolchain=rust"));
    }

    #[tokio::test]
    async fn test_cli_adapter_decode_analyze_complexity() {
        let adapter = CliAdapter::new();
        let command = Commands::Analyze(AnalyzeCommands::Complexity {
            path: PathBuf::from("."),
            project_path: None,
            file: None,
            files: vec![],
            toolchain: Some("rust".to_string()),
            format: ComplexityOutputFormat::Json,
            output: None,
            max_cyclomatic: Some(10),
            max_cognitive: Some(15),
            include: vec!["**/*.rs".to_string()],
            watch: false,
            top_files: 0,
            fail_on_violation: false,
            timeout: 60,
            ml: false,
        });

        let input = CliInput::from_commands(command);
        let unified_request = adapter.decode(input).await.unwrap();

        assert_eq!(unified_request.method, Method::POST);
        assert_eq!(unified_request.path, "/api/v1/analyze/complexity");

        // Verify body contains expected fields
        let body_bytes = axum::body::to_bytes(unified_request.body, usize::MAX)
            .await
            .unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["toolchain"], "rust");
        assert_eq!(body_json["max_cyclomatic"], 10);
        assert_eq!(body_json["max_cognitive"], 15);
    }

    #[tokio::test]
    async fn test_cli_adapter_encode_success() {
        let adapter = CliAdapter::new();
        let response = UnifiedResponse::ok()
            .with_json(&json!({"message": "success"}))
            .unwrap();

        let output = adapter.encode(response).await.unwrap();
        match output {
            CliOutput::Success { content, exit_code } => {
                assert_eq!(exit_code, 0);
                assert!(content.contains("success"));
            }
            _ => panic!("Expected success output"),
        }
    }

    #[tokio::test]
    async fn test_cli_adapter_encode_error() {
        let adapter = CliAdapter::new();
        let response = UnifiedResponse::new(axum::http::StatusCode::BAD_REQUEST)
            .with_json(&json!({"error": "Invalid request"}))
            .unwrap();

        let output = adapter.encode(response).await.unwrap();
        match output {
            CliOutput::Error { message, exit_code } => {
                assert_eq!(exit_code, 1);
                assert!(message.contains("Invalid request"));
            }
            _ => panic!("Expected error output"),
        }
    }

    #[test]
    fn test_format_conversions() {
        assert_eq!(format_to_string(&ContextFormat::Markdown), "markdown");
        assert_eq!(format_to_string(&ContextFormat::Json), "json");

        assert_eq!(
            churn_format_to_string(&ChurnOutputFormat::Summary),
            "summary"
        );
        assert_eq!(churn_format_to_string(&ChurnOutputFormat::Json), "json");

        assert_eq!(
            complexity_format_to_string(&ComplexityOutputFormat::Sarif),
            "sarif"
        );

        assert_eq!(dag_type_to_string(&DagType::CallGraph), "call-graph");
        assert_eq!(
            dag_type_to_string(&DagType::FullDependency),
            "full-dependency"
        );
    }

    #[test]
    fn test_cli_output_methods() {
        let success = CliOutput::Success {
            content: "test content".to_string(),
            exit_code: 0,
        };
        assert_eq!(success.exit_code(), 0);
        assert_eq!(success.content(), "test content");

        let error = CliOutput::Error {
            message: "test error".to_string(),
            exit_code: 1,
        };
        assert_eq!(error.exit_code(), 1);
        assert_eq!(error.content(), "test error");
    }

    #[test]
    fn test_cli_adapter_new() {
        let _ = CliAdapter::new();
        // Verify the adapter is created successfully
        // size_of_val always returns >= 0 for any type
    }

    #[test]
    fn test_cli_adapter_default() {
        let _ = CliAdapter;
        // Verify default creation works
        // size_of_val always returns >= 0 for any type
    }

    #[tokio::test]
    async fn test_decode_scaffold_project() {
        let adapter = CliAdapter::new();
        let params = vec![(
            "project_name".to_string(),
            Value::String("test_project".to_string()),
        )];

        let command = Commands::Scaffold {
            command: ScaffoldCommands::Project {
                toolchain: "rust".to_string(),
                templates: vec!["cli".to_string()],
                params,
                parallel: 4,
            },
        };

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/scaffold");
    }

    #[tokio::test]
    async fn test_decode_search() {
        let adapter = CliAdapter::new();
        let command = Commands::Search {
            query: "rust cli".to_string(),
            toolchain: Some("rust".to_string()),
            limit: 10,
        };

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        // For POST search, just verify the path
        assert_eq!(request.path, "/api/v1/search");
    }

    #[tokio::test]
    async fn test_decode_validate() {
        let adapter = CliAdapter::new();
        let params = vec![("key".to_string(), Value::String("value".to_string()))];
        let command = Commands::Validate {
            uri: "template://rust/cli".to_string(),
            params,
        };

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/validate");
    }

    #[tokio::test]
    async fn test_decode_context() {
        let adapter = CliAdapter::new();
        let command = Commands::Context {
            toolchain: Some("rust".to_string()),
            project_path: PathBuf::from("/test/project"),
            output: Some(PathBuf::from("context.md")),
            format: ContextFormat::Markdown,
            include_large_files: false,
            skip_expensive_metrics: true,
            language: None,
            languages: None,
        };

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/analyze/context");
    }

    #[tokio::test]
    async fn test_decode_analyze_churn() {
        let adapter = CliAdapter::new();
        let command = Commands::Analyze(AnalyzeCommands::Churn {
            project_path: PathBuf::from("."),
            days: 30,
            format: ChurnOutputFormat::Json,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        });

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/analyze/churn");
    }

    #[tokio::test]
    async fn test_decode_analyze_dag() {
        let adapter = CliAdapter::new();
        let command = Commands::Analyze(AnalyzeCommands::Dag {
            dag_type: DagType::FullDependency,
            project_path: PathBuf::from("."),
            output: None,
            max_depth: Some(5),
            target_nodes: None,
            filter_external: false,
            show_complexity: false,
            include_duplicates: false,
            enhanced: false,
            include_dead_code: false,
        });

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/analyze/dag");
    }

    #[tokio::test]
    async fn test_decode_analyze_dead_code() {
        let adapter = CliAdapter::new();
        let command = Commands::Analyze(AnalyzeCommands::DeadCode {
            path: PathBuf::from("."),
            format: DeadCodeOutputFormat::Json,
            top_files: Some(10),
            include_unreachable: true,
            min_dead_lines: 10,
            include_tests: false,
            output: None,
            fail_on_violation: false,
            max_percentage: 15.0,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            max_depth: 8,
            no_cache: false,
        });

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/analyze/dead-code");
    }

    #[tokio::test]
    async fn test_decode_analyze_satd() {
        let adapter = CliAdapter::new();
        let command = Commands::Analyze(AnalyzeCommands::Satd {
            path: PathBuf::from("."),
            format: SatdOutputFormat::Json,
            severity: None,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: None,
            top_files: 10,
            fail_on_violation: false,
            timeout: 60,
            include: vec![],
            exclude: vec![],
        });

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/analyze/satd");
    }

    #[tokio::test]
    async fn test_decode_demo() {
        let adapter = CliAdapter::new();
        let command = Commands::Demo {
            path: Some(PathBuf::from(".")),
            url: None,
            repo: None,
            format: OutputFormat::Table,
            protocol: DemoProtocol::Http,
            show_api: false,
            no_browser: false,
            port: Some(8080),
            cli: true,
            target_nodes: 100,
            centrality_threshold: 0.5,
            merge_threshold: 3,
            debug: false,
            debug_output: None,
            skip_vendor: true,
            no_skip_vendor: false,
            max_line_length: None,
        };

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/demo");
    }

    #[tokio::test]
    async fn test_decode_serve() {
        let adapter = CliAdapter::new();
        let command = Commands::Serve {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors: true,
            transport: ServeTransport::Http,
        };

        let input = CliInput::from_commands(command);
        let result = adapter.decode(input).await;
