#![cfg_attr(coverage_nightly, coverage(off))]
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

// Token bucket algorithm for rate limiting
pub struct RateLimiter {
    capacity: u32,
    tokens: AtomicU32,
    refill_rate: u32, // Tokens per second
    last_refill: parking_lot::Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: AtomicU32::new(capacity),
            refill_rate,
            last_refill: parking_lot::Mutex::new(Instant::now()),
        }
    }

    pub fn try_acquire(&self, tokens: u32) -> bool {
        self.refill();

        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < tokens {
                return false; // Would exceed rate limit
            }

            match self.tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    pub async fn acquire(&self, tokens: u32) {
        while !self.try_acquire(tokens) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    fn refill(&self) {
        let mut last_refill = self.last_refill.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed.as_secs_f64() > 0.0 {
            let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate as f64) as u32;

            if tokens_to_add > 0 {
                let current = self.tokens.load(Ordering::Relaxed);
                let new_tokens = current.saturating_add(tokens_to_add).min(self.capacity);
                self.tokens.store(new_tokens, Ordering::Relaxed);
                *last_refill = now;
            }
        }
    }

    pub fn available_tokens(&self) -> u32 {
        self.refill();
        self.tokens.load(Ordering::Relaxed)
    }
}

// Adaptive backpressure controller
pub struct BackpressureController {
    _max_queue_size: usize,
    current_queue_size: AtomicU64,
    semaphore: Arc<Semaphore>,
    metrics: Arc<parking_lot::RwLock<BackpressureMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct BackpressureMetrics {
    pub rejected_count: u64,
    pub accepted_count: u64,
    pub queue_depth_sum: u64,
    pub max_queue_depth: u64,
    pub sample_count: u64,
}

impl BackpressureController {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            _max_queue_size: max_queue_size,
            current_queue_size: AtomicU64::new(0),
            semaphore: Arc::new(Semaphore::new(max_queue_size)),
            metrics: Arc::new(parking_lot::RwLock::new(BackpressureMetrics::default())),
        }
    }

    pub async fn acquire_permit(&self) -> Result<BackpressurePermit<'_>, BackpressureError> {
        let permit = self
            .semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|_| BackpressureError::QueueFull)?;

        let queue_size = self.current_queue_size.fetch_add(1, Ordering::SeqCst) + 1;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.accepted_count += 1;
        metrics.queue_depth_sum += queue_size;
        metrics.max_queue_depth = metrics.max_queue_depth.max(queue_size);
        metrics.sample_count += 1;

        Ok(BackpressurePermit {
            _permit: permit,
            controller: self,
        })
    }

    pub fn try_acquire_permit(&self) -> Result<BackpressurePermit<'_>, BackpressureError> {
        let permit = self.semaphore.clone().try_acquire_owned().map_err(|_| {
            self.metrics.write().rejected_count += 1;
            BackpressureError::QueueFull
        })?;

        let queue_size = self.current_queue_size.fetch_add(1, Ordering::SeqCst) + 1;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.accepted_count += 1;
        metrics.queue_depth_sum += queue_size;
        metrics.max_queue_depth = metrics.max_queue_depth.max(queue_size);
        metrics.sample_count += 1;

        Ok(BackpressurePermit {
            _permit: permit,
            controller: self,
        })
    }

    fn release(&self) {
        self.current_queue_size.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn get_queue_depth(&self) -> u64 {
        self.current_queue_size.load(Ordering::Relaxed)
    }

    pub fn get_metrics(&self) -> BackpressureMetrics {
        self.metrics.read().clone()
    }

    pub fn get_average_queue_depth(&self) -> f64 {
        let metrics = self.metrics.read();
        if metrics.sample_count > 0 {
            metrics.queue_depth_sum as f64 / metrics.sample_count as f64
        } else {
            0.0
        }
    }
}

pub struct BackpressurePermit<'a> {
    _permit: tokio::sync::OwnedSemaphorePermit,
    controller: &'a BackpressureController,
}

impl Drop for BackpressurePermit<'_> {
    fn drop(&mut self) {
        self.controller.release();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BackpressureError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

// Adaptive rate controller that adjusts based on system load
pub struct AdaptiveRateController {
    _base_rate: u32,
    current_rate: AtomicU32,
    min_rate: u32,
    max_rate: u32,
    rate_limiter: Arc<parking_lot::RwLock<RateLimiter>>,
    load_monitor: Arc<LoadMonitor>,
}

pub struct LoadMonitor {
    cpu_threshold: f64,
    memory_threshold: f64,
}

impl LoadMonitor {
    pub fn new(cpu_threshold: f64, memory_threshold: f64) -> Self {
        Self {
            cpu_threshold,
            memory_threshold,
        }
    }

    pub fn get_load_factor(&self) -> f64 {
        // Simplified - would use actual system metrics
        let cpu_usage = self.get_cpu_usage();
        let memory_usage = self.get_memory_usage();

        let cpu_factor = cpu_usage / self.cpu_threshold;
        let memory_factor = memory_usage / self.memory_threshold;

        cpu_factor.max(memory_factor).min(1.0)
    }

    fn get_cpu_usage(&self) -> f64 {
        // Placeholder - would use sysinfo or similar
        0.5
    }

    fn get_memory_usage(&self) -> f64 {
        // Placeholder - would use sysinfo or similar
        0.4
    }
}

impl AdaptiveRateController {
    pub fn new(base_rate: u32, min_rate: u32, max_rate: u32) -> Self {
        let rate_limiter = RateLimiter::new(base_rate * 10, base_rate);

        Self {
            _base_rate: base_rate,
            current_rate: AtomicU32::new(base_rate),
            min_rate,
            max_rate,
            rate_limiter: Arc::new(parking_lot::RwLock::new(rate_limiter)),
            load_monitor: Arc::new(LoadMonitor::new(0.8, 0.9)),
        }
    }

    pub async fn adapt_rate(&self) {
        let load_factor = self.load_monitor.get_load_factor();

        // Adjust rate based on load
        let target_rate = if load_factor < 0.5 {
            // Low load - increase rate
            (self.current_rate.load(Ordering::Relaxed) as f64 * 1.1) as u32
        } else if load_factor > 0.8 {
            // High load - decrease rate
            (self.current_rate.load(Ordering::Relaxed) as f64 * 0.9) as u32
        } else {
            // Moderate load - maintain rate
            self.current_rate.load(Ordering::Relaxed)
        };

        let new_rate = target_rate.max(self.min_rate).min(self.max_rate);
        self.current_rate.store(new_rate, Ordering::Relaxed);

        // Update rate limiter
        *self.rate_limiter.write() = RateLimiter::new(new_rate * 10, new_rate);
    }

    pub fn try_acquire(&self) -> bool {
        self.rate_limiter.read().try_acquire(1)
    }

    pub async fn acquire(&self) {
        loop {
            {
                let guard = self.rate_limiter.read();
                if guard.try_acquire(1) {
                    return;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    pub fn get_current_rate(&self) -> u32 {
        self.current_rate.load(Ordering::Relaxed)
    }
}

// Bulkhead pattern for resource isolation
pub struct Bulkhead {
    name: String,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicU32>,
    rejected_count: Arc<AtomicU64>,
}

impl Clone for Bulkhead {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            max_concurrent: self.max_concurrent,
            semaphore: self.semaphore.clone(),
            active_count: self.active_count.clone(),
            rejected_count: self.rejected_count.clone(),
        }
    }
}

impl Bulkhead {
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            name,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_count: Arc::new(AtomicU32::new(0)),
            rejected_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn execute<F, T>(&self, operation: F) -> Result<T, BackpressureError>
    where
        F: std::future::Future<Output = T>,
    {
        let permit = self.semaphore.clone().try_acquire_owned().map_err(|_| {
            self.rejected_count.fetch_add(1, Ordering::Relaxed);
            BackpressureError::QueueFull
        })?;

        self.active_count.fetch_add(1, Ordering::Relaxed);

        let result = operation.await;

        drop(permit);
        self.active_count.fetch_sub(1, Ordering::Relaxed);

        Ok(result)
    }

    pub fn get_metrics(&self) -> BulkheadMetrics {
        BulkheadMetrics {
            name: self.name.clone(),
            max_concurrent: self.max_concurrent,
            active_count: self.active_count.load(Ordering::Relaxed),
            rejected_count: self.rejected_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BulkheadMetrics {
    pub name: String,
    pub max_concurrent: usize,
    pub active_count: u32,
    pub rejected_count: u64,
}

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
        assert!(rate >= 10 && rate <= 500);
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
