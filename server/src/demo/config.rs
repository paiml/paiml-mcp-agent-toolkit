use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub version: String,
    pub panels: PanelConfig,
    pub export: ExportConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    pub dependency: DependencyPanelConfig,
    pub complexity: ComplexityPanelConfig,
    pub churn: ChurnPanelConfig,
    pub context: ContextPanelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPanelConfig {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub grouping: GroupingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupingStrategy {
    Module,
    Directory,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityPanelConfig {
    pub threshold: u32,
    pub max_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnPanelConfig {
    pub days: u32,
    pub max_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPanelConfig {
    pub include_ast: bool,
    pub include_metrics: bool,
    pub max_file_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub formats: Vec<String>,
    pub include_metadata: bool,
    pub include_raw_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cache_enabled: bool,
    pub cache_ttl: u64,
    pub parallel_workers: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            panels: PanelConfig {
                dependency: DependencyPanelConfig {
                    max_nodes: 20,
                    max_edges: 60,
                    grouping: GroupingStrategy::Module,
                },
                complexity: ComplexityPanelConfig {
                    threshold: 15,
                    max_items: 50,
                },
                churn: ChurnPanelConfig {
                    days: 30,
                    max_items: 20,
                },
                context: ContextPanelConfig {
                    include_ast: true,
                    include_metrics: true,
                    max_file_size: 500000,
                },
            },
            export: ExportConfig {
                formats: vec![
                    "markdown".to_string(),
                    "json".to_string(),
                    "sarif".to_string(),
                ],
                include_metadata: true,
                include_raw_data: false,
            },
            performance: PerformanceConfig {
                cache_enabled: true,
                cache_ttl: 3600,
                parallel_workers: 4,
            },
        }
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<DisplayConfig>>,
    update_tx: broadcast::Sender<DisplayConfig>,
    _watcher: Option<RecommendedWatcher>,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let (update_tx, _) = broadcast::channel(16);
        let config = Arc::new(RwLock::new(DisplayConfig::default()));

        Ok(Self {
            config,
            update_tx,
            _watcher: None,
        })
    }

    pub fn load_from_file(path: &Path) -> Result<DisplayConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: DisplayConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    pub async fn load(&mut self, path: &Path) -> Result<()> {
        let config = Self::load_from_file(path)?;
        *self.config.write().await = config.clone();
        let _ = self.update_tx.send(config);
        Ok(())
    }

    pub async fn watch(&mut self, path: PathBuf) -> Result<()> {
        let config = self.config.clone();
        let tx = self.update_tx.clone();
        let watch_path = path.clone();

        // Create watcher with closure that captures necessary state
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    match event.kind {
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                            // Load config synchronously since we're in a sync context
                            if let Ok(new_config) = Self::load_from_file(&watch_path) {
                                // Use blocking write since we're in sync context
                                if let Ok(mut guard) = config.try_write() {
                                    *guard = new_config.clone();
                                    let _ = tx.send(new_config);
                                    tracing::info!("Configuration reloaded from {:?}", watch_path);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })?;

        // Watch the config file
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // Store the watcher to keep it alive
        self._watcher = Some(watcher);

        // Initial load
        self.load(&path).await?;

        Ok(())
    }

    pub async fn get_config(&self) -> DisplayConfig {
        self.config.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DisplayConfig> {
        self.update_tx.subscribe()
    }

    // Convenience methods for accessing specific configurations
    pub async fn get_dependency_config(&self) -> DependencyPanelConfig {
        self.config.read().await.panels.dependency.clone()
    }

    pub async fn get_complexity_config(&self) -> ComplexityPanelConfig {
        self.config.read().await.panels.complexity.clone()
    }

    pub async fn get_churn_config(&self) -> ChurnPanelConfig {
        self.config.read().await.panels.churn.clone()
    }

    pub async fn get_context_config(&self) -> ContextPanelConfig {
        self.config.read().await.panels.context.clone()
    }

    pub async fn get_export_config(&self) -> ExportConfig {
        self.config.read().await.export.clone()
    }

    pub async fn get_performance_config(&self) -> PerformanceConfig {
        self.config.read().await.performance.clone()
    }
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            dependency: DependencyPanelConfig {
                max_nodes: 20,
                max_edges: 60,
                grouping: GroupingStrategy::Module,
            },
            complexity: ComplexityPanelConfig {
                threshold: 15,
                max_items: 50,
            },
            churn: ChurnPanelConfig {
                days: 30,
                max_items: 20,
            },
            context: ContextPanelConfig {
                include_ast: true,
                include_metrics: true,
                max_file_size: 500000,
            },
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConfigManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = DisplayConfig::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.panels.dependency.max_nodes, 20);
        assert_eq!(config.panels.complexity.threshold, 15);
    }

    #[test]
    fn test_load_from_yaml() {
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
  formats: ["json"]
  include_metadata: false
  include_raw_data: true
performance:
  cache_enabled: false
  cache_ttl: 7200
  parallel_workers: 8
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();

        let config = ConfigManager::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.panels.dependency.max_nodes, 30);
        assert_eq!(config.panels.complexity.threshold, 20);
        assert_eq!(config.performance.parallel_workers, 8);
    }

    #[tokio::test]
    async fn test_config_manager() {
        let manager = ConfigManager::new().unwrap();

        // Test default config
        let config = manager.get_config().await;
        assert_eq!(config.version, "1.0");

        // Test specific config accessors
        let dep_config = manager.get_dependency_config().await;
        assert_eq!(dep_config.max_nodes, 20);

        let perf_config = manager.get_performance_config().await;
        assert!(perf_config.cache_enabled);
    }
}
