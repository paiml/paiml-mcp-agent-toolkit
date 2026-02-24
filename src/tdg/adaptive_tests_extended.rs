fn create_sample_full(
    duration_ms: u64,
    cache_hit_ratio: f32,
    memory_mb: f32,
    cpu: f32,
    queue: usize,
) -> PerformanceSample {
    PerformanceSample {
        timestamp: Instant::now(),
        analysis_duration_ms: duration_ms,
        cache_hit_ratio,
        memory_usage_mb: memory_mb,
        cpu_utilization: cpu,
        queue_depth: queue,
    }
}

// =============================================================================
// ADAPTIVE CONFIG TESTS
// =============================================================================

#[test]
fn test_adaptive_config_default() {
    let config = AdaptiveConfig::default();
    assert_eq!(config.target_analysis_time_ms, 100);
    assert_eq!(config.min_cache_hit_ratio, 0.6);
    assert_eq!(config.max_memory_mb, 512.0);
    assert_eq!(config.max_cpu_utilization, 0.8);
    assert_eq!(config.sample_window_size, 50);
    assert_eq!(config.adjustment_sensitivity, 0.1);
}

#[test]
fn test_adaptive_config_clone() {
    let config = AdaptiveConfig::default();
    let cloned = config.clone();
    assert_eq!(
        cloned.target_analysis_time_ms,
        config.target_analysis_time_ms
    );
    assert_eq!(cloned.min_cache_hit_ratio, config.min_cache_hit_ratio);
}

#[test]
fn test_adaptive_config_serialization() {
    let config = AdaptiveConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AdaptiveConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.target_analysis_time_ms,
        config.target_analysis_time_ms
    );
}

// =============================================================================
// CURRENT THRESHOLDS TESTS
// =============================================================================

#[test]
fn test_current_thresholds_default() {
    let thresholds = CurrentThresholds::default();
    assert_eq!(thresholds.hot_cache_size, 1000);
    assert_eq!(thresholds.high_priority_permits, 10);
    assert_eq!(thresholds.low_priority_permits, 2);
    assert_eq!(thresholds.compression_level, 4);
    assert_eq!(thresholds.archive_after_hours, 24 * 30);
    assert_eq!(thresholds.cleanup_interval_minutes, 60);
}

#[test]
fn test_current_thresholds_clone() {
    let thresholds = CurrentThresholds::default();
    let cloned = thresholds.clone();
    assert_eq!(cloned.hot_cache_size, thresholds.hot_cache_size);
}

#[test]
fn test_current_thresholds_serialization() {
    let thresholds = CurrentThresholds::default();
    let json = serde_json::to_string(&thresholds).unwrap();
    let deserialized: CurrentThresholds = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.hot_cache_size, thresholds.hot_cache_size);
}

// =============================================================================
// THRESHOLD ADJUSTMENT TESTS
// =============================================================================

#[test]
fn test_threshold_adjustment_variants() {
    let adjustments = vec![
        ThresholdAdjustment::ScaleUp {
            cache_factor: 1.2,
            permit_factor: 1.1,
        },
        ThresholdAdjustment::ScaleDown {
            cache_factor: 0.8,
            permit_factor: 0.9,
        },
        ThresholdAdjustment::MoreCompression {
            compression_level: 8,
        },
        ThresholdAdjustment::LessCompression {
            compression_level: 2,
        },
        ThresholdAdjustment::Maintain,
    ];

    for adj in adjustments {
        let cloned = adj.clone();
        let json = serde_json::to_string(&adj).unwrap();
        let _deserialized: ThresholdAdjustment = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", cloned);
    }
}

// =============================================================================
// PERFORMANCE SAMPLE TESTS
// =============================================================================

#[test]
fn test_performance_sample_clone() {
    let sample = create_sample_full(100, 0.8, 256.0, 0.5, 3);
    let cloned = sample.clone();
    assert_eq!(cloned.analysis_duration_ms, 100);
    assert_eq!(cloned.cache_hit_ratio, 0.8);
    assert_eq!(cloned.memory_usage_mb, 256.0);
    assert_eq!(cloned.cpu_utilization, 0.5);
    assert_eq!(cloned.queue_depth, 3);
}

#[test]
fn test_performance_sample_debug() {
    let sample = create_sample_full(100, 0.8, 256.0, 0.5, 3);
    let debug = format!("{:?}", sample);
    assert!(debug.contains("PerformanceSample"));
}

// =============================================================================
// PERFORMANCE TREND TESTS
// =============================================================================

#[test]
fn test_performance_trend_all_variants() {
    let trends = vec![
        PerformanceTrend::Improving,
        PerformanceTrend::Stable,
        PerformanceTrend::Degrading,
    ];

    for trend in trends {
        let cloned = trend.clone();
        let json = serde_json::to_string(&trend).unwrap();
        let _deserialized: PerformanceTrend = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", cloned);
    }
}

// =============================================================================
// PERFORMANCE STATISTICS TESTS
// =============================================================================

#[test]
fn test_performance_statistics_default() {
    let stats = PerformanceStatistics::default();
    assert_eq!(stats.avg_analysis_duration_ms, 0.0);
    assert_eq!(stats.avg_cache_hit_ratio, 0.0);
    assert_eq!(stats.avg_memory_usage_mb, 0.0);
    assert_eq!(stats.avg_cpu_utilization, 0.0);
    assert_eq!(stats.total_samples, 0);
    assert_eq!(stats.recent_adjustments_count, 0);
    assert!(matches!(stats.performance_trend, PerformanceTrend::Stable));
}

#[test]
fn test_performance_statistics_clone() {
    let stats = PerformanceStatistics {
        avg_analysis_duration_ms: 100.0,
        avg_cache_hit_ratio: 0.85,
        avg_memory_usage_mb: 256.0,
        avg_cpu_utilization: 0.6,
        total_samples: 50,
        recent_adjustments_count: 3,
        performance_trend: PerformanceTrend::Improving,
    };
    let cloned = stats.clone();
    assert_eq!(cloned.avg_analysis_duration_ms, 100.0);
    assert_eq!(cloned.total_samples, 50);
}

#[test]
fn test_performance_statistics_serialization() {
    let stats = PerformanceStatistics::default();
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: PerformanceStatistics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_samples, stats.total_samples);
}

#[test]
fn test_format_diagnostic_improving() {
    let stats = PerformanceStatistics {
        avg_analysis_duration_ms: 80.5,
        avg_cache_hit_ratio: 0.9,
        avg_memory_usage_mb: 256.0,
        avg_cpu_utilization: 0.5,
        total_samples: 100,
        recent_adjustments_count: 5,
        performance_trend: PerformanceTrend::Improving,
    };

    let output = stats.format_diagnostic();
    assert!(output.contains("IMPROVING"));
    assert!(output.contains("80.5ms"));
    assert!(output.contains("90.0%")); // cache hit ratio
    assert!(output.contains("256.0MB"));
    assert!(output.contains("50.0%")); // CPU
    assert!(output.contains("100")); // samples
    assert!(output.contains("5")); // adjustments
}

#[test]
fn test_format_diagnostic_stable() {
    let stats = PerformanceStatistics {
        avg_analysis_duration_ms: 100.0,
        avg_cache_hit_ratio: 0.75,
        avg_memory_usage_mb: 300.0,
        avg_cpu_utilization: 0.6,
        total_samples: 50,
        recent_adjustments_count: 2,
        performance_trend: PerformanceTrend::Stable,
    };

    let output = stats.format_diagnostic();
    assert!(output.contains("STABLE"));
}

#[test]
fn test_format_diagnostic_degrading() {
    let stats = PerformanceStatistics {
        avg_analysis_duration_ms: 200.0,
        avg_cache_hit_ratio: 0.4,
        avg_memory_usage_mb: 450.0,
        avg_cpu_utilization: 0.9,
        total_samples: 25,
        recent_adjustments_count: 8,
        performance_trend: PerformanceTrend::Degrading,
    };

    let output = stats.format_diagnostic();
    assert!(output.contains("DEGRADING"));
}
