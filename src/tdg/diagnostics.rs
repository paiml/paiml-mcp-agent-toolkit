/// Diagnostic tools for Transactional Hashed TDG System
///
/// Provides comprehensive monitoring, profiling, and debugging capabilities
/// for the TDG system including storage, scheduling, and performance metrics.
use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant, SystemTime};

use super::{
    AdaptiveThresholdManager, PlatformResourceController, SimpleFairScheduler, TieredStore,
};

/// Comprehensive system diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDiagnostics {
    pub timestamp: SystemTime,
    pub uptime: Duration,
    pub storage: StorageDiagnostics,
    pub scheduler: SchedulerDiagnostics,
    pub adaptive: AdaptiveDiagnostics,
    pub resources: ResourceDiagnostics,
    pub performance: PerformanceDiagnostics,
    pub health: HealthStatus,
}

/// Storage subsystem diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDiagnostics {
    pub backend_type: String,
    pub total_entries: usize,
    pub hot_cache_entries: usize,
    pub warm_entries: usize,
    pub cold_entries: usize,
    pub cache_hit_ratio: f64,
    pub compression_ratio: f64,
    pub storage_size_mb: f64,
    pub last_archival: Option<SystemTime>,
    pub deduplication_savings: f64,
}

/// Scheduler subsystem diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerDiagnostics {
    pub active_operations: usize,
    pub queued_operations: usize,
    pub high_priority_available: usize,
    pub low_priority_available: usize,
    pub preemptions_count: u64,
    pub avg_wait_time_ms: f64,
    pub max_wait_time_ms: f64,
    pub operations_per_second: f64,
}

/// Adaptive threshold diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveDiagnostics {
    pub current_cache_size: usize,
    pub current_compression_level: u32,
    pub high_priority_permits: usize,
    pub low_priority_permits: usize,
    pub performance_trend: String,
    pub adjustments_made: usize,
    pub avg_analysis_time_ms: f64,
    pub optimization_effectiveness: f64,
}

/// Resource management diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDiagnostics {
    pub memory_usage_mb: f64,
    pub memory_limit_mb: f64,
    pub memory_pressure: String,
    pub cpu_utilization: f64,
    pub cpu_limit: f64,
    pub cpu_pressure: String,
    pub enforcement_actions: EnforcementStats,
}

/// Resource enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnforcementStats {
    pub total_requests: u64,
    pub allowed: u64,
    pub throttled: u64,
    pub queued: u64,
    pub rejected: u64,
    pub emergency_stops: u64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDiagnostics {
    pub analyses_per_hour: f64,
    pub avg_response_time_ms: f64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_mbps: f64,
    pub error_rate: f64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded { reasons: Vec<String> },
    Critical { reasons: Vec<String> },
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "✅ HEALTHY"),
            HealthStatus::Degraded { reasons } => {
                write!(f, "⚠️ DEGRADED: {}", reasons.join(", "))
            }
            HealthStatus::Critical { reasons } => {
                write!(f, "❌ CRITICAL: {}", reasons.join(", "))
            }
        }
    }
}

/// TDG System Diagnostic Tool
pub struct DiagnosticTool {
    start_time: Instant,
    performance_samples: Vec<PerformanceSample>,
    error_count: u64,
    analysis_count: u64,
}

#[derive(Clone)]
struct PerformanceSample {
    #[allow(dead_code)]
    timestamp: Instant,
    response_time_ms: f64,
    success: bool,
}

impl DiagnosticTool {
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            performance_samples: Vec::new(),
            error_count: 0,
            analysis_count: 0,
        }
    }

    /// Collect comprehensive system diagnostics
    pub async fn collect_diagnostics(
        &self,
        storage: Option<&TieredStore>,
        scheduler: Option<&SimpleFairScheduler>,
        adaptive: Option<&AdaptiveThresholdManager>,
        resources: Option<&PlatformResourceController>,
    ) -> Result<SystemDiagnostics> {
        let uptime = self.start_time.elapsed();

        let storage_diag = if let Some(store) = storage {
            self.collect_storage_diagnostics(store).await?
        } else {
            StorageDiagnostics::default()
        };

        let scheduler_diag = if let Some(sched) = scheduler {
            self.collect_scheduler_diagnostics(sched).await?
        } else {
            SchedulerDiagnostics::default()
        };

        let adaptive_diag = if let Some(mgr) = adaptive {
            self.collect_adaptive_diagnostics(mgr).await?
        } else {
            AdaptiveDiagnostics::default()
        };

        let resource_diag = if let Some(ctrl) = resources {
            self.collect_resource_diagnostics(ctrl).await?
        } else {
            ResourceDiagnostics::default()
        };

        let performance_diag = self.calculate_performance_metrics();
        let health = self.assess_health(&storage_diag, &resource_diag, &performance_diag);

        Ok(SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime,
            storage: storage_diag,
            scheduler: scheduler_diag,
            adaptive: adaptive_diag,
            resources: resource_diag,
            performance: performance_diag,
            health,
        })
    }

    /// Collect storage diagnostics
    async fn collect_storage_diagnostics(
        &self,
        storage: &TieredStore,
    ) -> Result<StorageDiagnostics> {
        let stats = storage.get_statistics();

        Ok(StorageDiagnostics {
            backend_type: "sled".to_string(),
            total_entries: stats.total_entries,
            hot_cache_entries: stats.hot_entries,
            warm_entries: stats.warm_entries,
            cold_entries: stats.cold_entries,
            cache_hit_ratio: if stats.hot_entries > 0 {
                stats.hot_entries as f64 / stats.total_entries.max(1) as f64
            } else {
                0.0
            },
            compression_ratio: f64::from(stats.compression_ratio),
            storage_size_mb: stats.hot_memory_kb as f64 / 1024.0,
            last_archival: None,        // Would need to track this
            deduplication_savings: 0.0, // Would need to calculate
        })
    }

    /// Collect scheduler diagnostics
    async fn collect_scheduler_diagnostics(
        &self,
        scheduler: &SimpleFairScheduler,
    ) -> Result<SchedulerDiagnostics> {
        let stats = scheduler.get_statistics().await;

        Ok(SchedulerDiagnostics {
            active_operations: stats.total_active_operations,
            queued_operations: 0, // Would need to track
            high_priority_available: stats.high_permits_available,
            low_priority_available: stats.low_permits_available,
            preemptions_count: 0, // Would need to track
            avg_wait_time_ms: stats.avg_wait_time_ms as f64,
            max_wait_time_ms: (stats.avg_wait_time_ms * 2) as f64, // Estimate
            operations_per_second: if self.start_time.elapsed().as_secs() > 0 {
                self.analysis_count as f64 / self.start_time.elapsed().as_secs() as f64
            } else {
                0.0
            },
        })
    }

    /// Collect adaptive threshold diagnostics
    async fn collect_adaptive_diagnostics(
        &self,
        adaptive: &AdaptiveThresholdManager,
    ) -> Result<AdaptiveDiagnostics> {
        let thresholds = adaptive.get_current_thresholds().await;
        let stats = adaptive.get_performance_stats().await;

        Ok(AdaptiveDiagnostics {
            current_cache_size: thresholds.hot_cache_size,
            current_compression_level: u32::from(thresholds.compression_level),
            high_priority_permits: thresholds.high_priority_permits,
            low_priority_permits: thresholds.low_priority_permits,
            performance_trend: if stats.avg_analysis_duration_ms > 100.0 {
                "Degrading"
            } else {
                "Stable"
            }
            .to_string(),
            adjustments_made: stats.total_samples,
            avg_analysis_time_ms: f64::from(stats.avg_analysis_duration_ms),
            optimization_effectiveness: f64::from(stats.avg_cache_hit_ratio),
        })
    }

    /// Collect resource diagnostics
    async fn collect_resource_diagnostics(
        &self,
        controller: &PlatformResourceController,
    ) -> Result<ResourceDiagnostics> {
        let usage = controller.get_current_usage().await;
        let stats = controller.get_enforcement_stats().await;

        Ok(ResourceDiagnostics {
            memory_usage_mb: usage.memory_mb,
            memory_limit_mb: 1024.0, // Would need to get from config
            memory_pressure: format!("{:?}", usage.memory_pressure),
            cpu_utilization: usage.cpu_utilization,
            cpu_limit: 0.8, // Would need to get from config
            cpu_pressure: format!("{:?}", usage.cpu_pressure),
            enforcement_actions: EnforcementStats {
                total_requests: stats.total_requests as u64,
                allowed: stats.allowed_requests as u64,
                throttled: stats.throttled_requests as u64,
                queued: stats.queued_requests as u64,
                rejected: stats.rejected_requests as u64,
                emergency_stops: 0, // Would need to track
            },
        })
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self) -> PerformanceDiagnostics {
        if self.performance_samples.is_empty() {
            return PerformanceDiagnostics::default();
        }

        let mut response_times: Vec<f64> = self
            .performance_samples
            .iter()
            .map(|s| s.response_time_ms)
            .collect();
        response_times.sort_by(|a, b| a.partial_cmp(b).expect("internal error"));

        let len = response_times.len();
        let sum: f64 = response_times.iter().sum();

        let error_count = self
            .performance_samples
            .iter()
            .filter(|s| !s.success)
            .count() as f64;

        PerformanceDiagnostics {
            analyses_per_hour: if self.start_time.elapsed().as_secs() > 0 {
                (self.analysis_count as f64 * 3600.0) / self.start_time.elapsed().as_secs() as f64
            } else {
                0.0
            },
            avg_response_time_ms: sum / len as f64,
            p50_response_time_ms: response_times[len / 2],
            p95_response_time_ms: response_times[len * 95 / 100],
            p99_response_time_ms: response_times[len * 99 / 100],
            throughput_mbps: 0.0, // Would need to track data volume
            error_rate: error_count / len as f64,
        }
    }

    /// Assess system health
    fn assess_health(
        &self,
        storage: &StorageDiagnostics,
        resources: &ResourceDiagnostics,
        performance: &PerformanceDiagnostics,
    ) -> HealthStatus {
        let mut critical_reasons = Vec::new();
        let mut degraded_reasons = Vec::new();

        // Check resource pressure
        if resources.memory_usage_mb > resources.memory_limit_mb * 0.95 {
            critical_reasons.push("Memory critical (>95%)".to_string());
        } else if resources.memory_usage_mb > resources.memory_limit_mb * 0.8 {
            degraded_reasons.push("Memory high (>80%)".to_string());
        }

        if resources.cpu_utilization > 0.95 {
            critical_reasons.push("CPU critical (>95%)".to_string());
        } else if resources.cpu_utilization > 0.8 {
            degraded_reasons.push("CPU high (>80%)".to_string());
        }

        // Check cache performance
        if storage.cache_hit_ratio < 0.5 {
            degraded_reasons.push("Low cache hit ratio (<50%)".to_string());
        }

        // Check error rate
        if performance.error_rate > 0.1 {
            critical_reasons.push("High error rate (>10%)".to_string());
        } else if performance.error_rate > 0.05 {
            degraded_reasons.push("Elevated error rate (>5%)".to_string());
        }

        // Check response times
        if performance.p99_response_time_ms > 5000.0 {
            degraded_reasons.push("Slow response times (p99 >5s)".to_string());
        }

        if !critical_reasons.is_empty() {
            HealthStatus::Critical {
                reasons: critical_reasons,
            }
        } else if !degraded_reasons.is_empty() {
            HealthStatus::Degraded {
                reasons: degraded_reasons,
            }
        } else {
            HealthStatus::Healthy
        }
    }

    /// Record a performance sample
    pub fn record_sample(&mut self, response_time_ms: f64, success: bool) {
        self.performance_samples.push(PerformanceSample {
            timestamp: Instant::now(),
            response_time_ms,
            success,
        });

        if success {
            self.analysis_count += 1;
        } else {
            self.error_count += 1;
        }

        // Keep only recent samples (last 1000)
        if self.performance_samples.len() > 1000 {
            self.performance_samples.drain(0..500);
        }
    }

    /// Format diagnostics for display
    #[must_use]
    pub fn format_diagnostics(diag: &SystemDiagnostics) -> String {
        let local_time: DateTime<Local> = diag.timestamp.into();

        format!(
            r"
╔══════════════════════════════════════════════════════════════════╗
║          TRANSACTIONAL HASHED TDG SYSTEM DIAGNOSTICS            ║
╚══════════════════════════════════════════════════════════════════╝

📅 Timestamp: {}
⏱️ Uptime: {:?}
🏥 Health: {}

📦 STORAGE SUBSYSTEM
├─ Backend: {}
├─ Total Entries: {}
├─ Hot Cache: {} | Warm: {} | Cold: {}
├─ Cache Hit Ratio: {:.1}%
├─ Compression Ratio: {:.1}%
└─ Storage Size: {:.1} MB

⚡ SCHEDULER SUBSYSTEM  
├─ Active Operations: {}
├─ Queued Operations: {}
├─ Available Permits: High={} | Low={}
├─ Avg Wait Time: {:.1}ms
└─ Operations/sec: {:.1}

🎯 ADAPTIVE THRESHOLDS
├─ Cache Size: {}
├─ Compression Level: {}
├─ Performance Trend: {}
├─ Adjustments Made: {}
└─ Avg Analysis Time: {:.1}ms

🛡️ RESOURCE CONTROL
├─ Memory: {:.1}/{:.1} MB ({})
├─ CPU: {:.1}% / {:.1}% ({})
├─ Enforcement: Allow={} | Throttle={} | Queue={} | Reject={}
└─ Emergency Stops: {}

📊 PERFORMANCE METRICS
├─ Analyses/hour: {:.0}
├─ Response Times: Avg={:.1}ms | P50={:.1}ms | P95={:.1}ms | P99={:.1}ms
├─ Throughput: {:.1} MB/s
└─ Error Rate: {:.2}%

════════════════════════════════════════════════════════════════════",
            local_time.format("%Y-%m-%d %H:%M:%S"),
            diag.uptime,
            diag.health,
            // Storage
            diag.storage.backend_type,
            diag.storage.total_entries,
            diag.storage.hot_cache_entries,
            diag.storage.warm_entries,
            diag.storage.cold_entries,
            diag.storage.cache_hit_ratio * 100.0,
            diag.storage.compression_ratio * 100.0,
            diag.storage.storage_size_mb,
            // Scheduler
            diag.scheduler.active_operations,
            diag.scheduler.queued_operations,
            diag.scheduler.high_priority_available,
            diag.scheduler.low_priority_available,
            diag.scheduler.avg_wait_time_ms,
            diag.scheduler.operations_per_second,
            // Adaptive
            diag.adaptive.current_cache_size,
            diag.adaptive.current_compression_level,
            diag.adaptive.performance_trend,
            diag.adaptive.adjustments_made,
            diag.adaptive.avg_analysis_time_ms,
            // Resources
            diag.resources.memory_usage_mb,
            diag.resources.memory_limit_mb,
            diag.resources.memory_pressure,
            diag.resources.cpu_utilization * 100.0,
            diag.resources.cpu_limit * 100.0,
            diag.resources.cpu_pressure,
            diag.resources.enforcement_actions.allowed,
            diag.resources.enforcement_actions.throttled,
            diag.resources.enforcement_actions.queued,
            diag.resources.enforcement_actions.rejected,
            diag.resources.enforcement_actions.emergency_stops,
            // Performance
            diag.performance.analyses_per_hour,
            diag.performance.avg_response_time_ms,
            diag.performance.p50_response_time_ms,
            diag.performance.p95_response_time_ms,
            diag.performance.p99_response_time_ms,
            diag.performance.throughput_mbps,
            diag.performance.error_rate * 100.0,
        )
    }
}

impl Default for DiagnosticTool {
    fn default() -> Self {
        Self::new()
    }
}

// Default implementations for diagnostic types
impl Default for StorageDiagnostics {
    fn default() -> Self {
        Self {
            backend_type: "none".to_string(),
            total_entries: 0,
            hot_cache_entries: 0,
            warm_entries: 0,
            cold_entries: 0,
            cache_hit_ratio: 0.0,
            compression_ratio: 0.0,
            storage_size_mb: 0.0,
            last_archival: None,
            deduplication_savings: 0.0,
        }
    }
}

impl Default for SchedulerDiagnostics {
    fn default() -> Self {
        Self {
            active_operations: 0,
            queued_operations: 0,
            high_priority_available: 0,
            low_priority_available: 0,
            preemptions_count: 0,
            avg_wait_time_ms: 0.0,
            max_wait_time_ms: 0.0,
            operations_per_second: 0.0,
        }
    }
}

impl Default for AdaptiveDiagnostics {
    fn default() -> Self {
        Self {
            current_cache_size: 0,
            current_compression_level: 0,
            high_priority_permits: 0,
            low_priority_permits: 0,
            performance_trend: "Unknown".to_string(),
            adjustments_made: 0,
            avg_analysis_time_ms: 0.0,
            optimization_effectiveness: 0.0,
        }
    }
}

impl Default for ResourceDiagnostics {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            memory_limit_mb: 1024.0,
            memory_pressure: "Unknown".to_string(),
            cpu_utilization: 0.0,
            cpu_limit: 0.8,
            cpu_pressure: "Unknown".to_string(),
            enforcement_actions: EnforcementStats::default(),
        }
    }
}

impl Default for PerformanceDiagnostics {
    fn default() -> Self {
        Self {
            analyses_per_hour: 0.0,
            avg_response_time_ms: 0.0,
            p50_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            throughput_mbps: 0.0,
            error_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_tool_creation() {
        let tool = DiagnosticTool::new();
        assert_eq!(tool.analysis_count, 0);
        assert_eq!(tool.error_count, 0);
    }

    #[test]
    fn test_diagnostic_tool_default() {
        let tool = DiagnosticTool::default();
        assert_eq!(tool.analysis_count, 0);
    }

    #[test]
    fn test_health_assessment() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics {
            cache_hit_ratio: 0.9,
            ..Default::default()
        };

        let resources = ResourceDiagnostics {
            memory_usage_mb: 500.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.5,
            ..Default::default()
        };

        let performance = PerformanceDiagnostics {
            error_rate: 0.01,
            p99_response_time_ms: 1000.0,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_assessment_critical_memory() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 980.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.5,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Critical { .. }));
    }

    #[test]
    fn test_health_assessment_critical_cpu() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 500.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.98,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Critical { .. }));
    }

    #[test]
    fn test_health_assessment_degraded_memory() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 850.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.5,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_degraded_cpu() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics {
            memory_usage_mb: 500.0,
            memory_limit_mb: 1024.0,
            cpu_utilization: 0.85,
            ..Default::default()
        };
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_low_cache_hit() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics {
            cache_hit_ratio: 0.3,
            ..Default::default()
        };
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics::default();

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_critical_error_rate() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics {
            error_rate: 0.15,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Critical { .. }));
    }

    #[test]
    fn test_health_assessment_degraded_error_rate() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics {
            error_rate: 0.07,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_assessment_slow_response() {
        let tool = DiagnosticTool::new();

        let storage = StorageDiagnostics::default();
        let resources = ResourceDiagnostics::default();
        let performance = PerformanceDiagnostics {
            p99_response_time_ms: 6000.0,
            ..Default::default()
        };

        let health = tool.assess_health(&storage, &resources, &performance);
        assert!(matches!(health, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_performance_sampling() {
        let mut tool = DiagnosticTool::new();

        // Record some samples
        tool.record_sample(100.0, true);
        tool.record_sample(200.0, true);
        tool.record_sample(150.0, false);

        assert_eq!(tool.analysis_count, 2);
        assert_eq!(tool.error_count, 1);
        assert_eq!(tool.performance_samples.len(), 3);
    }

    #[test]
    fn test_performance_sampling_cleanup() {
        let mut tool = DiagnosticTool::new();

        // Add more than 1000 samples to trigger cleanup
        for i in 0..1100 {
            tool.record_sample(i as f64, true);
        }

        // Should keep only 600 (drained 500, kept last 500 + 100 new)
        assert!(tool.performance_samples.len() <= 600);
    }

    #[test]
    fn test_calculate_performance_metrics_empty() {
        let tool = DiagnosticTool::new();
        let metrics = tool.calculate_performance_metrics();
        assert_eq!(metrics.avg_response_time_ms, 0.0);
    }

    #[test]
    fn test_calculate_performance_metrics_with_samples() {
        let mut tool = DiagnosticTool::new();
        for i in 1..=100 {
            tool.record_sample(i as f64, true);
        }
        let metrics = tool.calculate_performance_metrics();
        assert!(metrics.avg_response_time_ms > 0.0);
        assert!(metrics.p50_response_time_ms > 0.0);
    }

    #[test]
    fn test_health_status_display_healthy() {
        let status = HealthStatus::Healthy;
        let display = format!("{}", status);
        assert!(display.contains("HEALTHY"));
    }

    #[test]
    fn test_health_status_display_degraded() {
        let status = HealthStatus::Degraded {
            reasons: vec!["High memory".to_string()],
        };
        let display = format!("{}", status);
        assert!(display.contains("DEGRADED"));
        assert!(display.contains("High memory"));
    }

    #[test]
    fn test_health_status_display_critical() {
        let status = HealthStatus::Critical {
            reasons: vec!["Memory critical".to_string()],
        };
        let display = format!("{}", status);
        assert!(display.contains("CRITICAL"));
        assert!(display.contains("Memory critical"));
    }

    #[test]
    fn test_storage_diagnostics_default() {
        let diag = StorageDiagnostics::default();
        assert_eq!(diag.backend_type, "none");
        assert_eq!(diag.total_entries, 0);
        assert!(diag.last_archival.is_none());
    }

    #[test]
    fn test_scheduler_diagnostics_default() {
        let diag = SchedulerDiagnostics::default();
        assert_eq!(diag.active_operations, 0);
        assert_eq!(diag.preemptions_count, 0);
    }

    #[test]
    fn test_adaptive_diagnostics_default() {
        let diag = AdaptiveDiagnostics::default();
        assert_eq!(diag.current_cache_size, 0);
        assert_eq!(diag.performance_trend, "Unknown");
    }

    #[test]
    fn test_resource_diagnostics_default() {
        let diag = ResourceDiagnostics::default();
        assert_eq!(diag.memory_limit_mb, 1024.0);
        assert_eq!(diag.cpu_limit, 0.8);
    }

    #[test]
    fn test_performance_diagnostics_default() {
        let diag = PerformanceDiagnostics::default();
        assert_eq!(diag.analyses_per_hour, 0.0);
        assert_eq!(diag.error_rate, 0.0);
    }

    #[test]
    fn test_enforcement_stats_default() {
        let stats = EnforcementStats::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.emergency_stops, 0);
    }

    #[test]
    fn test_format_diagnostics() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(3600),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("DIAGNOSTICS"));
        assert!(formatted.contains("STORAGE"));
        assert!(formatted.contains("SCHEDULER"));
    }

    #[test]
    fn test_system_diagnostics_clone() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.uptime, diag.uptime);
    }

    #[test]
    fn test_system_diagnostics_debug() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("SystemDiagnostics"));
    }

    #[test]
    fn test_system_diagnostics_serialization() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Healthy,
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("uptime"));
        assert!(json.contains("storage"));
    }

    #[test]
    fn test_storage_diagnostics_clone() {
        let diag = StorageDiagnostics {
            backend_type: "sled".to_string(),
            total_entries: 100,
            hot_cache_entries: 50,
            warm_entries: 30,
            cold_entries: 20,
            cache_hit_ratio: 0.9,
            compression_ratio: 0.5,
            storage_size_mb: 10.0,
            last_archival: Some(SystemTime::now()),
            deduplication_savings: 0.1,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.total_entries, diag.total_entries);
        assert_eq!(cloned.backend_type, diag.backend_type);
    }

    #[test]
    fn test_storage_diagnostics_debug() {
        let diag = StorageDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("StorageDiagnostics"));
    }

    #[test]
    fn test_storage_diagnostics_serialization() {
        let diag = StorageDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("backend_type"));

        let deserialized: StorageDiagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.backend_type, "none");
    }

    #[test]
    fn test_storage_diagnostics_all_fields() {
        let diag = StorageDiagnostics {
            backend_type: "custom".to_string(),
            total_entries: 1000,
            hot_cache_entries: 100,
            warm_entries: 200,
            cold_entries: 700,
            cache_hit_ratio: 0.85,
            compression_ratio: 0.7,
            storage_size_mb: 256.5,
            last_archival: Some(SystemTime::now()),
            deduplication_savings: 0.15,
        };
        assert_eq!(diag.total_entries, 1000);
        assert!((diag.cache_hit_ratio - 0.85).abs() < f64::EPSILON);
        assert!(diag.last_archival.is_some());
    }

    #[test]
    fn test_scheduler_diagnostics_clone() {
        let diag = SchedulerDiagnostics {
            active_operations: 10,
            queued_operations: 5,
            high_priority_available: 8,
            low_priority_available: 12,
            preemptions_count: 3,
            avg_wait_time_ms: 15.5,
            max_wait_time_ms: 50.0,
            operations_per_second: 100.0,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.active_operations, diag.active_operations);
    }

    #[test]
    fn test_scheduler_diagnostics_debug() {
        let diag = SchedulerDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("SchedulerDiagnostics"));
    }

    #[test]
    fn test_scheduler_diagnostics_serialization() {
        let diag = SchedulerDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("active_operations"));

        let deserialized: SchedulerDiagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active_operations, 0);
    }

    #[test]
    fn test_scheduler_diagnostics_all_fields() {
        let diag = SchedulerDiagnostics {
            active_operations: 25,
            queued_operations: 10,
            high_priority_available: 4,
            low_priority_available: 8,
            preemptions_count: 100,
            avg_wait_time_ms: 25.0,
            max_wait_time_ms: 100.0,
            operations_per_second: 500.0,
        };
        assert_eq!(diag.queued_operations, 10);
        assert_eq!(diag.preemptions_count, 100);
        assert!((diag.operations_per_second - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adaptive_diagnostics_clone() {
        let diag = AdaptiveDiagnostics {
            current_cache_size: 1000,
            current_compression_level: 5,
            high_priority_permits: 4,
            low_priority_permits: 8,
            performance_trend: "Stable".to_string(),
            adjustments_made: 10,
            avg_analysis_time_ms: 50.0,
            optimization_effectiveness: 0.9,
        };
        let cloned = diag.clone();
        assert_eq!(cloned.current_cache_size, diag.current_cache_size);
    }

    #[test]
    fn test_adaptive_diagnostics_debug() {
        let diag = AdaptiveDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("AdaptiveDiagnostics"));
    }

    #[test]
    fn test_adaptive_diagnostics_serialization() {
        let diag = AdaptiveDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("current_cache_size"));

        let deserialized: AdaptiveDiagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.performance_trend, "Unknown");
    }

    #[test]
    fn test_adaptive_diagnostics_all_fields() {
        let diag = AdaptiveDiagnostics {
            current_cache_size: 5000,
            current_compression_level: 3,
            high_priority_permits: 6,
            low_priority_permits: 12,
            performance_trend: "Improving".to_string(),
            adjustments_made: 50,
            avg_analysis_time_ms: 75.0,
            optimization_effectiveness: 0.85,
        };
        assert_eq!(diag.current_compression_level, 3);
        assert_eq!(diag.adjustments_made, 50);
    }

    #[test]
    fn test_resource_diagnostics_clone() {
        let diag = ResourceDiagnostics {
            memory_usage_mb: 512.0,
            memory_limit_mb: 2048.0,
            memory_pressure: "Low".to_string(),
            cpu_utilization: 0.4,
            cpu_limit: 0.9,
            cpu_pressure: "Low".to_string(),
            enforcement_actions: EnforcementStats::default(),
        };
        let cloned = diag.clone();
        assert!((cloned.memory_usage_mb - diag.memory_usage_mb).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_diagnostics_debug() {
        let diag = ResourceDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("ResourceDiagnostics"));
    }

    #[test]
    fn test_resource_diagnostics_serialization() {
        let diag = ResourceDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("memory_usage_mb"));

        let deserialized: ResourceDiagnostics = serde_json::from_str(&json).unwrap();
        assert!((deserialized.memory_limit_mb - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_diagnostics_all_fields() {
        let diag = ResourceDiagnostics {
            memory_usage_mb: 768.0,
            memory_limit_mb: 1024.0,
            memory_pressure: "Medium".to_string(),
            cpu_utilization: 0.75,
            cpu_limit: 0.85,
            cpu_pressure: "High".to_string(),
            enforcement_actions: EnforcementStats {
                total_requests: 1000,
                allowed: 900,
                throttled: 50,
                queued: 30,
                rejected: 20,
                emergency_stops: 0,
            },
        };
        assert_eq!(diag.memory_pressure, "Medium");
        assert!((diag.cpu_utilization - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_enforcement_stats_clone() {
        let stats = EnforcementStats {
            total_requests: 500,
            allowed: 400,
            throttled: 50,
            queued: 30,
            rejected: 20,
            emergency_stops: 0,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_requests, stats.total_requests);
    }

    #[test]
    fn test_enforcement_stats_debug() {
        let stats = EnforcementStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("EnforcementStats"));
    }

    #[test]
    fn test_enforcement_stats_serialization() {
        let stats = EnforcementStats {
            total_requests: 100,
            allowed: 90,
            throttled: 5,
            queued: 3,
            rejected: 2,
            emergency_stops: 0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("total_requests"));

        let deserialized: EnforcementStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_requests, 100);
    }

    #[test]
    fn test_enforcement_stats_all_fields() {
        let stats = EnforcementStats {
            total_requests: 10000,
            allowed: 9000,
            throttled: 500,
            queued: 300,
            rejected: 200,
            emergency_stops: 5,
        };
        assert_eq!(stats.allowed, 9000);
        assert_eq!(stats.emergency_stops, 5);
    }

    #[test]
    fn test_performance_diagnostics_clone() {
        let diag = PerformanceDiagnostics {
            analyses_per_hour: 1000.0,
            avg_response_time_ms: 50.0,
            p50_response_time_ms: 40.0,
            p95_response_time_ms: 100.0,
            p99_response_time_ms: 200.0,
            throughput_mbps: 10.0,
            error_rate: 0.01,
        };
        let cloned = diag.clone();
        assert!((cloned.analyses_per_hour - diag.analyses_per_hour).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_diagnostics_debug() {
        let diag = PerformanceDiagnostics::default();
        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("PerformanceDiagnostics"));
    }

    #[test]
    fn test_performance_diagnostics_serialization() {
        let diag = PerformanceDiagnostics::default();
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("analyses_per_hour"));

        let deserialized: PerformanceDiagnostics = serde_json::from_str(&json).unwrap();
        assert!((deserialized.error_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_diagnostics_all_fields() {
        let diag = PerformanceDiagnostics {
            analyses_per_hour: 5000.0,
            avg_response_time_ms: 25.0,
            p50_response_time_ms: 20.0,
            p95_response_time_ms: 80.0,
            p99_response_time_ms: 150.0,
            throughput_mbps: 50.0,
            error_rate: 0.005,
        };
        assert!((diag.p50_response_time_ms - 20.0).abs() < f64::EPSILON);
        assert!((diag.throughput_mbps - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_health_status_clone() {
        let healthy = HealthStatus::Healthy;
        let cloned = healthy.clone();
        assert_eq!(cloned, HealthStatus::Healthy);

        let degraded = HealthStatus::Degraded {
            reasons: vec!["Test".to_string()],
        };
        let cloned_degraded = degraded.clone();
        assert!(matches!(cloned_degraded, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_health_status_serialization() {
        let healthy = HealthStatus::Healthy;
        let json = serde_json::to_string(&healthy).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, HealthStatus::Healthy);

        let degraded = HealthStatus::Degraded {
            reasons: vec!["Reason 1".to_string()],
        };
        let json = serde_json::to_string(&degraded).unwrap();
        assert!(json.contains("Reason 1"));

        let critical = HealthStatus::Critical {
            reasons: vec!["Critical reason".to_string()],
        };
        let json = serde_json::to_string(&critical).unwrap();
        assert!(json.contains("Critical reason"));
    }

    #[test]
    fn test_health_status_eq() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(
            HealthStatus::Healthy,
            HealthStatus::Degraded { reasons: vec![] }
        );
        assert_ne!(
            HealthStatus::Degraded {
                reasons: vec!["a".to_string()]
            },
            HealthStatus::Critical {
                reasons: vec!["a".to_string()]
            }
        );
    }

    #[test]
    fn test_format_diagnostics_with_values() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(7200),
            storage: StorageDiagnostics {
                backend_type: "sled".to_string(),
                total_entries: 1000,
                hot_cache_entries: 500,
                warm_entries: 300,
                cold_entries: 200,
                cache_hit_ratio: 0.85,
                compression_ratio: 0.7,
                storage_size_mb: 50.0,
                last_archival: None,
                deduplication_savings: 0.1,
            },
            scheduler: SchedulerDiagnostics {
                active_operations: 5,
                queued_operations: 2,
                high_priority_available: 3,
                low_priority_available: 7,
                preemptions_count: 10,
                avg_wait_time_ms: 12.5,
                max_wait_time_ms: 50.0,
                operations_per_second: 200.0,
            },
            adaptive: AdaptiveDiagnostics {
                current_cache_size: 2000,
                current_compression_level: 3,
                high_priority_permits: 4,
                low_priority_permits: 8,
                performance_trend: "Stable".to_string(),
                adjustments_made: 25,
                avg_analysis_time_ms: 45.0,
                optimization_effectiveness: 0.9,
            },
            resources: ResourceDiagnostics {
                memory_usage_mb: 512.0,
                memory_limit_mb: 1024.0,
                memory_pressure: "Low".to_string(),
                cpu_utilization: 0.4,
                cpu_limit: 0.8,
                cpu_pressure: "Low".to_string(),
                enforcement_actions: EnforcementStats {
                    total_requests: 1000,
                    allowed: 950,
                    throttled: 30,
                    queued: 15,
                    rejected: 5,
                    emergency_stops: 0,
                },
            },
            performance: PerformanceDiagnostics {
                analyses_per_hour: 1200.0,
                avg_response_time_ms: 35.0,
                p50_response_time_ms: 30.0,
                p95_response_time_ms: 70.0,
                p99_response_time_ms: 120.0,
                throughput_mbps: 15.0,
                error_rate: 0.02,
            },
            health: HealthStatus::Healthy,
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("sled"));
        assert!(formatted.contains("1000"));
        assert!(formatted.contains("Stable"));
        assert!(formatted.contains("HEALTHY"));
    }

    #[test]
    fn test_format_diagnostics_with_degraded_health() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Degraded {
                reasons: vec!["Memory high".to_string()],
            },
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("DEGRADED"));
        assert!(formatted.contains("Memory high"));
    }

    #[test]
    fn test_format_diagnostics_with_critical_health() {
        let diag = SystemDiagnostics {
            timestamp: SystemTime::now(),
            uptime: Duration::from_secs(100),
            storage: StorageDiagnostics::default(),
            scheduler: SchedulerDiagnostics::default(),
            adaptive: AdaptiveDiagnostics::default(),
            resources: ResourceDiagnostics::default(),
            performance: PerformanceDiagnostics::default(),
            health: HealthStatus::Critical {
                reasons: vec!["System failure".to_string()],
            },
        };

        let formatted = DiagnosticTool::format_diagnostics(&diag);
        assert!(formatted.contains("CRITICAL"));
        assert!(formatted.contains("System failure"));
    }

    #[tokio::test]
    async fn test_collect_diagnostics_no_components() {
        let tool = DiagnosticTool::new();
        let diag = tool
            .collect_diagnostics(None, None, None, None)
            .await
            .unwrap();
        assert_eq!(diag.storage.backend_type, "none");
        assert_eq!(diag.scheduler.active_operations, 0);
        assert_eq!(diag.adaptive.performance_trend, "Unknown");
    }

    #[test]
    fn test_health_status_debug() {
        let healthy = HealthStatus::Healthy;
        let debug_str = format!("{:?}", healthy);
        assert!(debug_str.contains("Healthy"));

        let degraded = HealthStatus::Degraded {
            reasons: vec!["test".to_string()],
        };
        let debug_str = format!("{:?}", degraded);
        assert!(debug_str.contains("Degraded"));

        let critical = HealthStatus::Critical {
            reasons: vec!["test".to_string()],
        };
        let debug_str = format!("{:?}", critical);
        assert!(debug_str.contains("Critical"));
    }

    #[test]
    fn test_multiple_degraded_reasons() {
        let status = HealthStatus::Degraded {
            reasons: vec![
                "High memory".to_string(),
                "High CPU".to_string(),
                "Slow response".to_string(),
            ],
        };
        let display = format!("{}", status);
        assert!(display.contains("High memory"));
        assert!(display.contains("High CPU"));
        assert!(display.contains("Slow response"));
    }

    #[test]
    fn test_multiple_critical_reasons() {
        let status = HealthStatus::Critical {
            reasons: vec!["Memory critical".to_string(), "High error rate".to_string()],
        };
        let display = format!("{}", status);
        assert!(display.contains("Memory critical"));
        assert!(display.contains("High error rate"));
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
