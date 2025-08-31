/// Diagnostic tools for Transactional Hashed TDG System
/// 
/// Provides comprehensive monitoring, profiling, and debugging capabilities
/// for the TDG system including storage, scheduling, and performance metrics.

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use super::{
    AdaptiveThresholdManager, PlatformResourceController, SimpleFairScheduler,
    StorageStatistics, TdgAnalyzer, TieredStore,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    timestamp: Instant,
    response_time_ms: f64,
    success: bool,
}

impl DiagnosticTool {
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
    async fn collect_storage_diagnostics(&self, storage: &TieredStore) -> Result<StorageDiagnostics> {
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
            compression_ratio: stats.compression_ratio as f64,
            storage_size_mb: stats.hot_memory_kb as f64 / 1024.0,
            last_archival: None, // Would need to track this
            deduplication_savings: 0.0, // Would need to calculate
        })
    }
    
    /// Collect scheduler diagnostics
    async fn collect_scheduler_diagnostics(&self, scheduler: &SimpleFairScheduler) -> Result<SchedulerDiagnostics> {
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
    async fn collect_adaptive_diagnostics(&self, adaptive: &AdaptiveThresholdManager) -> Result<AdaptiveDiagnostics> {
        let thresholds = adaptive.get_current_thresholds().await;
        let stats = adaptive.get_performance_stats().await;
        
        Ok(AdaptiveDiagnostics {
            current_cache_size: thresholds.hot_cache_size,
            current_compression_level: thresholds.compression_level as u32,
            high_priority_permits: thresholds.high_priority_permits,
            low_priority_permits: thresholds.low_priority_permits,
            performance_trend: if stats.avg_analysis_duration_ms > 100.0 { "Degrading" } else { "Stable" }.to_string(),
            adjustments_made: stats.total_samples,
            avg_analysis_time_ms: stats.avg_analysis_duration_ms as f64,
            optimization_effectiveness: stats.avg_cache_hit_ratio as f64,
        })
    }
    
    /// Collect resource diagnostics
    async fn collect_resource_diagnostics(&self, controller: &PlatformResourceController) -> Result<ResourceDiagnostics> {
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
        
        let mut response_times: Vec<f64> = self.performance_samples
            .iter()
            .map(|s| s.response_time_ms)
            .collect();
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = response_times.len();
        let sum: f64 = response_times.iter().sum();
        
        let error_count = self.performance_samples
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
            HealthStatus::Critical { reasons: critical_reasons }
        } else if !degraded_reasons.is_empty() {
            HealthStatus::Degraded { reasons: degraded_reasons }
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
    pub fn format_diagnostics(diag: &SystemDiagnostics) -> String {
        let local_time: DateTime<Local> = diag.timestamp.into();
        
        format!(
            r#"
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

════════════════════════════════════════════════════════════════════"#,
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

impl Default for EnforcementStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            allowed: 0,
            throttled: 0,
            queued: 0,
            rejected: 0,
            emergency_stops: 0,
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
}