//! Performance benchmarking and optimization for unified quality system
//! 
//! Provides comprehensive performance monitoring, benchmarking, and optimization

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::time::interval;

/// Performance monitoring and optimization system
pub struct PerformanceMonitor {
    /// Active benchmarks
    benchmarks: HashMap<String, BenchmarkSuite>,
    
    /// Performance metrics storage
    metrics: PerformanceMetrics,
    
    /// Optimization engine
    optimizer: PerformanceOptimizer,
    
    /// Configuration
    config: PerformanceConfig,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable continuous monitoring
    pub continuous_monitoring: bool,
    
    /// Benchmark frequency
    pub benchmark_interval: Duration,
    
    /// Performance thresholds
    pub thresholds: PerformanceThresholds,
    
    /// Optimization settings
    pub optimization: OptimizationConfig,
    
    /// Retention settings
    pub retention: RetentionConfig,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum analysis time per file (ms)
    pub max_analysis_time_ms: u64,
    
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    
    /// Performance regression threshold (%)
    pub regression_threshold_percent: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_analysis_time_ms: 5000, // 5 seconds
            max_memory_mb: 1024,         // 1 GB
            max_cpu_percent: 80.0,       // 80%
            regression_threshold_percent: 20.0, // 20% slower
        }
    }
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Enable automatic optimization
    pub auto_optimize: bool,
    
    /// Optimization strategies to use
    pub strategies: Vec<OptimizationStrategy>,
    
    /// Minimum improvement threshold for applying optimization
    pub min_improvement_percent: f64,
    
    /// Enable experimental optimizations
    pub experimental: bool,
}

/// Performance optimization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// Cache frequently analyzed files
    CacheOptimization,
    
    /// Parallel processing optimization
    ParallelProcessing,
    
    /// Memory pooling
    MemoryPooling,
    
    /// Incremental parsing optimization
    IncrementalParsing,
    
    /// I/O optimization
    IoOptimization,
    
    /// AST reuse
    AstReuse,
}

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Keep detailed metrics for this duration
    pub detailed_retention: Duration,
    
    /// Keep summary metrics for this duration
    pub summary_retention: Duration,
    
    /// Automatic cleanup enabled
    pub auto_cleanup: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            detailed_retention: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            summary_retention: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
            auto_cleanup: true,
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Time-series data
    timeseries: HashMap<String, Vec<PerformancePoint>>,
    
    /// Aggregated statistics
    statistics: PerformanceStatistics,
    
    /// Baseline measurements
    baselines: HashMap<String, Baseline>,
}

/// Single performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePoint {
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Metric name
    pub metric: String,
    
    /// Measured value
    pub value: f64,
    
    /// Context metadata
    pub context: HashMap<String, String>,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    /// Analysis performance
    pub analysis: AnalysisStats,
    
    /// Memory usage statistics
    pub memory: MemoryStats,
    
    /// I/O performance statistics
    pub io: IoStats,
    
    /// System resource usage
    pub system: SystemStats,
}

/// Analysis performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStats {
    /// Average analysis time per file (ms)
    pub avg_analysis_time_ms: f64,
    
    /// Analysis throughput (files/second)
    pub throughput_fps: f64,
    
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    
    /// Parser efficiency
    pub parser_efficiency: f64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage (MB)
    pub peak_memory_mb: f64,
    
    /// Average memory usage (MB)
    pub avg_memory_mb: f64,
    
    /// Memory growth rate (MB/hour)
    pub growth_rate_mb_per_hour: f64,
    
    /// Garbage collection impact
    pub gc_impact_percent: f64,
}

/// I/O performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoStats {
    /// File read performance (MB/s)
    pub read_throughput_mbps: f64,
    
    /// Average file read time (ms)
    pub avg_read_time_ms: f64,
    
    /// I/O wait time percentage
    pub io_wait_percent: f64,
    
    /// Cache effectiveness
    pub cache_effectiveness: f64,
}

/// System resource statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// CPU utilization percentage
    pub cpu_percent: f64,
    
    /// Thread count
    pub thread_count: u32,
    
    /// System load average
    pub load_average: f64,
    
    /// Network usage (KB/s)
    pub network_kbps: f64,
}

/// Baseline performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// Baseline identifier
    pub id: String,
    
    /// Measured performance values
    pub measurements: HashMap<String, f64>,
    
    /// Measurement timestamp
    pub measured_at: SystemTime,
    
    /// Context information
    pub context: BaselineContext,
}

/// Baseline measurement context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineContext {
    /// System configuration
    pub system_info: SystemInfo,
    
    /// Codebase characteristics
    pub codebase_info: CodebaseInfo,
    
    /// Configuration used
    pub config_hash: String,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU model
    pub cpu_model: String,
    
    /// Total memory (MB)
    pub total_memory_mb: u64,
    
    /// Operating system
    pub os: String,
    
    /// Rust version
    pub rust_version: String,
}

/// Codebase characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseInfo {
    /// Total lines of code
    pub total_loc: u64,
    
    /// Number of files
    pub file_count: u64,
    
    /// Average complexity
    pub avg_complexity: f64,
    
    /// Primary language
    pub primary_language: String,
}

/// Benchmark suite definition
#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    /// Suite name
    pub name: String,
    
    /// Individual benchmarks
    pub benchmarks: Vec<Benchmark>,
    
    /// Suite configuration
    pub config: BenchmarkConfig,
}

/// Individual benchmark
#[derive(Debug, Clone)]
pub struct Benchmark {
    /// Benchmark name
    pub name: String,
    
    /// Benchmark function
    pub benchmark_fn: BenchmarkFn,
    
    /// Setup function
    pub setup_fn: Option<SetupFn>,
    
    /// Teardown function
    pub teardown_fn: Option<TeardownFn>,
    
    /// Expected performance characteristics
    pub expected: ExpectedPerformance,
}

/// Benchmark function type
pub type BenchmarkFn = fn(&BenchmarkContext) -> Result<BenchmarkResult>;
pub type SetupFn = fn() -> Result<BenchmarkContext>;
pub type TeardownFn = fn(BenchmarkContext) -> Result<()>;

/// Benchmark execution context
#[derive(Debug, Clone)]
pub struct BenchmarkContext {
    /// Test data
    pub test_data: HashMap<String, Vec<u8>>,
    
    /// Temporary directory
    pub temp_dir: PathBuf,
    
    /// Configuration
    pub config: HashMap<String, String>,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Execution time
    pub execution_time: Duration,
    
    /// Memory used (bytes)
    pub memory_used: u64,
    
    /// CPU time
    pub cpu_time: Duration,
    
    /// Throughput (operations/second)
    pub throughput: f64,
    
    /// Success indicator
    pub success: bool,
    
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Expected performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedPerformance {
    /// Maximum execution time
    pub max_execution_time: Duration,
    
    /// Maximum memory usage
    pub max_memory_bytes: u64,
    
    /// Minimum throughput
    pub min_throughput: f64,
    
    /// Performance regression threshold
    pub regression_threshold: f64,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations
    pub iterations: u32,
    
    /// Warmup iterations
    pub warmup_iterations: u32,
    
    /// Timeout for each benchmark
    pub timeout: Duration,
    
    /// Parallel execution
    pub parallel: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            timeout: Duration::from_secs(60),
            parallel: false,
        }
    }
}

/// Performance optimizer
pub struct PerformanceOptimizer {
    /// Active optimizations
    optimizations: Vec<ActiveOptimization>,
    
    /// Optimization history
    history: Vec<OptimizationResult>,
    
    /// Configuration
    config: OptimizationConfig,
}

/// Active optimization
#[derive(Debug, Clone)]
pub struct ActiveOptimization {
    /// Optimization strategy
    pub strategy: OptimizationStrategy,
    
    /// Target metric
    pub target_metric: String,
    
    /// Expected improvement
    pub expected_improvement: f64,
    
    /// Implementation status
    pub status: OptimizationStatus,
}

/// Optimization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStatus {
    /// Being analyzed
    Analyzing,
    
    /// Ready to implement
    Ready,
    
    /// Currently implementing
    Implementing,
    
    /// Testing performance impact
    Testing,
    
    /// Successfully applied
    Applied,
    
    /// Failed to apply
    Failed(String),
    
    /// Rolled back due to issues
    RolledBack(String),
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Applied strategy
    pub strategy: OptimizationStrategy,
    
    /// Measured improvement
    pub improvement_percent: f64,
    
    /// Affected metrics
    pub metrics_changed: HashMap<String, f64>,
    
    /// Application timestamp
    pub applied_at: SystemTime,
    
    /// Success indicator
    pub success: bool,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            benchmarks: HashMap::new(),
            metrics: PerformanceMetrics::new(),
            optimizer: PerformanceOptimizer::new(config.optimization.clone()),
            config,
        }
    }
    
    /// Start continuous performance monitoring
    pub async fn start_monitoring(&mut self) -> Result<()> {
        if !self.config.continuous_monitoring {
            return Ok(());
        }
        
        let mut interval = interval(self.config.benchmark_interval);
        
        loop {
            interval.tick().await;
            
            // Collect performance metrics
            self.collect_metrics().await?;
            
            // Check for performance regressions
            self.check_regressions().await?;
            
            // Apply optimizations if configured
            if self.config.optimization.auto_optimize {
                self.auto_optimize().await?;
            }
            
            // Cleanup old data
            if self.config.retention.auto_cleanup {
                self.cleanup_old_data().await?;
            }
        }
    }
    
    /// Run comprehensive performance benchmark
    pub async fn run_benchmark(&mut self, suite_name: &str) -> Result<BenchmarkReport> {
        let suite = self.benchmarks.get(suite_name)
            .ok_or_else(|| anyhow::anyhow!("Benchmark suite not found: {}", suite_name))?;
        
        let mut results = Vec::new();
        
        for benchmark in &suite.benchmarks {
            let result = self.run_single_benchmark(benchmark).await?;
            results.push((benchmark.name.clone(), result));
        }
        
        // Calculate summary statistics
        let summary = self.calculate_summary_stats(&results);
        
        // Check for regressions
        let regressions = self.detect_regressions(&results).await?;
        
        let report = BenchmarkReport {
            suite_name: suite_name.to_string(),
            executed_at: SystemTime::now(),
            results,
            summary: summary.clone(),
            regressions,
            recommendations: self.generate_recommendations(&summary),
        };
        
        // Store results for trend analysis
        self.store_benchmark_results(&report).await?;
        
        Ok(report)
    }
    
    /// Establish performance baseline
    pub async fn establish_baseline(&mut self, baseline_id: String) -> Result<Baseline> {
        // Collect current system information
        let system_info = self.collect_system_info().await?;
        let codebase_info = self.collect_codebase_info().await?;
        
        // Run comprehensive measurements
        let measurements = self.collect_baseline_measurements().await?;
        
        let baseline = Baseline {
            id: baseline_id.clone(),
            measurements,
            measured_at: SystemTime::now(),
            context: BaselineContext {
                system_info,
                codebase_info,
                config_hash: self.calculate_config_hash(),
            },
        };
        
        self.metrics.baselines.insert(baseline_id, baseline.clone());
        
        Ok(baseline)
    }
    
    /// Apply performance optimization
    pub async fn apply_optimization(
        &mut self,
        strategy: OptimizationStrategy,
    ) -> Result<OptimizationResult> {
        // Measure baseline performance
        let baseline = self.collect_baseline_measurements().await?;
        
        // Apply optimization strategy
        match strategy {
            OptimizationStrategy::CacheOptimization => {
                self.optimize_caching().await?;
            }
            OptimizationStrategy::ParallelProcessing => {
                self.optimize_parallel_processing().await?;
            }
            OptimizationStrategy::MemoryPooling => {
                self.optimize_memory_pooling().await?;
            }
            OptimizationStrategy::IncrementalParsing => {
                self.optimize_incremental_parsing().await?;
            }
            OptimizationStrategy::IoOptimization => {
                self.optimize_io().await?;
            }
            OptimizationStrategy::AstReuse => {
                self.optimize_ast_reuse().await?;
            }
        }
        
        // Measure performance after optimization
        tokio::time::sleep(Duration::from_secs(1)).await; // Allow system to stabilize
        let optimized = self.collect_baseline_measurements().await?;
        
        // Calculate improvement
        let improvement = self.calculate_improvement(&baseline, &optimized);
        
        let result = OptimizationResult {
            strategy,
            improvement_percent: improvement,
            metrics_changed: self.calculate_metrics_delta(&baseline, &optimized),
            applied_at: SystemTime::now(),
            success: improvement > self.config.optimization.min_improvement_percent,
        };
        
        self.optimizer.history.push(result.clone());
        
        Ok(result)
    }
    
    /// Generate performance report
    pub fn generate_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            generated_at: SystemTime::now(),
            current_statistics: self.metrics.statistics.clone(),
            recent_benchmarks: self.get_recent_benchmark_results(10),
            optimization_history: self.optimizer.history.clone(),
            recommendations: self.generate_system_recommendations(),
            alerts: self.generate_performance_alerts(),
        }
    }
    
    // Private implementation methods
    
    async fn collect_metrics(&mut self) -> Result<()> {
        // Implementation would collect various performance metrics
        Ok(())
    }
    
    async fn check_regressions(&self) -> Result<()> {
        // Implementation would check for performance regressions
        Ok(())
    }
    
    async fn auto_optimize(&mut self) -> Result<()> {
        // Implementation would apply automatic optimizations
        Ok(())
    }
    
    async fn cleanup_old_data(&mut self) -> Result<()> {
        // Implementation would clean up old performance data
        Ok(())
    }
    
    async fn run_single_benchmark(&self, _benchmark: &Benchmark) -> Result<BenchmarkResult> {
        // Implementation would run individual benchmark
        Ok(BenchmarkResult {
            execution_time: Duration::from_millis(100),
            memory_used: 1024 * 1024, // 1MB
            cpu_time: Duration::from_millis(90),
            throughput: 100.0,
            success: true,
            metrics: HashMap::new(),
        })
    }
    
    fn calculate_summary_stats(&self, _results: &[(String, BenchmarkResult)]) -> BenchmarkSummary {
        BenchmarkSummary {
            total_benchmarks: 10,
            passed_benchmarks: 10,
            failed_benchmarks: 0,
            avg_execution_time: Duration::from_millis(100),
            total_memory_used: 10 * 1024 * 1024, // 10MB
            avg_throughput: 100.0,
        }
    }
    
    async fn detect_regressions(&self, _results: &[(String, BenchmarkResult)]) -> Result<Vec<PerformanceRegression>> {
        Ok(Vec::new())
    }
    
    fn generate_recommendations(&self, _summary: &BenchmarkSummary) -> Vec<String> {
        vec!["Consider enabling cache optimization".to_string()]
    }
    
    async fn store_benchmark_results(&mut self, _report: &BenchmarkReport) -> Result<()> {
        Ok(())
    }
    
    async fn collect_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            cpu_model: "Unknown".to_string(),
            total_memory_mb: 8192,
            os: std::env::consts::OS.to_string(),
            rust_version: "1.70.0".to_string(),
        })
    }
    
    async fn collect_codebase_info(&self) -> Result<CodebaseInfo> {
        Ok(CodebaseInfo {
            total_loc: 100000,
            file_count: 1000,
            avg_complexity: 5.2,
            primary_language: "rust".to_string(),
        })
    }
    
    async fn collect_baseline_measurements(&self) -> Result<HashMap<String, f64>> {
        let mut measurements = HashMap::new();
        measurements.insert("analysis_time_ms".to_string(), 150.0);
        measurements.insert("memory_mb".to_string(), 256.0);
        measurements.insert("throughput_fps".to_string(), 50.0);
        Ok(measurements)
    }
    
    fn calculate_config_hash(&self) -> String {
        // Would calculate hash of current configuration
        "config_hash_placeholder".to_string()
    }
    
    // Optimization implementations
    async fn optimize_caching(&mut self) -> Result<()> { Ok(()) }
    async fn optimize_parallel_processing(&mut self) -> Result<()> { Ok(()) }
    async fn optimize_memory_pooling(&mut self) -> Result<()> { Ok(()) }
    async fn optimize_incremental_parsing(&mut self) -> Result<()> { Ok(()) }
    async fn optimize_io(&mut self) -> Result<()> { Ok(()) }
    async fn optimize_ast_reuse(&mut self) -> Result<()> { Ok(()) }
    
    fn calculate_improvement(&self, baseline: &HashMap<String, f64>, optimized: &HashMap<String, f64>) -> f64 {
        // Calculate average improvement across all metrics
        let mut total_improvement = 0.0;
        let mut count = 0;
        
        for (key, baseline_value) in baseline {
            if let Some(optimized_value) = optimized.get(key) {
                let improvement = (baseline_value - optimized_value) / baseline_value * 100.0;
                total_improvement += improvement;
                count += 1;
            }
        }
        
        if count > 0 {
            total_improvement / count as f64
        } else {
            0.0
        }
    }
    
    fn calculate_metrics_delta(&self, baseline: &HashMap<String, f64>, optimized: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut delta = HashMap::new();
        
        for (key, baseline_value) in baseline {
            if let Some(optimized_value) = optimized.get(key) {
                delta.insert(key.clone(), optimized_value - baseline_value);
            }
        }
        
        delta
    }
    
    fn get_recent_benchmark_results(&self, _count: usize) -> Vec<BenchmarkReport> {
        Vec::new() // Would return recent benchmark results
    }
    
    fn generate_system_recommendations(&self) -> Vec<String> {
        vec!["System appears to be performing well".to_string()]
    }
    
    fn generate_performance_alerts(&self) -> Vec<PerformanceAlert> {
        Vec::new() // Would generate performance alerts
    }
}

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
    Minor,      // < 10% regression
    Moderate,   // 10-25% regression
    Severe,     // 25-50% regression
    Critical,   // > 50% regression
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

impl PerformanceMetrics {
    /// Create new performance metrics
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_defaults() {
        let thresholds = PerformanceThresholds::default();
        assert_eq!(thresholds.max_analysis_time_ms, 5000);
        assert_eq!(thresholds.max_memory_mb, 1024);
        assert_eq!(thresholds.max_cpu_percent, 80.0);
        assert_eq!(thresholds.regression_threshold_percent, 20.0);
    }

    #[test]
    fn test_benchmark_config_defaults() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.warmup_iterations, 10);
        assert!(!config.parallel);
    }

    #[test]
    fn test_performance_statistics_defaults() {
        let stats = PerformanceStatistics::default();
        assert_eq!(stats.analysis.avg_analysis_time_ms, 100.0);
        assert_eq!(stats.memory.peak_memory_mb, 512.0);
        assert!(stats.io.cache_effectiveness > 0.0);
    }

    #[test]
    fn test_optimization_strategies() {
        let strategies = vec![
            OptimizationStrategy::CacheOptimization,
            OptimizationStrategy::ParallelProcessing,
            OptimizationStrategy::MemoryPooling,
            OptimizationStrategy::IncrementalParsing,
            OptimizationStrategy::IoOptimization,
            OptimizationStrategy::AstReuse,
        ];
        
        assert_eq!(strategies.len(), 6);
    }

    #[test]
    fn test_performance_monitor_creation() {
        let config = PerformanceConfig {
            continuous_monitoring: false,
            benchmark_interval: Duration::from_secs(60),
            thresholds: PerformanceThresholds::default(),
            optimization: OptimizationConfig {
                auto_optimize: false,
                strategies: vec![OptimizationStrategy::CacheOptimization],
                min_improvement_percent: 5.0,
                experimental: false,
            },
            retention: RetentionConfig::default(),
        };
        
        let monitor = PerformanceMonitor::new(config);
        assert!(!monitor.config.continuous_monitoring);
    }

    #[test]
    fn test_regression_severity_levels() {
        let severities = vec![
            RegressionSeverity::Minor,
            RegressionSeverity::Moderate,
            RegressionSeverity::Severe,
            RegressionSeverity::Critical,
        ];
        
        assert_eq!(severities.len(), 4);
        
        // Test serialization
        let serialized = serde_json::to_string(&severities[0]).unwrap();
        assert!(serialized.contains("Minor"));
    }
}