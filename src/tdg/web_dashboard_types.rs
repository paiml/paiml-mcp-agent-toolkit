//! TDG Web Dashboard - Type Definitions
//!
//! Data structures for the web dashboard metrics and state.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::SystemTime;

/// System metrics for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: SystemTime,
    pub storage_stats: StorageMetrics,
    pub performance_stats: PerformanceMetrics,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_entries: u64,
    pub cache_hit_ratio: f64,
    pub compression_ratio: f64,
    pub backend_type: String,
    pub storage_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_analysis_time_ms: f64,
    pub active_operations: u32,
    pub queue_depth: u32,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall: String, // "healthy", "warning", "critical"
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub uptime_seconds: u64,
}

/// Query parameters for analysis requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisQuery {
    pub path: String,
    pub backend: Option<String>,
    pub priority: Option<String>,
}

/// Storage operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOperation {
    pub action: String,
    pub options: Option<Value>,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            storage_stats: StorageMetrics {
                total_entries: 0,
                cache_hit_ratio: 0.0,
                compression_ratio: 0.0,
                backend_type: "sled".to_string(),
                storage_size_mb: 0.0,
            },
            performance_stats: PerformanceMetrics {
                avg_analysis_time_ms: 0.0,
                active_operations: 0,
                queue_depth: 0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
            },
            health_status: HealthStatus {
                overall: "healthy".to_string(),
                issues: Vec::new(),
                recommendations: Vec::new(),
                uptime_seconds: 0,
            },
        }
    }
}
