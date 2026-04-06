#![cfg_attr(coverage_nightly, coverage(off))]
/// DiagnosticTool implementation for the TDG System
use anyhow::Result;
use chrono::{DateTime, Local};
use std::time::{Instant, SystemTime};

use super::super::{
    AdaptiveThresholdManager, PlatformResourceController, SimpleFairScheduler, TieredStore,
};
use super::types::{
    AdaptiveDiagnostics, EnforcementStats, HealthStatus, PerformanceDiagnostics,
    ResourceDiagnostics, SchedulerDiagnostics, StorageDiagnostics, SystemDiagnostics,
};

/// TDG System Diagnostic Tool
pub struct DiagnosticTool {
    pub(super) start_time: Instant,
    pub(super) performance_samples: Vec<PerformanceSample>,
    pub(super) error_count: u64,
    pub(super) analysis_count: u64,
}

#[derive(Clone)]
pub(super) struct PerformanceSample {
    #[allow(dead_code)]
    pub(super) timestamp: Instant,
    pub(super) response_time_ms: f64,
    pub(super) success: bool,
}

impl DiagnosticTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            performance_samples: Vec::new(),
            error_count: 0,
            analysis_count: 0,
        }
    }

    /// Collect comprehensive system diagnostics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    pub(super) fn calculate_performance_metrics(&self) -> PerformanceDiagnostics {
        if self.performance_samples.is_empty() {
            return PerformanceDiagnostics::default();
        }

        let mut response_times: Vec<f64> = self
            .performance_samples
            .iter()
            .map(|s| s.response_time_ms)
            .collect();
        response_times.sort_by(|a, b| a.total_cmp(b));

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
    pub(super) fn assess_health(
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
