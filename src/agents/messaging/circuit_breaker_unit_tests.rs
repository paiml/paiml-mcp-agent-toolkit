#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // CircuitState tests
    #[test]
    fn test_circuit_state_debug() {
        let closed = CircuitState::Closed;
        let open = CircuitState::Open;
        let half_open = CircuitState::HalfOpen;

        assert!(format!("{:?}", closed).contains("Closed"));
        assert!(format!("{:?}", open).contains("Open"));
        assert!(format!("{:?}", half_open).contains("HalfOpen"));
    }

    #[test]
    fn test_circuit_state_clone_copy() {
        let state = CircuitState::Open;
        let cloned = state;
        assert_eq!(cloned, state);

        let state2 = CircuitState::HalfOpen;
        let cloned2 = state2;
        assert_eq!(cloned2, CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_eq!(CircuitState::Open, CircuitState::Open);
        assert_eq!(CircuitState::HalfOpen, CircuitState::HalfOpen);

        assert_ne!(CircuitState::Closed, CircuitState::Open);
        assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
        assert_ne!(CircuitState::HalfOpen, CircuitState::Closed);
    }

    // CircuitBreakerConfig tests
    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();

        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.timeout_duration, Duration::from_secs(30));
        assert_eq!(config.fallback_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            timeout_duration: Duration::from_secs(60),
            fallback_timeout: Duration::from_secs(10),
        };

        let cloned = config.clone();

        assert_eq!(cloned.failure_threshold, 10);
        assert_eq!(cloned.success_threshold, 5);
        assert_eq!(cloned.timeout_duration, Duration::from_secs(60));
        assert_eq!(cloned.fallback_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_circuit_breaker_config_custom_values() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout_duration: Duration::from_millis(100),
            fallback_timeout: Duration::from_millis(50),
        };

        assert_eq!(config.failure_threshold, 1);
        assert_eq!(config.success_threshold, 1);
        assert_eq!(config.timeout_duration.as_millis(), 100);
        assert_eq!(config.fallback_timeout.as_millis(), 50);
    }

    // CircuitBreaker tests
    #[test]
    fn test_circuit_breaker_new() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_get_state() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_get_metrics() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let metrics = breaker.get_metrics();

        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.last_failure_time, 0);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Manually set some internal state by calling on_failure
        breaker.on_failure();

        // Verify state changed
        let metrics_before = breaker.get_metrics();
        assert!(metrics_before.failure_count > 0 || metrics_before.last_failure_time > 0);

        // Reset
        breaker.reset();

        // Verify reset
        let metrics_after = breaker.get_metrics();
        assert_eq!(metrics_after.failure_count, 0);
        assert_eq!(metrics_after.success_count, 0);
        assert_eq!(metrics_after.state, CircuitState::Closed);
        assert_eq!(metrics_after.last_failure_time, 0);
    }

    #[test]
    fn test_circuit_breaker_should_attempt_reset_no_failures() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        // When last_failure_time is 0, should_attempt_reset returns true
        assert!(breaker.should_attempt_reset());
    }

    #[test]
    fn test_circuit_breaker_on_success_closed_state() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Record a failure first
        breaker.on_failure();
        let metrics = breaker.get_metrics();
        assert_eq!(metrics.failure_count, 1);

        // Record a success - should reset failure count in closed state
        breaker.on_success();
        let metrics = breaker.get_metrics();
        assert_eq!(metrics.failure_count, 0);
    }

    #[test]
    fn test_circuit_breaker_on_failure_opens_circuit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..CircuitBreakerConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        // First failure
        breaker.on_failure();
        assert_eq!(breaker.get_state(), CircuitState::Closed);

        // Second failure should open the circuit
        breaker.on_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);
    }

    // CircuitMetrics tests
    #[test]
    fn test_circuit_metrics_debug() {
        let metrics = CircuitMetrics {
            failure_count: 5,
            success_count: 10,
            state: CircuitState::HalfOpen,
            last_failure_time: 12345,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("CircuitMetrics"));
        assert!(debug_str.contains("failure_count"));
        assert!(debug_str.contains("success_count"));
        assert!(debug_str.contains("state"));
        assert!(debug_str.contains("last_failure_time"));
    }

    #[test]
    fn test_circuit_metrics_values() {
        let metrics = CircuitMetrics {
            failure_count: 3,
            success_count: 7,
            state: CircuitState::Open,
            last_failure_time: 999999,
        };

        assert_eq!(metrics.failure_count, 3);
        assert_eq!(metrics.success_count, 7);
        assert_eq!(metrics.state, CircuitState::Open);
        assert_eq!(metrics.last_failure_time, 999999);
    }

    // CircuitBreakerError tests
    #[test]
    fn test_circuit_breaker_error_circuit_open() {
        let err: CircuitBreakerError<std::io::Error> = CircuitBreakerError::CircuitOpen;
        let display = format!("{}", err);
        assert!(display.contains("open"));

        let debug = format!("{:?}", err);
        assert!(debug.contains("CircuitOpen"));
    }

    #[test]
    fn test_circuit_breaker_error_operation_failed() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: CircuitBreakerError<std::io::Error> = CircuitBreakerError::OperationFailed(io_err);

        let display = format!("{}", err);
        assert!(display.contains("Operation failed"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_circuit_breaker_error_timeout() {
        let err: CircuitBreakerError<std::io::Error> = CircuitBreakerError::Timeout;
        let display = format!("{}", err);
        assert!(display.contains("timeout"));

        let debug = format!("{:?}", err);
        assert!(debug.contains("Timeout"));
    }
}
