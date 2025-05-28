use crate::services::cache::base::CacheStats;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Cache diagnostics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDiagnostics {
    pub session_id: Uuid,
    pub uptime: Duration,
    pub memory_usage_mb: f64,
    pub memory_pressure: f32,
    pub cache_stats: Vec<(String, CacheStatsSnapshot)>,
    pub hot_paths: Vec<(String, u32)>,
    pub effectiveness: CacheEffectiveness,
}

/// Snapshot of cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_bytes: usize,
    pub hit_rate: f64,
    pub entries: usize,
}

impl From<(&CacheStats, usize)> for CacheStatsSnapshot {
    fn from((stats, entries): (&CacheStats, usize)) -> Self {
        Self {
            hits: stats.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: stats.misses.load(std::sync::atomic::Ordering::Relaxed),
            evictions: stats.evictions.load(std::sync::atomic::Ordering::Relaxed),
            total_bytes: stats.total_bytes.load(std::sync::atomic::Ordering::Relaxed),
            hit_rate: stats.hit_rate(),
            entries,
        }
    }
}

/// Cache effectiveness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEffectiveness {
    pub overall_hit_rate: f64,
    pub memory_efficiency: f64,
    pub time_saved_ms: u64,
    pub most_valuable_caches: Vec<(String, f64)>,
}

/// Diagnostic report with recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDiagnosticReport {
    pub diagnostics: CacheDiagnostics,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

impl CacheDiagnosticReport {
    pub fn new(diagnostics: CacheDiagnostics) -> Self {
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Check memory pressure
        if diagnostics.memory_pressure > 0.9 {
            warnings.push("Cache memory pressure is very high (>90%)".to_string());
            recommendations.push("Consider increasing max_memory_mb configuration".to_string());
        } else if diagnostics.memory_pressure > 0.8 {
            warnings.push("Cache memory pressure is high (>80%)".to_string());
        }

        // Check hit rates
        for (name, stats) in &diagnostics.cache_stats {
            if stats.hit_rate < 0.3 && stats.hits + stats.misses > 10 {
                warnings.push(format!(
                    "{} cache hit rate is low: {:.1}%",
                    name,
                    stats.hit_rate * 100.0
                ));
                recommendations.push(format!(
                    "Consider increasing TTL or max_size for {} cache",
                    name
                ));
            }
        }

        // Check effectiveness
        if diagnostics.effectiveness.overall_hit_rate < 0.5 {
            recommendations
                .push("Overall cache effectiveness is low - review cache strategies".to_string());
        }

        Self {
            diagnostics,
            warnings,
            recommendations,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Helper to format cache metrics as Prometheus-style output
pub fn format_prometheus_metrics(diagnostics: &CacheDiagnostics) -> String {
    let mut output = String::new();

    // Cache hits
    output.push_str("# HELP cache_hits_total Total cache hits\n");
    output.push_str("# TYPE cache_hits_total counter\n");
    for (name, stats) in &diagnostics.cache_stats {
        output.push_str(&format!(
            "cache_hits_total{{cache=\"{}\"}} {}\n",
            name, stats.hits
        ));
    }

    // Cache misses
    output.push_str("\n# HELP cache_misses_total Total cache misses\n");
    output.push_str("# TYPE cache_misses_total counter\n");
    for (name, stats) in &diagnostics.cache_stats {
        output.push_str(&format!(
            "cache_misses_total{{cache=\"{}\"}} {}\n",
            name, stats.misses
        ));
    }

    // Memory usage
    output.push_str("\n# HELP cache_memory_bytes Current cache memory usage\n");
    output.push_str("# TYPE cache_memory_bytes gauge\n");
    for (name, stats) in &diagnostics.cache_stats {
        output.push_str(&format!(
            "cache_memory_bytes{{cache=\"{}\"}} {}\n",
            name, stats.total_bytes
        ));
    }

    // Hit rate
    output.push_str("\n# HELP cache_hit_rate Hit rate percentage\n");
    output.push_str("# TYPE cache_hit_rate gauge\n");
    for (name, stats) in &diagnostics.cache_stats {
        output.push_str(&format!(
            "cache_hit_rate{{cache=\"{}\"}} {}\n",
            name, stats.hit_rate
        ));
    }

    // Overall metrics
    output.push_str("\n# HELP cache_memory_pressure Memory pressure ratio\n");
    output.push_str("# TYPE cache_memory_pressure gauge\n");
    output.push_str(&format!(
        "cache_memory_pressure {}\n",
        diagnostics.memory_pressure
    ));

    output.push_str("\n# HELP cache_uptime_seconds Cache uptime in seconds\n");
    output.push_str("# TYPE cache_uptime_seconds counter\n");
    output.push_str(&format!(
        "cache_uptime_seconds {}\n",
        diagnostics.uptime.as_secs()
    ));

    output
}
