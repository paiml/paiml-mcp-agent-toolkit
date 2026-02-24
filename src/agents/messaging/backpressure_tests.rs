#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(10, 5);

        // Should allow initial burst
        assert!(limiter.try_acquire(5));
        assert!(limiter.try_acquire(5));
        assert!(!limiter.try_acquire(1)); // Should be empty

        // Wait for refill
        std::thread::sleep(Duration::from_millis(201));
        assert!(limiter.try_acquire(1)); // Should have refilled
    }

    #[actix_rt::test]
    async fn test_backpressure_controller() {
        let controller = BackpressureController::new(3);

        // Acquire permits
        let permit1 = controller.try_acquire_permit().unwrap();
        let _permit2 = controller.try_acquire_permit().unwrap();
        let _permit3 = controller.try_acquire_permit().unwrap();

        // Should be full
        assert!(matches!(
            controller.try_acquire_permit(),
            Err(BackpressureError::QueueFull)
        ));

        // Release one
        drop(permit1);

        // Should be able to acquire again
        let _permit4 = controller.try_acquire_permit().unwrap();
    }

    #[actix_rt::test]
    async fn test_adaptive_rate_controller() {
        let controller = AdaptiveRateController::new(100, 10, 1000);

        // Initial rate
        assert_eq!(controller.get_current_rate(), 100);

        // Adapt based on load
        controller.adapt_rate().await;

        // Rate should have changed
        let new_rate = controller.get_current_rate();
        assert!((10..=1000).contains(&new_rate));
    }

    #[actix_rt::test]
    async fn test_bulkhead() {
        let bulkhead = Bulkhead::new("test".to_string(), 2);

        // Should allow concurrent execution up to limit
        let handle1 = tokio::spawn({
            let bulkhead = bulkhead.clone();
            async move {
                bulkhead
                    .execute(async {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        1
                    })
                    .await
            }
        });

        let handle2 = tokio::spawn({
            let bulkhead = bulkhead.clone();
            async move {
                bulkhead
                    .execute(async {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        2
                    })
                    .await
            }
        });

        // Third should be rejected
        tokio::time::sleep(Duration::from_millis(10)).await;
        let result = bulkhead.execute(async { 3 }).await;
        assert!(matches!(result, Err(BackpressureError::QueueFull)));

        // Wait for completion
        let _r1 = handle1.await.unwrap();
        let _r2 = handle2.await.unwrap();
    }
}
