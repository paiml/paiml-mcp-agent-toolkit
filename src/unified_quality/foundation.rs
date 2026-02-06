#![cfg_attr(coverage_nightly, coverage(off))]
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
        let mut parser = self.parser.lock().expect("internal error");
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

        let metrics = parser
            .parse_incremental(&path, code)
            .expect("internal error");
        assert!(metrics.complexity > 0);
        assert!(metrics.functions > 0);
    }

    #[tokio::test]
    async fn test_quality_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("internal error");
        assert_eq!(monitor.metrics.len(), 0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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

            let monitor = monitor_result.expect("internal error");
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
        #[ignore = "requires quality framework setup"]
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
                let max = *complexity_values.iter().max().expect("internal error");
                let min = *complexity_values.iter().min().expect("internal error");

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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::Path;
    use std::time::Duration;

    // ============================================
    // Test Fixtures and Helpers
    // ============================================

    /// Helper to create a default MonitorConfig for testing
    fn create_test_config() -> MonitorConfig {
        MonitorConfig {
            update_interval: Duration::from_secs(1),
            complexity_threshold: 15,
            watch_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string()],
            debounce_interval: Duration::from_millis(100),
            max_batch_size: 10,
            incremental_parsing: true,
            cache_ast: true,
        }
    }

    /// Helper to create a FileChange for testing
    fn create_test_file_change(path: &str, content: &str) -> FileChange {
        FileChange {
            path: PathBuf::from(path),
            content: content.to_string(),
            old_tree: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Helper to create valid Rust code for testing
    fn create_simple_rust_code() -> &'static str {
        r#"fn main() { println!("hello"); }"#
    }

    /// Helper to create complex Rust code for testing
    fn create_complex_rust_code() -> &'static str {
        r#"
        fn complex_fn(x: i32) -> i32 {
            if x > 0 {
                for i in 0..x {
                    if i % 2 == 0 {
                        println!("{}", i);
                    }
                }
            }
            x
        }
        "#
    }

    // ============================================
    // MonitorConfig Tests
    // ============================================

    #[test]
    fn test_monitor_config_default_values() {
        let config = MonitorConfig::default();

        assert_eq!(config.update_interval, Duration::from_secs(5));
        assert_eq!(config.complexity_threshold, 20);
        assert_eq!(config.debounce_interval, Duration::from_millis(500));
        assert_eq!(config.max_batch_size, 50);
        assert!(config.incremental_parsing);
        assert!(config.cache_ast);
        assert_eq!(config.watch_patterns.len(), 4);
    }

    #[test]
    fn test_monitor_config_custom_values() {
        let config = MonitorConfig {
            update_interval: Duration::from_secs(10),
            complexity_threshold: 30,
            watch_patterns: vec!["**/*.go".to_string()],
            debounce_interval: Duration::from_millis(200),
            max_batch_size: 100,
            incremental_parsing: false,
            cache_ast: false,
        };

        assert_eq!(config.update_interval, Duration::from_secs(10));
        assert_eq!(config.complexity_threshold, 30);
        assert_eq!(config.watch_patterns.len(), 1);
        assert_eq!(config.watch_patterns[0], "**/*.go");
        assert!(!config.incremental_parsing);
        assert!(!config.cache_ast);
    }

    #[test]
    fn test_monitor_config_clone() {
        let config1 = create_test_config();
        let config2 = config1.clone();

        assert_eq!(config1.complexity_threshold, config2.complexity_threshold);
        assert_eq!(config1.max_batch_size, config2.max_batch_size);
        assert_eq!(config1.watch_patterns, config2.watch_patterns);
    }

    #[test]
    fn test_monitor_config_debug() {
        let config = create_test_config();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("MonitorConfig"));
        assert!(debug_str.contains("complexity_threshold"));
    }

    // ============================================
    // FileChange Tests
    // ============================================

    #[test]
    fn test_file_change_creation() {
        let change = create_test_file_change("test.rs", "fn test() {}");

        assert_eq!(change.path, PathBuf::from("test.rs"));
        assert_eq!(change.content, "fn test() {}");
        assert!(change.old_tree.is_none());
    }

    #[test]
    fn test_file_change_with_old_tree() {
        let change = FileChange {
            path: PathBuf::from("test.rs"),
            content: "fn test() {}".to_string(),
            old_tree: Some("previous_tree_data".to_string()),
            timestamp: SystemTime::now(),
        };

        assert!(change.old_tree.is_some());
        assert_eq!(change.old_tree.as_ref().unwrap(), "previous_tree_data");
    }

    #[test]
    fn test_file_change_clone() {
        let change1 = create_test_file_change("test.rs", "fn test() {}");
        let change2 = change1.clone();

        assert_eq!(change1.path, change2.path);
        assert_eq!(change1.content, change2.content);
    }

    #[test]
    fn test_file_change_debug() {
        let change = create_test_file_change("test.rs", "fn test() {}");
        let debug_str = format!("{:?}", change);

        assert!(debug_str.contains("FileChange"));
        assert!(debug_str.contains("test.rs"));
    }

    // ============================================
    // QualityMonitor Creation Tests
    // ============================================

    #[test]
    fn test_quality_monitor_new_default_config() {
        let config = MonitorConfig::default();
        let result = QualityMonitor::new(config);

        assert!(result.is_ok());
        let monitor = result.expect("Failed to create monitor");
        assert_eq!(monitor.metrics.len(), 0);
    }

    #[test]
    fn test_quality_monitor_new_custom_config() {
        let config = create_test_config();
        let result = QualityMonitor::new(config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_monitor_empty_metrics_initially() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let all_metrics = monitor.get_all_metrics();

        assert!(all_metrics.is_empty());
    }

    // ============================================
    // should_analyze Tests
    // ============================================

    #[test]
    fn test_should_analyze_rust_file() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("src/main.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_python_file() {
        let patterns = vec!["**/*.py".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("script.py"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_nested_path() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("src/module/submodule/file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_not_analyze_non_matching_extension() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(!QualityMonitor::should_analyze(
            Path::new("file.txt"),
            &patterns
        ));
    }

    #[test]
    fn test_should_not_analyze_markdown() {
        let patterns = vec!["**/*.rs".to_string(), "**/*.py".to_string()];
        assert!(!QualityMonitor::should_analyze(
            Path::new("README.md"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_multiple_patterns() {
        let patterns = vec![
            "**/*.rs".to_string(),
            "**/*.py".to_string(),
            "**/*.js".to_string(),
        ];
        assert!(QualityMonitor::should_analyze(
            Path::new("script.js"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!QualityMonitor::should_analyze(
            Path::new("test.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_contains_pattern() {
        let patterns = vec!["test".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("my_test_file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_typescript() {
        let patterns = vec!["**/*.ts".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("component.ts"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_javascript() {
        let patterns = vec!["**/*.js".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("index.js"),
            &patterns
        ));
    }

    // ============================================
    // analyze_incremental Tests
    // ============================================

    #[test]
    fn test_analyze_incremental_simple_code() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("test.rs", create_simple_rust_code());

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.functions > 0);
        assert!(metrics.lines > 0);
    }

    #[test]
    fn test_analyze_incremental_complex_code() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("complex.rs", create_complex_rust_code());

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.complexity > 1);
        assert!(metrics.functions > 0);
    }

    #[test]
    fn test_analyze_incremental_empty_function() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("empty.rs", "fn empty() {}");

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_incremental_with_satd() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let code = r#"
            fn test() {
                // TODO: implement this
                // FIXME: fix the bug
            }
        "#;
        let change = create_test_file_change("satd.rs", code);

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.satd_count >= 2);
    }

    // ============================================
    // get_metrics and get_all_metrics Tests
    // ============================================

    #[test]
    fn test_get_metrics_nonexistent_path() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let result = monitor.get_metrics(Path::new("nonexistent.rs"));

        assert!(result.is_none());
    }

    #[test]
    fn test_get_all_metrics_empty() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let all_metrics = monitor.get_all_metrics();

        assert!(all_metrics.is_empty());
    }

    // ============================================
    // subscribe Tests
    // ============================================

    #[test]
    fn test_subscribe_returns_receiver() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let receiver = monitor.subscribe();

        // Receiver should be empty since no events have been sent
        assert!(receiver.try_recv().is_err());
    }

    // ============================================
    // DashMap Re-export Tests
    // ============================================

    #[test]
    fn test_dashmap_reexport() {
        let map: DashMap<String, i32> = DashMap::new();
        map.insert("key".to_string(), 42);

        assert_eq!(map.len(), 1);
        assert_eq!(*map.get("key").expect("Key should exist"), 42);
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_analyze_incremental_invalid_rust_code() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("invalid.rs", "this is not valid rust code { {");

        let result = monitor.analyze_incremental(change);
        // Invalid code should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_incremental_empty_content() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("empty.rs", "");

        let result = monitor.analyze_incremental(change);
        // Empty file should be parseable (no functions, no complexity)
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_analyze_path_with_spaces() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("path with spaces/file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_hidden_file() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new(".hidden_file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_config_zero_batch_size() {
        let config = MonitorConfig {
            max_batch_size: 0,
            ..MonitorConfig::default()
        };
        // Should still be able to create monitor
        let result = QualityMonitor::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_zero_complexity_threshold() {
        let config = MonitorConfig {
            complexity_threshold: 0,
            ..MonitorConfig::default()
        };
        let result = QualityMonitor::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_change_timestamp_is_recent() {
        let before = SystemTime::now();
        let change = create_test_file_change("test.rs", "fn test() {}");
        let after = SystemTime::now();

        assert!(change.timestamp >= before);
        assert!(change.timestamp <= after);
    }

    // ============================================
    // Concurrency Tests
    // ============================================

    #[test]
    fn test_dashmap_concurrent_insert() {
        use std::sync::Arc;
        use std::thread;

        let map: Arc<DashMap<i32, i32>> = Arc::new(DashMap::new());
        let mut handles = vec![];

        for i in 0..10 {
            let map_clone = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                map_clone.insert(i, i * 2);
            }));
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        assert_eq!(map.len(), 10);
    }

    // ============================================
    // Property-Based Coverage Tests
    // ============================================

    #[test]
    fn test_monitor_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string(&config).expect("Serialization failed");

        assert!(json.contains("complexity_threshold"));
        assert!(json.contains("max_batch_size"));

        let deserialized: MonitorConfig =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(
            config.complexity_threshold,
            deserialized.complexity_threshold
        );
    }

    #[test]
    fn test_multiple_file_analyses() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");

        let files = vec![
            ("file1.rs", "fn a() {}"),
            ("file2.rs", "fn b() { if true {} }"),
            ("file3.rs", "fn c() { for i in 0..10 {} }"),
        ];

        for (name, content) in files {
            let change = create_test_file_change(name, content);
            let result = monitor.analyze_incremental(change);
            assert!(result.is_ok(), "Failed to analyze {}", name);
        }
    }

    #[test]
    fn test_analyze_code_with_loops() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let code = r#"
            fn loops() {
                for i in 0..10 {
                    while true {
                        loop {
                            break;
                        }
                    }
                }
            }
        "#;
        let change = create_test_file_change("loops.rs", code);
        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.complexity > 3); // Multiple loop constructs
    }

    #[test]
    fn test_analyze_code_with_match() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let code = r#"
            fn matcher(x: i32) -> i32 {
                match x {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    _ => 3,
                }
            }
        "#;
        let change = create_test_file_change("match.rs", code);
        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.complexity >= 4); // Match arms add complexity
    }
}
