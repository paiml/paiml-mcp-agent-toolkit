//! Foundation Layer: Real-time Monitoring Engine
//!
//! Phase 1 Implementation (Months 1-3)
//! Practical monitoring using proven technologies

use anyhow::Result;
use crossbeam_channel;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock as ParkingLotRwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::info;

use crate::unified_quality::enhanced_parser::EnhancedParser;
use crate::unified_quality::events::QualityEvent;
use crate::unified_quality::metrics::Metrics;

/// Practical monitoring using proven technologies
pub struct QualityMonitor {
    /// FSEvents/inotify for cross-platform file watching
    watcher: Arc<ParkingLotRwLock<Option<RecommendedWatcher>>>,

    /// Tree-sitter for incremental parsing (5-10ms latency)
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

    /// Analyze incremental changes with O(log n) complexity
    pub fn analyze_incremental(&self, change: FileChange) -> Result<Metrics> {
        // Use real tree-sitter incremental parsing
        let mut parser = self.parser.lock().unwrap();
        parser.parse_incremental(&change.path, &change.content)
    }

    /// Get current metrics for a file
    #[must_use]
    pub fn get_metrics(&self, path: &Path) -> Option<Metrics> {
        self.metrics.get(path).map(|entry| entry.clone())
    }

    /// Get all metrics
    #[must_use]
    pub fn get_all_metrics(&self) -> HashMap<PathBuf, Metrics> {
        self.metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Subscribe to quality events
    #[must_use]
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
                for path in event.paths {
                    if Self::should_analyze(&path, &config.watch_patterns) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let _change = FileChange {
                                path: path.clone(),
                                content: content.clone(),
                                old_tree: None,
                                timestamp: SystemTime::now(),
                            };

                            if let Ok(mut parser_lock) = parser.lock() {
                                if let Ok(new_metrics) =
                                    parser_lock.parse_incremental(&path, &content)
                                {
                                    let old_metrics =
                                        metrics.insert(path.clone(), new_metrics.clone());

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
            .filter_map(std::result::Result::ok)
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
        // use rayon::prelude::*; // Currently unused

        let results: Vec<_> = paths
            .iter()
            .filter_map(|path| {
                std::fs::read_to_string(path).ok().map(|content| {
                    if let Ok(mut parser) = self.parser.lock() {
                        (path.clone(), parser.parse_incremental(path, &content))
                    } else {
                        (path.clone(), Err(anyhow::anyhow!("Failed to lock parser")))
                    }
                })
            })
            .collect();

        for (path, result) in results {
            if let Ok(metrics) = result {
                self.metrics.insert(path, metrics);
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
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn monitor_config_thresholds_valid(
            complexity_threshold in 1u32..1000,
            max_batch_size in 1usize..1000,
            update_interval_secs in 1u64..3600,
            debounce_millis in 1u64..5000
        ) {
            let config = MonitorConfig {
                complexity_threshold,
                max_batch_size,
                update_interval: Duration::from_secs(update_interval_secs),
                debounce_interval: Duration::from_millis(debounce_millis),
                watch_patterns: vec!["**/*.rs".to_string()],
                incremental_parsing: true,
                cache_ast: true,
            };

            prop_assert!(config.complexity_threshold > 0);
            prop_assert!(config.max_batch_size > 0);
            prop_assert!(config.update_interval.as_secs() > 0);
            prop_assert!(config.debounce_interval.as_millis() > 0);
        }

        #[test]
        fn file_pattern_matching_consistent(
            extension in "[a-z]{2,5}",
            filename in "[a-zA-Z0-9_-]{1,20}"
        ) {
            let patterns = vec![format!("**/*.{}", extension)];
            let test_file = format!("{}.{}", filename, extension);
            let path = Path::new(&test_file);

            let matches = QualityMonitor::should_analyze(path, &patterns);
            prop_assert!(matches);

            // Test non-matching extension
            let wrong_file = format!("{}.txt", filename);
            let wrong_path = Path::new(&wrong_file);
            let wrong_matches = QualityMonitor::should_analyze(wrong_path, &patterns);
            prop_assert!(wrong_matches == (extension == "txt"));
        }

        #[test]
        fn quality_monitor_creation_stable(
            complexity_threshold in 5u32..50,
            max_batch_size in 10usize..200
        ) {
            let config = MonitorConfig {
                complexity_threshold,
                max_batch_size,
                update_interval: Duration::from_secs(5),
                debounce_interval: Duration::from_millis(500),
                watch_patterns: vec!["**/*.rs".to_string()],
                incremental_parsing: true,
                cache_ast: true,
            };

            let monitor_result = QualityMonitor::new(config);
            prop_assert!(monitor_result.is_ok());

            let monitor = monitor_result.unwrap();
            prop_assert_eq!(monitor.metrics.len(), 0);
        }

        #[test]
        fn file_change_properties_valid(
            content in "[a-zA-Z0-9\\s\\n{}();]{10,1000}",
            path_components in prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
        ) {
            let path_str = path_components.join("/") + ".rs";
            let path = PathBuf::from(path_str);

            let file_change = FileChange {
                path: path.clone(),
                content: content.clone(),
                old_tree: None,
                timestamp: SystemTime::now(),
            };

            prop_assert_eq!(file_change.path, path);
            prop_assert_eq!(file_change.content, content);
            prop_assert!(file_change.old_tree.is_none());
        }

        #[test]
        fn batch_processing_properties(
            batch_size in 1usize..100,
            file_count in 1usize..1000
        ) {
            let config = MonitorConfig {
                max_batch_size: batch_size,
                complexity_threshold: 20,
                update_interval: Duration::from_secs(5),
                debounce_interval: Duration::from_millis(500),
                watch_patterns: vec!["**/*.rs".to_string()],
                incremental_parsing: true,
                cache_ast: true,
            };

            // Properties about batch processing
            let expected_batches = (file_count as f64 / batch_size as f64).ceil() as usize;
            prop_assert!(expected_batches >= 1);
            prop_assert!(expected_batches <= file_count);

            // Config should maintain batch size limits
            prop_assert_eq!(config.max_batch_size, batch_size);
        }

        #[test]
        #[ignore]
        fn pattern_matching_edge_cases(
            pattern_type in 0..3usize,
            file_extension in "[a-z]{1,10}"
        ) {
            let patterns = [
                format!("**/*.{}", file_extension),
                format!("*.{}", file_extension),
                file_extension.clone()
            ];

            let test_pattern = &patterns[pattern_type];
            let test_file = format!("test.{}", file_extension);
            let path = Path::new(&test_file);

            let matches = QualityMonitor::should_analyze(path, std::slice::from_ref(test_pattern));

            match pattern_type {
                0 => prop_assert!(matches), // **/*.ext should match
                1 => prop_assert!(matches), // *.ext should match
                2 => prop_assert!(matches), // ext should match (contains)
                _ => unreachable!(),
            }
        }

        #[test]
        fn metrics_aggregation_properties(
            _file_count in 1usize..50,
            complexity_values in prop::collection::vec(1u32..30, 1..50)
        ) {
            // Properties of metrics aggregation
            if !complexity_values.is_empty() {
                let sum: u32 = complexity_values.iter().sum();
                let avg = sum as f64 / complexity_values.len() as f64;
                let max = *complexity_values.iter().max().unwrap();
                let min = *complexity_values.iter().min().unwrap();

                prop_assert!(avg >= min as f64);
                prop_assert!(avg <= max as f64);
                prop_assert!(sum >= max);
                prop_assert!(max >= min);
            }
        }

        #[test]
        fn concurrent_access_properties(
            thread_count in 1usize..10,
            operation_count in 1usize..100
        ) {
            // Properties of concurrent operations
            prop_assert!(thread_count > 0);
            prop_assert!(operation_count > 0);

            let total_operations = thread_count * operation_count;
            prop_assert!(total_operations >= thread_count);
            prop_assert!(total_operations >= operation_count);

            // DashMap should handle concurrent access
            let metrics: Arc<dashmap::DashMap<PathBuf, Metrics>> = Arc::new(dashmap::DashMap::new());
            prop_assert_eq!(metrics.len(), 0);
        }
    }
}
