#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality Monitoring Engine for Claude Code Agent Mode
//!
//! PMAT-7002: Real-time complexity tracking, file change event processing,
//! basic notification system, and metrics collection and storage.

use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Quality monitoring engine for continuous code quality tracking
pub struct QualityMonitorEngine {
    /// Configuration
    config: QualityMonitorConfig,

    /// File system watchers by project
    watchers: Arc<RwLock<HashMap<String, RecommendedWatcher>>>,

    /// Current metrics by project
    metrics: Arc<RwLock<HashMap<String, QualityMetrics>>>,

    /// Event sender for quality updates
    event_sender: Option<mpsc::Sender<QualityEvent>>,
}

/// Configuration for quality monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMonitorConfig {
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
}

impl Default for QualityMonitorConfig {
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
        }
    }
}

/// Quality metrics for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub project_id: String,
    pub last_updated: SystemTime,
    pub quality_score: f64,
    pub files_analyzed: usize,
    pub functions_analyzed: usize,
    pub avg_complexity: f64,
    pub max_complexity: u32,
    pub hotspot_functions: usize,
    pub satd_issues: usize,
    pub complexity_distribution: ComplexityDistribution,
    pub file_metrics: HashMap<String, FileQualityMetrics>,
    pub quality_trend: f64,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            last_updated: SystemTime::now(),
            quality_score: 0.0,
            files_analyzed: 0,
            functions_analyzed: 0,
            avg_complexity: 0.0,
            max_complexity: 0,
            hotspot_functions: 0,
            satd_issues: 0,
            complexity_distribution: ComplexityDistribution {
                low: 0,
                medium: 0,
                high: 0,
                very_high: 0,
                violations: 0,
            },
            file_metrics: HashMap::new(),
            quality_trend: 0.0,
        }
    }
}

/// Complexity distribution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityDistribution {
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub very_high: usize,
    pub violations: usize,
}

/// Quality metrics for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileQualityMetrics {
    pub file_path: String,
    pub last_modified: SystemTime,
    pub last_analyzed: SystemTime,
    pub function_count: usize,
    pub avg_complexity: f64,
    pub max_complexity: u32,
    pub satd_issues: usize,
    pub quality_score: f64,
    pub needs_attention: bool,
}

/// Quality events for notifications
#[derive(Debug, Clone)]
pub enum QualityEvent {
    MetricsUpdated {
        project_id: String,
        metrics: QualityMetrics,
        changes: Vec<QualityChange>,
    },
    ThresholdViolated {
        project_id: String,
        violation: QualityViolation,
    },
    FileAnalyzed {
        project_id: String,
        file_path: String,
        metrics: FileQualityMetrics,
    },
    TrendDetected {
        project_id: String,
        trend: QualityTrend,
    },
    Error {
        project_id: String,
        error: String,
    },
}

/// Types of quality changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityChange {
    ComplexityIncrease { file: String, old_complexity: f64, new_complexity: f64 },
    ComplexityDecrease { file: String, old_complexity: f64, new_complexity: f64 },
    SatdAdded { file: String, count: usize },
    SatdRemoved { file: String, count: usize },
    FileAdded { file: String },
    FileRemoved { file: String },
    QualityImproved { old_score: f64, new_score: f64 },
    QualityDegraded { old_score: f64, new_score: f64 },
}

/// Quality violations that trigger alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityViolation {
    ComplexityThreshold { file: String, function: String, complexity: u32 },
    QualityScoreBelow { current_score: f64, threshold: f64 },
    TooManySatdIssues { count: usize, threshold: usize },
    QualityTrendNegative { trend: f64, duration: Duration },
}

/// Quality trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTrend {
    Improving { rate: f64, duration: Duration },
    Stable { score: f64, duration: Duration },
    Degrading { rate: f64, duration: Duration },
}

// --- Implementation split into include files ---
include!("quality_monitor_engine.rs");
include!("quality_monitor_events.rs");
include!("quality_monitor_analysis.rs");

// Tests extracted to quality_monitor_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "quality_monitor_tests.rs"]
mod tests;
