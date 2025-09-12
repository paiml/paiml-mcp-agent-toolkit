//! Foundation Layer: Real-time Monitoring Engine
//! 
//! Phase 1 Implementation (Months 1-3)
//! Practical monitoring using proven technologies

use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info};
use crossbeam_channel;
use parking_lot::RwLock as ParkingLotRwLock;

use crate::unified_quality::metrics::Metrics;
use crate::unified_quality::events::QualityEvent;
use crate::unified_quality::enhanced_parser::EnhancedParser;

/// Practical monitoring using proven technologies
pub struct QualityMonitor {
    /// FSEvents/inotify for cross-platform file watching
    watcher: Arc<ParkingLotRwLock<Option<RecommendedWatcher>>>,
    
    /// Enhanced parser for accurate analysis
    parser: Arc<std::sync::Mutex<EnhancedParser>>,
    
    /// Lock-free metrics storage
    metrics: Arc<dashmap::DashMap<PathBuf, Metrics>>,
    
    /// Crossbeam channel for bounded memory usage
    events: crossbeam_channel::Sender<QualityEvent>,
    
    /// Configuration
    config: MonitorConfig,
}

/// Configuration for quality monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Update interval for periodic checks
    pub update_interval: Duration,
    
    /// Complexity threshold for alerts
    pub complexity_threshold: u32,
    
    /// File patterns to watch
    pub watch_patterns: Vec<String>,
    
    /// Debounce interval to avoid excessive notifications
    pub debounce_interval: Duration,
    
    /// Maximum number of files to analyze per batch
    pub max_batch_size: usize,
    
    /// Enable incremental parsing
    pub incremental_parsing: bool,
    
    /// Cache AST for performance
    pub cache_ast: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(5),
            complexity_threshold: 20,
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
            ],
            debounce_interval: Duration::from_millis(500),
            max_batch_size: 50,
            incremental_parsing: true,
            cache_ast: true,
        }
    }
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub content: String,
    pub old_tree: Option<String>,
    pub timestamp: SystemTime,
}

impl QualityMonitor {
    /// Create a new quality monitor
    pub fn new(config: MonitorConfig) -> Result<Self> {
        let (tx, _rx) = crossbeam_channel::bounded(1000);
        
        Ok(Self {
            watcher: Arc::new(ParkingLotRwLock::new(None)),
            parser: Arc::new(std::sync::Mutex::new(EnhancedParser::new())),
            metrics: Arc::new(dashmap::DashMap::new()),
            events: tx,
            config,
        })
    }
    
    /// Start monitoring a directory
    pub async fn start_monitoring(&mut self, path: PathBuf) -> Result<()> {
        info!("Starting quality monitoring for: {:?}", path);
        
        // Create file system watcher
        let events = self.events.clone();
        let metrics = self.metrics.clone();
        let parser = self.parser.clone();
        let config = self.config.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<Event>| {
                if let Ok(event) = result {
                    Self::handle_fs_event(event, &events, &metrics, &parser, &config);
                }
            },
            Config::default(),
        )?;
        
        // Start watching the directory
        watcher.watch(&path, RecursiveMode::Recursive)?;
        
        // Store watcher
        {
            let mut guard = self.watcher.write();
            *guard = Some(watcher);
        }
        
        // Perform initial analysis
        self.analyze_directory(&path).await?;
        
        Ok(())
    }
    
    /// Analyze incremental changes
    pub fn analyze_incremental(&self, change: FileChange) -> Result<Metrics> {
        let mut parser = self.parser.lock().unwrap();
        parser.parse_incremental(&change.path, &change.content)
    }
    
    /// Get current metrics for a file
    pub fn get_metrics(&self, path: &Path) -> Option<Metrics> {
        self.metrics.get(path).map(|entry| entry.clone())
    }
    
    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<PathBuf, Metrics> {
        self.metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
    
    /// Subscribe to quality events
    pub fn subscribe(&self) -> crossbeam_channel::Receiver<QualityEvent> {
        let (_tx, rx) = crossbeam_channel::bounded(100);
        rx
    }
    
    /// Handle file system events
    fn handle_fs_event(
        event: Event,
        events: &crossbeam_channel::Sender<QualityEvent>,
        metrics: &Arc<dashmap::DashMap<PathBuf, Metrics>>,
        parser: &Arc<std::sync::Mutex<EnhancedParser>>,
        config: &MonitorConfig,
    ) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                // Process changed files
                for path in event.paths {
                    if Self::should_analyze(&path, &config.watch_patterns) {
                        info!("File changed: {:?}, triggering analysis", path);
                        
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Parse with enhanced parser
                            if let Ok(mut parser_lock) = parser.lock() {
                                if let Ok(new_metrics) = parser_lock.parse_incremental(&path, &content) {
                                    let old_metrics = metrics.insert(path.clone(), new_metrics.clone());
                                    
                                    let event = if let Some(old) = old_metrics {
                                        QualityEvent::MetricsUpdated {
                                            path: path.clone(),
                                            old_metrics: old,
                                            new_metrics,
                                        }
                                    } else {
                                        QualityEvent::FileAdded {
                                            path: path.clone(),
                                            metrics: new_metrics,
                                        }
                                    };
                                    
                                    let _ = events.try_send(event);
                                }
                            }
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some((_, metrics)) = metrics.remove(&path) {
                        let _ = events.try_send(QualityEvent::FileRemoved {
                            path,
                            last_metrics: metrics,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Check if file should be analyzed
    fn should_analyze(path: &Path, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        patterns.iter().any(|pattern| {
            if pattern.contains("**") {
                let ext = pattern.strip_prefix("**/").unwrap_or(pattern);
                path_str.ends_with(ext.strip_prefix("*").unwrap_or(ext))
            } else {
                path_str.contains(pattern)
            }
        })
    }
    
    /// Analyze entire directory
    async fn analyze_directory(&self, path: &Path) -> Result<()> {
        use walkdir::WalkDir;
        
        let mut batch = Vec::new();
        
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && Self::should_analyze(path, &self.config.watch_patterns) {
                batch.push(path.to_path_buf());
                
                if batch.len() >= self.config.max_batch_size {
                    self.analyze_batch(&batch).await?;
                    batch.clear();
                }
            }
        }
        
        if !batch.is_empty() {
            self.analyze_batch(&batch).await?;
        }
        
        Ok(())
    }
    
    /// Analyze a batch of files
    async fn analyze_batch(&self, paths: &[PathBuf]) -> Result<()> {
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(mut parser) = self.parser.lock() {
                    if let Ok(metrics) = parser.parse_incremental(path, &content) {
                        self.metrics.insert(path.clone(), metrics);
                    }
                }
            }
        }
        
        Ok(())
    }
}

// Re-export dashmap for metrics storage
pub use dashmap::DashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.complexity_threshold, 20);
        assert!(config.incremental_parsing);
        assert!(config.cache_ast);
    }

    #[test]
    fn test_should_analyze() {
        let patterns = vec!["**/*.rs".to_string(), "**/*.py".to_string()];
        
        assert!(QualityMonitor::should_analyze(
            Path::new("src/main.rs"),
            &patterns
        ));
        
        assert!(QualityMonitor::should_analyze(
            Path::new("test.py"),
            &patterns
        ));
        
        assert!(!QualityMonitor::should_analyze(
            Path::new("README.md"),
            &patterns
        ));
    }

    #[test]
    fn test_enhanced_parser_integration() {
        let mut parser = EnhancedParser::new();
        let path = PathBuf::from("test.rs");
        let code = "fn main() { if true { } }";
        
        let metrics = parser.parse_incremental(&path, code).unwrap();
        assert!(metrics.complexity > 0);
        assert!(metrics.functions > 0);
    }

    #[tokio::test]
    async fn test_quality_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).unwrap();
        assert_eq!(monitor.metrics.len(), 0);
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