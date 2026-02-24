#![cfg_attr(coverage_nightly, coverage(off))]
//! Foundation Layer: Real-time Monitoring Engine
//!
//! Phase 1 Implementation (Months 1-3)
//! Practical monitoring using proven technologies

use anyhow::Result;
use crossbeam_channel;
#[cfg(feature = "watch")]
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

/// Opaque watcher type - only holds a real watcher when the watch feature is enabled
#[cfg(feature = "watch")]
type WatcherType = RecommendedWatcher;
#[cfg(not(feature = "watch"))]
type WatcherType = ();

/// Practical monitoring using proven technologies
#[allow(dead_code)]
pub struct QualityMonitor {
    /// FSEvents/inotify for cross-platform file watching
    watcher: Arc<ParkingLotRwLock<Option<WatcherType>>>,

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

// QualityMonitor implementation
include!("foundation_impl.rs");

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

// Property-based tests
include!("foundation_property_tests.rs");

// Coverage tests - config, file change, should_analyze
include!("foundation_coverage_tests.rs");

// Coverage tests - analyze_incremental, edge cases, concurrency
include!("foundation_coverage_tests_part2.rs");
