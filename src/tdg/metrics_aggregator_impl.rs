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
        debug_assert!(!values.is_empty(), "values must not be empty");
        debug_assert!(!data.is_empty(), "data must not be empty");
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
        debug_assert!(!values.is_empty(), "values must not be empty");
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
        debug_assert!(!values.is_empty(), "values must not be empty");
        debug_assert!(!data.is_empty(), "data must not be empty");
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
