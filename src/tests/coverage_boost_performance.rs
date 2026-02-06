#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for unified_quality/performance module
//! Tests for PerformanceMonitor, benchmarks, and related types

use crate::unified_quality::performance::{
    ActiveOptimization, AlertSeverity, AlertType, AnalysisStats, Baseline, BaselineContext,
    BenchmarkConfig, BenchmarkContext, BenchmarkReport, BenchmarkResult, BenchmarkSummary,
    CodebaseInfo, ExpectedPerformance, IoStats, MemoryStats, OptimizationConfig,
    OptimizationResult, OptimizationStatus, OptimizationStrategy, PerformanceAlert,
    PerformanceConfig, PerformanceMetrics, PerformanceMonitor, PerformancePoint,
    PerformanceRegression, PerformanceReport, PerformanceStatistics, PerformanceThresholds,
    RegressionSeverity, RetentionConfig, SystemInfo, SystemStats,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ============ PerformanceThresholds Tests ============

#[test]
fn test_performance_thresholds_default() {
    let thresholds = PerformanceThresholds::default();
    assert_eq!(thresholds.max_analysis_time_ms, 5000);
    assert_eq!(thresholds.max_memory_mb, 1024);
    assert_eq!(thresholds.max_cpu_percent, 80.0);
    assert_eq!(thresholds.regression_threshold_percent, 20.0);
}

#[test]
fn test_performance_thresholds_custom() {
    let thresholds = PerformanceThresholds {
        max_analysis_time_ms: 10000,
        max_memory_mb: 2048,
        max_cpu_percent: 90.0,
        regression_threshold_percent: 10.0,
    };
    assert_eq!(thresholds.max_analysis_time_ms, 10000);
}

#[test]
fn test_performance_thresholds_serialization() {
    let thresholds = PerformanceThresholds::default();
    let serialized = serde_json::to_string(&thresholds).unwrap();
    let deserialized: PerformanceThresholds = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.max_memory_mb, 1024);
}

// ============ OptimizationStrategy Tests ============

#[test]
fn test_optimization_strategy_cache() {
    let strategy = OptimizationStrategy::CacheOptimization;
    let serialized = serde_json::to_string(&strategy).unwrap();
    assert!(serialized.contains("CacheOptimization"));
}

#[test]
fn test_optimization_strategy_parallel() {
    let strategy = OptimizationStrategy::ParallelProcessing;
    let cloned = strategy.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_optimization_strategy_memory_pooling() {
    let strategy = OptimizationStrategy::MemoryPooling;
    let serialized = serde_json::to_string(&strategy).unwrap();
    let _: OptimizationStrategy = serde_json::from_str(&serialized).unwrap();
}

#[test]
fn test_optimization_strategy_incremental() {
    let strategy = OptimizationStrategy::IncrementalParsing;
    let _ = format!("{:?}", strategy);
}

#[test]
fn test_optimization_strategy_io() {
    let strategy = OptimizationStrategy::IoOptimization;
    let cloned = strategy.clone();
    let serialized = serde_json::to_string(&cloned).unwrap();
    assert!(serialized.contains("IoOptimization"));
}

#[test]
fn test_optimization_strategy_ast_reuse() {
    let strategy = OptimizationStrategy::AstReuse;
    let serialized = serde_json::to_string(&strategy).unwrap();
    let deserialized: OptimizationStrategy = serde_json::from_str(&serialized).unwrap();
    let _ = format!("{:?}", deserialized);
}

// ============ RetentionConfig Tests ============

#[test]
fn test_retention_config_default() {
    let config = RetentionConfig::default();
    assert_eq!(config.detailed_retention, Duration::from_secs(7 * 24 * 60 * 60));
    assert!(config.auto_cleanup);
}

#[test]
fn test_retention_config_custom() {
    let config = RetentionConfig {
        detailed_retention: Duration::from_secs(3600),
        summary_retention: Duration::from_secs(86400),
        auto_cleanup: false,
    };
    assert!(!config.auto_cleanup);
}

#[test]
fn test_retention_config_serialization() {
    let config = RetentionConfig::default();
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: RetentionConfig = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.auto_cleanup);
}

// ============ OptimizationConfig Tests ============

#[test]
fn test_optimization_config_creation() {
    let config = OptimizationConfig {
        auto_optimize: true,
        strategies: vec![
            OptimizationStrategy::CacheOptimization,
            OptimizationStrategy::ParallelProcessing,
        ],
        min_improvement_percent: 5.0,
        experimental: false,
    };
    assert!(config.auto_optimize);
    assert_eq!(config.strategies.len(), 2);
}

#[test]
fn test_optimization_config_serialization() {
    let config = OptimizationConfig {
        auto_optimize: false,
        strategies: vec![OptimizationStrategy::MemoryPooling],
        min_improvement_percent: 10.0,
        experimental: true,
    };
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: OptimizationConfig = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.experimental);
}

// ============ BenchmarkConfig Tests ============

#[test]
fn test_benchmark_config_default() {
    let config = BenchmarkConfig::default();
    assert_eq!(config.iterations, 100);
    assert_eq!(config.warmup_iterations, 10);
    assert!(!config.parallel);
}

#[test]
fn test_benchmark_config_custom() {
    let config = BenchmarkConfig {
        iterations: 500,
        warmup_iterations: 50,
        timeout: Duration::from_secs(120),
        parallel: true,
    };
    assert!(config.parallel);
    assert_eq!(config.iterations, 500);
}

#[test]
fn test_benchmark_config_serialization() {
    let config = BenchmarkConfig::default();
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: BenchmarkConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.iterations, 100);
}

// ============ PerformancePoint Tests ============

#[test]
fn test_performance_point_creation() {
    let point = PerformancePoint {
        timestamp: SystemTime::now(),
        metric: "analysis_time".to_string(),
        value: 150.5,
        context: HashMap::new(),
    };
    assert_eq!(point.metric, "analysis_time");
    assert_eq!(point.value, 150.5);
}

#[test]
fn test_performance_point_with_context() {
    let mut context = HashMap::new();
    context.insert("file".to_string(), "test.rs".to_string());

    let point = PerformancePoint {
        timestamp: SystemTime::now(),
        metric: "parse_time".to_string(),
        value: 50.0,
        context,
    };
    assert!(point.context.contains_key("file"));
}

#[test]
fn test_performance_point_serialization() {
    let point = PerformancePoint {
        timestamp: SystemTime::UNIX_EPOCH,
        metric: "throughput".to_string(),
        value: 100.0,
        context: HashMap::new(),
    };
    let serialized = serde_json::to_string(&point).unwrap();
    let deserialized: PerformancePoint = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.value, 100.0);
}

// ============ AnalysisStats Tests ============

#[test]
fn test_analysis_stats_creation() {
    let stats = AnalysisStats {
        avg_analysis_time_ms: 100.0,
        throughput_fps: 50.0,
        cache_hit_ratio: 0.85,
        parser_efficiency: 0.95,
    };
    assert_eq!(stats.cache_hit_ratio, 0.85);
}

#[test]
fn test_analysis_stats_serialization() {
    let stats = AnalysisStats {
        avg_analysis_time_ms: 200.0,
        throughput_fps: 25.0,
        cache_hit_ratio: 0.5,
        parser_efficiency: 0.8,
    };
    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: AnalysisStats = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.throughput_fps, 25.0);
}

// ============ MemoryStats Tests ============

#[test]
fn test_memory_stats_creation() {
    let stats = MemoryStats {
        peak_memory_mb: 512.0,
        avg_memory_mb: 256.0,
        growth_rate_mb_per_hour: 10.0,
        gc_impact_percent: 3.0,
    };
    assert_eq!(stats.peak_memory_mb, 512.0);
}

#[test]
fn test_memory_stats_serialization() {
    let stats = MemoryStats {
        peak_memory_mb: 1024.0,
        avg_memory_mb: 512.0,
        growth_rate_mb_per_hour: 5.0,
        gc_impact_percent: 2.0,
    };
    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: MemoryStats = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.avg_memory_mb, 512.0);
}

// ============ IoStats Tests ============

#[test]
fn test_io_stats_creation() {
    let stats = IoStats {
        read_throughput_mbps: 500.0,
        avg_read_time_ms: 5.0,
        io_wait_percent: 2.0,
        cache_effectiveness: 0.9,
    };
    assert_eq!(stats.read_throughput_mbps, 500.0);
}

#[test]
fn test_io_stats_serialization() {
    let stats = IoStats {
        read_throughput_mbps: 100.0,
        avg_read_time_ms: 10.0,
        io_wait_percent: 5.0,
        cache_effectiveness: 0.75,
    };
    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: IoStats = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.cache_effectiveness, 0.75);
}

// ============ SystemStats Tests ============

#[test]
fn test_system_stats_creation() {
    let stats = SystemStats {
        cpu_percent: 45.0,
        thread_count: 16,
        load_average: 2.5,
        network_kbps: 2048.0,
    };
    assert_eq!(stats.thread_count, 16);
}

#[test]
fn test_system_stats_serialization() {
    let stats = SystemStats {
        cpu_percent: 80.0,
        thread_count: 8,
        load_average: 4.0,
        network_kbps: 1024.0,
    };
    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: SystemStats = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.cpu_percent, 80.0);
}

// ============ PerformanceStatistics Tests ============

#[test]
fn test_performance_statistics_default() {
    let stats = PerformanceStatistics::default();
    assert_eq!(stats.analysis.avg_analysis_time_ms, 100.0);
    assert_eq!(stats.memory.peak_memory_mb, 512.0);
    assert_eq!(stats.io.read_throughput_mbps, 100.0);
    assert_eq!(stats.system.thread_count, 8);
}

#[test]
fn test_performance_statistics_serialization() {
    let stats = PerformanceStatistics::default();
    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: PerformanceStatistics = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.analysis.throughput_fps, 10.0);
}

// ============ SystemInfo Tests ============

#[test]
fn test_system_info_creation() {
    let info = SystemInfo {
        cpu_model: "Intel i7".to_string(),
        total_memory_mb: 16384,
        os: "linux".to_string(),
        rust_version: "1.75.0".to_string(),
    };
    assert_eq!(info.total_memory_mb, 16384);
}

#[test]
fn test_system_info_serialization() {
    let info = SystemInfo {
        cpu_model: "AMD Ryzen".to_string(),
        total_memory_mb: 32768,
        os: "windows".to_string(),
        rust_version: "1.70.0".to_string(),
    };
    let serialized = serde_json::to_string(&info).unwrap();
    let deserialized: SystemInfo = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.cpu_model, "AMD Ryzen");
}

// ============ CodebaseInfo Tests ============

#[test]
fn test_codebase_info_creation() {
    let info = CodebaseInfo {
        total_loc: 500000,
        file_count: 2500,
        avg_complexity: 6.5,
        primary_language: "rust".to_string(),
    };
    assert_eq!(info.total_loc, 500000);
}

#[test]
fn test_codebase_info_serialization() {
    let info = CodebaseInfo {
        total_loc: 100000,
        file_count: 1000,
        avg_complexity: 4.2,
        primary_language: "python".to_string(),
    };
    let serialized = serde_json::to_string(&info).unwrap();
    let deserialized: CodebaseInfo = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.primary_language, "python");
}

// ============ BaselineContext Tests ============

#[test]
fn test_baseline_context_creation() {
    let context = BaselineContext {
        system_info: SystemInfo {
            cpu_model: "Test CPU".to_string(),
            total_memory_mb: 8192,
            os: "linux".to_string(),
            rust_version: "1.70.0".to_string(),
        },
        codebase_info: CodebaseInfo {
            total_loc: 50000,
            file_count: 500,
            avg_complexity: 5.0,
            primary_language: "rust".to_string(),
        },
        config_hash: "abc123".to_string(),
    };
    assert_eq!(context.config_hash, "abc123");
}

// ============ Baseline Tests ============

#[test]
fn test_baseline_creation() {
    let mut measurements = HashMap::new();
    measurements.insert("analysis_time".to_string(), 100.0);

    let baseline = Baseline {
        id: "baseline1".to_string(),
        measurements,
        measured_at: SystemTime::now(),
        context: BaselineContext {
            system_info: SystemInfo {
                cpu_model: "Test".to_string(),
                total_memory_mb: 8192,
                os: "linux".to_string(),
                rust_version: "1.70.0".to_string(),
            },
            codebase_info: CodebaseInfo {
                total_loc: 10000,
                file_count: 100,
                avg_complexity: 5.0,
                primary_language: "rust".to_string(),
            },
            config_hash: "hash".to_string(),
        },
    };
    assert_eq!(baseline.id, "baseline1");
}

// ============ BenchmarkContext Tests ============

#[test]
fn test_benchmark_context_creation() {
    let context = BenchmarkContext {
        test_data: HashMap::new(),
        temp_dir: std::path::PathBuf::from("/tmp/test"),
        config: HashMap::new(),
    };
    assert_eq!(context.temp_dir, std::path::PathBuf::from("/tmp/test"));
}

#[test]
fn test_benchmark_context_with_data() {
    let mut test_data = HashMap::new();
    test_data.insert("file1".to_string(), vec![0u8; 100]);

    let mut config = HashMap::new();
    config.insert("iterations".to_string(), "100".to_string());

    let context = BenchmarkContext {
        test_data,
        temp_dir: std::path::PathBuf::from("/tmp"),
        config,
    };
    assert!(context.test_data.contains_key("file1"));
}

// ============ BenchmarkResult Tests ============

#[test]
fn test_benchmark_result_creation() {
    let result = BenchmarkResult {
        execution_time: Duration::from_millis(100),
        memory_used: 1024 * 1024,
        cpu_time: Duration::from_millis(90),
        throughput: 100.0,
        success: true,
        metrics: HashMap::new(),
    };
    assert!(result.success);
    assert_eq!(result.throughput, 100.0);
}

#[test]
fn test_benchmark_result_serialization() {
    let result = BenchmarkResult {
        execution_time: Duration::from_secs(1),
        memory_used: 2048,
        cpu_time: Duration::from_millis(500),
        throughput: 50.0,
        success: false,
        metrics: HashMap::new(),
    };
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: BenchmarkResult = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.success);
}

// ============ ExpectedPerformance Tests ============

#[test]
fn test_expected_performance_creation() {
    let expected = ExpectedPerformance {
        max_execution_time: Duration::from_secs(5),
        max_memory_bytes: 1024 * 1024 * 100,
        min_throughput: 10.0,
        regression_threshold: 0.2,
    };
    assert_eq!(expected.min_throughput, 10.0);
}

#[test]
fn test_expected_performance_serialization() {
    let expected = ExpectedPerformance {
        max_execution_time: Duration::from_secs(10),
        max_memory_bytes: 1024 * 1024,
        min_throughput: 5.0,
        regression_threshold: 0.1,
    };
    let serialized = serde_json::to_string(&expected).unwrap();
    let deserialized: ExpectedPerformance = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.regression_threshold, 0.1);
}

// ============ OptimizationStatus Tests ============

#[test]
fn test_optimization_status_analyzing() {
    let status = OptimizationStatus::Analyzing;
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(serialized.contains("Analyzing"));
}

#[test]
fn test_optimization_status_ready() {
    let status = OptimizationStatus::Ready;
    let _ = format!("{:?}", status);
}

#[test]
fn test_optimization_status_implementing() {
    let status = OptimizationStatus::Implementing;
    let cloned = status.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_optimization_status_testing() {
    let status = OptimizationStatus::Testing;
    let serialized = serde_json::to_string(&status).unwrap();
    let _: OptimizationStatus = serde_json::from_str(&serialized).unwrap();
}

#[test]
fn test_optimization_status_applied() {
    let status = OptimizationStatus::Applied;
    let _ = format!("{:?}", status);
}

#[test]
fn test_optimization_status_failed() {
    let status = OptimizationStatus::Failed("Timeout".to_string());
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(serialized.contains("Failed"));
}

#[test]
fn test_optimization_status_rolled_back() {
    let status = OptimizationStatus::RolledBack("Performance degraded".to_string());
    let cloned = status.clone();
    let _ = format!("{:?}", cloned);
}

// ============ ActiveOptimization Tests ============

#[test]
fn test_active_optimization_creation() {
    let optimization = ActiveOptimization {
        strategy: OptimizationStrategy::CacheOptimization,
        target_metric: "analysis_time".to_string(),
        expected_improvement: 15.0,
        status: OptimizationStatus::Ready,
    };
    assert_eq!(optimization.expected_improvement, 15.0);
}

// ============ OptimizationResult Tests ============

#[test]
fn test_optimization_result_creation() {
    let result = OptimizationResult {
        strategy: OptimizationStrategy::ParallelProcessing,
        improvement_percent: 25.0,
        metrics_changed: HashMap::new(),
        applied_at: SystemTime::now(),
        success: true,
    };
    assert!(result.success);
    assert_eq!(result.improvement_percent, 25.0);
}

#[test]
fn test_optimization_result_serialization() {
    let result = OptimizationResult {
        strategy: OptimizationStrategy::MemoryPooling,
        improvement_percent: 10.0,
        metrics_changed: HashMap::new(),
        applied_at: SystemTime::UNIX_EPOCH,
        success: false,
    };
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: OptimizationResult = serde_json::from_str(&serialized).unwrap();
    assert!(!deserialized.success);
}

// ============ BenchmarkSummary Tests ============

#[test]
fn test_benchmark_summary_creation() {
    let summary = BenchmarkSummary {
        total_benchmarks: 100,
        passed_benchmarks: 95,
        failed_benchmarks: 5,
        avg_execution_time: Duration::from_millis(150),
        total_memory_used: 1024 * 1024 * 50,
        avg_throughput: 75.0,
    };
    assert_eq!(summary.passed_benchmarks, 95);
}

#[test]
fn test_benchmark_summary_serialization() {
    let summary = BenchmarkSummary {
        total_benchmarks: 50,
        passed_benchmarks: 50,
        failed_benchmarks: 0,
        avg_execution_time: Duration::from_millis(100),
        total_memory_used: 1024 * 1024,
        avg_throughput: 100.0,
    };
    let serialized = serde_json::to_string(&summary).unwrap();
    let deserialized: BenchmarkSummary = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.total_benchmarks, 50);
}

// ============ RegressionSeverity Tests ============

#[test]
fn test_regression_severity_minor() {
    let severity = RegressionSeverity::Minor;
    let serialized = serde_json::to_string(&severity).unwrap();
    assert!(serialized.contains("Minor"));
}

#[test]
fn test_regression_severity_moderate() {
    let severity = RegressionSeverity::Moderate;
    let cloned = severity.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_regression_severity_severe() {
    let severity = RegressionSeverity::Severe;
    let serialized = serde_json::to_string(&severity).unwrap();
    let _: RegressionSeverity = serde_json::from_str(&serialized).unwrap();
}

#[test]
fn test_regression_severity_critical() {
    let severity = RegressionSeverity::Critical;
    let _ = format!("{:?}", severity);
}

// ============ PerformanceRegression Tests ============

#[test]
fn test_performance_regression_creation() {
    let regression = PerformanceRegression {
        benchmark_name: "analysis_benchmark".to_string(),
        metric_name: "execution_time".to_string(),
        current_value: 200.0,
        baseline_value: 100.0,
        regression_percent: 100.0,
        severity: RegressionSeverity::Critical,
    };
    assert_eq!(regression.regression_percent, 100.0);
}

// ============ AlertType Tests ============

#[test]
fn test_alert_type_high_latency() {
    let alert_type = AlertType::HighLatency;
    let serialized = serde_json::to_string(&alert_type).unwrap();
    assert!(serialized.contains("HighLatency"));
}

#[test]
fn test_alert_type_high_memory() {
    let alert_type = AlertType::HighMemoryUsage;
    let _ = format!("{:?}", alert_type);
}

#[test]
fn test_alert_type_high_cpu() {
    let alert_type = AlertType::HighCpuUsage;
    let cloned = alert_type.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_alert_type_low_throughput() {
    let alert_type = AlertType::LowThroughput;
    let serialized = serde_json::to_string(&alert_type).unwrap();
    let _: AlertType = serde_json::from_str(&serialized).unwrap();
}

#[test]
fn test_alert_type_regression() {
    let alert_type = AlertType::PerformanceRegression;
    let _ = format!("{:?}", alert_type);
}

// ============ AlertSeverity Tests ============

#[test]
fn test_alert_severity_info() {
    let severity = AlertSeverity::Info;
    let serialized = serde_json::to_string(&severity).unwrap();
    assert!(serialized.contains("Info"));
}

#[test]
fn test_alert_severity_warning() {
    let severity = AlertSeverity::Warning;
    let cloned = severity.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_alert_severity_critical() {
    let severity = AlertSeverity::Critical;
    let serialized = serde_json::to_string(&severity).unwrap();
    let _: AlertSeverity = serde_json::from_str(&serialized).unwrap();
}

// ============ PerformanceAlert Tests ============

#[test]
fn test_performance_alert_creation() {
    let alert = PerformanceAlert {
        alert_type: AlertType::HighLatency,
        message: "Analysis taking too long".to_string(),
        severity: AlertSeverity::Warning,
        metric_name: "analysis_time".to_string(),
        current_value: 6000.0,
        threshold_value: 5000.0,
        triggered_at: SystemTime::now(),
    };
    assert_eq!(alert.current_value, 6000.0);
}

// ============ BenchmarkReport Tests ============

#[test]
fn test_benchmark_report_creation() {
    let report = BenchmarkReport {
        suite_name: "performance_suite".to_string(),
        executed_at: SystemTime::now(),
        results: vec![],
        summary: BenchmarkSummary {
            total_benchmarks: 10,
            passed_benchmarks: 10,
            failed_benchmarks: 0,
            avg_execution_time: Duration::from_millis(100),
            total_memory_used: 1024 * 1024,
            avg_throughput: 100.0,
        },
        regressions: vec![],
        recommendations: vec!["Enable caching".to_string()],
    };
    assert_eq!(report.suite_name, "performance_suite");
}

// ============ PerformanceReport Tests ============

#[test]
fn test_performance_report_creation() {
    let report = PerformanceReport {
        generated_at: SystemTime::now(),
        current_statistics: PerformanceStatistics::default(),
        recent_benchmarks: vec![],
        optimization_history: vec![],
        recommendations: vec!["Optimize cache".to_string()],
        alerts: vec![],
    };
    assert!(report.recommendations.len() > 0);
}

// ============ PerformanceMetrics Tests ============

#[test]
fn test_performance_metrics_new() {
    let metrics = PerformanceMetrics::new();
    let _ = format!("{:?}", metrics);
}

#[test]
fn test_performance_metrics_default() {
    let metrics = PerformanceMetrics::default();
    let cloned = metrics.clone();
    let _ = format!("{:?}", cloned);
}

// ============ PerformanceConfig Tests ============

fn create_test_config() -> PerformanceConfig {
    PerformanceConfig {
        continuous_monitoring: true,
        benchmark_interval: Duration::from_secs(60),
        thresholds: PerformanceThresholds::default(),
        optimization: OptimizationConfig {
            auto_optimize: false,
            strategies: vec![],
            min_improvement_percent: 5.0,
            experimental: false,
        },
        retention: RetentionConfig::default(),
    }
}

#[test]
fn test_performance_config_creation() {
    let config = create_test_config();
    assert!(config.continuous_monitoring);
}

#[test]
fn test_performance_config_serialization() {
    let config = create_test_config();
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: PerformanceConfig = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.continuous_monitoring);
}

// ============ PerformanceMonitor Tests ============

#[test]
fn test_performance_monitor_new() {
    let config = create_test_config();
    let monitor = PerformanceMonitor::new(config);
    let _ = monitor;
}

#[test]
fn test_performance_monitor_generate_report() {
    let config = create_test_config();
    let monitor = PerformanceMonitor::new(config);
    let report = monitor.generate_performance_report();
    assert!(report.recommendations.len() > 0);
}

#[tokio::test]
async fn test_performance_monitor_establish_baseline() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let baseline = monitor.establish_baseline("test_baseline".to_string()).await.unwrap();
    assert_eq!(baseline.id, "test_baseline");
    assert!(baseline.measurements.len() > 0);
}

#[tokio::test]
async fn test_performance_monitor_run_benchmark_not_found() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.run_benchmark("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_performance_monitor_apply_optimization_cache() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.apply_optimization(OptimizationStrategy::CacheOptimization).await.unwrap();
    let _ = result;
}

#[tokio::test]
async fn test_performance_monitor_apply_optimization_parallel() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.apply_optimization(OptimizationStrategy::ParallelProcessing).await.unwrap();
    let _ = result;
}

#[tokio::test]
async fn test_performance_monitor_apply_optimization_memory() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.apply_optimization(OptimizationStrategy::MemoryPooling).await.unwrap();
    let _ = result;
}

#[tokio::test]
async fn test_performance_monitor_apply_optimization_incremental() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.apply_optimization(OptimizationStrategy::IncrementalParsing).await.unwrap();
    let _ = result;
}

#[tokio::test]
async fn test_performance_monitor_apply_optimization_io() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.apply_optimization(OptimizationStrategy::IoOptimization).await.unwrap();
    let _ = result;
}

#[tokio::test]
async fn test_performance_monitor_apply_optimization_ast() {
    let config = create_test_config();
    let mut monitor = PerformanceMonitor::new(config);
    let result = monitor.apply_optimization(OptimizationStrategy::AstReuse).await.unwrap();
    let _ = result;
}
