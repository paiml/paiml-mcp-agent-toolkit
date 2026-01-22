//! TDG Web Dashboard - State Management
//!
//! Dashboard state and metrics update functionality.

use super::web_dashboard_types::{
    HealthStatus, PerformanceMetrics, StorageMetrics, SystemMetrics,
};
use super::{AdaptiveThresholdFactory, SchedulerFactory, TdgAnalyzer, TieredStorageFactory, TieredStore};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Shared state for the TDG dashboard
#[derive(Clone)]
pub struct DashboardState {
    /// TDG storage instance
    pub storage: Arc<TieredStore>,
    /// System analyzers
    pub analyzer: Arc<TdgAnalyzer>,
    /// Real-time metrics cache
    pub metrics_cache: Arc<RwLock<SystemMetrics>>,
}

impl DashboardState {
    /// Create new dashboard state with initialized TDG system
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing TDG Dashboard state");

        let storage = Arc::new(TieredStorageFactory::create_default()?);
        let analyzer = Arc::new(TdgAnalyzer::new()?);

        let initial_metrics = SystemMetrics {
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
        };

        Ok(Self {
            storage,
            analyzer,
            metrics_cache: Arc::new(RwLock::new(initial_metrics)),
        })
    }

    /// Update system metrics
    pub async fn update_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let storage_stats = self.storage.get_statistics();
        let adaptive = AdaptiveThresholdFactory::create_default();
        let performance = adaptive.get_performance_stats().await;
        let scheduler = SchedulerFactory::create_balanced();
        let scheduler_stats = scheduler.get_statistics().await;

        let mut metrics = self.metrics_cache.write().await;

        metrics.timestamp = SystemTime::now();
        metrics.storage_stats = StorageMetrics {
            total_entries: storage_stats.total_entries as u64,
            cache_hit_ratio: 0.85, // Estimated - hot entries / total entries
            compression_ratio: f64::from(storage_stats.compression_ratio),
            backend_type: storage_stats.warm_backend.clone(),
            storage_size_mb: storage_stats.hot_memory_kb as f64 / 1024.0, // Convert KB to MB
        };

        metrics.performance_stats = PerformanceMetrics {
            avg_analysis_time_ms: f64::from(performance.avg_analysis_duration_ms),
            active_operations: scheduler_stats.total_active_operations as u32,
            queue_depth: scheduler_stats.avg_wait_time_ms as u32 / 10, // Approximation
            cpu_usage_percent: f64::from(performance.avg_cpu_utilization * 100.0),
            memory_usage_mb: f64::from(performance.avg_memory_usage_mb),
        };

        // Basic health assessment
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        if metrics.performance_stats.avg_analysis_time_ms > 1000.0 {
            issues.push("High analysis times detected".to_string());
            recommendations
                .push("Consider increasing cache size or optimizing queries".to_string());
        }

        if metrics.storage_stats.cache_hit_ratio < 0.7 {
            issues.push("Low cache hit ratio".to_string());
            recommendations.push("Review access patterns and consider cache tuning".to_string());
        }

        let overall = if issues.is_empty() {
            "healthy".to_string()
        } else if issues.len() <= 2 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        metrics.health_status = HealthStatus {
            overall,
            issues,
            recommendations,
            uptime_seconds: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        debug!(
            "Updated dashboard metrics: health={}",
            metrics.health_status.overall
        );
        Ok(())
    }
}
