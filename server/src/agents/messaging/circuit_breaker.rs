use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
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
    pub failure_threshold: u32,      // Failures before opening
    pub success_threshold: u32,      // Successes to close from half-open
    pub timeout_duration: Duration,  // Time before half-open
    pub fallback_timeout: Duration,  // Max time for fallback
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
            .unwrap()
            .as_secs();

        now - last_failure > self.config.timeout_duration.as_secs()
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
            _ => {}
        }
    }

    fn on_failure(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

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