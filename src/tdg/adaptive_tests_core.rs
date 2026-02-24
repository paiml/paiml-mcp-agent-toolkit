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
