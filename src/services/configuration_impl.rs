// ConfigurationService implementation — core service logic
// Included by configuration_service.rs — shares parent module scope

/// Configuration service providing centralized config management
pub struct ConfigurationService {
    config: Arc<RwLock<PmatConfig>>,
    config_path: PathBuf,
    metrics: Arc<RwLock<ServiceMetrics>>,
    watchers: Arc<RwLock<Vec<Box<dyn ConfigWatcher + Send + Sync>>>>,
}

/// Trait for configuration change watchers
pub trait ConfigWatcher {
    fn on_config_changed(&self, config: &PmatConfig) -> Result<()>;
}

impl ConfigurationService {
    /// Create a new configuration service
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let default_path = config_path.unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .join("pmat.toml")
        });

        // Read the file here. `load()` is async and was only ever reached from
        // `start()`, which nothing calls, so every reader saw the built-in
        // defaults: `config --set quality.max_complexity=5` wrote pmat.toml,
        // reported success, and the next `config` invocation still printed 30.
        let config = Self::read_config_file(&default_path).unwrap_or_else(Self::default_config);

        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: default_path,
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Read and parse a config file, returning None when there is nothing
    /// usable on disk. A malformed file is reported on stderr rather than
    /// silently replaced by defaults.
    fn read_config_file(path: &std::path::Path) -> Option<PmatConfig> {
        if !path.exists() {
            return None;
        }
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<PmatConfig>(&content) {
                Ok(config) => Some(config),
                Err(e) => {
                    eprintln!(
                        "warning: {} is not valid pmat configuration ({e}); using defaults",
                        path.display()
                    );
                    None
                }
            },
            Err(e) => {
                eprintln!(
                    "warning: could not read {} ({e}); using defaults",
                    path.display()
                );
                None
            }
        }
    }

    /// Load configuration from file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn load(&self) -> Result<()> {
        if self.config_path.exists() {
            let content = tokio::fs::read_to_string(&self.config_path).await?;
            let config: PmatConfig = toml::from_str(&content)?;

            {
                let mut config_lock = self
                    .config
                    .write()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire config write lock"))?;
                *config_lock = config.clone();
            }

            // Notify watchers
            self.notify_watchers(&config)?;

            // Update metrics
            {
                let mut metrics = self
                    .metrics
                    .write()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?;
                metrics.record_request(std::time::Duration::from_millis(1), true);
            }
        }

        Ok(())
    }

    /// Save configuration to file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn save(&self) -> Result<()> {
        let config = {
            self.config
                .read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?
                .clone()
        };

        let content = toml::to_string_pretty(&config)?;
        tokio::fs::write(&self.config_path, content).await?;

        // Update metrics
        {
            let mut metrics = self
                .metrics
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?;
            metrics.record_request(std::time::Duration::from_millis(1), true);
        }

        Ok(())
    }

    /// Get current configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_config(&self) -> Result<PmatConfig> {
        Ok(self
            .config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?
            .clone())
    }

    /// Update configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut PmatConfig) -> Result<()>,
    {
        let config_clone = {
            let mut config = self
                .config
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire config write lock"))?;

            updater(&mut config)?;
            config.clone()
        }; // Guard is dropped here

        // Save to file
        self.save().await?;

        // Notify watchers
        self.notify_watchers(&config_clone)?;

        Ok(())
    }

    /// Add configuration watcher
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_watcher(&self, watcher: Box<dyn ConfigWatcher + Send + Sync>) -> Result<()> {
        let mut watchers = self
            .watchers
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire watchers lock"))?;
        watchers.push(watcher);
        Ok(())
    }

    /// Get specific configuration section
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_quality_config(&self) -> Result<QualityConfig> {
        Ok(self.get_config()?.quality)
    }

    /// Get analysis config.
    pub fn get_analysis_config(&self) -> Result<AnalysisConfig> {
        Ok(self.get_config()?.analysis)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get performance config.
    pub fn get_performance_config(&self) -> Result<PerformanceConfig> {
        Ok(self.get_config()?.performance)
    }

    /// Get mcp config.
    pub fn get_mcp_config(&self) -> Result<McpConfig> {
        Ok(self.get_config()?.mcp)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get roadmap config.
    pub fn get_roadmap_config(&self) -> Result<RoadmapConfig> {
        Ok(self.get_config()?.roadmap)
    }

    /// Get telemetry config.
    pub fn get_telemetry_config(&self) -> Result<TelemetryConfig> {
        Ok(self.get_config()?.telemetry)
    }

    /// Get semantic search configuration (PMAT-SEARCH-011, PMAT-SEARCH-012)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_semantic_config(&self) -> Result<SemanticConfig> {
        Ok(self.get_config()?.semantic)
    }

    /// Get semantic configuration with environment variable fallbacks
    ///
    /// Priority order:
    /// 1. Config file values (if explicitly set)
    /// 2. Environment variables
    /// 3. Defaults
    ///
    /// Environment variables:
    /// - PMAT_VECTOR_DB_PATH: Path to vector database
    /// - PMAT_WORKSPACE: Workspace path for code indexing
    ///
    /// NOTE: No API keys required - uses local embeddings via aprender/trueno-rag
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_semantic_config_with_env_fallback(&self) -> Result<SemanticConfig> {
        let mut config = self.get_semantic_config()?;

        // Workspace path fallback: config file > env var > current directory.
        // Resolved BEFORE the DB path, which is derived from it.
        if config.workspace_path.is_none() {
            config.workspace_path = std::env::var("PMAT_WORKSPACE")
                .ok()
                .map(PathBuf::from)
                .or_else(|| std::env::current_dir().ok());
        }

        // Vector DB path fallback: config file > env var > per-workspace default.
        //
        // This used to fall back to a single machine-global
        // `~/.pmat/embeddings.db` shared by every project, while chunk paths
        // are stored workspace-relative (`./src/main.rs`). One crate's leftover
        // index was therefore served to every OTHER directory, at paths that do
        // not resolve there: `pmat embed status` reported the same "5 chunks
        // indexed" in an empty directory as in a 4,000-file repo, and
        // `semantic search` in the empty directory returned another crate's
        // `./src/main.rs`. A caller-side workspace default already existed but
        // could never fire, because this ran first and always returned `Some`.
        if config.vector_db_path.is_none() {
            config.vector_db_path = std::env::var("PMAT_VECTOR_DB_PATH")
                .ok()
                .or_else(|| config.workspace_path.as_deref().map(default_vector_db_path));
        }

        Ok(config)
    }

    /// Notify all watchers of configuration changes
    fn notify_watchers(&self, config: &PmatConfig) -> Result<()> {
        let watchers = self
            .watchers
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire watchers lock"))?;

        for watcher in watchers.iter() {
            if let Err(e) = watcher.on_config_changed(config) {
                tracing::warn!("Configuration watcher failed: {}", e);
            }
        }

        Ok(())
    }
}

impl ConfigurationService {
    /// Start the configuration service
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn start(&self) -> Result<()> {
        // Load configuration from file if it exists
        self.load().await?;

        // Update metrics
        {
            let mut metrics = self
                .metrics
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?;
            metrics.record_request(Duration::from_millis(10), true);
        }

        tracing::info!(
            "Configuration service started with config at: {:?}",
            self.config_path
        );
        Ok(())
    }

    /// Stop the configuration service
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn stop(&self) -> Result<()> {
        // Save current configuration
        self.save().await?;

        // Update metrics
        {
            let mut metrics = self
                .metrics
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?;
            metrics.record_request(Duration::from_millis(5), true);
        }

        tracing::info!("Configuration service stopped");
        Ok(())
    }

    /// Get service status
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn status(&self) -> Result<String> {
        let config_exists = self.config_path.exists();
        let _config = self.get_config()?;

        Ok(format!(
            "Configuration service: {} (file: {}, sections: {})",
            if config_exists { "loaded" } else { "default" },
            self.config_path.display(),
            8 // Number of main config sections (system, quality, analysis, performance, mcp, roadmap, telemetry, semantic)
        ))
    }

    /// Get service metrics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_metrics(&self) -> Result<ServiceMetrics> {
        Ok(self
            .metrics
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?
            .clone())
    }

    /// Check service health
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn health_check(&self) -> Result<bool> {
        // Check if we can read the configuration
        self.get_config().map(|_| true)
    }
}

/// Where a workspace's embeddings live when neither config nor
/// `PMAT_VECTOR_DB_PATH` names a path.
///
/// THE one definition. Embedding chunk paths are stored workspace-relative, so
/// a store shared between workspaces returns rows whose paths mean nothing to
/// the caller; keying the store to the workspace is what makes a relative chunk
/// path resolvable again. Three separate copies of a machine-global
/// `~/.pmat/embeddings.db` default used to exist (config service, CLI
/// dispatcher, MCP server config) — they all call this now.
#[must_use]
pub fn default_vector_db_path(workspace: &std::path::Path) -> String {
    workspace
        .join(".pmat")
        .join("embeddings.db")
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod new_reads_disk_tests {
    use super::ConfigurationService;

    #[test]
    fn test_new_reports_the_values_on_disk() {
        // Regression: `config --set quality.max_complexity=5` wrote pmat.toml
        // and said "Configuration updated successfully", but the readers only
        // ever saw default_config() because load() was never called.
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("pmat.toml");

        let mut on_disk = ConfigurationService::default_config();
        on_disk.quality.max_complexity = 5;
        on_disk.system.project_name = "written-by-set".to_string();
        std::fs::write(&path, toml::to_string_pretty(&on_disk).unwrap()).unwrap();

        let service = ConfigurationService::new(Some(path));
        let config = service.get_config().unwrap();
        assert_eq!(config.quality.max_complexity, 5);
        assert_eq!(config.system.project_name, "written-by-set");
    }

    #[test]
    fn test_new_falls_back_to_defaults_when_no_file() {
        let temp = tempfile::tempdir().unwrap();
        let service = ConfigurationService::new(Some(temp.path().join("absent.toml")));
        let config = service.get_config().unwrap();
        assert_eq!(
            config.quality.max_complexity,
            ConfigurationService::default_config().quality.max_complexity
        );
    }

    #[test]
    fn test_new_survives_a_malformed_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("pmat.toml");
        std::fs::write(&path, "this is not toml {{{").unwrap();
        let service = ConfigurationService::new(Some(path));
        assert!(service.get_config().is_ok());
    }
}

// Global configuration service instance (singleton pattern)
lazy_static::lazy_static! {
    static ref CONFIGURATION: Arc<ConfigurationService> = Arc::new(ConfigurationService::new(None));
}

/// Get the global configuration service instance - THE ONE way to access configuration
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn configuration() -> Arc<ConfigurationService> {
    CONFIGURATION.clone()
}
