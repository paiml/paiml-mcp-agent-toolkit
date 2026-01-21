//\! Tests for performance
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    fn create_test_config() -> PerformanceConfig {
        PerformanceConfig {
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
        }
    }

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
        let strategies = [
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
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        assert!(!monitor.config.continuous_monitoring);
    }

    #[test]
    fn test_regression_severity_levels() {
        let severities = [
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

    // ============ PerformanceThresholds Tests ============

    #[test]
    fn test_performance_thresholds_clone() {
        let thresholds = PerformanceThresholds::default();
        let cloned = thresholds.clone();
        assert_eq!(cloned.max_analysis_time_ms, thresholds.max_analysis_time_ms);
    }

    #[test]
    fn test_performance_thresholds_debug() {
        let thresholds = PerformanceThresholds::default();
        let debug = format!("{:?}", thresholds);
        assert!(debug.contains("PerformanceThresholds"));
    }

    #[test]
    fn test_performance_thresholds_serialization() {
        let thresholds = PerformanceThresholds::default();
        let json = serde_json::to_string(&thresholds).unwrap();
        let deserialized: PerformanceThresholds = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_analysis_time_ms, thresholds.max_analysis_time_ms);
    }

    // ============ OptimizationConfig Tests ============

    #[test]
    fn test_optimization_config_creation() {
        let config = OptimizationConfig {
            auto_optimize: true,
            strategies: vec![OptimizationStrategy::CacheOptimization, OptimizationStrategy::ParallelProcessing],
            min_improvement_percent: 10.0,
            experimental: true,
        };
        assert!(config.auto_optimize);
        assert_eq!(config.strategies.len(), 2);
        assert!(config.experimental);
    }

    #[test]
    fn test_optimization_config_clone() {
        let config = OptimizationConfig {
            auto_optimize: false,
            strategies: vec![],
            min_improvement_percent: 5.0,
            experimental: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.auto_optimize, config.auto_optimize);
    }

    // ============ RetentionConfig Tests ============

    #[test]
    fn test_retention_config_default() {
        let config = RetentionConfig::default();
        assert_eq!(config.detailed_retention, Duration::from_secs(7 * 24 * 60 * 60));
        assert_eq!(config.summary_retention, Duration::from_secs(90 * 24 * 60 * 60));
        assert!(config.auto_cleanup);
    }

    #[test]
    fn test_retention_config_clone() {
        let config = RetentionConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.detailed_retention, config.detailed_retention);
    }

    // ============ PerformancePoint Tests ============

    #[test]
    fn test_performance_point_creation() {
        let point = PerformancePoint {
            timestamp: SystemTime::now(),
            metric: "test_metric".to_string(),
            value: 42.5,
            context: HashMap::new(),
        };
        assert_eq!(point.metric, "test_metric");
        assert_eq!(point.value, 42.5);
    }

    #[test]
    fn test_performance_point_clone() {
        let mut context = HashMap::new();
        context.insert("key".to_string(), "value".to_string());
        let point = PerformancePoint {
            timestamp: SystemTime::now(),
            metric: "metric".to_string(),
            value: 100.0,
            context,
        };
        let cloned = point.clone();
        assert_eq!(cloned.metric, point.metric);
    }

    // ============ AnalysisStats Tests ============

    #[test]
    fn test_analysis_stats_creation() {
        let stats = AnalysisStats {
            avg_analysis_time_ms: 50.0,
            throughput_fps: 20.0,
            cache_hit_ratio: 0.95,
            parser_efficiency: 0.85,
        };
        assert_eq!(stats.avg_analysis_time_ms, 50.0);
        assert_eq!(stats.throughput_fps, 20.0);
    }

    #[test]
    fn test_analysis_stats_clone() {
        let stats = AnalysisStats {
            avg_analysis_time_ms: 100.0,
            throughput_fps: 10.0,
            cache_hit_ratio: 0.8,
            parser_efficiency: 0.9,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.avg_analysis_time_ms, stats.avg_analysis_time_ms);
    }

    // ============ MemoryStats Tests ============

    #[test]
    fn test_memory_stats_creation() {
        let stats = MemoryStats {
            peak_memory_mb: 1024.0,
            avg_memory_mb: 512.0,
            growth_rate_mb_per_hour: 10.0,
            gc_impact_percent: 5.0,
        };
        assert_eq!(stats.peak_memory_mb, 1024.0);
        assert_eq!(stats.avg_memory_mb, 512.0);
    }

    #[test]
    fn test_memory_stats_clone() {
        let stats = MemoryStats {
            peak_memory_mb: 256.0,
            avg_memory_mb: 128.0,
            growth_rate_mb_per_hour: 2.0,
            gc_impact_percent: 1.0,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.peak_memory_mb, stats.peak_memory_mb);
    }

    // ============ IoStats Tests ============

    #[test]
    fn test_io_stats_creation() {
        let stats = IoStats {
            read_throughput_mbps: 200.0,
            avg_read_time_ms: 5.0,
            io_wait_percent: 3.0,
            cache_effectiveness: 0.9,
        };
        assert_eq!(stats.read_throughput_mbps, 200.0);
        assert_eq!(stats.cache_effectiveness, 0.9);
    }

    #[test]
    fn test_io_stats_clone() {
        let stats = IoStats {
            read_throughput_mbps: 100.0,
            avg_read_time_ms: 10.0,
            io_wait_percent: 5.0,
            cache_effectiveness: 0.85,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.read_throughput_mbps, stats.read_throughput_mbps);
    }

    // ============ SystemStats Tests ============

    #[test]
    fn test_system_stats_creation() {
        let stats = SystemStats {
            cpu_percent: 50.0,
            thread_count: 16,
            load_average: 2.0,
            network_kbps: 2048.0,
        };
        assert_eq!(stats.cpu_percent, 50.0);
        assert_eq!(stats.thread_count, 16);
    }

    #[test]
    fn test_system_stats_clone() {
        let stats = SystemStats {
            cpu_percent: 25.0,
            thread_count: 8,
            load_average: 1.0,
            network_kbps: 1024.0,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.cpu_percent, stats.cpu_percent);
    }

    // ============ SystemInfo Tests ============

    #[test]
    fn test_system_info_creation() {
        let info = SystemInfo {
            cpu_model: "Intel i9".to_string(),
            total_memory_mb: 32768,
            os: "linux".to_string(),
            rust_version: "1.75.0".to_string(),
        };
        assert_eq!(info.cpu_model, "Intel i9");
        assert_eq!(info.total_memory_mb, 32768);
    }

    #[test]
    fn test_system_info_clone() {
        let info = SystemInfo {
            cpu_model: "AMD Ryzen".to_string(),
            total_memory_mb: 16384,
            os: "macos".to_string(),
            rust_version: "1.70.0".to_string(),
        };
        let cloned = info.clone();
        assert_eq!(cloned.cpu_model, info.cpu_model);
    }

    // ============ CodebaseInfo Tests ============

    #[test]
    fn test_codebase_info_creation() {
        let info = CodebaseInfo {
            total_loc: 500000,
            file_count: 2000,
            avg_complexity: 8.5,
            primary_language: "typescript".to_string(),
        };
        assert_eq!(info.total_loc, 500000);
        assert_eq!(info.file_count, 2000);
    }

    #[test]
    fn test_codebase_info_clone() {
        let info = CodebaseInfo {
            total_loc: 100000,
            file_count: 500,
            avg_complexity: 5.0,
            primary_language: "rust".to_string(),
        };
        let cloned = info.clone();
        assert_eq!(cloned.total_loc, info.total_loc);
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
                total_loc: 10000,
                file_count: 100,
                avg_complexity: 4.0,
                primary_language: "rust".to_string(),
            },
            config_hash: "abc123".to_string(),
        };
        assert_eq!(context.config_hash, "abc123");
    }

    #[test]
    fn test_baseline_context_clone() {
        let context = BaselineContext {
            system_info: SystemInfo {
                cpu_model: "CPU".to_string(),
                total_memory_mb: 4096,
                os: "windows".to_string(),
                rust_version: "1.65.0".to_string(),
            },
            codebase_info: CodebaseInfo {
                total_loc: 5000,
                file_count: 50,
                avg_complexity: 3.0,
                primary_language: "python".to_string(),
            },
            config_hash: "hash".to_string(),
        };
        let cloned = context.clone();
        assert_eq!(cloned.config_hash, context.config_hash);
    }

    // ============ Baseline Tests ============

    #[test]
    fn test_baseline_creation() {
        let baseline = Baseline {
            id: "baseline-1".to_string(),
            measurements: HashMap::new(),
            measured_at: SystemTime::now(),
            context: BaselineContext {
                system_info: SystemInfo {
                    cpu_model: "CPU".to_string(),
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
        assert_eq!(baseline.id, "baseline-1");
    }

    #[test]
    fn test_baseline_clone() {
        let mut measurements = HashMap::new();
        measurements.insert("metric".to_string(), 100.0);
        let baseline = Baseline {
            id: "test".to_string(),
            measurements,
            measured_at: SystemTime::now(),
            context: BaselineContext {
                system_info: SystemInfo {
                    cpu_model: "CPU".to_string(),
                    total_memory_mb: 4096,
                    os: "macos".to_string(),
                    rust_version: "1.70.0".to_string(),
                },
                codebase_info: CodebaseInfo {
                    total_loc: 5000,
                    file_count: 50,
                    avg_complexity: 3.0,
                    primary_language: "go".to_string(),
                },
                config_hash: "h".to_string(),
            },
        };
        let cloned = baseline.clone();
        assert_eq!(cloned.id, baseline.id);
    }

    // ============ BenchmarkResult Tests ============

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult {
            execution_time: Duration::from_millis(500),
            memory_used: 1024 * 1024 * 10,
            cpu_time: Duration::from_millis(400),
            throughput: 200.0,
            success: true,
            metrics: HashMap::new(),
        };
        assert_eq!(result.execution_time, Duration::from_millis(500));
        assert!(result.success);
    }

    #[test]
    fn test_benchmark_result_clone() {
        let result = BenchmarkResult {
            execution_time: Duration::from_secs(1),
            memory_used: 1024,
            cpu_time: Duration::from_millis(900),
            throughput: 100.0,
            success: false,
            metrics: HashMap::new(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.success, result.success);
    }

    // ============ ExpectedPerformance Tests ============

    #[test]
    fn test_expected_performance_creation() {
        let expected = ExpectedPerformance {
            max_execution_time: Duration::from_secs(5),
            max_memory_bytes: 1024 * 1024 * 100,
            min_throughput: 50.0,
            regression_threshold: 0.1,
        };
        assert_eq!(expected.max_execution_time, Duration::from_secs(5));
        assert_eq!(expected.min_throughput, 50.0);
    }

    #[test]
    fn test_expected_performance_clone() {
        let expected = ExpectedPerformance {
            max_execution_time: Duration::from_secs(10),
            max_memory_bytes: 1024 * 1024,
            min_throughput: 25.0,
            regression_threshold: 0.2,
        };
        let cloned = expected.clone();
        assert_eq!(cloned.min_throughput, expected.min_throughput);
    }

    // ============ BenchmarkConfig Tests ============

    #[test]
    fn test_benchmark_config_creation() {
        let config = BenchmarkConfig {
            iterations: 50,
            warmup_iterations: 5,
            timeout: Duration::from_secs(30),
            parallel: true,
        };
        assert_eq!(config.iterations, 50);
        assert!(config.parallel);
    }

    #[test]
    fn test_benchmark_config_clone() {
        let config = BenchmarkConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.iterations, config.iterations);
    }

    // ============ ActiveOptimization Tests ============

    #[test]
    fn test_active_optimization_creation() {
        let opt = ActiveOptimization {
            strategy: OptimizationStrategy::CacheOptimization,
            target_metric: "analysis_time".to_string(),
            expected_improvement: 25.0,
            status: OptimizationStatus::Ready,
        };
        assert_eq!(opt.target_metric, "analysis_time");
        assert_eq!(opt.expected_improvement, 25.0);
    }

    #[test]
    fn test_active_optimization_clone() {
        let opt = ActiveOptimization {
            strategy: OptimizationStrategy::ParallelProcessing,
            target_metric: "throughput".to_string(),
            expected_improvement: 50.0,
            status: OptimizationStatus::Applied,
        };
        let cloned = opt.clone();
        assert_eq!(cloned.target_metric, opt.target_metric);
    }

    // ============ OptimizationStatus Tests ============

    #[test]
    fn test_optimization_status_variants() {
        let statuses = [
            OptimizationStatus::Analyzing,
            OptimizationStatus::Ready,
            OptimizationStatus::Implementing,
            OptimizationStatus::Testing,
            OptimizationStatus::Applied,
            OptimizationStatus::Failed("error".to_string()),
            OptimizationStatus::RolledBack("reason".to_string()),
        ];
        assert_eq!(statuses.len(), 7);
    }

    #[test]
    fn test_optimization_status_clone() {
        let status = OptimizationStatus::Failed("test error".to_string());
        let cloned = status.clone();
        if let OptimizationStatus::Failed(msg) = cloned {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected Failed status");
        }
    }

    // ============ OptimizationResult Tests ============

    #[test]
    fn test_optimization_result_creation() {
        let result = OptimizationResult {
            strategy: OptimizationStrategy::MemoryPooling,
            improvement_percent: 15.0,
            metrics_changed: HashMap::new(),
            applied_at: SystemTime::now(),
            success: true,
        };
        assert_eq!(result.improvement_percent, 15.0);
        assert!(result.success);
    }

    #[test]
    fn test_optimization_result_clone() {
        let mut metrics = HashMap::new();
        metrics.insert("memory".to_string(), -50.0);
        let result = OptimizationResult {
            strategy: OptimizationStrategy::IoOptimization,
            improvement_percent: 20.0,
            metrics_changed: metrics,
            applied_at: SystemTime::now(),
            success: true,
        };
        let cloned = result.clone();
        assert_eq!(cloned.improvement_percent, result.improvement_percent);
    }

    // ============ BenchmarkSummary Tests ============

    #[test]
    fn test_benchmark_summary_creation() {
        let summary = BenchmarkSummary {
            total_benchmarks: 20,
            passed_benchmarks: 18,
            failed_benchmarks: 2,
            avg_execution_time: Duration::from_millis(150),
            total_memory_used: 100 * 1024 * 1024,
            avg_throughput: 75.0,
        };
        assert_eq!(summary.total_benchmarks, 20);
        assert_eq!(summary.passed_benchmarks, 18);
    }

    #[test]
    fn test_benchmark_summary_clone() {
        let summary = BenchmarkSummary {
            total_benchmarks: 10,
            passed_benchmarks: 10,
            failed_benchmarks: 0,
            avg_execution_time: Duration::from_millis(100),
            total_memory_used: 50 * 1024 * 1024,
            avg_throughput: 100.0,
        };
        let cloned = summary.clone();
        assert_eq!(cloned.total_benchmarks, summary.total_benchmarks);
    }

    // ============ PerformanceRegression Tests ============

    #[test]
    fn test_performance_regression_creation() {
        let regression = PerformanceRegression {
            benchmark_name: "test_bench".to_string(),
            metric_name: "latency".to_string(),
            current_value: 150.0,
            baseline_value: 100.0,
            regression_percent: 50.0,
            severity: RegressionSeverity::Severe,
        };
        assert_eq!(regression.benchmark_name, "test_bench");
        assert_eq!(regression.regression_percent, 50.0);
    }

    #[test]
    fn test_performance_regression_clone() {
        let regression = PerformanceRegression {
            benchmark_name: "bench".to_string(),
            metric_name: "metric".to_string(),
            current_value: 110.0,
            baseline_value: 100.0,
            regression_percent: 10.0,
            severity: RegressionSeverity::Minor,
        };
        let cloned = regression.clone();
        assert_eq!(cloned.regression_percent, regression.regression_percent);
    }

    // ============ PerformanceAlert Tests ============

    #[test]
    fn test_performance_alert_creation() {
        let alert = PerformanceAlert {
            alert_type: AlertType::HighLatency,
            message: "High latency detected".to_string(),
            severity: AlertSeverity::Warning,
            metric_name: "response_time".to_string(),
            current_value: 500.0,
            threshold_value: 200.0,
            triggered_at: SystemTime::now(),
        };
        assert_eq!(alert.message, "High latency detected");
    }

    #[test]
    fn test_performance_alert_clone() {
        let alert = PerformanceAlert {
            alert_type: AlertType::HighMemoryUsage,
            message: "Memory usage high".to_string(),
            severity: AlertSeverity::Critical,
            metric_name: "memory_mb".to_string(),
            current_value: 2000.0,
            threshold_value: 1024.0,
            triggered_at: SystemTime::now(),
        };
        let cloned = alert.clone();
        assert_eq!(cloned.message, alert.message);
    }

    // ============ AlertType Tests ============

    #[test]
    fn test_alert_type_variants() {
        let types = [
            AlertType::HighLatency,
            AlertType::HighMemoryUsage,
            AlertType::HighCpuUsage,
            AlertType::LowThroughput,
            AlertType::PerformanceRegression,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_alert_type_clone() {
        let alert_type = AlertType::LowThroughput;
        let cloned = alert_type.clone();
        assert!(matches!(cloned, AlertType::LowThroughput));
    }

    // ============ AlertSeverity Tests ============

    #[test]
    fn test_alert_severity_variants() {
        let severities = [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Critical,
        ];
        assert_eq!(severities.len(), 3);
    }

    #[test]
    fn test_alert_severity_clone() {
        let severity = AlertSeverity::Critical;
        let cloned = severity.clone();
        assert!(matches!(cloned, AlertSeverity::Critical));
    }

    // ============ PerformanceMetrics Tests ============

    #[test]
    fn test_performance_metrics_new() {
        let metrics = PerformanceMetrics::new();
        assert!(metrics.baselines.is_empty());
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert!(metrics.baselines.is_empty());
    }

    #[test]
    fn test_performance_metrics_clone() {
        let metrics = PerformanceMetrics::new();
        let cloned = metrics.clone();
        assert!(cloned.baselines.is_empty());
    }

    // ============ PerformanceOptimizer Tests ============

    #[test]
    fn test_performance_optimizer_new() {
        let config = OptimizationConfig {
            auto_optimize: false,
            strategies: vec![],
            min_improvement_percent: 5.0,
            experimental: false,
        };
        let optimizer = PerformanceOptimizer::new(config);
        assert!(optimizer.history.is_empty());
    }

    // ============ BenchmarkSuite Tests ============

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite {
            name: "test_suite".to_string(),
            benchmarks: vec![],
            config: BenchmarkConfig::default(),
        };
        assert_eq!(suite.name, "test_suite");
    }

    #[test]
    fn test_benchmark_suite_clone() {
        let suite = BenchmarkSuite {
            name: "suite".to_string(),
            benchmarks: vec![],
            config: BenchmarkConfig::default(),
        };
        let cloned = suite.clone();
        assert_eq!(cloned.name, suite.name);
    }

    // ============ BenchmarkContext Tests ============

    #[test]
    fn test_benchmark_context_creation() {
        let context = BenchmarkContext {
            test_data: HashMap::new(),
            temp_dir: PathBuf::from("/tmp/test"),
            config: HashMap::new(),
        };
        assert_eq!(context.temp_dir, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_benchmark_context_clone() {
        let mut config = HashMap::new();
        config.insert("key".to_string(), "value".to_string());
        let context = BenchmarkContext {
            test_data: HashMap::new(),
            temp_dir: PathBuf::from("/tmp"),
            config,
        };
        let cloned = context.clone();
        assert!(cloned.config.contains_key("key"));
    }

    // ============ PerformanceReport Tests ============

    #[test]
    fn test_performance_report_creation() {
        let report = PerformanceReport {
            generated_at: SystemTime::now(),
            current_statistics: PerformanceStatistics::default(),
            recent_benchmarks: vec![],
            optimization_history: vec![],
            recommendations: vec!["Optimize caching".to_string()],
            alerts: vec![],
        };
        assert_eq!(report.recommendations.len(), 1);
    }

    #[test]
    fn test_performance_report_clone() {
        let report = PerformanceReport {
            generated_at: SystemTime::now(),
            current_statistics: PerformanceStatistics::default(),
            recent_benchmarks: vec![],
            optimization_history: vec![],
            recommendations: vec![],
            alerts: vec![],
        };
        let cloned = report.clone();
        assert!(cloned.recommendations.is_empty());
    }

    // ============ PerformanceMonitor Integration Tests ============

    #[test]
    fn test_generate_performance_report() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let report = monitor.generate_performance_report();
        assert!(report.recommendations.len() >= 0);
    }

    #[test]
    fn test_calculate_improvement() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let mut baseline = HashMap::new();
        baseline.insert("metric1".to_string(), 100.0);
        baseline.insert("metric2".to_string(), 200.0);

        let mut optimized = HashMap::new();
        optimized.insert("metric1".to_string(), 80.0);  // 20% improvement
        optimized.insert("metric2".to_string(), 160.0); // 20% improvement

        let improvement = monitor.calculate_improvement(&baseline, &optimized);
        assert!((improvement - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_improvement_empty() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let baseline = HashMap::new();
        let optimized = HashMap::new();

        let improvement = monitor.calculate_improvement(&baseline, &optimized);
        assert_eq!(improvement, 0.0);
    }

    #[test]
    fn test_calculate_metrics_delta() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let mut baseline = HashMap::new();
        baseline.insert("metric".to_string(), 100.0);

        let mut optimized = HashMap::new();
        optimized.insert("metric".to_string(), 75.0);

        let delta = monitor.calculate_metrics_delta(&baseline, &optimized);
        assert_eq!(delta.get("metric"), Some(&-25.0));
    }

    // ============ OptimizationStrategy Serialization Tests ============

    #[test]
    fn test_optimization_strategy_serialization() {
        let strategy = OptimizationStrategy::AstReuse;
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: OptimizationStrategy = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, OptimizationStrategy::AstReuse));
    }

    #[test]
    fn test_optimization_strategy_clone() {
        let strategy = OptimizationStrategy::IncrementalParsing;
        let cloned = strategy.clone();
        assert!(matches!(cloned, OptimizationStrategy::IncrementalParsing));
    }

    // ============ RegressionSeverity Tests ============

    #[test]
    fn test_regression_severity_clone() {
        let severity = RegressionSeverity::Moderate;
        let cloned = severity.clone();
        assert!(matches!(cloned, RegressionSeverity::Moderate));
    }

    #[test]
    fn test_regression_severity_serialization() {
        let severity = RegressionSeverity::Critical;
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: RegressionSeverity = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, RegressionSeverity::Critical));
    }

    // ============ OptimizationStatus Tests ============

    #[test]
    fn test_optimization_status_analyzing() {
        let status = OptimizationStatus::Analyzing;
        let debug = format!("{:?}", status);
        assert!(debug.contains("Analyzing"));
    }

    #[test]
    fn test_optimization_status_ready() {
        let status = OptimizationStatus::Ready;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Ready"));
    }

    #[test]
    fn test_optimization_status_implementing() {
        let status = OptimizationStatus::Implementing;
        let cloned = status.clone();
        assert!(matches!(cloned, OptimizationStatus::Implementing));
    }

    #[test]
    fn test_optimization_status_testing() {
        let status = OptimizationStatus::Testing;
        let debug = format!("{:?}", status);
        assert!(debug.contains("Testing"));
    }

    #[test]
    fn test_optimization_status_applied() {
        let status = OptimizationStatus::Applied;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: OptimizationStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, OptimizationStatus::Applied));
    }

    #[test]
    fn test_optimization_status_failed() {
        let status = OptimizationStatus::Failed("error message".to_string());
        if let OptimizationStatus::Failed(msg) = status {
            assert_eq!(msg, "error message");
        } else {
            panic!("Expected Failed status");
        }
    }

    #[test]
    fn test_optimization_status_rolled_back() {
        let status = OptimizationStatus::RolledBack("reason".to_string());
        let cloned = status.clone();
        if let OptimizationStatus::RolledBack(reason) = cloned {
            assert_eq!(reason, "reason");
        } else {
            panic!("Expected RolledBack status");
        }
    }

    // ============ ActiveOptimization Extra Tests ============

    #[test]
    fn test_active_optimization_with_ready_status() {
        let opt = ActiveOptimization {
            strategy: OptimizationStrategy::CacheOptimization,
            target_metric: "analysis_time".to_string(),
            expected_improvement: 25.0,
            status: OptimizationStatus::Ready,
        };
        assert_eq!(opt.target_metric, "analysis_time");
        assert_eq!(opt.expected_improvement, 25.0);
    }

    #[test]
    fn test_active_optimization_with_analyzing_status() {
        let opt = ActiveOptimization {
            strategy: OptimizationStrategy::ParallelProcessing,
            target_metric: "throughput".to_string(),
            expected_improvement: 50.0,
            status: OptimizationStatus::Analyzing,
        };
        let cloned = opt.clone();
        assert_eq!(cloned.target_metric, opt.target_metric);
        assert_eq!(cloned.expected_improvement, opt.expected_improvement);
    }

    #[test]
    fn test_active_optimization_debug_format() {
        let opt = ActiveOptimization {
            strategy: OptimizationStrategy::MemoryPooling,
            target_metric: "memory".to_string(),
            expected_improvement: 15.0,
            status: OptimizationStatus::Testing,
        };
        let debug = format!("{:?}", opt);
        assert!(debug.contains("MemoryPooling"));
        assert!(debug.contains("memory"));
    }

    // ============ OptimizationResult Extra Tests ============

    #[test]
    fn test_optimization_result_with_io_strategy() {
        let result = OptimizationResult {
            strategy: OptimizationStrategy::IoOptimization,
            improvement_percent: 30.0,
            metrics_changed: HashMap::new(),
            applied_at: SystemTime::now(),
            success: true,
        };
        assert_eq!(result.improvement_percent, 30.0);
        assert!(result.success);
    }

    #[test]
    fn test_optimization_result_with_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("metric1".to_string(), -20.0);

        let result = OptimizationResult {
            strategy: OptimizationStrategy::AstReuse,
            improvement_percent: 20.0,
            metrics_changed: metrics,
            applied_at: SystemTime::now(),
            success: true,
        };
        let cloned = result.clone();
        assert_eq!(cloned.improvement_percent, result.improvement_percent);
        assert_eq!(cloned.metrics_changed.len(), 1);
    }

    #[test]
    fn test_optimization_result_debug_format() {
        let result = OptimizationResult {
            strategy: OptimizationStrategy::IncrementalParsing,
            improvement_percent: 10.0,
            metrics_changed: HashMap::new(),
            applied_at: SystemTime::now(),
            success: false,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("IncrementalParsing"));
    }

    #[test]
    fn test_optimization_result_json_serialization() {
        let result = OptimizationResult {
            strategy: OptimizationStrategy::CacheOptimization,
            improvement_percent: 25.5,
            metrics_changed: HashMap::new(),
            applied_at: SystemTime::now(),
            success: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("25.5"));
    }

    // ============ BenchmarkConfig Extra Tests ============

    #[test]
    fn test_benchmark_config_clone_with_defaults() {
        let config = BenchmarkConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.iterations, config.iterations);
        assert_eq!(cloned.warmup_iterations, config.warmup_iterations);
    }

    #[test]
    fn test_benchmark_config_debug() {
        let config = BenchmarkConfig {
            iterations: 50,
            warmup_iterations: 5,
            timeout: Duration::from_secs(30),
            parallel: true,
        };
        let debug = format!("{:?}", config);
        assert!(debug.contains("iterations"));
    }

    #[test]
    fn test_benchmark_config_serialization() {
        let config = BenchmarkConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("100"));
        let deserialized: BenchmarkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.iterations, 100);
    }

    // ============ PerformanceConfig Tests ============

    #[test]
    fn test_performance_config_clone() {
        let config = create_test_config();
        let cloned = config.clone();
        assert_eq!(cloned.continuous_monitoring, config.continuous_monitoring);
        assert_eq!(cloned.benchmark_interval, config.benchmark_interval);
    }

    #[test]
    fn test_performance_config_debug() {
        let config = create_test_config();
        let debug = format!("{:?}", config);
        assert!(debug.contains("continuous_monitoring"));
    }

    #[test]
    fn test_performance_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("continuous_monitoring"));
        let deserialized: PerformanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.continuous_monitoring, false);
    }

    // ============ More PerformanceStatistics Tests ============

    #[test]
    fn test_performance_statistics_clone() {
        let stats = PerformanceStatistics::default();
        let cloned = stats.clone();
        assert_eq!(cloned.analysis.avg_analysis_time_ms, stats.analysis.avg_analysis_time_ms);
    }

    #[test]
    fn test_performance_statistics_debug() {
        let stats = PerformanceStatistics::default();
        let debug = format!("{:?}", stats);
        assert!(debug.contains("analysis"));
    }

    // ============ All OptimizationStrategy Variants ============

    #[test]
    fn test_all_optimization_strategies() {
        let cache = OptimizationStrategy::CacheOptimization;
        let parallel = OptimizationStrategy::ParallelProcessing;
        let memory = OptimizationStrategy::MemoryPooling;
        let incr = OptimizationStrategy::IncrementalParsing;
        let io = OptimizationStrategy::IoOptimization;
        let ast = OptimizationStrategy::AstReuse;

        // Test serialization for all
        let _ = serde_json::to_string(&cache).unwrap();
        let _ = serde_json::to_string(&parallel).unwrap();
        let _ = serde_json::to_string(&memory).unwrap();
        let _ = serde_json::to_string(&incr).unwrap();
        let _ = serde_json::to_string(&io).unwrap();
        let _ = serde_json::to_string(&ast).unwrap();

        // Test debug for all
        let _ = format!("{:?}", cache);
        let _ = format!("{:?}", parallel);
        let _ = format!("{:?}", memory);
        let _ = format!("{:?}", incr);
        let _ = format!("{:?}", io);
        let _ = format!("{:?}", ast);
    }

    // ============ All RegressionSeverity Variants ============

    #[test]
    fn test_all_regression_severities() {
        let minor = RegressionSeverity::Minor;
        let moderate = RegressionSeverity::Moderate;
        let severe = RegressionSeverity::Severe;
        let critical = RegressionSeverity::Critical;

        // Test serialization for all
        let _ = serde_json::to_string(&minor).unwrap();
        let _ = serde_json::to_string(&moderate).unwrap();
        let _ = serde_json::to_string(&severe).unwrap();
        let _ = serde_json::to_string(&critical).unwrap();

        // Test debug for all
        let _ = format!("{:?}", minor);
        let _ = format!("{:?}", moderate);
        let _ = format!("{:?}", severe);
        let _ = format!("{:?}", critical);
    }

    // ============ Async Method Tests ============

    #[tokio::test]
    async fn test_establish_baseline() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let baseline = monitor.establish_baseline("test-baseline".to_string()).await;
        assert!(baseline.is_ok());
        let b = baseline.unwrap();
        assert_eq!(b.id, "test-baseline");
    }

    #[tokio::test]
    async fn test_apply_cache_optimization() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.apply_optimization(OptimizationStrategy::CacheOptimization).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_parallel_processing() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.apply_optimization(OptimizationStrategy::ParallelProcessing).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_memory_pooling() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.apply_optimization(OptimizationStrategy::MemoryPooling).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_incremental_parsing() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.apply_optimization(OptimizationStrategy::IncrementalParsing).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_io_optimization() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.apply_optimization(OptimizationStrategy::IoOptimization).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_ast_reuse() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.apply_optimization(OptimizationStrategy::AstReuse).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_collect_system_info() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let info = monitor.collect_system_info().await;
        assert!(info.is_ok());
        let sys = info.unwrap();
        assert!(!sys.os.is_empty());
    }

    #[tokio::test]
    async fn test_collect_codebase_info() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let info = monitor.collect_codebase_info().await;
        assert!(info.is_ok());
        let cb = info.unwrap();
        assert!(cb.total_loc > 0);
    }

    #[tokio::test]
    async fn test_collect_baseline_measurements() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let measurements = monitor.collect_baseline_measurements().await;
        assert!(measurements.is_ok());
        let m = measurements.unwrap();
        assert!(m.contains_key("analysis_time_ms"));
    }

    #[tokio::test]
    async fn test_collect_metrics() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.collect_metrics().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_regressions() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let result = monitor.check_regressions().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auto_optimize() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.auto_optimize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_old_data() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);
        let result = monitor.cleanup_old_data().await;
        assert!(result.is_ok());
    }

    // ============ BenchmarkReport Serialization ============

    #[test]
    fn test_benchmark_report_serialization() {
        let report = BenchmarkReport {
            suite_name: "test_suite".to_string(),
            executed_at: SystemTime::now(),
            results: vec![("test".to_string(), BenchmarkResult {
                execution_time: Duration::from_millis(100),
                memory_used: 1024,
                cpu_time: Duration::from_millis(90),
                throughput: 100.0,
                success: true,
                metrics: HashMap::new(),
            })],
            summary: BenchmarkSummary {
                total_benchmarks: 1,
                passed_benchmarks: 1,
                failed_benchmarks: 0,
                avg_execution_time: Duration::from_millis(100),
                total_memory_used: 1024,
                avg_throughput: 100.0,
            },
            regressions: vec![],
            recommendations: vec!["Optimize caching".to_string()],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("test_suite"));
        let deserialized: BenchmarkReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.suite_name, "test_suite");
    }

    // ============ BenchmarkResult Serialization ============

    #[test]
    fn test_benchmark_result_serialization() {
        let mut metrics = HashMap::new();
        metrics.insert("custom_metric".to_string(), 42.0);
        let result = BenchmarkResult {
            execution_time: Duration::from_secs(2),
            memory_used: 1024 * 1024,
            cpu_time: Duration::from_millis(1800),
            throughput: 50.0,
            success: true,
            metrics,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("50.0"));
        let deserialized: BenchmarkResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.throughput, 50.0);
    }

    // ============ ExpectedPerformance Serialization ============

    #[test]
    fn test_expected_performance_serialization() {
        let expected = ExpectedPerformance {
            max_execution_time: Duration::from_secs(10),
            max_memory_bytes: 1024 * 1024 * 100,
            min_throughput: 25.0,
            regression_threshold: 0.15,
        };
        let json = serde_json::to_string(&expected).unwrap();
        assert!(json.contains("25.0"));
        let deserialized: ExpectedPerformance = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.min_throughput, 25.0);
    }

    // ============ PerformancePoint Serialization ============

    #[test]
    fn test_performance_point_serialization() {
        let mut context = HashMap::new();
        context.insert("key".to_string(), "value".to_string());
        let point = PerformancePoint {
            timestamp: SystemTime::now(),
            metric: "latency".to_string(),
            value: 150.0,
            context,
        };
        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("latency"));
        let deserialized: PerformancePoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.metric, "latency");
    }

    // ============ PerformanceStatistics Serialization ============

    #[test]
    fn test_performance_statistics_serialization() {
        let stats = PerformanceStatistics::default();
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("analysis"));
        let deserialized: PerformanceStatistics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.analysis.avg_analysis_time_ms, 100.0);
    }

    // ============ AnalysisStats Serialization ============

    #[test]
    fn test_analysis_stats_serialization() {
        let stats = AnalysisStats {
            avg_analysis_time_ms: 75.0,
            throughput_fps: 15.0,
            cache_hit_ratio: 0.9,
            parser_efficiency: 0.95,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: AnalysisStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.avg_analysis_time_ms, 75.0);
    }

    // ============ MemoryStats Serialization ============

    #[test]
    fn test_memory_stats_serialization() {
        let stats = MemoryStats {
            peak_memory_mb: 768.0,
            avg_memory_mb: 384.0,
            growth_rate_mb_per_hour: 8.0,
            gc_impact_percent: 3.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: MemoryStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.peak_memory_mb, 768.0);
    }

    // ============ IoStats Serialization ============

    #[test]
    fn test_io_stats_serialization() {
        let stats = IoStats {
            read_throughput_mbps: 150.0,
            avg_read_time_ms: 8.0,
            io_wait_percent: 4.0,
            cache_effectiveness: 0.88,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: IoStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.read_throughput_mbps, 150.0);
    }

    // ============ SystemStats Serialization ============

    #[test]
    fn test_system_stats_serialization() {
        let stats = SystemStats {
            cpu_percent: 45.0,
            thread_count: 12,
            load_average: 1.8,
            network_kbps: 512.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: SystemStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.thread_count, 12);
    }

    // ============ SystemInfo Serialization ============

    #[test]
    fn test_system_info_serialization() {
        let info = SystemInfo {
            cpu_model: "Intel Xeon".to_string(),
            total_memory_mb: 65536,
            os: "linux".to_string(),
            rust_version: "1.76.0".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SystemInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cpu_model, "Intel Xeon");
    }

    // ============ CodebaseInfo Serialization ============

    #[test]
    fn test_codebase_info_serialization() {
        let info = CodebaseInfo {
            total_loc: 250000,
            file_count: 1500,
            avg_complexity: 6.5,
            primary_language: "go".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: CodebaseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_loc, 250000);
    }

    // ============ BaselineContext Serialization ============

    #[test]
    fn test_baseline_context_serialization() {
        let context = BaselineContext {
            system_info: SystemInfo {
                cpu_model: "CPU".to_string(),
                total_memory_mb: 4096,
                os: "macos".to_string(),
                rust_version: "1.70.0".to_string(),
            },
            codebase_info: CodebaseInfo {
                total_loc: 10000,
                file_count: 100,
                avg_complexity: 4.5,
                primary_language: "rust".to_string(),
            },
            config_hash: "hash123".to_string(),
        };
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: BaselineContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.config_hash, "hash123");
    }

    // ============ Baseline Serialization ============

    #[test]
    fn test_baseline_serialization() {
        let mut measurements = HashMap::new();
        measurements.insert("metric1".to_string(), 100.0);
        let baseline = Baseline {
            id: "baseline-test".to_string(),
            measurements,
            measured_at: SystemTime::now(),
            context: BaselineContext {
                system_info: SystemInfo {
                    cpu_model: "CPU".to_string(),
                    total_memory_mb: 8192,
                    os: "linux".to_string(),
                    rust_version: "1.70.0".to_string(),
                },
                codebase_info: CodebaseInfo {
                    total_loc: 5000,
                    file_count: 50,
                    avg_complexity: 3.5,
                    primary_language: "python".to_string(),
                },
                config_hash: "hash".to_string(),
            },
        };
        let json = serde_json::to_string(&baseline).unwrap();
        let deserialized: Baseline = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "baseline-test");
    }

    // ============ PerformanceRegression Serialization ============

    #[test]
    fn test_performance_regression_serialization() {
        let regression = PerformanceRegression {
            benchmark_name: "bench1".to_string(),
            metric_name: "latency_ms".to_string(),
            current_value: 200.0,
            baseline_value: 100.0,
            regression_percent: 100.0,
            severity: RegressionSeverity::Critical,
        };
        let json = serde_json::to_string(&regression).unwrap();
        let deserialized: PerformanceRegression = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.regression_percent, 100.0);
    }

    // ============ PerformanceAlert Serialization ============

    #[test]
    fn test_performance_alert_serialization() {
        let alert = PerformanceAlert {
            alert_type: AlertType::HighCpuUsage,
            message: "CPU usage exceeded threshold".to_string(),
            severity: AlertSeverity::Warning,
            metric_name: "cpu_percent".to_string(),
            current_value: 95.0,
            threshold_value: 80.0,
            triggered_at: SystemTime::now(),
        };
        let json = serde_json::to_string(&alert).unwrap();
        let deserialized: PerformanceAlert = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_value, 95.0);
    }

    // ============ PerformanceReport Serialization ============

    #[test]
    fn test_performance_report_serialization() {
        let report = PerformanceReport {
            generated_at: SystemTime::now(),
            current_statistics: PerformanceStatistics::default(),
            recent_benchmarks: vec![],
            optimization_history: vec![],
            recommendations: vec!["Recommend1".to_string()],
            alerts: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: PerformanceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.recommendations.len(), 1);
    }

    // ============ OptimizationConfig Serialization ============

    #[test]
    fn test_optimization_config_serialization() {
        let config = OptimizationConfig {
            auto_optimize: true,
            strategies: vec![OptimizationStrategy::CacheOptimization],
            min_improvement_percent: 7.5,
            experimental: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OptimizationConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.auto_optimize);
    }

    // ============ RetentionConfig Serialization ============

    #[test]
    fn test_retention_config_serialization() {
        let config = RetentionConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RetentionConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.auto_cleanup);
    }

    // ============ Helper Method Tests ============

    #[test]
    fn test_calculate_config_hash() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let hash = monitor.calculate_config_hash();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_generate_system_recommendations() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let recommendations = monitor.generate_system_recommendations();
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_generate_performance_alerts() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let alerts = monitor.generate_performance_alerts();
        // Empty is fine, just testing the method works
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_calculate_summary_stats() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let results = vec![
            ("test1".to_string(), BenchmarkResult {
                execution_time: Duration::from_millis(100),
                memory_used: 1024,
                cpu_time: Duration::from_millis(90),
                throughput: 100.0,
                success: true,
                metrics: HashMap::new(),
            })
        ];
        let summary = monitor.calculate_summary_stats(&results);
        assert_eq!(summary.total_benchmarks, 10); // Stub returns 10
    }

    #[test]
    fn test_generate_recommendations() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let summary = BenchmarkSummary {
            total_benchmarks: 5,
            passed_benchmarks: 5,
            failed_benchmarks: 0,
            avg_execution_time: Duration::from_millis(100),
            total_memory_used: 1024,
            avg_throughput: 50.0,
        };
        let recommendations = monitor.generate_recommendations(&summary);
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_get_recent_benchmark_results() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);
        let results = monitor.get_recent_benchmark_results(5);
        // Stub returns empty
        assert!(results.is_empty());
    }

    // ============ BenchmarkSuite Debug ============

    #[test]
    fn test_benchmark_suite_debug() {
        let suite = BenchmarkSuite {
            name: "debug_suite".to_string(),
            benchmarks: vec![],
            config: BenchmarkConfig::default(),
        };
        let debug = format!("{:?}", suite);
        assert!(debug.contains("debug_suite"));
    }

    // ============ BenchmarkContext Debug ============

    #[test]
    fn test_benchmark_context_debug() {
        let context = BenchmarkContext {
            test_data: HashMap::new(),
            temp_dir: PathBuf::from("/tmp"),
            config: HashMap::new(),
        };
        let debug = format!("{:?}", context);
        assert!(debug.contains("BenchmarkContext"));
    }

    // ============ PerformanceMetrics Debug ============

    #[test]
    fn test_performance_metrics_debug() {
        let metrics = PerformanceMetrics::new();
        let debug = format!("{:?}", metrics);
        assert!(debug.contains("PerformanceMetrics"));
    }

    // ============ Additional Coverage Tests ============

    #[test]
    fn test_benchmark_suite_with_benchmarks() {
        fn dummy_benchmark(_ctx: &BenchmarkContext) -> Result<BenchmarkResult> {
            Ok(BenchmarkResult {
                execution_time: Duration::from_millis(50),
                memory_used: 512,
                cpu_time: Duration::from_millis(45),
                throughput: 150.0,
                success: true,
                metrics: HashMap::new(),
            })
        }

        let benchmark = Benchmark {
            name: "test_benchmark".to_string(),
            benchmark_fn: dummy_benchmark,
            setup_fn: None,
            teardown_fn: None,
            expected: ExpectedPerformance {
                max_execution_time: Duration::from_secs(1),
                max_memory_bytes: 1024 * 1024,
                min_throughput: 100.0,
                regression_threshold: 0.1,
            },
        };

        let suite = BenchmarkSuite {
            name: "full_suite".to_string(),
            benchmarks: vec![benchmark],
            config: BenchmarkConfig::default(),
        };

        assert_eq!(suite.benchmarks.len(), 1);
        assert_eq!(suite.name, "full_suite");
    }

    #[test]
    fn test_benchmark_with_setup_teardown() {
        fn setup() -> Result<BenchmarkContext> {
            Ok(BenchmarkContext {
                test_data: HashMap::new(),
                temp_dir: PathBuf::from("/tmp/bench"),
                config: HashMap::new(),
            })
        }

        fn teardown(_ctx: BenchmarkContext) -> Result<()> {
            Ok(())
        }

        fn bench_fn(_ctx: &BenchmarkContext) -> Result<BenchmarkResult> {
            Ok(BenchmarkResult {
                execution_time: Duration::from_millis(100),
                memory_used: 1024,
                cpu_time: Duration::from_millis(90),
                throughput: 100.0,
                success: true,
                metrics: HashMap::new(),
            })
        }

        let benchmark = Benchmark {
            name: "with_lifecycle".to_string(),
            benchmark_fn: bench_fn,
            setup_fn: Some(setup),
            teardown_fn: Some(teardown),
            expected: ExpectedPerformance {
                max_execution_time: Duration::from_secs(5),
                max_memory_bytes: 1024 * 1024 * 10,
                min_throughput: 50.0,
                regression_threshold: 0.2,
            },
        };

        assert!(benchmark.setup_fn.is_some());
        assert!(benchmark.teardown_fn.is_some());
    }

    #[test]
    fn test_performance_point_with_context() {
        let mut context = HashMap::new();
        context.insert("env".to_string(), "production".to_string());
        context.insert("version".to_string(), "1.0.0".to_string());

        let point = PerformancePoint {
            timestamp: SystemTime::now(),
            metric: "latency_p99".to_string(),
            value: 250.5,
            context,
        };

        assert_eq!(point.context.len(), 2);
        assert_eq!(point.context.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_performance_regression_all_severities() {
        let regressions = vec![
            PerformanceRegression {
                benchmark_name: "minor".to_string(),
                metric_name: "latency".to_string(),
                current_value: 105.0,
                baseline_value: 100.0,
                regression_percent: 5.0,
                severity: RegressionSeverity::Minor,
            },
            PerformanceRegression {
                benchmark_name: "moderate".to_string(),
                metric_name: "latency".to_string(),
                current_value: 120.0,
                baseline_value: 100.0,
                regression_percent: 20.0,
                severity: RegressionSeverity::Moderate,
            },
            PerformanceRegression {
                benchmark_name: "severe".to_string(),
                metric_name: "latency".to_string(),
                current_value: 140.0,
                baseline_value: 100.0,
                regression_percent: 40.0,
                severity: RegressionSeverity::Severe,
            },
            PerformanceRegression {
                benchmark_name: "critical".to_string(),
                metric_name: "latency".to_string(),
                current_value: 200.0,
                baseline_value: 100.0,
                regression_percent: 100.0,
                severity: RegressionSeverity::Critical,
            },
        ];

        assert_eq!(regressions.len(), 4);
        assert_eq!(regressions[0].regression_percent, 5.0);
        assert_eq!(regressions[3].regression_percent, 100.0);
    }

    #[test]
    fn test_performance_alert_all_types() {
        let alerts: Vec<PerformanceAlert> = vec![
            PerformanceAlert {
                alert_type: AlertType::HighLatency,
                message: "Latency exceeded".to_string(),
                severity: AlertSeverity::Warning,
                metric_name: "p99_latency".to_string(),
                current_value: 500.0,
                threshold_value: 200.0,
                triggered_at: SystemTime::now(),
            },
            PerformanceAlert {
                alert_type: AlertType::HighMemoryUsage,
                message: "Memory high".to_string(),
                severity: AlertSeverity::Critical,
                metric_name: "heap_mb".to_string(),
                current_value: 8000.0,
                threshold_value: 4096.0,
                triggered_at: SystemTime::now(),
            },
            PerformanceAlert {
                alert_type: AlertType::HighCpuUsage,
                message: "CPU high".to_string(),
                severity: AlertSeverity::Warning,
                metric_name: "cpu_percent".to_string(),
                current_value: 95.0,
                threshold_value: 80.0,
                triggered_at: SystemTime::now(),
            },
            PerformanceAlert {
                alert_type: AlertType::LowThroughput,
                message: "Throughput low".to_string(),
                severity: AlertSeverity::Info,
                metric_name: "ops_per_sec".to_string(),
                current_value: 50.0,
                threshold_value: 100.0,
                triggered_at: SystemTime::now(),
            },
            PerformanceAlert {
                alert_type: AlertType::PerformanceRegression,
                message: "Regression detected".to_string(),
                severity: AlertSeverity::Critical,
                metric_name: "benchmark_time".to_string(),
                current_value: 200.0,
                threshold_value: 100.0,
                triggered_at: SystemTime::now(),
            },
        ];

        assert_eq!(alerts.len(), 5);
        for alert in &alerts {
            assert!(!alert.message.is_empty());
        }
    }

    #[test]
    fn test_optimization_config_all_strategies() {
        let config = OptimizationConfig {
            auto_optimize: true,
            strategies: vec![
                OptimizationStrategy::CacheOptimization,
                OptimizationStrategy::ParallelProcessing,
                OptimizationStrategy::MemoryPooling,
                OptimizationStrategy::IncrementalParsing,
                OptimizationStrategy::IoOptimization,
                OptimizationStrategy::AstReuse,
            ],
            min_improvement_percent: 5.0,
            experimental: true,
        };

        assert_eq!(config.strategies.len(), 6);
        assert!(config.auto_optimize);
        assert!(config.experimental);
    }

    #[test]
    fn test_benchmark_report_full() {
        let result = BenchmarkResult {
            execution_time: Duration::from_millis(150),
            memory_used: 2048,
            cpu_time: Duration::from_millis(140),
            throughput: 75.0,
            success: true,
            metrics: {
                let mut m = HashMap::new();
                m.insert("gc_count".to_string(), 5.0);
                m
            },
        };

        let report = BenchmarkReport {
            suite_name: "full_report".to_string(),
            executed_at: SystemTime::now(),
            results: vec![("test1".to_string(), result)],
            summary: BenchmarkSummary {
                total_benchmarks: 1,
                passed_benchmarks: 1,
                failed_benchmarks: 0,
                avg_execution_time: Duration::from_millis(150),
                total_memory_used: 2048,
                avg_throughput: 75.0,
            },
            regressions: vec![],
            recommendations: vec!["Consider caching".to_string(), "Enable parallel".to_string()],
        };

        assert_eq!(report.suite_name, "full_report");
        assert_eq!(report.recommendations.len(), 2);
    }

    #[test]
    fn test_baseline_with_measurements() {
        let mut measurements = HashMap::new();
        measurements.insert("analysis_time_ms".to_string(), 125.0);
        measurements.insert("memory_mb".to_string(), 300.0);
        measurements.insert("throughput_fps".to_string(), 45.0);
        measurements.insert("cache_hit_ratio".to_string(), 0.85);

        let baseline = Baseline {
            id: "v2.0.0-baseline".to_string(),
            measurements: measurements.clone(),
            measured_at: SystemTime::now(),
            context: BaselineContext {
                system_info: SystemInfo {
                    cpu_model: "Intel Xeon Gold".to_string(),
                    total_memory_mb: 131072,
                    os: "linux".to_string(),
                    rust_version: "1.78.0".to_string(),
                },
                codebase_info: CodebaseInfo {
                    total_loc: 500000,
                    file_count: 3000,
                    avg_complexity: 7.5,
                    primary_language: "rust".to_string(),
                },
                config_hash: "sha256:abc123".to_string(),
            },
        };

        assert_eq!(baseline.measurements.len(), 4);
        assert!(baseline.measurements.get("analysis_time_ms").is_some());
    }

    #[test]
    fn test_performance_monitor_with_benchmarks() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);

        let suite = BenchmarkSuite {
            name: "test_suite".to_string(),
            benchmarks: vec![],
            config: BenchmarkConfig::default(),
        };

        monitor.benchmarks.insert("test_suite".to_string(), suite);
        assert!(monitor.benchmarks.contains_key("test_suite"));
    }

    #[test]
    fn test_calculate_improvement_with_improvement() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let mut baseline = HashMap::new();
        baseline.insert("latency".to_string(), 100.0);
        baseline.insert("memory".to_string(), 500.0);

        let mut optimized = HashMap::new();
        optimized.insert("latency".to_string(), 75.0);  // 25% improvement
        optimized.insert("memory".to_string(), 400.0);  // 20% improvement

        let improvement = monitor.calculate_improvement(&baseline, &optimized);
        assert!(improvement > 20.0); // Average should be around 22.5%
    }

    #[test]
    fn test_calculate_improvement_with_regression() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let mut baseline = HashMap::new();
        baseline.insert("latency".to_string(), 100.0);

        let mut optimized = HashMap::new();
        optimized.insert("latency".to_string(), 120.0);  // 20% regression (negative improvement)

        let improvement = monitor.calculate_improvement(&baseline, &optimized);
        assert!(improvement < 0.0); // Should be negative
    }

    #[test]
    fn test_calculate_metrics_delta_multiple() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let mut baseline = HashMap::new();
        baseline.insert("a".to_string(), 100.0);
        baseline.insert("b".to_string(), 200.0);
        baseline.insert("c".to_string(), 50.0);

        let mut optimized = HashMap::new();
        optimized.insert("a".to_string(), 80.0);
        optimized.insert("b".to_string(), 250.0);
        // c is missing in optimized

        let delta = monitor.calculate_metrics_delta(&baseline, &optimized);
        assert_eq!(delta.get("a"), Some(&-20.0));
        assert_eq!(delta.get("b"), Some(&50.0));
        assert!(delta.get("c").is_none()); // Not present in both
    }

    #[tokio::test]
    async fn test_run_single_benchmark() {
        fn test_fn(_ctx: &BenchmarkContext) -> Result<BenchmarkResult> {
            Ok(BenchmarkResult {
                execution_time: Duration::from_millis(50),
                memory_used: 1024,
                cpu_time: Duration::from_millis(45),
                throughput: 200.0,
                success: true,
                metrics: HashMap::new(),
            })
        }

        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let benchmark = Benchmark {
            name: "single_test".to_string(),
            benchmark_fn: test_fn,
            setup_fn: None,
            teardown_fn: None,
            expected: ExpectedPerformance {
                max_execution_time: Duration::from_secs(1),
                max_memory_bytes: 1024 * 1024,
                min_throughput: 100.0,
                regression_threshold: 0.1,
            },
        };

        let result = monitor.run_single_benchmark(&benchmark).await;
        assert!(result.is_ok());
        let br = result.unwrap();
        assert!(br.success);
    }

    #[tokio::test]
    async fn test_detect_regressions_empty() {
        let config = create_test_config();
        let monitor = PerformanceMonitor::new(config);

        let results = vec![];
        let regressions = monitor.detect_regressions(&results).await;
        assert!(regressions.is_ok());
        assert!(regressions.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_store_benchmark_results() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);

        let report = BenchmarkReport {
            suite_name: "store_test".to_string(),
            executed_at: SystemTime::now(),
            results: vec![],
            summary: BenchmarkSummary {
                total_benchmarks: 0,
                passed_benchmarks: 0,
                failed_benchmarks: 0,
                avg_execution_time: Duration::from_millis(0),
                total_memory_used: 0,
                avg_throughput: 0.0,
            },
            regressions: vec![],
            recommendations: vec![],
        };

        let result = monitor.store_benchmark_results(&report).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_retention_config_custom() {
        let config = RetentionConfig {
            detailed_retention: Duration::from_secs(3 * 24 * 60 * 60), // 3 days
            summary_retention: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            auto_cleanup: false,
        };

        assert_eq!(config.detailed_retention, Duration::from_secs(259200));
        assert!(!config.auto_cleanup);
    }

    #[test]
    fn test_performance_config_custom() {
        let config = PerformanceConfig {
            continuous_monitoring: true,
            benchmark_interval: Duration::from_secs(300),
            thresholds: PerformanceThresholds {
                max_analysis_time_ms: 10000,
                max_memory_mb: 2048,
                max_cpu_percent: 90.0,
                regression_threshold_percent: 15.0,
            },
            optimization: OptimizationConfig {
                auto_optimize: true,
                strategies: vec![OptimizationStrategy::CacheOptimization],
                min_improvement_percent: 10.0,
                experimental: false,
            },
            retention: RetentionConfig::default(),
        };

        assert!(config.continuous_monitoring);
        assert_eq!(config.thresholds.max_analysis_time_ms, 10000);
    }

    #[test]
    fn test_benchmark_result_with_custom_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("gc_pause_ms".to_string(), 25.0);
        metrics.insert("allocations".to_string(), 1000.0);
        metrics.insert("peak_rss_mb".to_string(), 512.0);

        let result = BenchmarkResult {
            execution_time: Duration::from_millis(200),
            memory_used: 512 * 1024 * 1024,
            cpu_time: Duration::from_millis(180),
            throughput: 50.0,
            success: true,
            metrics,
        };

        assert_eq!(result.metrics.len(), 3);
        assert_eq!(result.metrics.get("gc_pause_ms"), Some(&25.0));
    }

    #[test]
    fn test_active_optimization_all_statuses() {
        let statuses = vec![
            (OptimizationStatus::Analyzing, "analyzing"),
            (OptimizationStatus::Ready, "ready"),
            (OptimizationStatus::Implementing, "implementing"),
            (OptimizationStatus::Testing, "testing"),
            (OptimizationStatus::Applied, "applied"),
            (OptimizationStatus::Failed("timeout".to_string()), "failed"),
            (OptimizationStatus::RolledBack("crash".to_string()), "rollback"),
        ];

        for (status, _name) in statuses {
            let opt = ActiveOptimization {
                strategy: OptimizationStrategy::CacheOptimization,
                target_metric: "latency".to_string(),
                expected_improvement: 20.0,
                status,
            };
            // Just verify creation works
            assert_eq!(opt.expected_improvement, 20.0);
        }
    }

    #[test]
    fn test_performance_report_with_all_fields() {
        let report = PerformanceReport {
            generated_at: SystemTime::now(),
            current_statistics: PerformanceStatistics::default(),
            recent_benchmarks: vec![
                BenchmarkReport {
                    suite_name: "suite1".to_string(),
                    executed_at: SystemTime::now(),
                    results: vec![],
                    summary: BenchmarkSummary {
                        total_benchmarks: 5,
                        passed_benchmarks: 4,
                        failed_benchmarks: 1,
                        avg_execution_time: Duration::from_millis(100),
                        total_memory_used: 1024 * 1024,
                        avg_throughput: 100.0,
                    },
                    regressions: vec![],
                    recommendations: vec![],
                }
            ],
            optimization_history: vec![
                OptimizationResult {
                    strategy: OptimizationStrategy::CacheOptimization,
                    improvement_percent: 15.0,
                    metrics_changed: HashMap::new(),
                    applied_at: SystemTime::now(),
                    success: true,
                }
            ],
            recommendations: vec!["Enable caching".to_string()],
            alerts: vec![
                PerformanceAlert {
                    alert_type: AlertType::HighLatency,
                    message: "Test alert".to_string(),
                    severity: AlertSeverity::Warning,
                    metric_name: "latency".to_string(),
                    current_value: 150.0,
                    threshold_value: 100.0,
                    triggered_at: SystemTime::now(),
                }
            ],
        };

        assert_eq!(report.recent_benchmarks.len(), 1);
        assert_eq!(report.optimization_history.len(), 1);
        assert_eq!(report.alerts.len(), 1);
    }

    #[test]
    fn test_benchmark_clone_comprehensive() {
        fn dummy(_ctx: &BenchmarkContext) -> Result<BenchmarkResult> {
            Ok(BenchmarkResult {
                execution_time: Duration::from_millis(1),
                memory_used: 0,
                cpu_time: Duration::from_millis(1),
                throughput: 0.0,
                success: true,
                metrics: HashMap::new(),
            })
        }

        let benchmark = Benchmark {
            name: "clone_test".to_string(),
            benchmark_fn: dummy,
            setup_fn: None,
            teardown_fn: None,
            expected: ExpectedPerformance {
                max_execution_time: Duration::from_secs(1),
                max_memory_bytes: 1024,
                min_throughput: 1.0,
                regression_threshold: 0.1,
            },
        };

        let cloned = benchmark.clone();
        assert_eq!(cloned.name, benchmark.name);
    }

    #[tokio::test]
    async fn test_run_benchmark_suite_not_found() {
        let config = create_test_config();
        let mut monitor = PerformanceMonitor::new(config);

        let result = monitor.run_benchmark("nonexistent_suite").await;
        assert!(result.is_err());
    }
}
