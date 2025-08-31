//! Sprint 31 Week 2 - Advanced Metrics Aggregation and Trending
//! 
//! Provides sophisticated metrics aggregation, time-series analysis, and trending
//! capabilities for the TDG system. Supports rolling windows, percentile calculations,
//! and anomaly detection.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;

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

    pub fn get_window(&self) -> Vec<DataPoint<T>> {
        self.data.iter().cloned().collect()
    }

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

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            storage_metrics: Arc::new(RwLock::new(RollingWindow::new(
                Duration::from_secs(3600), // 1 hour window
                360, // Max 360 points (10 second intervals)
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

        let values: Vec<f64> = data.iter()
            .map(|p| p.value.avg_analysis_time_ms)
            .collect();

        self.calculate_stats(&values, &data)
    }

    /// Calculate statistics from raw values
    fn calculate_stats<T>(&self, values: &[f64], data: &[DataPoint<T>]) -> AggregatedStats {
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        // Calculate median
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };

        let min = *sorted.first().unwrap_or(&0.0);
        let max = *sorted.last().unwrap_or(&0.0);

        // Calculate standard deviation
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count as f64;
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
                    message: format!("Memory usage critical: {:.1} MB", latest.value.memory_usage_mb),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rolling_window() {
        let mut window = RollingWindow::new(Duration::from_secs(60), 10);
        
        for i in 0..5 {
            window.push(i as f64, HashMap::new());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        assert_eq!(window.get_window().len(), 5);
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
            aggregator.record_performance_metrics(metrics).await.unwrap();
        }
        
        let stats = aggregator.aggregate_performance_stats().await;
        assert!(stats.count > 0);
        assert!(stats.mean > 0.0);
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
        
        aggregator.record_performance_metrics(critical_metrics).await.unwrap();
        
        let alerts = aggregator.get_alert_status().await;
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.severity == AlertSeverity::Critical));
    }
}