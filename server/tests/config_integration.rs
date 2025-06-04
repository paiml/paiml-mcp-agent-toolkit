#[cfg(test)]
mod config_integration_tests {
    use anyhow::Result;
    use paiml_mcp_agent_toolkit::demo::config::{ConfigManager, GroupingStrategy};
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_config_loading_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join(".paiml-display.yaml");

        // Write test configuration
        let yaml_content = r#"
version: "1.0"
panels:
  dependency:
    max_nodes: 30
    max_edges: 90
    grouping: directory
  complexity:
    threshold: 20
    max_items: 100
  churn:
    days: 60
    max_items: 30
  context:
    include_ast: false
    include_metrics: true
    max_file_size: 1000000
export:
  formats: ["json", "markdown"]
  include_metadata: false
  include_raw_data: true
performance:
  cache_enabled: false
  cache_ttl: 7200
  parallel_workers: 8
"#;
        fs::write(&config_path, yaml_content)?;

        // Load configuration
        let mut manager = ConfigManager::new()?;
        manager.load(&config_path).await?;

        // Verify loaded configuration
        let config = manager.get_config().await;
        assert_eq!(config.panels.dependency.max_nodes, 30);
        assert_eq!(config.panels.dependency.max_edges, 90);
        assert!(matches!(
            config.panels.dependency.grouping,
            GroupingStrategy::Directory
        ));
        assert_eq!(config.panels.complexity.threshold, 20);
        assert_eq!(config.performance.parallel_workers, 8);
        assert!(!config.performance.cache_enabled);

        Ok(())
    }

    #[tokio::test]
    async fn test_config_hot_reload() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join(".paiml-display.yaml");

        // Write initial configuration
        let initial_yaml = r#"
version: "1.0"
panels:
  dependency:
    max_nodes: 20
    max_edges: 60
    grouping: module
  complexity:
    threshold: 15
    max_items: 50
  churn:
    days: 30
    max_items: 20
  context:
    include_ast: true
    include_metrics: true
    max_file_size: 500000
export:
  formats: ["markdown", "json", "sarif"]
  include_metadata: true
  include_raw_data: false
performance:
  cache_enabled: true
  cache_ttl: 3600
  parallel_workers: 4
"#;
        fs::write(&config_path, initial_yaml)?;

        // Start watching
        let mut manager = ConfigManager::new()?;
        let mut subscriber = manager.subscribe();
        manager.watch(config_path.clone()).await?;

        // Verify initial config
        let initial_config = manager.get_config().await;
        assert_eq!(initial_config.panels.dependency.max_nodes, 20);

        // Give the watcher time to initialize
        sleep(Duration::from_millis(1000)).await;

        // Update configuration
        let updated_yaml = r#"
version: "1.0"
panels:
  dependency:
    max_nodes: 40
    max_edges: 120
    grouping: none
  complexity:
    threshold: 25
    max_items: 75
  churn:
    days: 45
    max_items: 25
  context:
    include_ast: false
    include_metrics: true
    max_file_size: 750000
export:
  formats: ["json"]
  include_metadata: false
  include_raw_data: true
performance:
  cache_enabled: false
  cache_ttl: 7200
  parallel_workers: 8
"#;
        fs::write(&config_path, updated_yaml)?;
        // Force sync to ensure file is written
        {
            use std::fs::OpenOptions;
            let file = OpenOptions::new().write(true).open(&config_path)?;
            file.sync_all()?;
        }

        // Give filesystem and watcher time to detect change
        sleep(Duration::from_millis(200)).await;

        // Wait for update notification
        tokio::select! {
            update = subscriber.recv() => {
                let updated_config = update?;
                assert_eq!(updated_config.panels.dependency.max_nodes, 40);
                assert_eq!(updated_config.panels.dependency.max_edges, 120);
                assert!(matches!(updated_config.panels.dependency.grouping, GroupingStrategy::None));
                assert_eq!(updated_config.panels.complexity.threshold, 25);
                assert_eq!(updated_config.performance.parallel_workers, 8);
            }
            _ = sleep(Duration::from_secs(5)) => {
                panic!("Config update notification not received within timeout");
            }
        }

        // Verify manager also has updated config
        let current_config = manager.get_config().await;
        assert_eq!(current_config.panels.dependency.max_nodes, 40);

        Ok(())
    }

    #[tokio::test]
    async fn test_config_default_values() -> Result<()> {
        let manager = ConfigManager::new()?;
        let config = manager.get_config().await;

        // Verify default values
        assert_eq!(config.version, "1.0");
        assert_eq!(config.panels.dependency.max_nodes, 20);
        assert_eq!(config.panels.dependency.max_edges, 60);
        assert!(matches!(
            config.panels.dependency.grouping,
            GroupingStrategy::Module
        ));
        assert_eq!(config.panels.complexity.threshold, 15);
        assert_eq!(config.panels.churn.days, 30);
        assert!(config.panels.context.include_ast);
        assert!(config.performance.cache_enabled);
        assert_eq!(config.performance.cache_ttl, 3600);
        assert_eq!(config.performance.parallel_workers, 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_config_accessor_methods() -> Result<()> {
        let manager = ConfigManager::new()?;

        // Test all accessor methods
        let dep_config = manager.get_dependency_config().await;
        assert_eq!(dep_config.max_nodes, 20);

        let comp_config = manager.get_complexity_config().await;
        assert_eq!(comp_config.threshold, 15);

        let churn_config = manager.get_churn_config().await;
        assert_eq!(churn_config.days, 30);

        let ctx_config = manager.get_context_config().await;
        assert!(ctx_config.include_ast);

        let export_config = manager.get_export_config().await;
        assert_eq!(export_config.formats.len(), 3);
        assert!(export_config.formats.contains(&"markdown".to_string()));

        let perf_config = manager.get_performance_config().await;
        assert!(perf_config.cache_enabled);

        Ok(())
    }

    #[test]
    fn test_invalid_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".paiml-display.yaml");

        // Write invalid YAML
        fs::write(&config_path, "invalid: yaml: content: [").unwrap();

        // Should fail to load
        let result = ConfigManager::load_from_file(&config_path);
        assert!(result.is_err());
    }
}
