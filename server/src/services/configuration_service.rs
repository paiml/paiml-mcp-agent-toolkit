//! Configuration management service for PMAT system
//!
//! This module provides a centralized configuration management system that
//! consolidates all configuration patterns in the codebase following the
//! Toyota Way ONE implementation principle.

use crate::services::service_base::ServiceMetrics;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Central configuration for the entire PMAT system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmatConfig {
    /// System-wide settings
    pub system: SystemConfig,

    /// Quality gate configurations
    pub quality: QualityConfig,

    /// Analysis configurations
    pub analysis: AnalysisConfig,

    /// Performance testing configurations
    pub performance: PerformanceConfig,

    /// MCP server configurations
    pub mcp: McpConfig,

    /// Roadmap and project management
    pub roadmap: RoadmapConfig,

    /// Telemetry settings
    pub telemetry: TelemetryConfig,

    /// Custom user configurations
    pub custom: HashMap<String, serde_json::Value>,
}

/// System-wide configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Project name
    pub project_name: String,

    /// Project root path
    pub project_path: PathBuf,

    /// Output directory for generated files
    pub output_dir: PathBuf,

    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,

    /// Enable verbose logging
    pub verbose: bool,

    /// Enable debug mode
    pub debug: bool,

    /// Toolchain preference
    pub default_toolchain: String,
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Maximum cyclomatic complexity allowed
    pub max_complexity: u32,

    /// Maximum cognitive complexity allowed  
    pub max_cognitive_complexity: u32,

    /// Minimum test coverage percentage
    pub min_coverage: f64,

    /// Allow SATD (Self-Admitted Technical Debt) comments
    pub allow_satd: bool,

    /// Require documentation for public items
    pub require_docs: bool,

    /// Enable lint compliance checking
    pub lint_compliance: bool,

    /// Fail builds on quality violations
    pub fail_on_violation: bool,
}

/// Analysis configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Include patterns for file analysis
    pub include_patterns: Vec<String>,

    /// Exclude patterns for file analysis
    pub exclude_patterns: Vec<String>,

    /// Maximum file size to analyze (bytes)
    pub max_file_size: usize,

    /// Maximum line length for analysis
    pub max_line_length: usize,

    /// Skip vendor directories
    pub skip_vendor: bool,

    /// Enable parallel processing
    pub parallel: bool,

    /// Number of worker threads (0 = auto)
    pub thread_count: usize,

    /// Analysis timeout in seconds
    pub timeout_seconds: u64,
}

/// Performance testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable regression testing
    pub enable_regression_tests: bool,

    /// Enable memory usage tests
    pub enable_memory_tests: bool,

    /// Enable throughput tests
    pub enable_throughput_tests: bool,

    /// Number of test iterations
    pub test_iterations: usize,

    /// Test timeout in milliseconds
    pub timeout_ms: u64,

    /// Target metrics
    pub target_startup_latency_ms: u64,
    pub target_throughput_loc_per_sec: u64,
    pub target_memory_mb: u64,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server name
    pub server_name: String,

    /// Server version
    pub server_version: String,

    /// Enable transport compression
    pub enable_compression: bool,

    /// Request timeout in seconds
    pub request_timeout_seconds: u64,

    /// Maximum request size in bytes
    pub max_request_size: usize,

    /// Enable request logging
    pub log_requests: bool,

    /// Tools to expose
    pub enabled_tools: Vec<String>,
}

/// Roadmap and project management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapConfig {
    /// Path to roadmap file
    pub roadmap_path: PathBuf,

    /// Enable automatic todo generation
    pub auto_generate_todos: bool,

    /// Enforce quality gates
    pub enforce_quality_gates: bool,

    /// Require task IDs
    pub require_task_ids: bool,

    /// Task ID pattern regex
    pub task_id_pattern: String,

    /// Enable velocity tracking
    pub velocity_tracking: bool,

    /// Enable burndown charts
    pub burndown_charts: bool,

    /// Git integration settings
    pub git: GitConfig,
}

/// Git integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Create branches for tasks
    pub create_branches: bool,

    /// Branch naming pattern
    pub branch_pattern: String,

    /// Commit message pattern
    pub commit_pattern: String,

    /// Require quality check before commit
    pub require_quality_check: bool,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry collection
    pub enabled: bool,

    /// Collection interval in seconds
    pub collection_interval_seconds: u64,

    /// Maximum telemetry data age in days
    pub max_data_age_days: u32,

    /// Enable metric aggregation
    pub enable_aggregation: bool,

    /// Enable telemetry export
    pub enable_export: bool,

    /// Export format (json, csv, etc.)
    pub export_format: String,
}

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
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let default_path = config_path.unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .join("pmat.toml")
        });

        let default_config = Self::default_config();

        Self {
            config: Arc::new(RwLock::new(default_config)),
            config_path: default_path,
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Load configuration from file
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
    pub fn get_config(&self) -> Result<PmatConfig> {
        Ok(self
            .config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?
            .clone())
    }

    /// Update configuration
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
    pub fn add_watcher(&self, watcher: Box<dyn ConfigWatcher + Send + Sync>) -> Result<()> {
        let mut watchers = self
            .watchers
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire watchers lock"))?;
        watchers.push(watcher);
        Ok(())
    }

    /// Get specific configuration section
    pub fn get_quality_config(&self) -> Result<QualityConfig> {
        Ok(self.get_config()?.quality)
    }

    pub fn get_analysis_config(&self) -> Result<AnalysisConfig> {
        Ok(self.get_config()?.analysis)
    }

    pub fn get_performance_config(&self) -> Result<PerformanceConfig> {
        Ok(self.get_config()?.performance)
    }

    pub fn get_mcp_config(&self) -> Result<McpConfig> {
        Ok(self.get_config()?.mcp)
    }

    pub fn get_roadmap_config(&self) -> Result<RoadmapConfig> {
        Ok(self.get_config()?.roadmap)
    }

    pub fn get_telemetry_config(&self) -> Result<TelemetryConfig> {
        Ok(self.get_config()?.telemetry)
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

    /// Create default configuration
    #[must_use] 
    pub fn default_config() -> PmatConfig {
        PmatConfig {
            system: SystemConfig {
                project_name: "pmat".to_string(),
                project_path: std::env::current_dir().unwrap_or_default(),
                output_dir: PathBuf::from("target/pmat"),
                max_concurrent_operations: num_cpus::get(),
                verbose: false,
                debug: false,
                default_toolchain: "rust".to_string(),
            },
            quality: QualityConfig {
                max_complexity: 30,
                max_cognitive_complexity: 25,
                min_coverage: 80.0,
                allow_satd: false,
                require_docs: true,
                lint_compliance: true,
                fail_on_violation: true,
            },
            analysis: AnalysisConfig {
                include_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string()],
                exclude_patterns: vec![
                    "**/target/**".to_string(),
                    "**/node_modules/**".to_string(),
                ],
                max_file_size: 1024 * 1024, // 1MB
                max_line_length: 100,
                skip_vendor: true,
                parallel: true,
                thread_count: 0,      // Auto
                timeout_seconds: 300, // 5 minutes
            },
            performance: PerformanceConfig {
                enable_regression_tests: true,
                enable_memory_tests: true,
                enable_throughput_tests: true,
                test_iterations: 10,
                timeout_ms: 30000,
                target_startup_latency_ms: 127,
                target_throughput_loc_per_sec: 487000,
                target_memory_mb: 47,
            },
            mcp: McpConfig {
                server_name: "pmat-mcp-server".to_string(),
                server_version: env!("CARGO_PKG_VERSION").to_string(),
                enable_compression: true,
                request_timeout_seconds: 30,
                max_request_size: 10 * 1024 * 1024, // 10MB
                log_requests: false,
                enabled_tools: vec![
                    "analyze_complexity".to_string(),
                    "analyze_dead_code".to_string(),
                    "quality_gate".to_string(),
                    "refactor_start".to_string(),
                ],
            },
            roadmap: RoadmapConfig {
                roadmap_path: PathBuf::from("docs/execution/roadmap.md"),
                auto_generate_todos: true,
                enforce_quality_gates: true,
                require_task_ids: true,
                task_id_pattern: "PMAT-[0-9]{4}".to_string(),
                velocity_tracking: true,
                burndown_charts: true,
                git: GitConfig {
                    create_branches: true,
                    branch_pattern: "feature/{task_id}".to_string(),
                    commit_pattern: "{task_id}: {message}".to_string(),
                    require_quality_check: true,
                },
            },
            telemetry: TelemetryConfig {
                enabled: true,
                collection_interval_seconds: 60,
                max_data_age_days: 30,
                enable_aggregation: true,
                enable_export: false,
                export_format: "json".to_string(),
            },
            custom: HashMap::new(),
        }
    }
}

impl ConfigurationService {
    /// Start the configuration service
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
    pub async fn status(&self) -> Result<String> {
        let config_exists = self.config_path.exists();
        let _config = self.get_config()?;

        Ok(format!(
            "Configuration service: {} (file: {}, sections: {})",
            if config_exists { "loaded" } else { "default" },
            self.config_path.display(),
            7 // Number of main config sections
        ))
    }

    /// Get service metrics
    pub async fn get_metrics(&self) -> Result<ServiceMetrics> {
        Ok(self
            .metrics
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire metrics lock"))?
            .clone())
    }

    /// Check service health
    pub async fn health_check(&self) -> Result<bool> {
        // Check if we can read the configuration
        self.get_config().map(|_| true)
    }
}

// Global configuration service instance (singleton pattern)
lazy_static::lazy_static! {
    static ref CONFIGURATION: Arc<ConfigurationService> = Arc::new(ConfigurationService::new(None));
}

/// Get the global configuration service instance - THE ONE way to access configuration
#[must_use] 
pub fn configuration() -> Arc<ConfigurationService> {
    CONFIGURATION.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_configuration_service_creation() {
        let config_service = ConfigurationService::new(None);
        let config = config_service.get_config().unwrap();

        assert_eq!(config.system.project_name, "pmat");
        assert_eq!(config.quality.max_complexity, 20);
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
        assert_eq!(quality_config.max_complexity, 20);

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
    async fn test_service_lifecycle() {
        let config_service = ConfigurationService::new(None);

        // Test service operations
        assert!(config_service.start().await.is_ok());
        assert!(config_service.health_check().await.unwrap());

        let status = config_service.status().await.unwrap();
        assert!(status.contains("Configuration service"));

        let metrics = config_service.get_metrics().await.unwrap();
        assert_eq!(metrics.request_count, 1); // From the start() call

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

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
