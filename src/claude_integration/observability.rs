// Observability and metrics collection for Claude integration
// Implements RED method: Rate, Errors, Duration

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Metrics collector for Claude bridge operations
pub struct MetricsCollector {
    // Request metrics
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_error: AtomicU64,

    // Latency tracking (microseconds)
    latency_sum: AtomicU64,
    latency_count: AtomicU64,
    latency_min: AtomicU64,
    latency_max: AtomicU64,

    // Error tracking by type
    timeout_errors: AtomicU64,
    circuit_open_errors: AtomicU64,
    transport_errors: AtomicU64,
    application_errors: AtomicU64,

    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,

    // Pool metrics
    pool_acquisitions: AtomicU64,
    pool_exhaustions: AtomicU64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            latency_sum: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
            latency_min: AtomicU64::new(u64::MAX),
            latency_max: AtomicU64::new(0),
            timeout_errors: AtomicU64::new(0),
            circuit_open_errors: AtomicU64::new(0),
            transport_errors: AtomicU64::new(0),
            application_errors: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            pool_acquisitions: AtomicU64::new(0),
            pool_exhaustions: AtomicU64::new(0),
        }
    }

    /// Record a successful request
    pub fn record_success(&self, latency: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_success.fetch_add(1, Ordering::Relaxed);
        self.record_latency(latency);
    }

    /// Record a failed request
    pub fn record_error(&self, error_type: ErrorType, latency: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_error.fetch_add(1, Ordering::Relaxed);
        self.record_latency(latency);

        match error_type {
            ErrorType::Timeout => {
                self.timeout_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::CircuitOpen => {
                self.circuit_open_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Transport => {
                self.transport_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Application => {
                self.application_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Record latency measurement
    fn record_latency(&self, latency: Duration) {
        let micros = latency.as_micros() as u64;

        self.latency_sum.fetch_add(micros, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);

        // Update min
        self.latency_min.fetch_min(micros, Ordering::Relaxed);

        // Update max
        self.latency_max.fetch_max(micros, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record pool acquisition
    pub fn record_pool_acquisition(&self) {
        self.pool_acquisitions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record pool exhaustion
    pub fn record_pool_exhaustion(&self) {
        self.pool_exhaustions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> BridgeMetrics {
        let total = self.requests_total.load(Ordering::Relaxed);
        let success = self.requests_success.load(Ordering::Relaxed);
        let error = self.requests_error.load(Ordering::Relaxed);

        let latency_count = self.latency_count.load(Ordering::Relaxed);
        let avg_latency_us = if latency_count > 0 {
            self.latency_sum.load(Ordering::Relaxed) / latency_count
        } else {
            0
        };

        let cache_total =
            self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        let cache_hit_rate = if cache_total > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / cache_total as f64
        } else {
            0.0
        };

        BridgeMetrics {
            requests_total: total,
            requests_success: success,
            requests_error: error,
            success_rate: if total > 0 {
                success as f64 / total as f64
            } else {
                0.0
            },
            avg_latency_us,
            min_latency_us: self.latency_min.load(Ordering::Relaxed),
            max_latency_us: self.latency_max.load(Ordering::Relaxed),
            timeout_errors: self.timeout_errors.load(Ordering::Relaxed),
            circuit_open_errors: self.circuit_open_errors.load(Ordering::Relaxed),
            transport_errors: self.transport_errors.load(Ordering::Relaxed),
            application_errors: self.application_errors.load(Ordering::Relaxed),
            cache_hit_rate,
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            pool_acquisitions: self.pool_acquisitions.load(Ordering::Relaxed),
            pool_exhaustions: self.pool_exhaustions.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.requests_success.store(0, Ordering::Relaxed);
        self.requests_error.store(0, Ordering::Relaxed);
        self.latency_sum.store(0, Ordering::Relaxed);
        self.latency_count.store(0, Ordering::Relaxed);
        self.latency_min.store(u64::MAX, Ordering::Relaxed);
        self.latency_max.store(0, Ordering::Relaxed);
        self.timeout_errors.store(0, Ordering::Relaxed);
        self.circuit_open_errors.store(0, Ordering::Relaxed);
        self.transport_errors.store(0, Ordering::Relaxed);
        self.application_errors.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.pool_acquisitions.store(0, Ordering::Relaxed);
        self.pool_exhaustions.store(0, Ordering::Relaxed);
    }
}

/// Type of error for metrics tracking
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Timeout,
    CircuitOpen,
    Transport,
    Application,
}

/// Snapshot of bridge metrics
#[derive(Debug, Clone, Copy)]
pub struct BridgeMetrics {
    // Request metrics
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub success_rate: f64,

    // Latency metrics (microseconds)
    pub avg_latency_us: u64,
    pub min_latency_us: u64,
    pub max_latency_us: u64,

    // Error breakdown
    pub timeout_errors: u64,
    pub circuit_open_errors: u64,
    pub transport_errors: u64,
    pub application_errors: u64,

    // Cache metrics
    pub cache_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,

    // Pool metrics
    pub pool_acquisitions: u64,
    pub pool_exhaustions: u64,
}

impl BridgeMetrics {
    /// Format metrics for logging
    pub fn format_summary(&self) -> String {
        format!(
            "Requests: {total} (success: {success}, error: {error}, rate: {rate:.2}%) | \
             Latency: avg={avg}μs min={min}μs max={max}μs | \
             Cache: hits={hits} misses={misses} rate={cache_rate:.1}% | \
             Pool: acquired={acq} exhausted={exh}",
            total = self.requests_total,
            success = self.requests_success,
            error = self.requests_error,
            rate = self.success_rate * 100.0,
            avg = self.avg_latency_us,
            min = self.min_latency_us,
            max = self.max_latency_us,
            hits = self.cache_hits,
            misses = self.cache_misses,
            cache_rate = self.cache_hit_rate * 100.0,
            acq = self.pool_acquisitions,
            exh = self.pool_exhaustions,
        )
    }
}

/// Helper for timing operations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        let metrics = collector.snapshot();

        assert_eq!(metrics.requests_total, 0);
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[test]
    fn test_record_success() {
        let collector = MetricsCollector::new();

        collector.record_success(Duration::from_micros(1000));
        collector.record_success(Duration::from_micros(2000));

        let metrics = collector.snapshot();
        assert_eq!(metrics.requests_total, 2);
        assert_eq!(metrics.requests_success, 2);
        assert_eq!(metrics.requests_error, 0);
        assert_eq!(metrics.success_rate, 1.0);
        assert_eq!(metrics.avg_latency_us, 1500);
    }

    #[test]
    fn test_record_error() {
        let collector = MetricsCollector::new();

        collector.record_error(ErrorType::Timeout, Duration::from_micros(5000));
        collector.record_error(ErrorType::Transport, Duration::from_micros(100));

        let metrics = collector.snapshot();
        assert_eq!(metrics.requests_total, 2);
        assert_eq!(metrics.requests_error, 2);
        assert_eq!(metrics.timeout_errors, 1);
        assert_eq!(metrics.transport_errors, 1);
    }

    #[test]
    fn test_success_rate() {
        let collector = MetricsCollector::new();

        collector.record_success(Duration::from_micros(1000));
        collector.record_success(Duration::from_micros(1000));
        collector.record_error(ErrorType::Application, Duration::from_micros(1000));

        let metrics = collector.snapshot();
        assert_eq!(metrics.requests_total, 3);
        assert!((metrics.success_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_metrics() {
        let collector = MetricsCollector::new();

        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_miss();

        let metrics = collector.snapshot();
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert!((metrics.cache_hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_latency_tracking() {
        let collector = MetricsCollector::new();

        collector.record_success(Duration::from_micros(100));
        collector.record_success(Duration::from_micros(500));
        collector.record_success(Duration::from_micros(300));

        let metrics = collector.snapshot();
        assert_eq!(metrics.min_latency_us, 100);
        assert_eq!(metrics.max_latency_us, 500);
        assert_eq!(metrics.avg_latency_us, 300);
    }

    #[test]
    fn test_reset() {
        let collector = MetricsCollector::new();

        collector.record_success(Duration::from_micros(1000));
        collector.record_cache_hit();

        collector.reset();

        let metrics = collector.snapshot();
        assert_eq!(metrics.requests_total, 0);
        assert_eq!(metrics.cache_hits, 0);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_metrics_format() {
        let collector = MetricsCollector::new();
        collector.record_success(Duration::from_micros(1500));
        collector.record_cache_hit();

        let metrics = collector.snapshot();
        let summary = metrics.format_summary();

        assert!(summary.contains("Requests: 1"));
        assert!(summary.contains("success: 1"));
        assert!(summary.contains("rate: 100.00%"));
    }
}
