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
        assert_eq!(
            deserialized.max_analysis_time_ms,
            thresholds.max_analysis_time_ms
        );
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
        assert_eq!(
            config.detailed_retention,
            Duration::from_secs(7 * 24 * 60 * 60)
        );
        assert_eq!(
            config.summary_retention,
            Duration::from_secs(90 * 24 * 60 * 60)
        );
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
        optimized.insert("metric1".to_string(), 80.0); // 20% improvement
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
        assert_eq!(
            cloned.analysis.avg_analysis_time_ms,
            stats.analysis.avg_analysis_time_ms
        );
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

    // Part 2 tests extracted to performance_tests_part2.rs for file health compliance (CB-040)
}

#[path = "performance_tests_part2.rs"]
mod performance_tests_part2;
