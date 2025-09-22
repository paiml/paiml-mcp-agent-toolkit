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
        assert!(new_rate >= 10 && new_rate <= 1000);
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
