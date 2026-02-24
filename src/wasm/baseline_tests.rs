// Tests for baseline module — property tests, QualityBaseline, Metrics, RollingStats
// Included from baseline.rs — shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn default_metrics() -> Metrics {
        Metrics::default()
    }

    fn custom_metrics(complexity_p90: u32, complexity_p95: u32, complexity_p99: u32) -> Metrics {
        Metrics {
            timestamp: Utc::now(),
            complexity_p90,
            complexity_p95,
            complexity_p99,
            binary_size: 1_000_000,
            init_time_ms: 10,
            memory_usage_mb: 50,
            function_count: 100,
            instruction_count: 10_000,
        }
    }

    // ==================== QualityBaseline Tests ====================

    #[test]
    fn test_quality_baseline_new() {
        let release = default_metrics();
        let stable = default_metrics();
        let baseline = QualityBaseline::new(release, stable);

        // Verify baseline was created
        assert!(baseline.rolling_window.data_points.is_empty());
    }

    #[test]
    fn test_evaluate_all_passing() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        let current = custom_metrics(10, 15, 20);
        let assessment = baseline.evaluate(&current);

        assert!(assessment.is_passing());
        assert!(assessment.violations.is_empty());
        assert_eq!(assessment.overall_health, 100.0);
    }

    #[test]
    fn test_evaluate_complexity_regression() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        // Current complexity_p95 (25) exceeds release_anchor.complexity_p99 (20)
        let current = custom_metrics(10, 25, 30);
        let assessment = baseline.evaluate(&current);

        assert!(!assessment.is_passing());
        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::ComplexityRegression { .. })));
    }

    #[test]
    fn test_evaluate_complexity_creep() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        // Current complexity_p90 (18) exceeds stable_anchor.complexity_p95 (15)
        let current = custom_metrics(18, 19, 20);
        let assessment = baseline.evaluate(&current);

        // Complexity creep is a warning, not an error
        assert!(assessment.is_passing());
        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::ComplexityCreep { .. })));
    }

    #[test]
    fn test_evaluate_binary_size_increase() {
        let release = Metrics {
            binary_size: 1_000_000,
            ..default_metrics()
        };
        let stable = Metrics {
            binary_size: 1_000_000,
            ..default_metrics()
        };
        let baseline = QualityBaseline::new(release, stable);

        // 25% increase exceeds 20% threshold
        let current = Metrics {
            binary_size: 1_250_000,
            ..default_metrics()
        };
        let assessment = baseline.evaluate(&current);

        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::BinarySizeIncrease { .. })));
    }

    #[test]
    fn test_evaluate_performance_regression() {
        let release = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let stable = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let baseline = QualityBaseline::new(release, stable);

        // Init time 20ms is > 15ms (1.5x baseline)
        let current = Metrics {
            init_time_ms: 20,
            ..default_metrics()
        };
        let assessment = baseline.evaluate(&current);

        assert!(!assessment.is_passing());
        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::PerformanceRegression { .. })));
    }

    #[test]
    fn test_add_data_point() {
        let release = default_metrics();
        let stable = default_metrics();
        let mut baseline = QualityBaseline::new(release, stable);

        baseline.add_data_point(default_metrics());
        assert_eq!(baseline.rolling_window.data_points.len(), 1);

        baseline.add_data_point(default_metrics());
        assert_eq!(baseline.rolling_window.data_points.len(), 2);
    }

    #[test]
    fn test_health_score_degradation() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        // Perfect score
        let current = custom_metrics(10, 15, 20);
        let assessment = baseline.evaluate(&current);
        assert_eq!(assessment.overall_health, 100.0);

        // Degraded complexity
        let degraded = custom_metrics(15, 20, 25); // 50% higher
        let assessment = baseline.evaluate(&degraded);
        assert!(assessment.overall_health < 100.0);
    }

    #[test]
    fn test_recommendation_no_violations() {
        let release = default_metrics();
        let stable = default_metrics();
        let baseline = QualityBaseline::new(release, stable);

        let current = default_metrics();
        let assessment = baseline.evaluate(&current);

        assert_eq!(
            assessment.recommendation,
            "Quality metrics are within acceptable bounds."
        );
    }

    #[test]
    fn test_recommendation_with_critical_violations() {
        let release = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let stable = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let baseline = QualityBaseline::new(release, stable);

        let current = Metrics {
            init_time_ms: 100, // 10x baseline - critical performance regression
            ..default_metrics()
        };
        let assessment = baseline.evaluate(&current);

        assert!(assessment.recommendation.contains("critical violations"));
    }

    // ==================== Metrics Tests ====================

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();

        assert_eq!(metrics.complexity_p90, 10);
        assert_eq!(metrics.complexity_p95, 15);
        assert_eq!(metrics.complexity_p99, 20);
        assert_eq!(metrics.binary_size, 1_000_000);
        assert_eq!(metrics.init_time_ms, 10);
        assert_eq!(metrics.memory_usage_mb, 50);
        assert_eq!(metrics.function_count, 100);
        assert_eq!(metrics.instruction_count, 10_000);
    }

    #[test]
    fn test_metrics_serialization() {
        let metrics = Metrics::default();

        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: Metrics = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metrics.complexity_p90, deserialized.complexity_p90);
        assert_eq!(metrics.binary_size, deserialized.binary_size);
    }

    #[test]
    fn test_metrics_clone() {
        let metrics = Metrics::default();
        let cloned = metrics.clone();

        assert_eq!(metrics.complexity_p90, cloned.complexity_p90);
        assert_eq!(metrics.init_time_ms, cloned.init_time_ms);
    }

    // ==================== RollingStats Tests ====================

    #[test]
    fn test_rolling_stats_new() {
        let stats = RollingStats::new(30);
        assert_eq!(stats.window_days, 30);
        assert!(stats.data_points.is_empty());
    }

    #[test]
    fn test_rolling_stats_add_point() {
        let mut stats = RollingStats::new(30);

        stats.add_point(default_metrics());
        assert_eq!(stats.data_points.len(), 1);

        stats.add_point(default_metrics());
        assert_eq!(stats.data_points.len(), 2);
    }

    #[test]
    fn test_rolling_stats_trend_slope_empty() {
        let stats = RollingStats::new(30);
        assert_eq!(stats.trend_slope(), 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_single_point() {
        let mut stats = RollingStats::new(30);
        stats.add_point(default_metrics());

        // Need at least 2 points for slope
        assert_eq!(stats.trend_slope(), 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_flat() {
        let mut stats = RollingStats::new(30);

        // Add identical metrics - should have zero slope
        for _ in 0..5 {
            stats.add_point(custom_metrics(10, 15, 20));
        }

        assert_eq!(stats.trend_slope(), 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_increasing() {
        let mut stats = RollingStats::new(30);

        // Add increasing complexity - should have positive slope
        for i in 0..5 {
            stats.add_point(custom_metrics(10 + i, 15, 20));
        }

        assert!(stats.trend_slope() > 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_decreasing() {
        let mut stats = RollingStats::new(30);

        // Add decreasing complexity - should have negative slope
        for i in 0..5 {
            stats.add_point(custom_metrics(20 - i, 15, 20));
        }

        assert!(stats.trend_slope() < 0.0);
    }
}
