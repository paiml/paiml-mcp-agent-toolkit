#[cfg(test)]
mod demo_core_extraction_tests {
    use anyhow::Result;
    use paiml_mcp_agent_toolkit::demo::config::{ConfigManager, DisplayConfig};
    use paiml_mcp_agent_toolkit::demo::export::{create_export_report, ExportService};
    use paiml_mcp_agent_toolkit::demo::DemoRunner;
    use paiml_mcp_agent_toolkit::stateless_server::StatelessTemplateServer;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_demo_runner_as_library() -> Result<()> {
        // Skip slow tests in CI
        if std::env::var("SKIP_SLOW_TESTS").is_ok() {
            return Ok(());
        }

        // Create a demo runner programmatically
        let server = Arc::new(StatelessTemplateServer::new()?);
        let mut runner = DemoRunner::new(server);

        // Execute analysis on current directory
        let report = runner.execute_with_diagram(Path::new("."), None).await?;

        // Verify report structure
        assert!(!report.repository.is_empty());
        assert!(!report.steps.is_empty());
        assert!(report.total_time_ms > 0);

        // Check that all expected steps are present
        let step_names: Vec<&str> = report.steps.iter().map(|s| s.capability).collect();

        assert!(step_names.contains(&"AST Context Analysis"));
        assert!(step_names.contains(&"Code Complexity Analysis"));
        assert!(step_names.contains(&"DAG Visualization"));
        assert!(step_names.contains(&"Code Churn Analysis"));

        Ok(())
    }

    #[tokio::test]
    async fn test_config_manager_as_library() -> Result<()> {
        // Create config manager
        let mut config_manager = ConfigManager::new()?;

        // Get default config
        let default_config = config_manager.get_config().await;
        assert_eq!(default_config.version, "1.0");

        // Create custom config
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join(".paiml-display.yaml");

        let custom_yaml = r#"
version: "1.0"
panels:
  dependency:
    max_nodes: 50
    max_edges: 150
    grouping: none
  complexity:
    threshold: 30
    max_items: 100
  churn:
    days: 90
    max_items: 50
  context:
    include_ast: true
    include_metrics: true
    max_file_size: 1000000
export:
  formats: ["markdown", "sarif"]
  include_metadata: true
  include_raw_data: true
performance:
  cache_enabled: true
  cache_ttl: 7200
  parallel_workers: 8
"#;
        std::fs::write(&config_path, custom_yaml)?;

        // Load custom config
        config_manager.load(&config_path).await?;
        let custom_config = config_manager.get_config().await;

        assert_eq!(custom_config.panels.dependency.max_nodes, 50);
        assert_eq!(custom_config.panels.complexity.threshold, 30);
        assert_eq!(custom_config.performance.parallel_workers, 8);

        Ok(())
    }

    #[tokio::test]
    async fn test_export_service_as_library() -> Result<()> {
        // Skip slow tests in CI
        if std::env::var("SKIP_SLOW_TESTS").is_ok() {
            return Ok(());
        }

        // Create export service
        let export_service = ExportService::new();

        // Create a demo report
        let server = Arc::new(StatelessTemplateServer::new()?);
        let mut runner = DemoRunner::new(server);
        let demo_report = runner.execute_with_diagram(Path::new("."), None).await?;

        // Convert demo report to export report
        // Note: In a real scenario, we'd extract more data from the demo report
        let export_report = create_export_report(
            &demo_report.repository,
            &Default::default(), // Empty DAG for this test
            None,
            None,
            "graph TD\n    A --> B",
            demo_report.total_time_ms,
        );

        // Test exporting to different formats
        let markdown = export_service.export("markdown", &export_report)?;
        assert!(markdown.contains(&format!("# Analysis: {}", demo_report.repository)));

        let json = export_service.export("json", &export_report)?;
        let parsed: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(parsed["repository"], demo_report.repository);

        let sarif = export_service.export("sarif", &export_report)?;
        let sarif_json: serde_json::Value = serde_json::from_str(&sarif)?;
        assert_eq!(sarif_json["version"], "2.1.0");

        Ok(())
    }

    #[tokio::test]
    async fn test_programmatic_demo_with_custom_config() -> Result<()> {
        // Skip slow tests in CI
        if std::env::var("SKIP_SLOW_TESTS").is_ok() {
            return Ok(());
        }

        // Create custom configuration
        let _config = DisplayConfig {
            version: "1.0".to_string(),
            panels: Default::default(),
            export: paiml_mcp_agent_toolkit::demo::config::ExportConfig {
                formats: vec!["json".to_string()],
                include_metadata: false,
                include_raw_data: false,
            },
            performance: paiml_mcp_agent_toolkit::demo::config::PerformanceConfig {
                cache_enabled: false,
                cache_ttl: 0,
                parallel_workers: 2,
            },
        };

        // Create demo runner with custom config
        let server = Arc::new(StatelessTemplateServer::new()?);
        let mut runner = DemoRunner::new(server);

        // Run analysis
        let report = runner.execute_with_diagram(Path::new("."), None).await?;

        // Verify report was generated
        assert!(!report.repository.is_empty());
        assert!(report.total_time_ms > 0);

        Ok(())
    }

    #[test]
    fn test_export_formats_discovery() {
        let export_service = ExportService::new();
        let formats = export_service.supported_formats();

        // Verify all required formats are supported
        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"sarif"));
        assert_eq!(formats.len(), 3);
    }

    #[tokio::test]
    async fn test_end_to_end_library_usage() -> Result<()> {
        // Skip slow tests in CI
        if std::env::var("SKIP_SLOW_TESTS").is_ok() {
            return Ok(());
        }

        let temp_dir = TempDir::new()?;

        // Step 1: Create configuration
        let config_path = temp_dir.path().join(".paiml-display.yaml");
        let config_content = r#"
version: "1.0"
panels:
  dependency:
    max_nodes: 10
    max_edges: 30
    grouping: module
  complexity:
    threshold: 10
    max_items: 20
  churn:
    days: 7
    max_items: 10
  context:
    include_ast: true
    include_metrics: true
    max_file_size: 100000
export:
  formats: ["markdown", "json"]
  include_metadata: true
  include_raw_data: false
performance:
  cache_enabled: true
  cache_ttl: 300
  parallel_workers: 2
"#;
        std::fs::write(&config_path, config_content)?;

        // Step 2: Load configuration
        let mut config_manager = ConfigManager::new()?;
        config_manager.load(&config_path).await?;
        let _config = config_manager.get_config().await;

        // Step 3: Run analysis
        let server = Arc::new(StatelessTemplateServer::new()?);
        let mut runner = DemoRunner::new(server);
        let report = runner.execute_with_diagram(Path::new("."), None).await?;

        // Step 4: Export results
        let export_service = ExportService::new();
        let export_report = create_export_report(
            &report.repository,
            &Default::default(),
            None,
            None,
            report.system_diagram.as_deref().unwrap_or("graph TD"),
            report.total_time_ms,
        );

        // Export to multiple formats
        for format in ["markdown", "json"] {
            let output_path = temp_dir.path().join(format!("report.{}", format));
            export_service.save_to_file(format, &export_report, &output_path)?;
            assert!(output_path.exists());
        }

        Ok(())
    }
}
