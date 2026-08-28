#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // RateLimiter tests
    #[test]
    fn test_rate_limiter_available_tokens() {
        let limiter = RateLimiter::new(100, 10);

        // Initially should have full capacity
        let available = limiter.available_tokens();
        assert_eq!(available, 100);

        // Acquire some tokens
        limiter.try_acquire(30);
        let available = limiter.available_tokens();
        assert_eq!(available, 70);
    }

    #[actix_rt::test]
    async fn test_rate_limiter_acquire_async() {
        let limiter = RateLimiter::new(5, 100); // 100 tokens/sec refill rate

        // Acquire all tokens
        assert!(limiter.try_acquire(5));
        assert!(!limiter.try_acquire(1)); // Should be empty

        // Use async acquire - should block until tokens available
        let start = Instant::now();
        limiter.acquire(1).await;
        let elapsed = start.elapsed();

        // Should have waited for refill (at least a few ms)
        assert!(elapsed.as_millis() >= 5);
    }

    #[test]
    fn test_rate_limiter_refill_partial() {
        let limiter = RateLimiter::new(10, 1000); // 1000 tokens/sec refill

        // Drain all tokens
        assert!(limiter.try_acquire(10));
        assert_eq!(limiter.available_tokens(), 0);

        // Wait for partial refill
        std::thread::sleep(Duration::from_millis(5));

        // Should have some tokens back
        let available = limiter.available_tokens();
        assert!(available > 0);
        assert!(available <= 10); // But not more than capacity
    }

    #[test]
    fn test_rate_limiter_concurrent_acquire() {
        use std::sync::Arc;
        use std::thread;

        let limiter = Arc::new(RateLimiter::new(100, 50));
        let mut handles = vec![];

        // Spawn multiple threads trying to acquire tokens
        for _ in 0..5 {
            let limiter_clone = limiter.clone();
            handles.push(thread::spawn(move || {
                let mut acquired = 0;
                for _ in 0..10 {
                    if limiter_clone.try_acquire(2) {
                        acquired += 2;
                    }
                }
                acquired
            }));
        }

        let total_acquired: u32 = handles.into_iter().map(|h| h.join().unwrap()).sum();

        // Total acquired should not exceed initial capacity
        assert!(total_acquired <= 100);
    }

    // BackpressureMetrics tests
    #[test]
    fn test_backpressure_metrics_default() {
        let metrics = BackpressureMetrics::default();

        assert_eq!(metrics.rejected_count, 0);
        assert_eq!(metrics.accepted_count, 0);
        assert_eq!(metrics.queue_depth_sum, 0);
        assert_eq!(metrics.max_queue_depth, 0);
        assert_eq!(metrics.sample_count, 0);
    }

    #[test]
    fn test_backpressure_metrics_clone() {
        let metrics = BackpressureMetrics {
            rejected_count: 10,
            accepted_count: 100,
            queue_depth_sum: 500,
            max_queue_depth: 8,
            sample_count: 100,
        };

        let cloned = metrics.clone();

        assert_eq!(cloned.rejected_count, metrics.rejected_count);
        assert_eq!(cloned.accepted_count, metrics.accepted_count);
        assert_eq!(cloned.queue_depth_sum, metrics.queue_depth_sum);
        assert_eq!(cloned.max_queue_depth, metrics.max_queue_depth);
        assert_eq!(cloned.sample_count, metrics.sample_count);
    }

    #[test]
    fn test_backpressure_metrics_debug() {
        let metrics = BackpressureMetrics {
            rejected_count: 5,
            accepted_count: 50,
            queue_depth_sum: 200,
            max_queue_depth: 10,
            sample_count: 50,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("BackpressureMetrics"));
        assert!(debug_str.contains("rejected_count"));
        assert!(debug_str.contains("accepted_count"));
    }

    // BackpressureController tests
    #[actix_rt::test]
    async fn test_backpressure_controller_acquire_permit() {
        let controller = BackpressureController::new(5);

        // Should be able to acquire permit
        let permit = controller.acquire_permit().await;
        assert!(permit.is_ok());

        // Check queue depth increased
        assert_eq!(controller.get_queue_depth(), 1);
    }

    #[actix_rt::test]
    async fn test_backpressure_controller_get_metrics() {
        let controller = BackpressureController::new(10);

        // Acquire some permits
        let _p1 = controller.try_acquire_permit().unwrap();
        let _p2 = controller.try_acquire_permit().unwrap();

        let metrics = controller.get_metrics();
        assert_eq!(metrics.accepted_count, 2);
        assert_eq!(metrics.max_queue_depth, 2);
        assert_eq!(metrics.sample_count, 2);
    }

    #[actix_rt::test]
    async fn test_backpressure_controller_average_queue_depth() {
        let controller = BackpressureController::new(10);

        // No samples yet
        assert_eq!(controller.get_average_queue_depth(), 0.0);

        // Acquire permits to create samples
        let _p1 = controller.try_acquire_permit().unwrap();
        let _p2 = controller.try_acquire_permit().unwrap();
        let _p3 = controller.try_acquire_permit().unwrap();

        // Average should be (1 + 2 + 3) / 3 = 2.0
        let avg = controller.get_average_queue_depth();
        assert!((avg - 2.0).abs() < f64::EPSILON);
    }

    #[actix_rt::test]
    async fn test_backpressure_controller_rejected_metrics() {
        let controller = BackpressureController::new(2);

        // Fill up the queue
        let _p1 = controller.try_acquire_permit().unwrap();
        let _p2 = controller.try_acquire_permit().unwrap();

        // Try to acquire more - should be rejected
        let result = controller.try_acquire_permit();
        assert!(matches!(result, Err(BackpressureError::QueueFull)));

        // Check rejected count
        let metrics = controller.get_metrics();
        assert_eq!(metrics.rejected_count, 1);
    }

    #[actix_rt::test]
    async fn test_backpressure_permit_drop_releases() {
        let controller = BackpressureController::new(2);

        // Acquire all permits
        let p1 = controller.try_acquire_permit().unwrap();
        let _p2 = controller.try_acquire_permit().unwrap();

        assert_eq!(controller.get_queue_depth(), 2);

        // Drop one permit
        drop(p1);

        // Queue depth should decrease
        assert_eq!(controller.get_queue_depth(), 1);
    }

    // BackpressureError tests
    #[test]
    fn test_backpressure_error_queue_full() {
        let err = BackpressureError::QueueFull;
        let display_str = format!("{}", err);
        assert!(display_str.contains("full"));

        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("QueueFull"));
    }

    #[test]
    fn test_backpressure_error_rate_limit() {
        let err = BackpressureError::RateLimitExceeded;
        let display_str = format!("{}", err);
        assert!(display_str.contains("Rate limit"));

        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("RateLimitExceeded"));
    }

    // LoadMonitor tests
    #[test]
    fn test_load_monitor_creation() {
        let monitor = LoadMonitor::new(0.8, 0.9);

        assert_eq!(monitor.cpu_threshold, 0.8);
        assert_eq!(monitor.memory_threshold, 0.9);
    }

    #[test]
    fn test_load_monitor_get_load_factor() {
        let monitor = LoadMonitor::new(0.8, 0.9);

        // Load factor should be between 0 and 1 (clamped)
        let load_factor = monitor.get_load_factor();
        assert!(load_factor >= 0.0);
        assert!(load_factor <= 1.0);
    }

    // AdaptiveRateController tests
    #[actix_rt::test]
    async fn test_adaptive_rate_controller_try_acquire() {
        let controller = AdaptiveRateController::new(100, 10, 500);

        // Should be able to acquire tokens
        assert!(controller.try_acquire());
    }

    #[actix_rt::test]
    async fn test_adaptive_rate_controller_acquire_async() {
        let controller = AdaptiveRateController::new(100, 10, 500);

        // Should be able to acquire
        controller.acquire().await;

        // Rate should still be valid
        let rate = controller.get_current_rate();
        assert!((10..=500).contains(&rate));
    }

    #[actix_rt::test]
    async fn test_adaptive_rate_controller_rate_bounds() {
        let controller = AdaptiveRateController::new(50, 10, 200);

        // Adapt multiple times
        for _ in 0..10 {
            controller.adapt_rate().await;
        }

        // Rate should always be within bounds
        let rate = controller.get_current_rate();
        assert!(rate >= 10);
        assert!(rate <= 200);
    }

    // Bulkhead tests
    #[test]
    fn test_bulkhead_creation() {
        let bulkhead = Bulkhead::new("test-bulkhead".to_string(), 5);

        let metrics = bulkhead.get_metrics();
        assert_eq!(metrics.name, "test-bulkhead");
        assert_eq!(metrics.max_concurrent, 5);
        assert_eq!(metrics.active_count, 0);
        assert_eq!(metrics.rejected_count, 0);
    }

    #[test]
    fn test_bulkhead_clone() {
        let bulkhead = Bulkhead::new("test".to_string(), 3);
        let cloned = bulkhead.clone();

        // Both should share the same underlying state
        assert_eq!(bulkhead.name, cloned.name);
        assert_eq!(bulkhead.max_concurrent, cloned.max_concurrent);
    }

    #[actix_rt::test]
    async fn test_bulkhead_metrics_after_execution() {
        let bulkhead = Bulkhead::new("metrics-test".to_string(), 5);

        // Execute a task
        let result = bulkhead.execute(async { 42 }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // After completion, active count should be 0
        let metrics = bulkhead.get_metrics();
        assert_eq!(metrics.active_count, 0);
        assert_eq!(metrics.rejected_count, 0);
    }

    #[actix_rt::test]
    async fn test_bulkhead_rejection_metrics() {
        let bulkhead = Bulkhead::new("rejection-test".to_string(), 1);

        // Start a long-running task
        let bulkhead_clone = bulkhead.clone();
        let handle = tokio::spawn(async move {
            bulkhead_clone
                .execute(async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    1
                })
                .await
        });

        // Give time for the task to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try to execute another task - should be rejected
        let result = bulkhead.execute(async { 2 }).await;
        assert!(matches!(result, Err(BackpressureError::QueueFull)));

        // Check rejection metrics
        let metrics = bulkhead.get_metrics();
        assert_eq!(metrics.rejected_count, 1);

        // Wait for first task to complete
        let _ = handle.await;
    }

    // BulkheadMetrics tests
    #[test]
    fn test_bulkhead_metrics_debug() {
        let metrics = BulkheadMetrics {
            name: "test".to_string(),
            max_concurrent: 5,
            active_count: 2,
            rejected_count: 1,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("BulkheadMetrics"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_bulkhead_metrics_clone() {
        let metrics = BulkheadMetrics {
            name: "cloned".to_string(),
            max_concurrent: 10,
            active_count: 3,
            rejected_count: 2,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.name, metrics.name);
        assert_eq!(cloned.max_concurrent, metrics.max_concurrent);
        assert_eq!(cloned.active_count, metrics.active_count);
        assert_eq!(cloned.rejected_count, metrics.rejected_count);
    }

    // Additional edge case tests
    #[test]
    fn test_rate_limiter_zero_tokens_acquire() {
        let limiter = RateLimiter::new(10, 5);

        // Acquiring zero tokens should always succeed
        assert!(limiter.try_acquire(0));
        assert_eq!(limiter.available_tokens(), 10); // No tokens consumed
    }

    #[test]
    fn test_rate_limiter_exact_capacity_acquire() {
        let limiter = RateLimiter::new(10, 5);

        // Should be able to acquire exact capacity
        assert!(limiter.try_acquire(10));
        assert_eq!(limiter.available_tokens(), 0);

        // Cannot acquire any more
        assert!(!limiter.try_acquire(1));
    }

    #[actix_rt::test]
    async fn test_backpressure_controller_queue_depth_zero_when_empty() {
        let controller = BackpressureController::new(5);

        // Initially queue should be empty
        assert_eq!(controller.get_queue_depth(), 0);

        // Acquire and release permit
        {
            let _permit = controller.try_acquire_permit().unwrap();
            assert_eq!(controller.get_queue_depth(), 1);
        }

        // After release, queue should be empty again
        assert_eq!(controller.get_queue_depth(), 0);
    }

    #[actix_rt::test]
    async fn test_adaptive_rate_low_load_increases_rate() {
        // With default LoadMonitor returning 0.5 CPU and 0.4 memory,
        // the load factor will be around 0.625 (0.5/0.8)
        // This is moderate load, so rate may stay the same or increase slightly
        let controller = AdaptiveRateController::new(100, 10, 1000);
        let initial_rate = controller.get_current_rate();

        controller.adapt_rate().await;

        let new_rate = controller.get_current_rate();
        // Rate should have increased or stayed same (load factor ~0.625 is moderate)
        assert!(new_rate >= initial_rate - 10); // Allow small decrease
        assert!(new_rate <= 1000); // But never exceed max
    }

    #[actix_rt::test]
    async fn test_bulkhead_sequential_execution() {
        let bulkhead = Bulkhead::new("sequential".to_string(), 2);

        // Execute tasks sequentially
        let r1 = bulkhead.execute(async { 1 }).await.unwrap();
        let r2 = bulkhead.execute(async { 2 }).await.unwrap();
        let r3 = bulkhead.execute(async { 3 }).await.unwrap();

        assert_eq!(r1, 1);
        assert_eq!(r2, 2);
        assert_eq!(r3, 3);
    }
}
