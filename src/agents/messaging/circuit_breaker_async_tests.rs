#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_async {
    use super::*;

    // CircuitBreakerManager tests
    #[test]
    fn test_circuit_breaker_manager_new() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // Verify manager is created with no breakers
        let metrics = manager.get_all_metrics();
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_circuit_breaker_manager_get_or_create() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // Create first breaker
        let breaker1 = manager.get_or_create("test-service");
        assert_eq!(breaker1.get_state(), CircuitState::Closed);

        // Get same breaker again
        let breaker2 = manager.get_or_create("test-service");
        assert_eq!(breaker2.get_state(), CircuitState::Closed);

        // Create different breaker
        let breaker3 = manager.get_or_create("other-service");
        assert_eq!(breaker3.get_state(), CircuitState::Closed);

        // Verify we have 2 breakers
        let metrics = manager.get_all_metrics();
        assert_eq!(metrics.len(), 2);
    }

    #[test]
    fn test_circuit_breaker_manager_get_all_metrics() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // Create multiple breakers
        manager.get_or_create("service-a");
        manager.get_or_create("service-b");
        manager.get_or_create("service-c");

        let metrics = manager.get_all_metrics();
        assert_eq!(metrics.len(), 3);
        assert!(metrics.contains_key("service-a"));
        assert!(metrics.contains_key("service-b"));
        assert!(metrics.contains_key("service-c"));

        // All should be closed
        for m in metrics.values() {
            assert_eq!(m.state, CircuitState::Closed);
        }
    }

    #[test]
    fn test_circuit_breaker_manager_reset_all() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..CircuitBreakerConfig::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Create breakers and open them
        let breaker1 = manager.get_or_create("service-1");
        let breaker2 = manager.get_or_create("service-2");

        breaker1.on_failure();
        breaker2.on_failure();

        assert_eq!(breaker1.get_state(), CircuitState::Open);
        assert_eq!(breaker2.get_state(), CircuitState::Open);

        // Reset all
        manager.reset_all();

        // Verify all reset
        let metrics = manager.get_all_metrics();
        for m in metrics.values() {
            assert_eq!(m.state, CircuitState::Closed);
            assert_eq!(m.failure_count, 0);
        }
    }

    // Async call tests
    #[actix_rt::test]
    async fn test_circuit_breaker_call_success() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        let result = breaker
            .call(async { Ok::<i32, std::io::Error>(42) }, || 0)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[actix_rt::test]
    async fn test_circuit_breaker_call_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10, // High threshold to not open
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        let result = breaker
            .call(
                async { Err::<i32, std::io::Error>(std::io::Error::other("error")) },
                || 0,
            )
            .await;

        assert!(result.is_err());
        let metrics = breaker.get_metrics();
        assert_eq!(metrics.failure_count, 1);
    }

    #[actix_rt::test]
    async fn test_circuit_breaker_fallback_when_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout_duration: Duration::from_secs(60), // Long timeout to stay open
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        let _ = breaker
            .call(
                async { Err::<i32, std::io::Error>(std::io::Error::other("error")) },
                || -1,
            )
            .await;

        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Next call should use fallback
        let result = breaker
            .call(async { Ok::<i32, std::io::Error>(42) }, || -1)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -1); // Fallback value
    }

    #[actix_rt::test]
    async fn test_circuit_breaker_timeout_uses_fallback() {
        let config = CircuitBreakerConfig {
            fallback_timeout: Duration::from_millis(10), // Very short timeout
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Operation that takes longer than timeout
        let result = breaker
            .call(
                async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok::<i32, std::io::Error>(42)
                },
                || -1,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -1); // Fallback due to timeout
    }

    #[actix_rt::test]
    async fn test_circuit_breaker_half_open_failure_reopens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout_duration: Duration::from_millis(50),
            fallback_timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        let _ = breaker
            .call(
                async { Err::<(), std::io::Error>(std::io::Error::other("error")) },
                || (),
            )
            .await;

        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Wait for timeout to transition to half-open
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Fail in half-open should reopen
        let _ = breaker
            .call(
                async { Err::<(), std::io::Error>(std::io::Error::other("error")) },
                || (),
            )
            .await;

        assert_eq!(breaker.get_state(), CircuitState::Open);
    }

    #[actix_rt::test]
    async fn test_circuit_breaker_multiple_successes_close() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 3,
            timeout_duration: Duration::from_millis(50),
            fallback_timeout: Duration::from_secs(1),
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        let _ = breaker
            .call(
                async { Err::<(), std::io::Error>(std::io::Error::other("error")) },
                || (),
            )
            .await;

        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(100)).await;

        // First success - goes to half-open
        let _ = breaker
            .call(async { Ok::<(), std::io::Error>(()) }, || ())
            .await;

        // After 1 success (need 3 to close)
        assert_eq!(breaker.get_state(), CircuitState::HalfOpen);

        // Second success
        let _ = breaker
            .call(async { Ok::<(), std::io::Error>(()) }, || ())
            .await;

        // Still half-open
        assert_eq!(breaker.get_state(), CircuitState::HalfOpen);

        // Third success - should close
        let _ = breaker
            .call(async { Ok::<(), std::io::Error>(()) }, || ())
            .await;

        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_on_success_open_state_does_nothing() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        breaker.on_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Calling on_success while Open should do nothing
        breaker.on_success();
        assert_eq!(breaker.get_state(), CircuitState::Open);
    }

    #[test]
    fn test_on_failure_open_state_does_nothing() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        breaker.on_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);

        let metrics_before = breaker.get_metrics();

        // Calling on_failure while Open should just update timestamp
        breaker.on_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);

        let metrics_after = breaker.get_metrics();
        // Failure count should remain the same (only last_failure_time updated)
        assert_eq!(metrics_after.failure_count, metrics_before.failure_count);
    }
}
