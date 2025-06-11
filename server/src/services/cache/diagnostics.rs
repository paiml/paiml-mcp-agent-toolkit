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
                    "Consider increasing TTL or max_size for {name} cache"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_snapshot() {
        let stats = CacheStats::new();
        stats.hits.store(100, std::sync::atomic::Ordering::Relaxed);
        stats.misses.store(50, std::sync::atomic::Ordering::Relaxed);
        stats
            .evictions
            .store(10, std::sync::atomic::Ordering::Relaxed);
        stats
            .total_bytes
            .store(1024, std::sync::atomic::Ordering::Relaxed);

        let snapshot = CacheStatsSnapshot::from((&stats, 150));
        assert_eq!(snapshot.hits, 100);
        assert_eq!(snapshot.misses, 50);
        assert_eq!(snapshot.evictions, 10);
        assert_eq!(snapshot.total_bytes, 1024);
        assert_eq!(snapshot.entries, 150);
    }

    #[test]
    fn test_diagnostic_report_high_memory_pressure() {
        let diagnostics = CacheDiagnostics {
            session_id: Uuid::new_v4(),
            uptime: Duration::from_secs(3600),
            memory_usage_mb: 900.0,
            memory_pressure: 0.95,
            cache_stats: vec![],
            hot_paths: vec![],
            effectiveness: CacheEffectiveness {
                overall_hit_rate: 0.8,
                memory_efficiency: 0.9,
                time_saved_ms: 10000,
                most_valuable_caches: vec![],
            },
        };

        let report = CacheDiagnosticReport::new(diagnostics);
        assert!(!report.warnings.is_empty());
        assert!(report.warnings[0].contains("very high"));
        assert!(!report.recommendations.is_empty());
        assert!(report.recommendations[0].contains("max_memory_mb"));
    }

    #[test]
    fn test_diagnostic_report_low_hit_rate() {
        let mut cache_stats = vec![];
        let stats_snapshot = CacheStatsSnapshot {
            hits: 2,
            misses: 18,
            evictions: 0,
            total_bytes: 1024,
            hit_rate: 0.1,
            entries: 10,
        };
        cache_stats.push(("test_cache".to_string(), stats_snapshot));

        let diagnostics = CacheDiagnostics {
            session_id: Uuid::new_v4(),
            uptime: Duration::from_secs(3600),
            memory_usage_mb: 100.0,
            memory_pressure: 0.5,
            cache_stats,
            hot_paths: vec![],
            effectiveness: CacheEffectiveness {
                overall_hit_rate: 0.1,
                memory_efficiency: 0.5,
                time_saved_ms: 1000,
                most_valuable_caches: vec![],
            },
        };

        let report = CacheDiagnosticReport::new(diagnostics);
        assert!(!report.warnings.is_empty());
        assert!(report.warnings[0].contains("hit rate is low"));
        assert!(report.warnings[0].contains("10.0%"));
    }

    #[test]
    fn test_cache_effectiveness() {
        let effectiveness = CacheEffectiveness {
            overall_hit_rate: 0.85,
            memory_efficiency: 0.75,
            time_saved_ms: 5000,
            most_valuable_caches: vec![
                ("ast_cache".to_string(), 0.9),
                ("content_cache".to_string(), 0.8),
            ],
        };

        assert_eq!(effectiveness.overall_hit_rate, 0.85);
        assert_eq!(effectiveness.memory_efficiency, 0.75);
        assert_eq!(effectiveness.time_saved_ms, 5000);
        assert_eq!(effectiveness.most_valuable_caches.len(), 2);
    }

    #[test]
    fn test_report_healthy() {
        let diagnostics = CacheDiagnostics {
            session_id: Uuid::new_v4(),
            uptime: Duration::from_secs(3600),
            memory_usage_mb: 100.0,
            memory_pressure: 0.5,
            cache_stats: vec![],
            hot_paths: vec![],
            effectiveness: CacheEffectiveness {
                overall_hit_rate: 0.8,
                memory_efficiency: 0.9,
                time_saved_ms: 10000,
                most_valuable_caches: vec![],
            },
        };

        let report = CacheDiagnosticReport::new(diagnostics);
        assert!(report.is_healthy());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_format_prometheus_metrics() {
        let diagnostics = CacheDiagnostics {
            session_id: Uuid::new_v4(),
            uptime: Duration::from_secs(3600),
            memory_usage_mb: 100.0,
            memory_pressure: 0.5,
            cache_stats: vec![(
                "test_cache".to_string(),
                CacheStatsSnapshot {
                    hits: 100,
                    misses: 50,
                    evictions: 10,
                    total_bytes: 1024,
                    hit_rate: 0.67,
                    entries: 150,
                },
            )],
            hot_paths: vec![("hot_path".to_string(), 42)],
            effectiveness: CacheEffectiveness {
                overall_hit_rate: 0.8,
                memory_efficiency: 0.9,
                time_saved_ms: 10000,
                most_valuable_caches: vec![],
            },
        };

        let output = format_prometheus_metrics(&diagnostics);
        assert!(output.contains("# HELP cache_hits_total"));
        assert!(output.contains("cache_hits_total{cache=\"test_cache\"} 100"));
        assert!(output.contains("cache_memory_pressure 0.5"));
        assert!(output.contains("cache_uptime_seconds 3600"));
    }
}
