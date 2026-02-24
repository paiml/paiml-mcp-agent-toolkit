#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_load_daemon_config_missing_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let result = load_daemon_config(&config_path).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Configuration file not found"));
    }

    #[tokio::test]
    async fn test_daemon_status_json_format() {
        let result = handle_agent_status(None, crate::cli::OutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_monitor_with_default_project_id() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let result = handle_agent_monitor(project_path, None, None).await;
        // This will fail because daemon is not running, but that's expected
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Agent daemon is not running"));
    }

    // ============ AgentStartConfig Tests ============

    #[test]
    fn test_agent_start_config_creation() {
        let config = AgentStartConfig {
            _project_path: PathBuf::from("/tmp/project"),
            config_path: None,
            working_dir: None,
            pid_file: None,
            log_file: None,
            foreground: true,
            health_interval: 60,
            max_memory_mb: 512,
            auto_restart: true,
        };

        assert!(config.foreground);
        assert_eq!(config.health_interval, 60);
        assert_eq!(config.max_memory_mb, 512);
        assert!(config.auto_restart);
    }

    #[test]
    fn test_agent_start_config_with_paths() {
        let config = AgentStartConfig {
            _project_path: PathBuf::from("/home/user/myproject"),
            config_path: Some(PathBuf::from("/etc/pmat/config.toml")),
            working_dir: Some(PathBuf::from("/var/run/pmat")),
            pid_file: Some(PathBuf::from("/var/run/pmat/agent.pid")),
            log_file: Some(PathBuf::from("/var/log/pmat/agent.log")),
            foreground: false,
            health_interval: 30,
            max_memory_mb: 1024,
            auto_restart: false,
        };

        assert!(config.config_path.is_some());
        assert!(config.working_dir.is_some());
        assert!(config.pid_file.is_some());
        assert!(config.log_file.is_some());
    }

    #[test]
    fn test_agent_start_config_clone() {
        let config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: None,
            pid_file: None,
            log_file: None,
            foreground: false,
            health_interval: 10,
            max_memory_mb: 256,
            auto_restart: true,
        };

        let cloned = config.clone();
        assert_eq!(cloned.health_interval, 10);
        assert_eq!(cloned.max_memory_mb, 256);
    }

    #[test]
    fn test_agent_start_config_debug() {
        let config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: None,
            pid_file: None,
            log_file: None,
            foreground: true,
            health_interval: 5,
            max_memory_mb: 128,
            auto_restart: false,
        };

        let debug = format!("{:?}", config);
        assert!(debug.contains("AgentStartConfig"));
    }

    // ============ apply_config_overrides Tests ============

    #[test]
    fn test_apply_config_overrides_basic() {
        let mut daemon_config = DaemonConfig::default();
        let start_config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: None,
            pid_file: None,
            log_file: None,
            foreground: true,
            health_interval: 120,
            max_memory_mb: 2048,
            auto_restart: true,
        };

        apply_config_overrides(&mut daemon_config, &start_config);

        assert_eq!(
            daemon_config.daemon.health_check_interval,
            Duration::from_secs(120)
        );
        assert_eq!(daemon_config.daemon.max_memory_mb, 2048);
        assert!(daemon_config.daemon.auto_restart);
    }

    // ============ apply_optional_overrides Tests ============

    #[test]
    fn test_apply_optional_overrides_all_none() {
        let mut daemon_config = DaemonConfig::default();
        let start_config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: None,
            pid_file: None,
            log_file: None,
            foreground: false,
            health_interval: 60,
            max_memory_mb: 512,
            auto_restart: true,
        };

        apply_optional_overrides(&mut daemon_config, &start_config);
        // Should not change anything when all optional paths are None
    }

    #[test]
    fn test_apply_optional_overrides_with_working_dir() {
        let mut daemon_config = DaemonConfig::default();
        let start_config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: Some(PathBuf::from("/custom/working/dir")),
            pid_file: None,
            log_file: None,
            foreground: false,
            health_interval: 60,
            max_memory_mb: 512,
            auto_restart: true,
        };

        apply_optional_overrides(&mut daemon_config, &start_config);
        assert_eq!(
            daemon_config.daemon.working_directory,
            PathBuf::from("/custom/working/dir")
        );
    }

    #[test]
    fn test_apply_optional_overrides_with_pid_file() {
        let mut daemon_config = DaemonConfig::default();
        let start_config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: None,
            pid_file: Some(PathBuf::from("/run/pmat.pid")),
            log_file: None,
            foreground: false,
            health_interval: 60,
            max_memory_mb: 512,
            auto_restart: true,
        };

        apply_optional_overrides(&mut daemon_config, &start_config);
        assert_eq!(
            daemon_config.daemon.pid_file,
            Some(PathBuf::from("/run/pmat.pid"))
        );
    }

    #[test]
    fn test_apply_optional_overrides_with_log_file() {
        let mut daemon_config = DaemonConfig::default();
        let start_config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: None,
            pid_file: None,
            log_file: Some(PathBuf::from("/var/log/pmat.log")),
            foreground: false,
            health_interval: 60,
            max_memory_mb: 512,
            auto_restart: true,
        };

        apply_optional_overrides(&mut daemon_config, &start_config);
        assert_eq!(
            daemon_config.daemon.log_file,
            Some(PathBuf::from("/var/log/pmat.log"))
        );
    }

    #[test]
    fn test_apply_optional_overrides_all_set() {
        let mut daemon_config = DaemonConfig::default();
        let start_config = AgentStartConfig {
            _project_path: PathBuf::from("/test"),
            config_path: None,
            working_dir: Some(PathBuf::from("/work")),
            pid_file: Some(PathBuf::from("/run/pmat.pid")),
            log_file: Some(PathBuf::from("/var/log/pmat.log")),
            foreground: false,
            health_interval: 60,
            max_memory_mb: 512,
            auto_restart: true,
        };

        apply_optional_overrides(&mut daemon_config, &start_config);
        assert_eq!(
            daemon_config.daemon.working_directory,
            PathBuf::from("/work")
        );
        assert_eq!(
            daemon_config.daemon.pid_file,
            Some(PathBuf::from("/run/pmat.pid"))
        );
        assert_eq!(
            daemon_config.daemon.log_file,
            Some(PathBuf::from("/var/log/pmat.log"))
        );
    }

    // ============ load_or_default_config Tests ============

    #[tokio::test]
    async fn test_load_or_default_config_none() {
        let result = load_or_default_config(&None).await;
        assert!(result.is_ok());
        let config = result.unwrap();
        // Should return default config
        assert!(config.daemon.health_check_interval.as_secs() > 0);
    }

    #[tokio::test]
    async fn test_load_or_default_config_missing_file() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result = load_or_default_config(&Some(path)).await;
        assert!(result.is_err());
    }

    // ============ check_daemon_not_running Tests ============

    #[tokio::test]
    async fn test_check_daemon_not_running_when_not_running() {
        // Assuming daemon is not running in test environment
        let result = check_daemon_not_running().await;
        assert!(result.is_ok());
    }

    // ============ Handler Error Path Tests ============

    #[tokio::test]
    async fn test_handle_agent_stop_not_running() {
        let result = handle_agent_stop(None, false, 30).await;
        // Should succeed even when not running
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_agent_unmonitor_not_running() {
        let result = handle_agent_unmonitor("test-project".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    #[tokio::test]
    async fn test_handle_agent_health_not_running() {
        let result = handle_agent_health(None, false).await;
        // Should succeed but report not running
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_agent_health_detailed_not_running() {
        let result = handle_agent_health(None, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_agent_reload_not_running() {
        let result = handle_agent_reload(None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    #[tokio::test]
    async fn test_handle_agent_quality_gate_not_running() {
        let result = handle_agent_quality_gate(
            "test-project".to_string(),
            None,
            crate::cli::QualityGateOutputFormat::Human,
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    #[tokio::test]
    async fn test_handle_agent_status_text_format() {
        let result = handle_agent_status(None, crate::cli::OutputFormat::Table).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_agent_status_yaml_format() {
        let result = handle_agent_status(None, crate::cli::OutputFormat::Yaml).await;
        assert!(result.is_ok());
    }

    // ============ Monitor with custom project_id Tests ============

    #[tokio::test]
    async fn test_handle_agent_monitor_with_custom_project_id() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let result =
            handle_agent_monitor(project_path, Some("custom-project-id".to_string()), None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    // ============ Load config with incomplete file Tests ============

    #[tokio::test]
    async fn test_load_daemon_config_incomplete_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Incomplete config - missing required fields like quality_monitor
        let config_content = r#"
[daemon]
health_check_interval_secs = 30
"#;
        fs::write(&config_path, config_content).await.unwrap();

        // Incomplete configs should fail to parse
        let result = load_daemon_config(&config_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_daemon_config_invalid_toml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");

        fs::write(&config_path, "this is not valid toml {{{{")
            .await
            .unwrap();

        let result = load_daemon_config(&config_path).await;
        assert!(result.is_err());
    }
}
