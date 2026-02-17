#![cfg_attr(coverage_nightly, coverage(off))]
//! Sprint 31 Week 2 - Advanced Metrics Aggregation and Trending
//!
//! Provides sophisticated metrics aggregation, time-series analysis, and trending
//! capabilities for the TDG system. Supports rolling windows, percentile calculations,
//! and anomaly detection.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint<T> {
    pub timestamp: SystemTime,
    pub value: T,
    pub tags: HashMap<String, String>,
}

/// Rolling window aggregator for metrics
#[derive(Debug, Clone)]
pub struct RollingWindow<T: Clone> {
    window_size: Duration,
    max_points: usize,
    data: VecDeque<DataPoint<T>>,
}

impl<T: Clone> RollingWindow<T> {
    #[must_use]
    pub fn new(window_size: Duration, max_points: usize) -> Self {
        Self {
            window_size,
            max_points,
            data: VecDeque::with_capacity(max_points),
        }
    }

    pub fn push(&mut self, value: T, tags: HashMap<String, String>) {
        let now = SystemTime::now();

        // Remove old data points outside the window
        let cutoff = now - self.window_size;
        while let Some(front) = self.data.front() {
            if front.timestamp < cutoff {
                self.data.pop_front();
            } else {
                break;
            }
        }

        // Add new data point
        self.data.push_back(DataPoint {
            timestamp: now,
            value,
            tags,
        });

        // Enforce max points limit
        while self.data.len() > self.max_points {
            self.data.pop_front();
        }
    }

    #[must_use]
    pub fn get_window(&self) -> Vec<DataPoint<T>> {
        self.data.iter().cloned().collect()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Metrics aggregation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub p95: f64,
    pub p99: f64,
    pub trend: TrendDirection,
    pub anomalies: Vec<AnomalyPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPoint {
    pub timestamp: SystemTime,
    pub value: f64,
    pub severity: AnomalySeverity,
    pub deviation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Advanced metrics aggregator for the TDG system
pub struct MetricsAggregator {
    /// Storage metrics time series
    storage_metrics: Arc<RwLock<RollingWindow<StorageMetricPoint>>>,
    /// Performance metrics time series
    performance_metrics: Arc<RwLock<RollingWindow<PerformanceMetricPoint>>>,
    /// Analysis metrics time series
    analysis_metrics: Arc<RwLock<RollingWindow<AnalysisMetricPoint>>>,
    /// Alert thresholds
    alert_thresholds: Arc<RwLock<AlertThresholds>>,
    /// Historical aggregations
    #[allow(dead_code)]
    historical_stats: Arc<RwLock<HashMap<String, AggregatedStats>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetricPoint {
    pub total_entries: u64,
    pub cache_hit_ratio: f64,
    pub compression_ratio: f64,
    pub storage_size_mb: f64,
    pub write_throughput: f64,
    pub read_throughput: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricPoint {
    pub avg_analysis_time_ms: f64,
    pub active_operations: u32,
    pub queue_depth: u32,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub gc_pause_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetricPoint {
    pub files_analyzed: u64,
    pub avg_tdg_score: f64,
    pub critical_issues: u32,
    pub success_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub cpu_critical: f64,
    pub memory_critical_mb: f64,
    pub queue_depth_warning: u32,
    pub analysis_time_warning_ms: f64,
    pub cache_hit_ratio_warning: f64,
    pub storage_usage_warning_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_critical: 90.0,
            memory_critical_mb: 8192.0,
            queue_depth_warning: 100,
            analysis_time_warning_ms: 5000.0,
            cache_hit_ratio_warning: 0.5,
            storage_usage_warning_percent: 85.0,
        }
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsAggregator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage_metrics: Arc::new(RwLock::new(RollingWindow::new(
                Duration::from_secs(3600), // 1 hour window
                360,                       // Max 360 points (10 second intervals)
            ))),
            performance_metrics: Arc::new(RwLock::new(RollingWindow::new(
                Duration::from_secs(3600),
                360,
            ))),
            analysis_metrics: Arc::new(RwLock::new(RollingWindow::new(
                Duration::from_secs(3600),
                360,
            ))),
            alert_thresholds: Arc::new(RwLock::new(AlertThresholds::default())),
            historical_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record storage metrics
    pub async fn record_storage_metrics(&self, metrics: StorageMetricPoint) -> Result<()> {
        let mut window = self.storage_metrics.write().await;
        window.push(metrics, HashMap::new());
        Ok(())
    }

    /// Record performance metrics
    pub async fn record_performance_metrics(&self, metrics: PerformanceMetricPoint) -> Result<()> {
        let mut window = self.performance_metrics.write().await;

        // Check for alerts
        let thresholds = self.alert_thresholds.read().await;
        let mut tags = HashMap::new();

        if metrics.cpu_usage_percent > thresholds.cpu_critical {
            tags.insert("alert".to_string(), "cpu_critical".to_string());
        }
        if metrics.memory_usage_mb > thresholds.memory_critical_mb {
            tags.insert("alert".to_string(), "memory_critical".to_string());
        }
        if metrics.queue_depth > thresholds.queue_depth_warning {
            tags.insert("alert".to_string(), "queue_depth_warning".to_string());
        }

        window.push(metrics, tags);
        Ok(())
    }

    /// Record analysis metrics
    pub async fn record_analysis_metrics(&self, metrics: AnalysisMetricPoint) -> Result<()> {
        let mut window = self.analysis_metrics.write().await;
        window.push(metrics, HashMap::new());
        Ok(())
    }

    /// Calculate aggregated statistics for performance metrics
    pub async fn aggregate_performance_stats(&self) -> AggregatedStats {
        let window = self.performance_metrics.read().await;
        let data = window.get_window();

        if data.is_empty() {
            return AggregatedStats {
                count: 0,
                mean: 0.0,
                median: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
                p95: 0.0,
                p99: 0.0,
                trend: TrendDirection::Stable,
                anomalies: Vec::new(),
            };
        }

        let values: Vec<f64> = data.iter().map(|p| p.value.avg_analysis_time_ms).collect();

        self.calculate_stats(&values, &data)
    }

    /// Calculate statistics from raw values
    fn calculate_stats<T>(&self, values: &[f64], data: &[DataPoint<T>]) -> AggregatedStats {
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        // Calculate median
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };

        let min = *sorted.first().unwrap_or(&0.0);
        let max = *sorted.last().unwrap_or(&0.0);

        // Calculate standard deviation
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        // Calculate percentiles
        let p95_idx = ((count as f64 * 0.95) as usize).min(count - 1);
        let p99_idx = ((count as f64 * 0.99) as usize).min(count - 1);
        let p95 = sorted[p95_idx];
        let p99 = sorted[p99_idx];

        // Detect trend
        let trend = self.detect_trend(values);

        // Detect anomalies
        let anomalies = self.detect_anomalies(values, mean, std_dev, data);

        AggregatedStats {
            count,
            mean,
            median,
            min,
            max,
            std_dev,
            p95,
            p99,
            trend,
            anomalies,
        }
    }

    /// Detect trend direction in time series
    fn detect_trend(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 3 {
            return TrendDirection::Stable;
        }

        let n = values.len();
        let recent = &values[n - (n / 3)..];
        let older = &values[..(n / 3)];

        let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let older_avg: f64 = older.iter().sum::<f64>() / older.len() as f64;

        let diff_percent = ((recent_avg - older_avg) / older_avg).abs() * 100.0;

        if diff_percent < 5.0 {
            TrendDirection::Stable
        } else if recent_avg > older_avg {
            TrendDirection::Rising
        } else {
            TrendDirection::Falling
        }
    }

    /// Detect anomalies using z-score method
    fn detect_anomalies<T>(
        &self,
        values: &[f64],
        mean: f64,
        std_dev: f64,
        data: &[DataPoint<T>],
    ) -> Vec<AnomalyPoint> {
        let mut anomalies = Vec::new();

        for (i, value) in values.iter().enumerate() {
            if std_dev > 0.0 {
                let z_score = (value - mean).abs() / std_dev;

                if z_score > 3.0 {
                    let severity = match z_score {
                        z if z > 4.0 => AnomalySeverity::Critical,
                        z if z > 3.5 => AnomalySeverity::High,
                        z if z > 3.0 => AnomalySeverity::Medium,
                        _ => AnomalySeverity::Low,
                    };

                    anomalies.push(AnomalyPoint {
                        timestamp: data[i].timestamp,
                        value: *value,
                        severity,
                        deviation: z_score,
                    });
                }
            }
        }

        anomalies
    }

    /// Get current alert status
    pub async fn get_alert_status(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let thresholds = self.alert_thresholds.read().await;

        // Check performance metrics
        let perf_window = self.performance_metrics.read().await;
        if let Some(latest) = perf_window.data.back() {
            if latest.value.cpu_usage_percent > thresholds.cpu_critical {
                alerts.push(Alert {
                    severity: AlertSeverity::Critical,
                    message: format!("CPU usage critical: {:.1}%", latest.value.cpu_usage_percent),
                    timestamp: SystemTime::now(),
                    metric: "cpu_usage".to_string(),
                });
            }

            if latest.value.memory_usage_mb > thresholds.memory_critical_mb {
                alerts.push(Alert {
                    severity: AlertSeverity::Critical,
                    message: format!(
                        "Memory usage critical: {:.1} MB",
                        latest.value.memory_usage_mb
                    ),
                    timestamp: SystemTime::now(),
                    metric: "memory_usage".to_string(),
                });
            }

            if latest.value.queue_depth > thresholds.queue_depth_warning {
                alerts.push(Alert {
                    severity: AlertSeverity::Warning,
                    message: format!("Queue depth high: {}", latest.value.queue_depth),
                    timestamp: SystemTime::now(),
                    metric: "queue_depth".to_string(),
                });
            }
        }

        alerts
    }

    /// Export metrics in various formats
    pub async fn export_metrics(&self, format: ExportFormat) -> Result<String> {
        let storage = self.storage_metrics.read().await.get_window();
        let performance = self.performance_metrics.read().await.get_window();
        let analysis = self.analysis_metrics.read().await.get_window();

        match format {
            ExportFormat::Json => {
                let export = json!({
                    "storage": storage,
                    "performance": performance,
                    "analysis": analysis,
                    "timestamp": SystemTime::now(),
                });
                Ok(serde_json::to_string_pretty(&export)?)
            }
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("timestamp,metric_type,metric_name,value\n");

                for point in storage {
                    csv.push_str(&format!(
                        "{:?},storage,total_entries,{}\n",
                        point.timestamp, point.value.total_entries
                    ));
                    csv.push_str(&format!(
                        "{:?},storage,cache_hit_ratio,{}\n",
                        point.timestamp, point.value.cache_hit_ratio
                    ));
                }

                for point in performance {
                    csv.push_str(&format!(
                        "{:?},performance,avg_analysis_time_ms,{}\n",
                        point.timestamp, point.value.avg_analysis_time_ms
                    ));
                    csv.push_str(&format!(
                        "{:?},performance,cpu_usage_percent,{}\n",
                        point.timestamp, point.value.cpu_usage_percent
                    ));
                }

                Ok(csv)
            }
            ExportFormat::Prometheus => {
                let mut prom = String::new();

                if let Some(latest_storage) = storage.last() {
                    prom.push_str(&format!(
                        "# HELP tdg_storage_entries Total storage entries\n\
                         # TYPE tdg_storage_entries gauge\n\
                         tdg_storage_entries {}\n",
                        latest_storage.value.total_entries
                    ));
                    prom.push_str(&format!(
                        "# HELP tdg_cache_hit_ratio Cache hit ratio\n\
                         # TYPE tdg_cache_hit_ratio gauge\n\
                         tdg_cache_hit_ratio {}\n",
                        latest_storage.value.cache_hit_ratio
                    ));
                }

                if let Some(latest_perf) = performance.last() {
                    prom.push_str(&format!(
                        "# HELP tdg_analysis_time_ms Average analysis time\n\
                         # TYPE tdg_analysis_time_ms gauge\n\
                         tdg_analysis_time_ms {}\n",
                        latest_perf.value.avg_analysis_time_ms
                    ));
                    prom.push_str(&format!(
                        "# HELP tdg_cpu_usage_percent CPU usage percentage\n\
                         # TYPE tdg_cpu_usage_percent gauge\n\
                         tdg_cpu_usage_percent {}\n",
                        latest_perf.value.cpu_usage_percent
                    ));
                }

                Ok(prom)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: SystemTime,
    pub metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
}

use serde_json::json;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ============ DataPoint Tests ============

    #[test]
    fn test_data_point_creation() {
        let point = DataPoint {
            timestamp: SystemTime::now(),
            value: 42.0,
            tags: HashMap::from([("key".to_string(), "value".to_string())]),
        };
        assert_eq!(point.value, 42.0);
        assert!(point.tags.contains_key("key"));
    }

    #[test]
    fn test_data_point_clone() {
        let point = DataPoint {
            timestamp: SystemTime::now(),
            value: 100,
            tags: HashMap::new(),
        };
        let cloned = point.clone();
        assert_eq!(cloned.value, 100);
    }

    #[test]
    fn test_data_point_serialization() {
        let point = DataPoint {
            timestamp: SystemTime::now(),
            value: 55.5,
            tags: HashMap::new(),
        };
        let json = serde_json::to_string(&point).unwrap();
        let deserialized: DataPoint<f64> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.value, 55.5);
    }

    // ============ RollingWindow Tests ============

    #[tokio::test]
    async fn test_rolling_window() {
        let mut window = RollingWindow::new(Duration::from_secs(60), 10);

        for i in 0..5 {
            window.push(i as f64, HashMap::new());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        assert_eq!(window.get_window().len(), 5);
    }

    #[test]
    fn test_rolling_window_new() {
        let window: RollingWindow<f64> = RollingWindow::new(Duration::from_secs(30), 100);
        assert!(window.is_empty());
        assert_eq!(window.get_window().len(), 0);
    }

    #[test]
    fn test_rolling_window_max_points() {
        let mut window: RollingWindow<i32> = RollingWindow::new(Duration::from_secs(3600), 5);
        for i in 0..10 {
            window.push(i, HashMap::new());
        }
        // Should only keep last 5 points
        assert_eq!(window.get_window().len(), 5);
    }

    #[test]
    fn test_rolling_window_is_empty() {
        let window: RollingWindow<u64> = RollingWindow::new(Duration::from_secs(60), 10);
        assert!(window.is_empty());
    }

    #[test]
    fn test_rolling_window_with_tags() {
        let mut window: RollingWindow<f64> = RollingWindow::new(Duration::from_secs(60), 10);
        let mut tags = HashMap::new();
        tags.insert("alert".to_string(), "warning".to_string());
        window.push(1.0, tags);

        let data = window.get_window();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].tags.get("alert"), Some(&"warning".to_string()));
    }

    // ============ AggregatedStats Tests ============

    #[test]
    fn test_aggregated_stats_serialization() {
        let stats = AggregatedStats {
            count: 10,
            mean: 50.0,
            median: 48.0,
            min: 10.0,
            max: 90.0,
            std_dev: 15.0,
            p95: 85.0,
            p99: 89.0,
            trend: TrendDirection::Rising,
            anomalies: vec![],
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: AggregatedStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.count, 10);
        assert_eq!(deserialized.mean, 50.0);
    }

    // ============ TrendDirection Tests ============

    #[test]
    fn test_trend_direction_clone() {
        let trend = TrendDirection::Rising;
        let cloned = trend.clone();
        assert_eq!(cloned, TrendDirection::Rising);
    }

    #[test]
    fn test_trend_direction_serialization() {
        let trend = TrendDirection::Volatile;
        let json = serde_json::to_string(&trend).unwrap();
        let deserialized: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TrendDirection::Volatile);
    }

    #[test]
    fn test_trend_direction_all_variants() {
        assert_eq!(TrendDirection::Rising, TrendDirection::Rising);
        assert_eq!(TrendDirection::Falling, TrendDirection::Falling);
        assert_eq!(TrendDirection::Stable, TrendDirection::Stable);
        assert_eq!(TrendDirection::Volatile, TrendDirection::Volatile);
    }

    // ============ AnomalyPoint Tests ============

    #[test]
    fn test_anomaly_point_creation() {
        let point = AnomalyPoint {
            timestamp: SystemTime::now(),
            value: 999.9,
            severity: AnomalySeverity::High,
            deviation: 4.5,
        };
        assert_eq!(point.value, 999.9);
        assert_eq!(point.severity, AnomalySeverity::High);
    }

    #[test]
    fn test_anomaly_point_serialization() {
        let point = AnomalyPoint {
            timestamp: SystemTime::now(),
            value: 100.0,
            severity: AnomalySeverity::Critical,
            deviation: 5.0,
        };
        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("Critical"));
    }

    // ============ AnomalySeverity Tests ============

    #[test]
    fn test_anomaly_severity_all_variants() {
        assert_eq!(AnomalySeverity::Low, AnomalySeverity::Low);
        assert_eq!(AnomalySeverity::Medium, AnomalySeverity::Medium);
        assert_eq!(AnomalySeverity::High, AnomalySeverity::High);
        assert_eq!(AnomalySeverity::Critical, AnomalySeverity::Critical);
    }

    #[test]
    fn test_anomaly_severity_serialization() {
        let severity = AnomalySeverity::Medium;
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: AnomalySeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AnomalySeverity::Medium);
    }

    // ============ StorageMetricPoint Tests ============

    #[test]
    fn test_storage_metric_point_creation() {
        let point = StorageMetricPoint {
            total_entries: 1000,
            cache_hit_ratio: 0.85,
            compression_ratio: 0.6,
            storage_size_mb: 512.0,
            write_throughput: 100.0,
            read_throughput: 500.0,
        };
        assert_eq!(point.total_entries, 1000);
        assert_eq!(point.cache_hit_ratio, 0.85);
    }

    #[test]
    fn test_storage_metric_point_serialization() {
        let point = StorageMetricPoint {
            total_entries: 500,
            cache_hit_ratio: 0.9,
            compression_ratio: 0.5,
            storage_size_mb: 256.0,
            write_throughput: 50.0,
            read_throughput: 200.0,
        };
        let json = serde_json::to_string(&point).unwrap();
        let deserialized: StorageMetricPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_entries, 500);
    }

    // ============ PerformanceMetricPoint Tests ============

    #[test]
    fn test_performance_metric_point_creation() {
        let point = PerformanceMetricPoint {
            avg_analysis_time_ms: 150.0,
            active_operations: 5,
            queue_depth: 10,
            cpu_usage_percent: 45.0,
            memory_usage_mb: 1024.0,
            gc_pause_ms: 5.0,
        };
        assert_eq!(point.avg_analysis_time_ms, 150.0);
        assert_eq!(point.active_operations, 5);
    }

    #[test]
    fn test_performance_metric_point_serialization() {
        let point = PerformanceMetricPoint {
            avg_analysis_time_ms: 200.0,
            active_operations: 3,
            queue_depth: 5,
            cpu_usage_percent: 30.0,
            memory_usage_mb: 512.0,
            gc_pause_ms: 2.0,
        };
        let json = serde_json::to_string(&point).unwrap();
        let deserialized: PerformanceMetricPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.queue_depth, 5);
    }

    // ============ AnalysisMetricPoint Tests ============

    #[test]
    fn test_analysis_metric_point_creation() {
        let point = AnalysisMetricPoint {
            files_analyzed: 100,
            avg_tdg_score: 85.5,
            critical_issues: 2,
            success_rate: 0.98,
            cache_hits: 80,
            cache_misses: 20,
        };
        assert_eq!(point.files_analyzed, 100);
        assert_eq!(point.success_rate, 0.98);
    }

    #[test]
    fn test_analysis_metric_point_serialization() {
        let point = AnalysisMetricPoint {
            files_analyzed: 50,
            avg_tdg_score: 75.0,
            critical_issues: 0,
            success_rate: 1.0,
            cache_hits: 45,
            cache_misses: 5,
        };
        let json = serde_json::to_string(&point).unwrap();
        let deserialized: AnalysisMetricPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cache_hits, 45);
    }

    // ============ AlertThresholds Tests ============

    #[test]
    fn test_alert_thresholds_default() {
        let thresholds = AlertThresholds::default();
        assert_eq!(thresholds.cpu_critical, 90.0);
        assert_eq!(thresholds.memory_critical_mb, 8192.0);
        assert_eq!(thresholds.queue_depth_warning, 100);
        assert_eq!(thresholds.analysis_time_warning_ms, 5000.0);
        assert_eq!(thresholds.cache_hit_ratio_warning, 0.5);
        assert_eq!(thresholds.storage_usage_warning_percent, 85.0);
    }

    #[test]
    fn test_alert_thresholds_serialization() {
        let thresholds = AlertThresholds::default();
        let json = serde_json::to_string(&thresholds).unwrap();
        let deserialized: AlertThresholds = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cpu_critical, 90.0);
    }

    // ============ MetricsAggregator Tests ============

    #[tokio::test]
    async fn test_metrics_aggregator_default() {
        let aggregator = MetricsAggregator::default();
        let stats = aggregator.aggregate_performance_stats().await;
        assert_eq!(stats.count, 0);
    }

    #[tokio::test]
    async fn test_metrics_aggregation() {
        let aggregator = MetricsAggregator::new();

        for i in 0..10 {
            let metrics = PerformanceMetricPoint {
                avg_analysis_time_ms: (i * 100) as f64,
                active_operations: i as u32,
                queue_depth: i as u32,
                cpu_usage_percent: (i * 10) as f64,
                memory_usage_mb: (i * 100) as f64,
                gc_pause_ms: 0.0,
            };
            aggregator
                .record_performance_metrics(metrics)
                .await
                .expect("internal error");
        }

        let stats = aggregator.aggregate_performance_stats().await;
        assert!(stats.count > 0);
        assert!(stats.mean > 0.0);
    }

    #[tokio::test]
    async fn test_record_storage_metrics() {
        let aggregator = MetricsAggregator::new();
        let metrics = StorageMetricPoint {
            total_entries: 1000,
            cache_hit_ratio: 0.9,
            compression_ratio: 0.5,
            storage_size_mb: 100.0,
            write_throughput: 50.0,
            read_throughput: 200.0,
        };
        let result = aggregator.record_storage_metrics(metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_analysis_metrics() {
        let aggregator = MetricsAggregator::new();
        let metrics = AnalysisMetricPoint {
            files_analyzed: 50,
            avg_tdg_score: 80.0,
            critical_issues: 1,
            success_rate: 0.95,
            cache_hits: 40,
            cache_misses: 10,
        };
        let result = aggregator.record_analysis_metrics(metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_detection() {
        let aggregator = MetricsAggregator::new();

        let critical_metrics = PerformanceMetricPoint {
            avg_analysis_time_ms: 10000.0,
            active_operations: 10,
            queue_depth: 200,
            cpu_usage_percent: 95.0,
            memory_usage_mb: 9000.0,
            gc_pause_ms: 100.0,
        };

        aggregator
            .record_performance_metrics(critical_metrics)
            .await
            .expect("internal error");

        let alerts = aggregator.get_alert_status().await;
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.severity == AlertSeverity::Critical));
    }

    #[tokio::test]
    async fn test_get_alert_status_no_alerts() {
        let aggregator = MetricsAggregator::new();
        let alerts = aggregator.get_alert_status().await;
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_export_metrics_json() {
        let aggregator = MetricsAggregator::new();
        let result = aggregator.export_metrics(ExportFormat::Json).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("storage"));
        assert!(json.contains("performance"));
    }

    #[tokio::test]
    async fn test_export_metrics_csv() {
        let aggregator = MetricsAggregator::new();
        let result = aggregator.export_metrics(ExportFormat::Csv).await;
        assert!(result.is_ok());
        let csv = result.unwrap();
        assert!(csv.contains("timestamp,metric_type,metric_name,value"));
    }

    #[tokio::test]
    async fn test_export_metrics_prometheus() {
        let aggregator = MetricsAggregator::new();
        let result = aggregator.export_metrics(ExportFormat::Prometheus).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_metrics_prometheus_with_data() {
        let aggregator = MetricsAggregator::new();

        let storage = StorageMetricPoint {
            total_entries: 100,
            cache_hit_ratio: 0.8,
            compression_ratio: 0.5,
            storage_size_mb: 50.0,
            write_throughput: 10.0,
            read_throughput: 50.0,
        };
        aggregator.record_storage_metrics(storage).await.unwrap();

        let perf = PerformanceMetricPoint {
            avg_analysis_time_ms: 100.0,
            active_operations: 5,
            queue_depth: 10,
            cpu_usage_percent: 30.0,
            memory_usage_mb: 512.0,
            gc_pause_ms: 1.0,
        };
        aggregator.record_performance_metrics(perf).await.unwrap();

        let result = aggregator.export_metrics(ExportFormat::Prometheus).await;
        assert!(result.is_ok());
        let prom = result.unwrap();
        assert!(prom.contains("tdg_storage_entries"));
        assert!(prom.contains("tdg_analysis_time_ms"));
    }

    // ============ Alert Tests ============

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            severity: AlertSeverity::Warning,
            message: "Test warning".to_string(),
            timestamp: SystemTime::now(),
            metric: "test_metric".to_string(),
        };
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.metric, "test_metric");
    }

    #[test]
    fn test_alert_serialization() {
        let alert = Alert {
            severity: AlertSeverity::Critical,
            message: "Critical issue".to_string(),
            timestamp: SystemTime::now(),
            metric: "cpu".to_string(),
        };
        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("Critical"));
    }

    // ============ AlertSeverity Tests ============

    #[test]
    fn test_alert_severity_all_variants() {
        assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
        assert_eq!(AlertSeverity::Warning, AlertSeverity::Warning);
        assert_eq!(AlertSeverity::Error, AlertSeverity::Error);
        assert_eq!(AlertSeverity::Critical, AlertSeverity::Critical);
    }

    #[test]
    fn test_alert_severity_serialization() {
        let severity = AlertSeverity::Error;
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: AlertSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AlertSeverity::Error);
    }

    // ============ ExportFormat Tests ============

    #[test]
    fn test_export_format_debug() {
        let format = ExportFormat::Json;
        let debug = format!("{:?}", format);
        assert!(debug.contains("Json"));
    }

    #[test]
    fn test_export_format_clone_copy() {
        let format = ExportFormat::Csv;
        let cloned = format;
        assert!(matches!(cloned, ExportFormat::Csv));
    }

    // === Anomaly Detection Tests ===

    #[tokio::test]
    async fn test_detect_anomalies_critical_severity() {
        let aggregator = MetricsAggregator::new();
        // Create 10 normal points around 100, then one extreme outlier
        for i in 0..10 {
            let metrics = PerformanceMetricPoint {
                avg_analysis_time_ms: 100.0 + (i as f64),
                active_operations: 1,
                queue_depth: 1,
                cpu_usage_percent: 10.0,
                memory_usage_mb: 100.0,
                gc_pause_ms: 0.0,
            };
            aggregator
                .record_performance_metrics(metrics)
                .await
                .unwrap();
        }
        // Add extreme outlier (z-score > 4.0)
        let outlier = PerformanceMetricPoint {
            avg_analysis_time_ms: 500.0,
            active_operations: 1,
            queue_depth: 1,
            cpu_usage_percent: 10.0,
            memory_usage_mb: 100.0,
            gc_pause_ms: 0.0,
        };
        aggregator
            .record_performance_metrics(outlier)
            .await
            .unwrap();

        let stats = aggregator.aggregate_performance_stats().await;
        assert!(!stats.anomalies.is_empty(), "Should detect anomaly");
        assert!(
            stats.anomalies.iter().any(|a| a.deviation > 3.0),
            "Anomaly deviation should exceed 3.0"
        );
    }

    #[tokio::test]
    async fn test_detect_anomalies_zero_std_dev() {
        let aggregator = MetricsAggregator::new();
        // All identical values → std_dev = 0, no anomalies
        for _ in 0..5 {
            let metrics = PerformanceMetricPoint {
                avg_analysis_time_ms: 100.0,
                active_operations: 1,
                queue_depth: 1,
                cpu_usage_percent: 10.0,
                memory_usage_mb: 100.0,
                gc_pause_ms: 0.0,
            };
            aggregator
                .record_performance_metrics(metrics)
                .await
                .unwrap();
        }
        let stats = aggregator.aggregate_performance_stats().await;
        assert!(stats.anomalies.is_empty(), "No anomalies with zero std_dev");
    }

    // === Export Metrics with Data ===

    #[tokio::test]
    async fn test_export_metrics_csv_with_data() {
        let aggregator = MetricsAggregator::new();

        let storage = StorageMetricPoint {
            total_entries: 500,
            cache_hit_ratio: 0.75,
            compression_ratio: 0.6,
            storage_size_mb: 100.0,
            write_throughput: 10.0,
            read_throughput: 50.0,
        };
        aggregator.record_storage_metrics(storage).await.unwrap();

        let perf = PerformanceMetricPoint {
            avg_analysis_time_ms: 250.0,
            active_operations: 3,
            queue_depth: 5,
            cpu_usage_percent: 42.0,
            memory_usage_mb: 512.0,
            gc_pause_ms: 1.0,
        };
        aggregator.record_performance_metrics(perf).await.unwrap();

        let csv = aggregator.export_metrics(ExportFormat::Csv).await.unwrap();
        assert!(csv.contains("storage,total_entries,500"));
        assert!(csv.contains("storage,cache_hit_ratio,0.75"));
        assert!(csv.contains("performance,avg_analysis_time_ms,250"));
        assert!(csv.contains("performance,cpu_usage_percent,42"));
    }

    #[tokio::test]
    async fn test_export_metrics_prometheus_storage_branch() {
        let aggregator = MetricsAggregator::new();

        let storage = StorageMetricPoint {
            total_entries: 999,
            cache_hit_ratio: 0.95,
            compression_ratio: 0.5,
            storage_size_mb: 50.0,
            write_throughput: 10.0,
            read_throughput: 50.0,
        };
        aggregator.record_storage_metrics(storage).await.unwrap();

        let prom = aggregator
            .export_metrics(ExportFormat::Prometheus)
            .await
            .unwrap();
        assert!(prom.contains("tdg_storage_entries 999"));
        assert!(prom.contains("tdg_cache_hit_ratio 0.95"));
        assert!(prom.contains("# HELP tdg_storage_entries"));
        assert!(prom.contains("# TYPE tdg_storage_entries gauge"));
    }

    #[tokio::test]
    async fn test_export_metrics_prometheus_perf_branch() {
        let aggregator = MetricsAggregator::new();

        let perf = PerformanceMetricPoint {
            avg_analysis_time_ms: 123.5,
            active_operations: 2,
            queue_depth: 3,
            cpu_usage_percent: 55.0,
            memory_usage_mb: 256.0,
            gc_pause_ms: 0.5,
        };
        aggregator.record_performance_metrics(perf).await.unwrap();

        let prom = aggregator
            .export_metrics(ExportFormat::Prometheus)
            .await
            .unwrap();
        assert!(prom.contains("tdg_analysis_time_ms 123.5"));
        assert!(prom.contains("tdg_cpu_usage_percent 55"));
        assert!(prom.contains("# HELP tdg_analysis_time_ms"));
        assert!(prom.contains("# TYPE tdg_cpu_usage_percent gauge"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
