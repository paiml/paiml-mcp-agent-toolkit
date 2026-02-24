#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_circuit_breaker_opens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_duration: Duration::from_secs(1),
            fallback_timeout: Duration::from_secs(1),
        };

        let breaker = CircuitBreaker::new(config);

        // Simulate failures
        for _ in 0..3 {
            let _ = breaker
                .call(
                    async { Err::<(), std::io::Error>(std::io::Error::other("test")) },
                    || (),
                )
                .await;
        }

        assert_eq!(breaker.get_state(), CircuitState::Open);
    }

    #[actix_rt::test]
    async fn test_circuit_breaker_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout_duration: Duration::from_millis(100),
            fallback_timeout: Duration::from_secs(1),
        };

        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        let _ = breaker
            .call(
                async { Err::<(), std::io::Error>(std::io::Error::other("test")) },
                || (),
            )
            .await;

        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        let _ = breaker
            .call(async { Ok::<(), std::io::Error>(()) }, || ())
            .await;

        // After one success in half-open, should close
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_metrics() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let metrics = breaker.get_metrics();

        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.state, CircuitState::Closed);
    }
}
