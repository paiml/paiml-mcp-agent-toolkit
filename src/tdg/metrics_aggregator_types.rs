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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(window_size: Duration, max_points: usize) -> Self {
        Self {
            window_size,
            max_points,
            data: VecDeque::with_capacity(max_points),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_window(&self) -> Vec<DataPoint<T>> {
        self.data.iter().cloned().collect()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
