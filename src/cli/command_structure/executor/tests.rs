#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for command executor

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::super::*;
    use crate::cli::Commands;
    use std::sync::Arc;

    // ============================================================================
    // Test Fixtures and Helpers
    // ============================================================================

    /// Creates a test server instance for testing command execution
    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("Failed to create test server"))
    }

    // ============================================================================
    // CommandExecutor Tests
    // ============================================================================

    #[test]
    fn test_command_executor_new() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server.clone());

        // Verify executor is created successfully
        assert!(Arc::ptr_eq(&executor.server, &server));
    }

    #[test]
    fn test_command_executor_has_registry() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // Registry should be accessible (implicitly tested via execute)
        let _ = &executor.registry;
    }

    // ============================================================================
    // CommandRegistry Tests
    // ============================================================================

    #[test]
    fn test_command_registry_creation() {
        let registry = CommandRegistry::default();

        // Verify all command groups are initialized by accessing them
        let _ = &registry.generate_handlers;
        let _ = &registry.analyze_handlers;
        let _ = &registry.utility_handlers;
        let _ = &registry.demo_handlers;
    }

    #[test]
    fn test_command_registry_default_trait() {
        // Test that Default trait is properly implemented
        let registry1 = CommandRegistry::default();
        let registry2 = CommandRegistry::default();

        // Both should be valid (we can't compare them, but they should exist)
        let _ = registry1;
        let _ = registry2;
    }

    // ============================================================================
    // Command Group Default Tests
    // ============================================================================

    #[test]
    fn test_generate_command_group_default() {
        let group = GenerateCommandGroup;
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_analyze_command_group_default() {
        let group = AnalyzeCommandGroup;
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_utility_command_group_default() {
        let group = UtilityCommandGroup;
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_demo_command_group_default() {
        let group = DemoCommandGroup;
        // Verify it can be created
        let _ = group;
    }

    #[test]
    fn test_command_group_defaults() {
        let _generate = GenerateCommandGroup;
        let _analyze = AnalyzeCommandGroup;
        let _utility = UtilityCommandGroup;
        let _demo = DemoCommandGroup;

        // All groups should be creatable via unit struct syntax
    }

    // ============================================================================
    // CommandExecutorFactory Tests
    // ============================================================================

    #[test]
    fn test_command_executor_factory_create() {
        let server = create_test_server();
        let executor = CommandExecutorFactory::create(server.clone());

        // Verify factory creates a valid executor
        assert!(Arc::ptr_eq(&executor.server, &server));
    }

    #[test]
    fn test_command_executor_factory_creates_with_registry() {
        let server = create_test_server();
        let executor = CommandExecutorFactory::create(server);

        // Verify registry is accessible
        let _ = &executor.registry;
    }

    // ============================================================================
    // Command Execution Error Cases (Commands that should bail)
    // ============================================================================

    #[tokio::test]
    async fn test_execute_show_metrics_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // ShowMetrics uses OutputFormat, not MetricsOutputFormat
        let command = Commands::ShowMetrics {
            trend: false,
            days: 30,
            metric: None,
            format: crate::cli::OutputFormat::Table,
            failures_only: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_predict_quality_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // PredictQuality uses correct field names
        let command = Commands::PredictQuality {
            metric: None,
            threshold: None,
            days: 30,
            format: crate::cli::OutputFormat::Table,
            all: false,
            failures_only: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_record_metric_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // RecordMetric uses String metric and f64 value
        let command = Commands::RecordMetric {
            metric: "lint".to_string(),
            value: 1000.0,
            timestamp: None,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_work_command_should_bail() {
        use crate::cli::commands::WorkCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // WorkCommands::Start requires id and optional fields
        let command = Commands::Work {
            command: WorkCommands::Start {
                id: "123".to_string(),
                with_spec: false,
                epic: false,
                path: None,
                create_github: false,
                profile: None,
                without: None,
                iteration: 1,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_qa_work_should_bail() {
        use crate::cli::commands::QaWorkCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // QaWork uses subcommand
        let command = Commands::QaWork {
            command: QaWorkCommands::Validate {
                task_id: "123".to_string(),
                path: std::path::PathBuf::from("."),
                strict: false,
                format: crate::cli::commands::QaOutputFormat::Text,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_comply_should_bail() {
        use crate::cli::commands::ComplyCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // ComplyCommands::Check is the correct variant
        let command = Commands::Comply {
            command: Some(ComplyCommands::Check {
                path: std::path::PathBuf::from("."),
                strict: false,
                failures_only: false,
                format: crate::cli::commands::ComplyOutputFormat::Text,
                include_project: vec![],
            }),
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_project_diag_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // ProjectDiag has correct field names
        let command = Commands::ProjectDiag {
            path: std::path::PathBuf::from("."),
            format: crate::cli::commands::ProjectDiagOutputFormat::Summary,
            category: None,
            failures_only: false,
            output: None,
            quiet: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_test_discovery_should_bail() {
        use crate::cli::commands::TestDiscoveryCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // TestDiscovery uses subcommand
        let command = Commands::TestDiscovery {
            command: TestDiscoveryCommands::Run {
                path: std::path::PathBuf::from("."),
                output: std::path::PathBuf::from("test-failures.json"),
                use_nextest: true,
                timeout: 600,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_debug_five_whys_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // DebugFiveWhys uses DebugOutputFormat
        let command = Commands::DebugFiveWhys {
            issue: "test issue".to_string(),
            depth: 5,
            format: crate::cli::DebugOutputFormat::Text,
            output: None,
            path: std::path::PathBuf::from("."),
            context: None,
            auto_analyze: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_localize_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // Localize has correct field names
        let command = Commands::Localize {
            passed_coverage: std::path::PathBuf::from("passed.lcov"),
            failed_coverage: std::path::PathBuf::from("failed.lcov"),
            passed_count: 10,
            failed_count: 2,
            formula: "tarantula".to_string(),
            top_n: 10,
            output: None,
            format: "terminal".to_string(),
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_oracle_should_bail() {
        use crate::cli::commands::OracleCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        let command = Commands::Oracle {
            command: OracleCommands::Status {
                path: std::path::PathBuf::from("."),
                format: crate::cli::commands::OracleOutputFormat::Text,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_perfection_score_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // PerfectionScore has correct field names
        let command = Commands::PerfectionScore {
            path: std::path::PathBuf::from("."),
            breakdown: false,
            target: None,
            format: crate::cli::commands::PerfectionScoreOutputFormat::Text,
            output: None,
            fast: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_spec_should_bail() {
        use crate::cli::commands::SpecCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        let command = Commands::Spec {
            command: SpecCommands::List {
                path: std::path::PathBuf::from("docs/specifications"),
                min_score: None,
                failing_only: false,
                format: crate::cli::commands::SpecOutputFormat::Text,
            },
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    #[tokio::test]
    async fn test_execute_cuda_tdg_should_bail() {
        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        // CudaTdg has correct field names
        let command = Commands::CudaTdg {
            path: std::path::PathBuf::from("."),
            command: None,
            format: crate::cli::commands::CudaTdgOutputFormat::Terminal,
            min_score: 85.0,
            fail_on_p0: false,
            simd: false,
            wgpu: false,
            output: None,
            quiet: false,
        };

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command_dispatcher.rs"));
    }

    // ============================================================================
    // Org Command Feature Gate Tests
    // ============================================================================

    #[tokio::test]
    #[cfg(not(feature = "org-intelligence"))]
    async fn test_execute_org_without_feature_should_bail() {
        use crate::cli::commands::OrgCommands;

        let server = create_test_server();
        let executor = CommandExecutor::new(server);

        let command = Commands::Org(OrgCommands::Analyze {
            org: "test-org".to_string(),
            output: std::path::PathBuf::from("output.json"),
            max_concurrent: 5,
            summarize: false,
            strip_pii: false,
            top_n: 10,
            min_frequency: 3,
        });

        let result = executor.execute(command).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("org-intelligence"));
    }
}
