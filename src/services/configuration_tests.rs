// Tests for configuration service
// Included by configuration_service.rs — shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_configuration_service_creation() {
        let config_service = ConfigurationService::new(None);
        let config = config_service.get_config().unwrap();

        assert_eq!(config.system.project_name, "pmat");
        assert_eq!(config.quality.max_complexity, 30);
        assert!(!config.quality.allow_satd);
    }

    #[tokio::test]
    async fn test_configuration_save_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config_service = ConfigurationService::new(Some(config_path.clone()));

        // Update config
        config_service
            .update_config(|config| {
                config.system.project_name = "test_project".to_string();
                config.quality.max_complexity = 25;
                Ok(())
            })
            .await
            .unwrap();

        // Create new service and load
        let new_service = ConfigurationService::new(Some(config_path));
        new_service.load().await.unwrap();

        let loaded_config = new_service.get_config().unwrap();
        assert_eq!(loaded_config.system.project_name, "test_project");
        assert_eq!(loaded_config.quality.max_complexity, 25);
    }

    #[tokio::test]
    async fn test_configuration_sections() {
        let config_service = ConfigurationService::new(None);

        let quality_config = config_service.get_quality_config().unwrap();
        assert_eq!(quality_config.max_complexity, 30);

        let analysis_config = config_service.get_analysis_config().unwrap();
        assert!(analysis_config.parallel);

        let performance_config = config_service.get_performance_config().unwrap();
        assert_eq!(performance_config.test_iterations, 10);

        let mcp_config = config_service.get_mcp_config().unwrap();
        assert_eq!(mcp_config.server_name, "pmat-mcp-server");

        let roadmap_config = config_service.get_roadmap_config().unwrap();
        assert!(roadmap_config.auto_generate_todos);

        let telemetry_config = config_service.get_telemetry_config().unwrap();
        assert!(telemetry_config.enabled);
    }

    #[tokio::test]
    #[ignore] // Five Whys: Process-global CWD modification causes race conditions under parallel execution
              // Root cause: std::env::set_current_dir() is process-wide, not thread-local
              // Fix attempted: RAII CwdGuard failed because current_dir() fails if CWD deleted
              // Decision: Mark as #[ignore] - unsuitable for parallel test execution
              // Run manually: cargo test test_service_lifecycle -- --ignored --test-threads=1
    async fn test_service_lifecycle() {
        let config_service = ConfigurationService::new(None);

        // Test service operations
        assert!(config_service.start().await.is_ok());
        assert!(config_service.health_check().await.unwrap());

        let status = config_service.status().await.unwrap();
        assert!(status.contains("Configuration service"));

        let metrics = config_service.get_metrics().await.unwrap();
        assert!(metrics.request_count >= 1); // From the start() call

        assert!(config_service.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_global_configuration_access() {
        let config_service = configuration();
        let config = config_service.get_config().unwrap();

        assert_eq!(config.system.project_name, "pmat");
        assert!(config.quality.fail_on_violation);
    }

    #[test]
    fn test_configuration_serialization() {
        let config = ConfigurationService::default_config();
        let serialized = toml::to_string(&config).unwrap();

        assert!(serialized.contains("[system]"));
        assert!(serialized.contains("[quality]"));
        assert!(serialized.contains("[analysis]"));
        assert!(serialized.contains("[performance]"));
        assert!(serialized.contains("[mcp]"));
        assert!(serialized.contains("[roadmap]"));
        assert!(serialized.contains("[telemetry]"));

        // Test deserialization
        let deserialized: PmatConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.system.project_name, config.system.project_name);
        assert_eq!(
            deserialized.quality.max_complexity,
            config.quality.max_complexity
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod getter_tests {
    use super::*;

    #[test]
    fn test_default_config_system_values() {
        let config = ConfigurationService::default_config();
        assert_eq!(config.system.project_name, "pmat");
        // max_concurrent_operations uses num_cpus::get(), just verify > 0
        assert!(config.system.max_concurrent_operations > 0);
        assert!(!config.system.verbose);
        assert!(!config.system.debug);
        assert_eq!(config.system.default_toolchain, "rust");
    }

    #[test]
    fn test_default_config_quality_values() {
        let config = ConfigurationService::default_config();
        assert_eq!(config.quality.max_complexity, 30);
        assert_eq!(config.quality.max_cognitive_complexity, 25);
        assert!(config.quality.fail_on_violation);
        assert!(!config.quality.allow_satd);
        assert!(config.quality.require_docs);
        assert!(config.quality.lint_compliance);
    }

    #[test]
    fn test_default_config_analysis_values() {
        let config = ConfigurationService::default_config();
        assert_eq!(config.analysis.max_file_size, 1024 * 1024); // 1MB
        assert_eq!(config.analysis.max_line_length, 100);
        assert!(config.analysis.skip_vendor);
        assert!(config.analysis.parallel);
        assert_eq!(config.analysis.thread_count, 0); // Auto
        assert_eq!(config.analysis.timeout_seconds, 300);
    }

    #[test]
    fn test_default_config_performance_values() {
        let config = ConfigurationService::default_config();
        assert_eq!(config.performance.test_iterations, 10);
        assert_eq!(config.performance.timeout_ms, 30000);
        assert!(config.performance.enable_regression_tests);
        assert!(config.performance.enable_memory_tests);
        assert!(config.performance.enable_throughput_tests);
    }

    #[test]
    fn test_default_config_mcp_values() {
        let config = ConfigurationService::default_config();
        assert_eq!(config.mcp.server_name, "pmat-mcp-server");
        assert!(!config.mcp.server_version.is_empty());
        assert!(config.mcp.enable_compression);
        assert_eq!(config.mcp.request_timeout_seconds, 30);
        assert!(!config.mcp.log_requests);
    }

    #[test]
    fn test_default_config_telemetry_values() {
        let config = ConfigurationService::default_config();
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.collection_interval_seconds, 60);
        assert_eq!(config.telemetry.max_data_age_days, 30);
        assert!(config.telemetry.enable_aggregation);
        assert!(!config.telemetry.enable_export);
    }

    #[test]
    fn test_default_config_semantic_values() {
        let config = ConfigurationService::default_config();
        assert!(!config.semantic.enabled);
        assert!(config.semantic.vector_db_path.is_none());
        assert_eq!(config.semantic.embedding_model, "aprender-tfidf-local");
        assert_eq!(config.semantic.embedding_dimensions, 256);
    }

    #[test]
    fn test_default_config_custom_empty() {
        let config = ConfigurationService::default_config();
        assert!(config.custom.is_empty());
    }

    #[test]
    fn test_default_config_roadmap_values() {
        let config = ConfigurationService::default_config();
        assert!(config.roadmap.auto_generate_todos);
        assert!(config.roadmap.enforce_quality_gates);
        assert!(config.roadmap.require_task_ids);
        assert!(config.roadmap.velocity_tracking);
        assert!(!config.roadmap.git.create_branches); // Per CLAUDE.md zero-branching
    }

    #[test]
    fn test_service_new_without_path() {
        let service = ConfigurationService::new(None);
        // config_path is PathBuf, defaults to cwd/pmat.toml
        assert!(service.config_path.to_string_lossy().ends_with("pmat.toml"));
    }

    #[test]
    fn test_service_new_with_path() {
        let path = PathBuf::from("/tmp/test-config.toml");
        let service = ConfigurationService::new(Some(path.clone()));
        assert_eq!(service.config_path, path);
    }

    /// A service whose config file does not exist, so it reports the built-in
    /// defaults. `ConfigurationService::new()` now reads the file it is pointed
    /// at (it used to ignore it entirely), so a getter test that wants defaults
    /// must not be pointed at the cwd's pmat.toml.
    fn default_service() -> ConfigurationService {
        ConfigurationService::new(Some(PathBuf::from(
            "/nonexistent-pmat-config-dir/pmat.toml",
        )))
    }

    #[test]
    fn test_get_quality_config() {
        let service = default_service();
        let quality = service.get_quality_config().unwrap();
        assert_eq!(quality.max_complexity, 30);
    }

    #[test]
    fn test_get_analysis_config() {
        let service = default_service();
        let analysis = service.get_analysis_config().unwrap();
        assert_eq!(analysis.max_file_size, 1024 * 1024);
    }

    #[test]
    fn test_get_performance_config() {
        let service = default_service();
        let perf = service.get_performance_config().unwrap();
        assert_eq!(perf.test_iterations, 10);
    }

    #[test]
    fn test_get_mcp_config() {
        let service = default_service();
        let mcp = service.get_mcp_config().unwrap();
        assert_eq!(mcp.server_name, "pmat-mcp-server");
    }

    #[test]
    fn test_get_roadmap_config() {
        let service = default_service();
        let roadmap = service.get_roadmap_config().unwrap();
        assert!(roadmap.auto_generate_todos);
    }

    #[test]
    fn test_get_telemetry_config() {
        let service = default_service();
        let telemetry = service.get_telemetry_config().unwrap();
        assert!(telemetry.enabled);
    }

    #[test]
    fn test_get_semantic_config() {
        let service = default_service();
        let semantic = service.get_semantic_config().unwrap();
        assert!(!semantic.enabled);
    }

    #[test]
    fn test_get_config_returns_full_config() {
        let service = default_service();
        let config = service.get_config().unwrap();
        assert_eq!(config.system.project_name, "pmat");
        assert_eq!(config.quality.max_complexity, 30);
        assert!(!config.semantic.enabled);
    }

    /// The embeddings store default is keyed to the workspace, never machine-global.
    ///
    /// This is the fallback `pmat embed status` / `pmat semantic search`
    /// actually consult. It used to return `~/.pmat/embeddings.db` for every
    /// directory on the machine while chunk paths are stored
    /// workspace-relative, so an empty directory reported the same "5 chunks
    /// indexed" as a 4,000-file repo and served that repo's `./src/main.rs`
    /// rows. A caller-side workspace default existed but could never fire —
    /// this ran first and always returned `Some`.
    #[test]
    #[serial_test::serial]
    fn test_vector_db_default_is_scoped_to_the_workspace() {
        let alpha = tempfile::tempdir().unwrap();
        let beta = tempfile::tempdir().unwrap();

        let prev_db = std::env::var("PMAT_VECTOR_DB_PATH").ok();
        let prev_ws = std::env::var("PMAT_WORKSPACE").ok();
        std::env::remove_var("PMAT_VECTOR_DB_PATH");

        let resolve = |ws: &std::path::Path| {
            std::env::set_var("PMAT_WORKSPACE", ws);
            ConfigurationService::new(Some(ws.join("pmat.toml")))
                .get_semantic_config_with_env_fallback()
                .unwrap()
                .vector_db_path
                .expect("a db path must be resolved")
        };

        let a = resolve(alpha.path());
        let b = resolve(beta.path());

        // Restore before asserting so a failure cannot leak env state.
        std::env::remove_var("PMAT_WORKSPACE");
        if let Some(v) = prev_db {
            std::env::set_var("PMAT_VECTOR_DB_PATH", v);
        }
        if let Some(v) = prev_ws {
            std::env::set_var("PMAT_WORKSPACE", v);
        }

        assert_ne!(a, b, "two workspaces resolved to ONE shared store: {a}");
        assert!(
            a.starts_with(alpha.path().to_str().unwrap()),
            "store must live under its own workspace, got {a}"
        );
        assert!(
            b.starts_with(beta.path().to_str().unwrap()),
            "store must live under its own workspace, got {b}"
        );
        let home = dirs::home_dir().unwrap_or_default().join(".pmat");
        assert!(
            !a.starts_with(home.to_str().unwrap_or("\u{0}")),
            "store fell back to the machine-global path: {a}"
        );
    }
}
