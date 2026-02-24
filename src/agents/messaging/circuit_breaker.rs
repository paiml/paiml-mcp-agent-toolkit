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

// --- Implementation ---
include!("circuit_breaker_impl.rs");

// --- Tests ---
include!("circuit_breaker_tests.rs");
include!("circuit_breaker_unit_tests.rs");
include!("circuit_breaker_async_tests.rs");
