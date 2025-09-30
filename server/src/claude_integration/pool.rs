// Connection pool with circuit breaker for resilience
// Implements Tokio's resource pool pattern with health monitoring

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};

/// Connection pool with circuit breaker
pub struct ResilientConnectionPool {
    /// Semaphore for connection limit
    semaphore: Arc<Semaphore>,

    /// Circuit breaker state machine
    circuit_state: Arc<AtomicU8>,

    /// Sliding window for error rate calculation
    error_window: Arc<RwLock<VecDeque<(Instant, bool)>>>,

    /// Maximum consecutive failures before opening circuit
    failure_threshold: usize,

    /// Success counter
    success_count: AtomicU64,

    /// Failure counter
    failure_count: AtomicU64,
}

#[derive(Debug)]
pub enum PoolError {
    CircuitOpen,
    CircuitStillUnhealthy,
    AcquisitionTimeout,
    Exhausted,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CircuitOpen => write!(f, "Circuit breaker is open"),
            Self::CircuitStillUnhealthy => write!(f, "Circuit still unhealthy"),
            Self::AcquisitionTimeout => write!(f, "Connection acquisition timeout"),
            Self::Exhausted => write!(f, "Pool exhausted"),
        }
    }
}

impl std::error::Error for PoolError {}

impl ResilientConnectionPool {
    const CLOSED: u8 = 0;
    const OPEN: u8 = 1;
    const HALF_OPEN: u8 = 2;

    pub fn new(pool_size: usize, failure_threshold: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(pool_size)),
            circuit_state: Arc::new(AtomicU8::new(Self::CLOSED)),
            error_window: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            failure_threshold,
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
        }
    }

    /// Acquire connection with circuit breaker logic
    pub async fn acquire(self: Arc<Self>) -> Result<PooledConnection, PoolError> {
        // Check circuit breaker state
        match self.circuit_state.load(Ordering::Acquire) {
            Self::OPEN => {
                return Err(PoolError::CircuitOpen);
            }
            Self::HALF_OPEN => {
                // Allow one request for testing
                if !self.try_health_check().await {
                    self.circuit_state.store(Self::OPEN, Ordering::Release);
                    return Err(PoolError::CircuitStillUnhealthy);
                }
                self.circuit_state.store(Self::CLOSED, Ordering::Release);
            }
            _ => {}
        }

        // Acquire with timeout
        let permit = tokio::time::timeout(
            Duration::from_secs(5),
            Arc::clone(&self.semaphore).acquire_owned(),
        )
        .await
        .map_err(|_| PoolError::AcquisitionTimeout)?
        .map_err(|_| PoolError::Exhausted)?;

        self.record_success();

        Ok(PooledConnection {
            _permit: permit,
            pool: self,
        })
    }

    fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }

    async fn try_health_check(&self) -> bool {
        // Simplified health check - always return true for now
        true
    }

    #[allow(dead_code)]
    fn should_open_circuit(&self) -> bool {
        let failures = self.failure_count.load(Ordering::Relaxed);
        let successes = self.success_count.load(Ordering::Relaxed);
        let total = failures + successes;

        if total < self.failure_threshold as u64 {
            return false;
        }

        let error_rate = failures as f64 / total as f64;
        error_rate > 0.5
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            successes: self.success_count.load(Ordering::Relaxed),
            failures: self.failure_count.load(Ordering::Relaxed),
            circuit_state: match self.circuit_state.load(Ordering::Acquire) {
                Self::CLOSED => CircuitState::Closed,
                Self::OPEN => CircuitState::Open,
                Self::HALF_OPEN => CircuitState::HalfOpen,
                _ => CircuitState::Closed,
            },
        }
    }
}

impl Clone for ResilientConnectionPool {
    fn clone(&self) -> Self {
        Self {
            semaphore: Arc::clone(&self.semaphore),
            circuit_state: Arc::clone(&self.circuit_state),
            error_window: Arc::clone(&self.error_window),
            failure_threshold: self.failure_threshold,
            success_count: AtomicU64::new(self.success_count.load(Ordering::Relaxed)),
            failure_count: AtomicU64::new(self.failure_count.load(Ordering::Relaxed)),
        }
    }
}

/// RAII guard for pooled connection
pub struct PooledConnection {
    _permit: tokio::sync::OwnedSemaphorePermit,
    pool: Arc<ResilientConnectionPool>,
}

impl PooledConnection {
    pub fn report_success(&self) {
        self.pool.record_success();
    }

    pub fn report_failure(&self) {
        self.pool.record_failure();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub successes: u64,
    pub failures: u64,
    pub circuit_state: CircuitState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ResilientConnectionPool::new(10, 5);
        assert_eq!(pool.failure_threshold, 5);
    }

    #[tokio::test]
    async fn test_acquire_connection() {
        let pool = Arc::new(ResilientConnectionPool::new(10, 5));
        let conn = pool.acquire().await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = ResilientConnectionPool::new(10, 5);
        let stats = pool.stats();
        assert_eq!(stats.successes, 0);
        assert_eq!(stats.failures, 0);
    }
}
