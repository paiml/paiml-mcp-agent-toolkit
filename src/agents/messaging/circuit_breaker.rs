#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,
    state: parking_lot::RwLock<CircuitState>,
    config: CircuitBreakerConfig,
}

#[derive(Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,     // Failures before opening
    pub success_threshold: u32,     // Successes to close from half-open
    pub timeout_duration: Duration, // Time before half-open
    pub fallback_timeout: Duration, // Max time for fallback
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(30),
            fallback_timeout: Duration::from_secs(5),
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            state: parking_lot::RwLock::new(CircuitState::Closed),
            config,
        }
    }

    pub async fn call<F, T, E>(
        &self,
        operation: F,
        fallback: impl Fn() -> T,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        // Check current state
        let current_state = *self.state.read();

        match current_state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    *self.state.write() = CircuitState::HalfOpen;
                } else {
                    return Ok(fallback());
                }
            }
            CircuitState::HalfOpen | CircuitState::Closed => {}
        }

        // Attempt operation with timeout
        match timeout(self.config.fallback_timeout, operation).await {
            Ok(Ok(result)) => {
                self.on_success();
                Ok(result)
            }
            Ok(Err(e)) => {
                self.on_failure();
                Err(CircuitBreakerError::OperationFailed(e))
            }
            Err(_) => {
                self.on_failure();
                Ok(fallback())
            }
        }
    }

    fn should_attempt_reset(&self) -> bool {
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        if last_failure == 0 {
            return true;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("internal error")
            .as_millis() as u64;

        now - last_failure > self.config.timeout_duration.as_millis() as u64
    }

    fn on_success(&self) {
        let current_state = *self.state.read();

        match current_state {
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;

                if success_count >= self.config.success_threshold {
                    *self.state.write() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Open => {
                // In Open state, we shouldn't be calling on_success
                // This is a logic error - do nothing
            }
        }
    }

    fn on_failure(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("internal error")
            .as_millis() as u64;

        self.last_failure_time.store(now, Ordering::SeqCst);

        let current_state = *self.state.read();

        match current_state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

                if failure_count >= self.config.failure_threshold {
                    *self.state.write() = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                *self.state.write() = CircuitState::Open;
                self.success_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    pub fn get_state(&self) -> CircuitState {
        *self.state.read()
    }

    pub fn get_metrics(&self) -> CircuitMetrics {
        CircuitMetrics {
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            state: self.get_state(),
            last_failure_time: self.last_failure_time.load(Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.last_failure_time.store(0, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct CircuitMetrics {
    pub failure_count: u32,
    pub success_count: u32,
    pub state: CircuitState,
    pub last_failure_time: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E: std::error::Error> {
    #[error("Circuit is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
    #[error("Operation timeout")]
    Timeout,
}

// Circuit breaker manager for multiple dependencies
pub struct CircuitBreakerManager {
    breakers: dashmap::DashMap<String, Arc<CircuitBreaker>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: dashmap::DashMap::new(),
            default_config,
        }
    }

    pub fn get_or_create(&self, name: &str) -> Arc<CircuitBreaker> {
        self.breakers
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(self.default_config.clone())))
            .clone()
    }

    pub fn get_all_metrics(&self) -> HashMap<String, CircuitMetrics> {
        self.breakers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().get_metrics()))
            .collect()
    }

    pub fn reset_all(&self) {
        for breaker in self.breakers.iter() {
            breaker.value().reset();
        }
    }
}

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
        for (_, m) in metrics.iter() {
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
        for (_, m) in metrics.iter() {
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
