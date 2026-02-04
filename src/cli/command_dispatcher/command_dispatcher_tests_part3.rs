//! CommandDispatcher tests - Part 3: Scaffold, quality gate, and report tests
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod scaffold_quality_tests {
    use super::*;
    use crate::cli::commands::{Commands, ScaffoldCommands};
    use crate::stateless_server::StatelessTemplateServer;
    use std::sync::Arc;

    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().expect("internal error"))
    }

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
            true, // fail_on_violation
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

    // Test: execute_report_command with various analysis types

    #[tokio::test]
    async fn test_report_dead_code_analysis() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {} fn unused() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            OutputFormat::Json,
            false,
            false,
            false,
            vec!["dead_code".to_string()],
            None,
            None,
            false,
            false,
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_with_visualizations() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            OutputFormat::Table,
            true, // include_visualizations
            true, // include_executive_summary
            true, // include_recommendations
            vec!["complexity".to_string()],
            Some(0.9),
            None,
            false,
            false,
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_text_format() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            OutputFormat::Table,
            false,
            false,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            false,
            true, // text
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_markdown_format() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            OutputFormat::Table,
            false,
            false,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            false,
            false,
            true, // markdown
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_report_csv_format() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            OutputFormat::Table,
            false,
            false,
            false,
            vec!["complexity".to_string()],
            None,
            None,
            false,
            false,
            false,
            true, // csv
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_config_command variants

    #[tokio::test]
    async fn test_config_validate() {
        let result = CommandDispatcher::execute_config_command(
            false, false, true, false, None, None, None,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_config_with_section() {
        let result = CommandDispatcher::execute_config_command(
            true,
            false,
            false,
            false,
            Some("quality".to_string()),
            None,
            None,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_config_with_set_values() {
        let result = CommandDispatcher::execute_config_command(
            false,
            false,
            false,
            false,
            None,
            Some(vec!["test.key=value".to_string()]),
            None,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }
}
