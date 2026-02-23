#![cfg_attr(coverage_nightly, coverage(off))]
/// Diagnostic type definitions for the Transactional Hashed TDG System
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime};

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
