/// Benchmark execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub suite_name: String,
    pub executed_at: SystemTime,
    pub results: Vec<(String, BenchmarkResult)>,
    pub summary: BenchmarkSummary,
    pub regressions: Vec<PerformanceRegression>,
    pub recommendations: Vec<String>,
}

/// Benchmark summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_benchmarks: u32,
    pub passed_benchmarks: u32,
    pub failed_benchmarks: u32,
    pub avg_execution_time: Duration,
    pub total_memory_used: u64,
    pub avg_throughput: f64,
}

/// Performance regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    pub benchmark_name: String,
    pub metric_name: String,
    pub current_value: f64,
    pub baseline_value: f64,
    pub regression_percent: f64,
    pub severity: RegressionSeverity,
}

/// Regression severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Minor,    // < 10% regression
    Moderate, // 10-25% regression
    Severe,   // 25-50% regression
    Critical, // > 50% regression
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub generated_at: SystemTime,
    pub current_statistics: PerformanceStatistics,
    pub recent_benchmarks: Vec<BenchmarkReport>,
    pub optimization_history: Vec<OptimizationResult>,
    pub recommendations: Vec<String>,
    pub alerts: Vec<PerformanceAlert>,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub triggered_at: SystemTime,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighLatency,
    HighMemoryUsage,
    HighCpuUsage,
    LowThroughput,
    PerformanceRegression,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics
    #[must_use]
    pub fn new() -> Self {
        Self {
            timeseries: HashMap::new(),
            statistics: PerformanceStatistics::default(),
            baselines: HashMap::new(),
        }
    }
}

impl PerformanceOptimizer {
    /// Create new performance optimizer
    #[must_use]
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            optimizations: Vec::new(),
            history: Vec::new(),
            config,
        }
    }
}

impl Default for PerformanceStatistics {
    fn default() -> Self {
        Self {
            analysis: AnalysisStats {
                avg_analysis_time_ms: 100.0,
                throughput_fps: 10.0,
                cache_hit_ratio: 0.8,
                parser_efficiency: 0.9,
            },
            memory: MemoryStats {
                peak_memory_mb: 512.0,
                avg_memory_mb: 256.0,
                growth_rate_mb_per_hour: 5.0,
                gc_impact_percent: 2.0,
            },
            io: IoStats {
                read_throughput_mbps: 100.0,
                avg_read_time_ms: 10.0,
                io_wait_percent: 5.0,
                cache_effectiveness: 0.85,
            },
            system: SystemStats {
                cpu_percent: 25.0,
                thread_count: 8,
                load_average: 1.5,
                network_kbps: 1024.0,
            },
        }
    }
}
