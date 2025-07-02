//! Fast demo core extraction tests
//!
//! Slow integration tests have been moved to `tests/slow_integration.rs` to ensure
//! fast test coverage completes within 3 minutes.

#[cfg(test)]
mod demo_core_extraction_tests {
    use anyhow::Result;
    use pmat::demo::config::ConfigManager;
    use pmat::demo::export::ExportService;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_creation() -> Result<()> {
        // Fast test - just verify config manager can be created
        let config_manager = ConfigManager::new()?;
        let default_config = config_manager.get_config().await;
        assert_eq!(default_config.version, "1.0");
        Ok(())
    }

    #[tokio::test]
    async fn test_config_manager_custom_load() -> Result<()> {
        // Fast test - verify custom config loading without running analysis
        let mut config_manager = ConfigManager::new()?;
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
    days: 45
    max_items: 25
  context:
    include_ast: true
    include_metrics: false
    max_file_size: 750000
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

        config_manager.load(&config_path).await?;
        let custom_config = config_manager.get_config().await;

        assert_eq!(custom_config.panels.dependency.max_nodes, 50);
        assert_eq!(custom_config.panels.complexity.threshold, 30);
        assert_eq!(custom_config.performance.parallel_workers, 8);

        Ok(())
    }

    #[test]
    fn test_export_formats_discovery() {
        // Synchronous test - very fast
        let export_service = ExportService::new();
        let formats = export_service.supported_formats();

        // Verify all required formats are supported
        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"sarif"));
        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_export_service_creation() {
        // Synchronous test - verify export service can be created
        let export_service = ExportService::new();
        assert!(!export_service.supported_formats().is_empty());
    }
}
