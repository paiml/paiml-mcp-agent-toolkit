//! Prometheus metrics exporter for unified quality system
//!
//! Exports quality metrics in Prometheus format for monitoring and alerting

use crate::unified_quality::events::QualityEvent;
use crate::unified_quality::foundation::QualityMonitor;
use crate::unified_quality::metrics::Metrics;
use anyhow::Result;
use prometheus::{
    Encoder, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry, TextEncoder,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use warp::{Filter, Reply};

/// Prometheus metrics registry for quality tracking
#[allow(dead_code)]
pub struct QualityMetricsRegistry {
    /// Registry for all metrics
    registry: Registry,

    /// File complexity metrics
    complexity_gauge: Gauge,

    /// Cognitive complexity metrics  
    cognitive_gauge: Gauge,

    /// SATD comment counter
    satd_counter: IntCounter,

    /// Test coverage gauge
    coverage_gauge: Gauge,

    /// Lines of code gauge
    lines_gauge: IntGauge,

    /// Function count gauge
    functions_gauge: IntGauge,

    /// Quality events counter
    events_counter: IntCounter,

    /// File analysis histogram
    analysis_duration: Histogram,

    /// Quality violations counter by type
    violations_counter: IntCounter,

    /// Error budget gauge
    error_budget_gauge: Gauge,

    /// Refactoring suggestions counter
    suggestions_counter: IntCounter,

    /// Files analyzed total
    files_analyzed_counter: IntCounter,
}

impl QualityMetricsRegistry {
    /// Create new metrics registry
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // Complexity metrics
        let complexity_gauge = Gauge::with_opts(
            Opts::new("pmat_complexity", "Current complexity score").namespace("quality"),
        )?;
        registry.register(Box::new(complexity_gauge.clone()))?;

        let cognitive_gauge = Gauge::with_opts(
            Opts::new("pmat_cognitive_complexity", "Current cognitive complexity")
                .namespace("quality"),
        )?;
        registry.register(Box::new(cognitive_gauge.clone()))?;

        // SATD tracking
        let satd_counter = IntCounter::with_opts(
            Opts::new("pmat_satd_comments_total", "Total SATD comments found").namespace("quality"),
        )?;
        registry.register(Box::new(satd_counter.clone()))?;

        // Coverage metrics
        let coverage_gauge = Gauge::with_opts(
            Opts::new("pmat_test_coverage", "Test coverage percentage").namespace("quality"),
        )?;
        registry.register(Box::new(coverage_gauge.clone()))?;

        // Code size metrics
        let lines_gauge = IntGauge::with_opts(
            Opts::new("pmat_lines_of_code", "Lines of code").namespace("quality"),
        )?;
        registry.register(Box::new(lines_gauge.clone()))?;

        let functions_gauge = IntGauge::with_opts(
            Opts::new("pmat_functions_count", "Number of functions").namespace("quality"),
        )?;
        registry.register(Box::new(functions_gauge.clone()))?;

        // Event tracking
        let events_counter = IntCounter::with_opts(
            Opts::new("pmat_quality_events_total", "Quality events processed").namespace("quality"),
        )?;
        registry.register(Box::new(events_counter.clone()))?;

        // Performance metrics
        let analysis_duration = Histogram::with_opts(
            HistogramOpts::new("pmat_analysis_duration_seconds", "File analysis duration")
                .namespace("quality")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        )?;
        registry.register(Box::new(analysis_duration.clone()))?;

        // Violation tracking
        let violations_counter = IntCounter::with_opts(
            Opts::new("pmat_violations_total", "Quality violations found").namespace("quality"),
        )?;
        registry.register(Box::new(violations_counter.clone()))?;

        // Error budget
        let error_budget_gauge = Gauge::with_opts(
            Opts::new("pmat_error_budget", "Remaining error budget").namespace("quality"),
        )?;
        registry.register(Box::new(error_budget_gauge.clone()))?;

        // Refactoring metrics
        let suggestions_counter = IntCounter::with_opts(
            Opts::new(
                "pmat_refactoring_suggestions_total",
                "Refactoring suggestions",
            )
            .namespace("quality"),
        )?;
        registry.register(Box::new(suggestions_counter.clone()))?;

        // Analysis volume
        let files_analyzed_counter = IntCounter::with_opts(
            Opts::new("pmat_files_analyzed_total", "Files analyzed").namespace("quality"),
        )?;
        registry.register(Box::new(files_analyzed_counter.clone()))?;

        Ok(Self {
            registry,
            complexity_gauge,
            cognitive_gauge,
            satd_counter,
            coverage_gauge,
            lines_gauge,
            functions_gauge,
            events_counter,
            analysis_duration,
            violations_counter,
            error_budget_gauge,
            suggestions_counter,
            files_analyzed_counter,
        })
    }

    /// Record file metrics
    pub fn record_file_metrics(&self, _path: &PathBuf, metrics: &Metrics) {
        self.complexity_gauge.set(f64::from(metrics.complexity));
        self.cognitive_gauge.set(f64::from(metrics.cognitive));
        self.satd_counter.inc_by(u64::from(metrics.satd_count));
        self.coverage_gauge.set(metrics.coverage);
        self.lines_gauge.set(i64::from(metrics.lines));
        self.functions_gauge.set(i64::from(metrics.functions));
        self.files_analyzed_counter.inc();
    }

    /// Record quality event
    pub fn record_quality_event(&self, event: &QualityEvent) {
        self.events_counter.inc();

        match event {
            QualityEvent::FileAdded { metrics, .. } => {
                self.record_file_metrics(&PathBuf::new(), metrics);
            }
            QualityEvent::MetricsUpdated { new_metrics, .. } => {
                self.record_file_metrics(&PathBuf::new(), new_metrics);
            }
            QualityEvent::FileRemoved { .. } => {
                // Could decrement counters if we track removals
            }
            QualityEvent::ThresholdViolated { .. } => {
                self.violations_counter.inc();
            }
            QualityEvent::QualityImproved { .. } => {
                // Quality improved event
            }
            QualityEvent::QualityDegraded { .. } => {
                // Quality degraded event
            }
            QualityEvent::BatchAnalysisComplete { .. } => {
                // Batch analysis complete event
            }
        }
    }

    /// Record analysis duration
    pub fn record_analysis_duration(&self, duration: Duration) {
        self.analysis_duration.observe(duration.as_secs_f64());
    }

    /// Update error budget
    pub fn update_error_budget(&self, remaining_percentage: f64) {
        self.error_budget_gauge.set(remaining_percentage);
    }

    /// Export metrics in Prometheus text format
    pub fn export_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get metrics registry for custom metrics
    #[must_use] 
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// Prometheus exporter configuration
#[derive(Debug, Clone)]
pub struct PrometheusConfig {
    /// Port to serve metrics on
    pub port: u16,

    /// Metrics update interval
    pub update_interval: Duration,

    /// Enable detailed per-file metrics
    pub detailed_metrics: bool,

    /// Metric retention period
    pub retention_period: Duration,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            port: 9090,
            update_interval: Duration::from_secs(15),
            detailed_metrics: false,
            retention_period: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }
}

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    /// Metrics registry
    metrics: Arc<QualityMetricsRegistry>,

    /// Configuration
    config: PrometheusConfig,

    /// Quality monitor reference
    monitor: Arc<QualityMonitor>,
}

impl PrometheusExporter {
    /// Create new Prometheus exporter
    pub fn new(monitor: Arc<QualityMonitor>, config: PrometheusConfig) -> Result<Self> {
        let metrics = Arc::new(QualityMetricsRegistry::new()?);

        Ok(Self {
            metrics,
            config,
            monitor,
        })
    }

    /// Start Prometheus metrics server
    pub async fn start(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        let monitor = self.monitor.clone();
        let update_interval = self.config.update_interval;

        // Start metrics collection task
        let collection_metrics = metrics.clone();
        let collection_monitor = monitor.clone();
        tokio::spawn(async move {
            let mut interval = interval(update_interval);

            loop {
                interval.tick().await;
                Self::collect_metrics(&collection_metrics, &collection_monitor).await;
            }
        });

        // Start HTTP server for metrics endpoint
        let metrics_route = warp::path("metrics").map(move || {
            let metrics_clone = metrics.clone();
            match metrics_clone.export_metrics() {
                Ok(metrics_text) => warp::reply::with_header(
                    metrics_text,
                    "content-type",
                    "text/plain; version=0.0.4",
                )
                .into_response(),
                Err(e) => warp::reply::with_status(
                    format!("Error exporting metrics: {e}"),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response(),
            }
        });

        let health_route =
            warp::path("health").map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

        let routes = metrics_route.or(health_route);

        println!(
            "Starting Prometheus metrics server on port {}",
            self.config.port
        );
        warp::serve(routes)
            .run(([0, 0, 0, 0], self.config.port))
            .await;

        Ok(())
    }

    /// Collect current metrics from quality monitor
    async fn collect_metrics(metrics: &QualityMetricsRegistry, monitor: &QualityMonitor) {
        let all_metrics = monitor.get_all_metrics();

        // Aggregate metrics across all files
        let mut total_complexity = 0.0;
        let mut total_cognitive = 0.0;
        let mut _total_satd = 0;
        let mut total_coverage = 0.0;
        let mut total_lines = 0;
        let mut total_functions = 0;

        for (path, file_metrics) in &all_metrics {
            total_complexity += f64::from(file_metrics.complexity);
            total_cognitive += f64::from(file_metrics.cognitive);
            _total_satd += file_metrics.satd_count;
            total_coverage += file_metrics.coverage;
            total_lines += file_metrics.lines;
            total_functions += file_metrics.functions;

            // Record individual file metrics
            metrics.record_file_metrics(path, file_metrics);
        }

        let file_count = all_metrics.len() as f64;
        if file_count > 0.0 {
            // Update aggregate metrics
            metrics.complexity_gauge.set(total_complexity / file_count);
            metrics.cognitive_gauge.set(total_cognitive / file_count);
            metrics.coverage_gauge.set(total_coverage / file_count);
            metrics.lines_gauge.set(i64::from(total_lines));
            metrics.functions_gauge.set(i64::from(total_functions));
        }

        // Calculate and update error budget (example: based on complexity)
        let avg_complexity = if file_count > 0.0 {
            total_complexity / file_count
        } else {
            0.0
        };
        let complexity_threshold = 20.0; // From quality standards
        let error_budget =
            ((complexity_threshold - avg_complexity) / complexity_threshold * 100.0).max(0.0);
        metrics.update_error_budget(error_budget);
    }

    /// Get metrics registry for custom metrics
    #[must_use] 
    pub fn metrics(&self) -> &Arc<QualityMetricsRegistry> {
        &self.metrics
    }

    /// Update configuration
    pub fn update_config(&mut self, config: PrometheusConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_quality::foundation::MonitorConfig;

    #[test]
    fn test_metrics_registry_creation() {
        let registry = QualityMetricsRegistry::new().unwrap();
        let metrics = registry.export_metrics().unwrap();
        assert!(metrics.contains("quality_pmat_complexity"));
        assert!(metrics.contains("quality_pmat_satd_comments_total"));
    }

    #[test]
    fn test_record_file_metrics() {
        let registry = QualityMetricsRegistry::new().unwrap();
        let metrics = Metrics {
            complexity: 15,
            cognitive: 12,
            satd_count: 3,
            coverage: 0.85,
            lines: 250,
            functions: 8,
            timestamp: std::time::SystemTime::now(),
        };

        registry.record_file_metrics(&PathBuf::from("test.rs"), &metrics);

        let exported = registry.export_metrics().unwrap();
        assert!(exported.contains("quality_pmat_complexity 15"));
        assert!(exported.contains("quality_pmat_cognitive_complexity 12"));
    }

    #[tokio::test]
    async fn test_prometheus_exporter_creation() {
        let config = MonitorConfig::default();
        let monitor = Arc::new(QualityMonitor::new(config).unwrap());
        let prometheus_config = PrometheusConfig::default();

        let exporter = PrometheusExporter::new(monitor, prometheus_config).unwrap();
        assert_eq!(exporter.config.port, 9090);
    }

    #[test]
    fn test_prometheus_config_default() {
        let config = PrometheusConfig::default();
        assert_eq!(config.port, 9090);
        assert_eq!(config.update_interval, Duration::from_secs(15));
        assert!(!config.detailed_metrics);
    }

    #[test]
    fn test_error_budget_calculation() {
        let registry = QualityMetricsRegistry::new().unwrap();

        // Good error budget (low complexity)
        registry.update_error_budget(80.0);
        let exported = registry.export_metrics().unwrap();
        assert!(exported.contains("quality_pmat_error_budget 80"));

        // Bad error budget (high complexity)
        registry.update_error_budget(20.0);
        let exported = registry.export_metrics().unwrap();
        assert!(exported.contains("quality_pmat_error_budget 20"));
    }

    #[test]
    fn test_quality_event_recording() {
        let registry = QualityMetricsRegistry::new().unwrap();
        let metrics = Metrics {
            complexity: 10,
            cognitive: 8,
            satd_count: 1,
            coverage: 0.9,
            lines: 100,
            functions: 5,
            timestamp: std::time::SystemTime::now(),
        };

        let event = QualityEvent::FileAdded {
            path: PathBuf::from("new_file.rs"),
            metrics,
        };

        registry.record_quality_event(&event);

        let exported = registry.export_metrics().unwrap();
        assert!(exported.contains("quality_pmat_quality_events_total 1"));
        assert!(exported.contains("quality_pmat_files_analyzed_total 1"));
    }

    #[test]
    fn test_analysis_duration_recording() {
        let registry = QualityMetricsRegistry::new().unwrap();
        let duration = Duration::from_millis(50);

        registry.record_analysis_duration(duration);

        let exported = registry.export_metrics().unwrap();
        assert!(exported.contains("quality_pmat_analysis_duration_seconds"));
        assert!(exported.contains("bucket"));
    }
}
