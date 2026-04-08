#![cfg_attr(coverage_nightly, coverage(off))]
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

// Token bucket algorithm for rate limiting
/// Limiter for controlling rate usage.
pub struct RateLimiter {
    capacity: u32,
    tokens: AtomicU32,
    refill_rate: u32, // Tokens per second
    last_refill: parking_lot::Mutex<Instant>,
}

// Adaptive backpressure controller
/// Backpressure controller.
pub struct BackpressureController {
    _max_queue_size: usize,
    current_queue_size: AtomicU64,
    semaphore: Arc<Semaphore>,
    metrics: Arc<parking_lot::RwLock<BackpressureMetrics>>,
}

#[derive(Debug, Clone, Default)]
/// Backpressure metrics.
pub struct BackpressureMetrics {
    pub rejected_count: u64,
    pub accepted_count: u64,
    pub queue_depth_sum: u64,
    pub max_queue_depth: u64,
    pub sample_count: u64,
}

/// Backpressure permit.
pub struct BackpressurePermit<'a> {
    _permit: tokio::sync::OwnedSemaphorePermit,
    controller: &'a BackpressureController,
}

#[derive(Debug, thiserror::Error)]
/// Error variants for backpressure operations.
pub enum BackpressureError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

// Adaptive rate controller that adjusts based on system load
/// Adaptive rate controller.
pub struct AdaptiveRateController {
    _base_rate: u32,
    current_rate: AtomicU32,
    min_rate: u32,
    max_rate: u32,
    rate_limiter: Arc<parking_lot::RwLock<RateLimiter>>,
    load_monitor: Arc<LoadMonitor>,
}

/// Monitor for load resources.
pub struct LoadMonitor {
    cpu_threshold: f64,
    memory_threshold: f64,
}

// Bulkhead pattern for resource isolation
/// Bulkhead.
pub struct Bulkhead {
    name: String,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicU32>,
    rejected_count: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
/// Bulkhead metrics.
pub struct BulkheadMetrics {
    pub name: String,
    pub max_concurrent: usize,
    pub active_count: u32,
    pub rejected_count: u64,
}

// --- Implementation includes ---
include!("backpressure_rate_limiter.rs");
include!("backpressure_controller.rs");
include!("backpressure_adaptive.rs");
include!("backpressure_bulkhead.rs");

// --- Test includes ---
include!("backpressure_tests.rs");
include!("backpressure_coverage_tests.rs");
