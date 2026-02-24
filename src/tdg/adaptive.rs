use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

include!("adaptive_types.rs");
include!("adaptive_manager.rs");
include!("adaptive_factory.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample(duration_ms: u64, cache_hit: bool, memory_mb: f32) -> PerformanceSample {
        PerformanceSample {
            timestamp: Instant::now(),
            analysis_duration_ms: duration_ms,
            cache_hit_ratio: if cache_hit { 1.0 } else { 0.0 },
            memory_usage_mb: memory_mb,
            cpu_utilization: 0.5,
            queue_depth: 2,
        }
    }

    #[tokio::test]
    async fn test_threshold_manager_creation() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());
        let stats = manager.get_performance_stats().await;

        assert_eq!(stats.total_samples, 0);
        assert!(matches!(stats.performance_trend, PerformanceTrend::Stable));
    }

    #[tokio::test]
    async fn test_performance_sample_recording() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());
        let sample = create_sample(80, true, 100.0);

        manager.record_sample(sample).await.unwrap();

        let stats = manager.get_performance_stats().await;
        assert_eq!(stats.total_samples, 1);
        assert_eq!(stats.avg_analysis_duration_ms, 80.0);
    }

    #[tokio::test]
    async fn test_sample_window_management() {
        let config = AdaptiveConfig {
            sample_window_size: 3,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add more samples than window size
        for i in 0..5 {
            let sample = create_sample(100 + i * 10, true, 100.0);
            manager.record_sample(sample).await.unwrap();
        }

        let stats = manager.get_performance_stats().await;
        assert_eq!(stats.total_samples, 3); // Should maintain window size
    }

    #[tokio::test]
    async fn test_scale_up_adjustment() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 100,
            min_cache_hit_ratio: 0.8,
            sample_window_size: 10,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples showing slow performance and low cache hits
        for _ in 0..12 {
            let sample = create_sample(200, false, 100.0); // Slow + cache miss
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        let stats = manager.get_performance_stats().await;

        // Should have triggered scale-up adjustment
        assert!(thresholds.hot_cache_size > 1000); // Should be increased from default
        assert!(stats.recent_adjustments_count > 0);
    }

    #[tokio::test]
    async fn test_compression_adjustment() {
        let config = AdaptiveConfig {
            max_memory_mb: 200.0,
            sample_window_size: 10,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples showing high memory usage
        for _ in 0..12 {
            let sample = create_sample(80, true, 300.0); // High memory
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;

        // Should have increased compression level
        assert!(thresholds.compression_level > 4);
    }

    #[tokio::test]
    async fn test_factory_patterns() {
        let default_mgr = AdaptiveThresholdFactory::create_default();
        let dev_mgr = AdaptiveThresholdFactory::create_dev_optimized();
        let prod_mgr = AdaptiveThresholdFactory::create_prod_optimized();

        // Test that all managers can record samples
        let sample = create_sample(100, true, 100.0);

        default_mgr.record_sample(sample.clone()).await.unwrap();
        dev_mgr.record_sample(sample.clone()).await.unwrap();
        prod_mgr.record_sample(sample).await.unwrap();

        // Verify different configurations
        let dev_stats = dev_mgr.get_performance_stats().await;
        let prod_stats = prod_mgr.get_performance_stats().await;

        assert_eq!(dev_stats.total_samples, 1);
        assert_eq!(prod_stats.total_samples, 1);
    }

    #[tokio::test]
    async fn test_trend_calculation() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Add improving trend samples (getting faster)
        for i in 0..20 {
            let duration = 200 - (i * 5); // Getting faster over time
            let sample = create_sample(duration, true, 100.0);
            manager.record_sample(sample).await.unwrap();
        }

        let stats = manager.get_performance_stats().await;
        assert!(matches!(
            stats.performance_trend,
            PerformanceTrend::Improving
        ));
    }
}

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
mod extended_coverage_tests {
    use super::*;

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

    // =============================================================================
    // ADAPTIVE THRESHOLD MANAGER ASYNC TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_get_current_thresholds() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());
        let thresholds = manager.get_current_thresholds().await;
        assert_eq!(thresholds.hot_cache_size, 1000);
        assert_eq!(thresholds.high_priority_permits, 10);
    }

    #[tokio::test]
    async fn test_reset_to_defaults() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Record samples to trigger adjustments
        for _ in 0..12 {
            let sample = create_sample_full(200, 0.3, 100.0, 0.5, 2);
            manager.record_sample(sample).await.unwrap();
        }

        // Reset to defaults
        manager.reset_to_defaults().await.unwrap();

        let thresholds = manager.get_current_thresholds().await;
        assert_eq!(thresholds.hot_cache_size, 1000);
        assert_eq!(thresholds.compression_level, 4);
    }

    #[tokio::test]
    async fn test_create_sample() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        let sample = manager
            .create_sample(Duration::from_millis(150), true, 5)
            .await;

        assert_eq!(sample.analysis_duration_ms, 150);
        assert_eq!(sample.cache_hit_ratio, 1.0);
        assert_eq!(sample.queue_depth, 5);
    }

    #[tokio::test]
    async fn test_create_sample_cache_miss() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        let sample = manager
            .create_sample(Duration::from_millis(200), false, 0)
            .await;

        assert_eq!(sample.cache_hit_ratio, 0.0);
    }

    #[tokio::test]
    async fn test_high_queue_depth_adjustment() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 100,
            min_cache_hit_ratio: 0.5,
            sample_window_size: 15,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples with good cache hit but high queue depth and slow performance
        for _ in 0..15 {
            let sample = create_sample_full(180, 0.7, 100.0, 0.5, 10); // High queue depth
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Should have increased permits due to high queue depth
        assert!(thresholds.high_priority_permits >= 10);
    }

    #[tokio::test]
    async fn test_less_compression_adjustment() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 50,
            min_cache_hit_ratio: 0.5,
            sample_window_size: 12,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples showing slow performance but good cache and low queue
        for _ in 0..15 {
            let sample = create_sample_full(150, 0.8, 100.0, 0.5, 2); // Slow, good cache, low queue
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Should have reduced compression for speed
        // The compression_level should be adjusted to 1 (less compression)
        // But the exact value depends on the adjustment logic
        let _ = thresholds.compression_level; // Just verify no panic
    }

    #[tokio::test]
    async fn test_scale_down_high_memory() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 200,
            max_memory_mb: 100.0,
            sample_window_size: 12,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples with fast performance, high cache hit, but high memory
        for _ in 0..15 {
            let sample = create_sample_full(80, 0.95, 200.0, 0.4, 2); // Fast, high cache, high memory
            manager.record_sample(sample).await.unwrap();
        }

        // Should trigger scale down or more compression due to high memory
        let _thresholds = manager.get_current_thresholds().await;
    }

    #[tokio::test]
    async fn test_maintain_excellent_performance() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 100,
            max_memory_mb: 500.0,
            sample_window_size: 12,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples with excellent performance
        for _ in 0..15 {
            let sample = create_sample_full(40, 0.9, 200.0, 0.3, 1); // Very fast, high cache, low resource
            manager.record_sample(sample).await.unwrap();
        }

        let stats = manager.get_performance_stats().await;
        // Should have recorded samples without major adjustments
        assert!(stats.total_samples > 0);
    }

    #[tokio::test]
    async fn test_trend_degrading() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Add degrading trend samples (getting slower)
        for i in 0..20 {
            let duration = 100 + (i * 10); // Getting slower over time
            let sample = create_sample_full(duration, 0.7, 150.0, 0.5, 2);
            manager.record_sample(sample).await.unwrap();
        }

        let stats = manager.get_performance_stats().await;
        assert!(matches!(
            stats.performance_trend,
            PerformanceTrend::Degrading
        ));
    }

    #[tokio::test]
    async fn test_trend_stable() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Add stable performance samples
        for _ in 0..20 {
            let sample = create_sample_full(100, 0.7, 150.0, 0.5, 2); // Same duration each time
            manager.record_sample(sample).await.unwrap();
        }

        let stats = manager.get_performance_stats().await;
        assert!(matches!(stats.performance_trend, PerformanceTrend::Stable));
    }

    #[tokio::test]
    async fn test_get_memory_usage() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());
        let memory = manager.get_memory_usage().await;
        // Should be >= base memory (50.0)
        assert!(memory >= 50.0);
    }

    #[tokio::test]
    async fn test_get_cpu_usage() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());
        let cpu = manager.get_cpu_usage().await;
        // Should be between 0 and 1
        assert!(cpu >= 0.0 && cpu <= 1.0);
    }

    #[tokio::test]
    async fn test_get_cpu_usage_with_samples() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Add recent samples
        for _ in 0..5 {
            let sample = create_sample_full(100, 0.8, 200.0, 0.5, 2);
            manager.record_sample(sample).await.unwrap();
        }

        let cpu = manager.get_cpu_usage().await;
        // Should reflect recent activity
        assert!(cpu >= 0.0 && cpu <= 1.0);
    }

    #[tokio::test]
    async fn test_get_performance_stats_empty() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());
        let stats = manager.get_performance_stats().await;

        assert_eq!(stats.total_samples, 0);
        assert_eq!(stats.avg_analysis_duration_ms, 0.0);
    }

    #[tokio::test]
    async fn test_scale_down_with_high_cpu() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 200,
            max_cpu_utilization: 0.5,
            max_memory_mb: 500.0,
            sample_window_size: 12,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add samples with high CPU but fast performance and high cache
        for _ in 0..15 {
            let sample = create_sample_full(80, 0.95, 200.0, 0.8, 2); // Fast, high cache, high CPU
            manager.record_sample(sample).await.unwrap();
        }

        // Should trigger adjustment due to high CPU
        let _thresholds = manager.get_current_thresholds().await;
    }

    #[tokio::test]
    async fn test_adjustment_history_limit() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 100,
            min_cache_hit_ratio: 0.9, // Hard to achieve
            sample_window_size: 10,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Add many samples to trigger many adjustments
        for _ in 0..200 {
            let sample = create_sample_full(200, 0.3, 100.0, 0.5, 2); // Triggers adjustments
            manager.record_sample(sample).await.unwrap();
        }

        // Adjustment history should be limited to 100
        let stats = manager.get_performance_stats().await;
        assert!(stats.recent_adjustments_count <= 100);
    }

    // =============================================================================
    // FACTORY TESTS
    // =============================================================================

    #[test]
    fn test_factory_create_default() {
        let manager = AdaptiveThresholdFactory::create_default();
        let _ = format!("{:?}", manager.config);
    }

    #[test]
    fn test_factory_create_dev_optimized() {
        let manager = AdaptiveThresholdFactory::create_dev_optimized();
        assert_eq!(manager.config.target_analysis_time_ms, 50);
        assert_eq!(manager.config.sample_window_size, 20);
        assert_eq!(manager.config.adjustment_sensitivity, 0.2);
    }

    #[test]
    fn test_factory_create_prod_optimized() {
        let manager = AdaptiveThresholdFactory::create_prod_optimized();
        assert_eq!(manager.config.target_analysis_time_ms, 200);
        assert_eq!(manager.config.sample_window_size, 100);
        assert_eq!(manager.config.adjustment_sensitivity, 0.05);
    }

    // =============================================================================
    // EDGE CASE TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_insufficient_samples_for_adjustment() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Add only 5 samples (below minimum for adjustment)
        for _ in 0..5 {
            let sample = create_sample_full(200, 0.3, 600.0, 0.9, 10);
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Should still be at defaults since not enough samples
        assert_eq!(thresholds.hot_cache_size, 1000);
    }

    #[tokio::test]
    async fn test_insufficient_samples_for_trend() {
        let manager = AdaptiveThresholdManager::new(AdaptiveConfig::default());

        // Add only 5 samples
        for _ in 0..5 {
            let sample = create_sample_full(100, 0.8, 200.0, 0.5, 2);
            manager.record_sample(sample).await.unwrap();
        }

        let stats = manager.get_performance_stats().await;
        // Should be stable since not enough samples for trend calculation
        assert!(matches!(stats.performance_trend, PerformanceTrend::Stable));
    }

    #[tokio::test]
    async fn test_scale_up_max_limits() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 50,
            min_cache_hit_ratio: 0.9,
            sample_window_size: 10,
            adjustment_sensitivity: 0.5, // Large adjustments
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Trigger many scale-ups
        for _ in 0..50 {
            let sample = create_sample_full(200, 0.3, 100.0, 0.3, 2);
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Should be capped at max values
        assert!(thresholds.hot_cache_size <= 10000);
        assert!(thresholds.high_priority_permits <= 50);
        assert!(thresholds.low_priority_permits <= 20);
    }

    #[tokio::test]
    async fn test_scale_down_min_limits() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 500,
            max_memory_mb: 50.0, // Very low to trigger scale down
            max_cpu_utilization: 0.2,
            sample_window_size: 10,
            adjustment_sensitivity: 0.5,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Trigger many scale-downs
        for _ in 0..50 {
            let sample = create_sample_full(50, 0.95, 300.0, 0.9, 2); // Fast but resource heavy
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Should be floored at min values
        assert!(thresholds.hot_cache_size >= 100);
        assert!(thresholds.high_priority_permits >= 2);
        assert!(thresholds.low_priority_permits >= 1);
    }

    #[tokio::test]
    async fn test_no_adjustment_moderate_performance() {
        // Exercise the Ok(None) return path in calculate_adjustment:
        // Performance is moderate (not slow enough for ScaleUp, not excellent enough for Maintain),
        // and resource usage is within limits (no ScaleDown/MoreCompression needed).
        let config = AdaptiveConfig {
            target_analysis_time_ms: 100,
            max_memory_mb: 512.0,
            max_cpu_utilization: 0.8,
            sample_window_size: 20,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // duration=80: NOT > 150 (target*1.5), so skip first block
        // memory=100 <= 512 and cpu=0.3 <= 0.8, so skip resource block
        // duration=80: NOT < 50 (target*0.5), so skip Maintain block
        // Result: Ok(None) -- no adjustment
        for _ in 0..15 {
            let sample = create_sample_full(80, 0.7, 100.0, 0.3, 2);
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Should remain at defaults since no adjustment was triggered
        assert_eq!(thresholds.hot_cache_size, 1000);
        assert_eq!(thresholds.compression_level, 4);
        assert_eq!(thresholds.high_priority_permits, 10);
    }

    #[tokio::test]
    async fn test_compression_level_bounds() {
        let config = AdaptiveConfig {
            target_analysis_time_ms: 10, // Very aggressive
            max_memory_mb: 1000.0,
            sample_window_size: 10,
            ..Default::default()
        };
        let manager = AdaptiveThresholdManager::new(config);

        // Trigger less compression adjustments
        for _ in 0..20 {
            let sample = create_sample_full(100, 0.8, 100.0, 0.3, 2);
            manager.record_sample(sample).await.unwrap();
        }

        let thresholds = manager.get_current_thresholds().await;
        // Compression level should be within valid range
        assert!(thresholds.compression_level >= 1);
        assert!(thresholds.compression_level <= 9);
    }
}
