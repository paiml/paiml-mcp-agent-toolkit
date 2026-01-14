use crate::services::cache::base::CacheStats;
use crate::services::cache::config::CacheConfig;
use crate::services::cache::diagnostics::{
    CacheDiagnostics, CacheEffectiveness, CacheStatsSnapshot,
};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_cache_config_default_values() {
    let config = CacheConfig::default();

    assert_eq!(config.max_memory_mb, 100);
    assert!(config.enable_watch);
    assert_eq!(config.ast_ttl_secs, 300);
    assert_eq!(config.template_ttl_secs, 600);
    assert_eq!(config.dag_ttl_secs, 180);
    assert_eq!(config.churn_ttl_secs, 1800);
    assert_eq!(config.git_stats_ttl_secs, 900);
    assert!(!config.warmup_on_startup);
    assert!(!config.warmup_patterns.is_empty());
    assert!(config.warmup_patterns.contains(&"src/**/*.rs".to_string()));
}

#[test]
fn test_cache_config_custom_values() {
    let config = CacheConfig {
        max_memory_mb: 256,
        enable_watch: false,
        ast_ttl_secs: 600,
        template_ttl_secs: 1200,
        dag_ttl_secs: 360,
        churn_ttl_secs: 3600,
        git_stats_ttl_secs: 1800,
        warmup_on_startup: true,
        warmup_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string()],
        git_cache_by_branch: true,
        git_cache_max_age_days: 7,
        parallel_warmup_threads: 8,
        cache_compression: true,
        eviction_batch_size: 50,
    };

    assert_eq!(config.max_memory_mb, 256);
    assert!(!config.enable_watch);
    assert_eq!(config.ast_ttl_secs, 600);
    assert_eq!(config.template_ttl_secs, 1200);
    assert_eq!(config.dag_ttl_secs, 360);
    assert_eq!(config.churn_ttl_secs, 3600);
    assert_eq!(config.git_stats_ttl_secs, 1800);
    assert!(config.warmup_on_startup);
    assert_eq!(config.warmup_patterns.len(), 2);
    assert!(config.git_cache_by_branch);
    assert_eq!(config.git_cache_max_age_days, 7);
    assert_eq!(config.parallel_warmup_threads, 8);
    assert!(config.cache_compression);
    assert_eq!(config.eviction_batch_size, 50);
}

#[test]
fn test_cache_config_serialization() {
    let config = CacheConfig::default();

    // Test serialization
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(serialized.contains("max_memory_mb"));
    assert!(serialized.contains("enable_watch"));
    assert!(serialized.contains("ast_ttl_secs"));

    // Test deserialization
    let deserialized: CacheConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.max_memory_mb, config.max_memory_mb);
    assert_eq!(deserialized.enable_watch, config.enable_watch);
    assert_eq!(deserialized.ast_ttl_secs, config.ast_ttl_secs);
}

#[test]
fn test_cache_stats_snapshot_creation() {
    let stats = CacheStats {
        hits: Arc::new(AtomicU64::new(150)),
        misses: Arc::new(AtomicU64::new(50)),
        evictions: Arc::new(AtomicU64::new(10)),
        total_bytes: Arc::new(AtomicUsize::new(1024000)),
    };

    let entries = 75;
    let snapshot = CacheStatsSnapshot::from((&stats, entries));

    assert_eq!(snapshot.hits, 150);
    assert_eq!(snapshot.misses, 50);
    assert_eq!(snapshot.evictions, 10);
    assert_eq!(snapshot.total_bytes, 1024000);
    assert_eq!(snapshot.entries, 75);
    assert_eq!(snapshot.hit_rate, 0.75); // 150 / (150 + 50)
}

#[test]
fn test_cache_stats_snapshot_zero_requests() {
    let stats = CacheStats {
        hits: Arc::new(AtomicU64::new(0)),
        misses: Arc::new(AtomicU64::new(0)),
        evictions: Arc::new(AtomicU64::new(0)),
        total_bytes: Arc::new(AtomicUsize::new(0)),
    };

    let entries = 0;
    let snapshot = CacheStatsSnapshot::from((&stats, entries));

    assert_eq!(snapshot.hits, 0);
    assert_eq!(snapshot.misses, 0);
    assert_eq!(snapshot.evictions, 0);
    assert_eq!(snapshot.total_bytes, 0);
    assert_eq!(snapshot.entries, 0);
    assert_eq!(snapshot.hit_rate, 0.0);
}

#[test]
fn test_cache_stats_snapshot_serialization() {
    let snapshot = CacheStatsSnapshot {
        hits: 100,
        misses: 25,
        evictions: 5,
        total_bytes: 512000,
        hit_rate: 0.8,
        entries: 50,
    };

    // Test serialization
    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert!(serialized.contains("hits"));
    assert!(serialized.contains("100"));
    assert!(serialized.contains("hit_rate"));
    assert!(serialized.contains("0.8"));

    // Test deserialization
    let deserialized: CacheStatsSnapshot = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.hits, 100);
    assert_eq!(deserialized.misses, 25);
    assert_eq!(deserialized.evictions, 5);
    assert_eq!(deserialized.total_bytes, 512000);
    assert_eq!(deserialized.hit_rate, 0.8);
    assert_eq!(deserialized.entries, 50);
}

#[test]
fn test_cache_effectiveness_structure() {
    let effectiveness = CacheEffectiveness {
        overall_hit_rate: 0.85,
        memory_efficiency: 0.72,
        time_saved_ms: 1500,
        most_valuable_caches: vec![
            ("ast_cache".to_string(), 0.9),
            ("template_cache".to_string(), 0.8),
            ("dag_cache".to_string(), 0.75),
        ],
    };

    assert_eq!(effectiveness.overall_hit_rate, 0.85);
    assert_eq!(effectiveness.memory_efficiency, 0.72);
    assert_eq!(effectiveness.time_saved_ms, 1500);
    assert_eq!(effectiveness.most_valuable_caches.len(), 3);
    assert_eq!(effectiveness.most_valuable_caches[0].0, "ast_cache");
    assert_eq!(effectiveness.most_valuable_caches[0].1, 0.9);
}

#[test]
fn test_cache_effectiveness_serialization() {
    let effectiveness = CacheEffectiveness {
        overall_hit_rate: 0.92,
        memory_efficiency: 0.85,
        time_saved_ms: 2500,
        most_valuable_caches: vec![
            ("complexity_cache".to_string(), 0.95),
            ("churn_cache".to_string(), 0.88),
        ],
    };

    // Test serialization
    let serialized = serde_json::to_string(&effectiveness).unwrap();
    assert!(serialized.contains("overall_hit_rate"));
    assert!(serialized.contains("0.92"));
    assert!(serialized.contains("complexity_cache"));

    // Test deserialization
    let deserialized: CacheEffectiveness = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.overall_hit_rate, 0.92);
    assert_eq!(deserialized.memory_efficiency, 0.85);
    assert_eq!(deserialized.time_saved_ms, 2500);
    assert_eq!(deserialized.most_valuable_caches.len(), 2);
}

#[test]
fn test_cache_diagnostics_structure() {
    let session_id = Uuid::new_v4();

    let diagnostics = CacheDiagnostics {
        session_id,
        uptime: Duration::from_secs(3600), // 1 hour
        memory_usage_mb: 64.5,
        memory_pressure: 0.65,
        cache_stats: vec![
            (
                "ast_cache".to_string(),
                CacheStatsSnapshot {
                    hits: 200,
                    misses: 50,
                    evictions: 5,
                    total_bytes: 1024000,
                    hit_rate: 0.8,
                    entries: 100,
                },
            ),
            (
                "template_cache".to_string(),
                CacheStatsSnapshot {
                    hits: 150,
                    misses: 25,
                    evictions: 2,
                    total_bytes: 512000,
                    hit_rate: 0.857,
                    entries: 75,
                },
            ),
        ],
        hot_paths: vec![
            ("src/main.rs".to_string(), 15),
            ("src/lib.rs".to_string(), 12),
            ("Cargo.toml".to_string(), 8),
        ],
        effectiveness: CacheEffectiveness {
            overall_hit_rate: 0.82,
            memory_efficiency: 0.78,
            time_saved_ms: 1800,
            most_valuable_caches: vec![
                ("ast_cache".to_string(), 0.8),
                ("template_cache".to_string(), 0.857),
            ],
        },
    };

    assert_eq!(diagnostics.session_id, session_id);
    assert_eq!(diagnostics.uptime, Duration::from_secs(3600));
    assert_eq!(diagnostics.memory_usage_mb, 64.5);
    assert_eq!(diagnostics.memory_pressure, 0.65);
    assert_eq!(diagnostics.cache_stats.len(), 2);
    assert_eq!(diagnostics.hot_paths.len(), 3);
    assert_eq!(diagnostics.effectiveness.overall_hit_rate, 0.82);
}

#[test]
fn test_cache_stats_hit_rate_calculation() {
    // Test with normal values
    let stats = CacheStats {
        hits: Arc::new(AtomicU64::new(80)),
        misses: Arc::new(AtomicU64::new(20)),
        evictions: Arc::new(AtomicU64::new(0)),
        total_bytes: Arc::new(AtomicUsize::new(0)),
    };

    assert_eq!(stats.hit_rate(), 0.8);

    // Test with zero hits and misses
    let empty_stats = CacheStats {
        hits: Arc::new(AtomicU64::new(0)),
        misses: Arc::new(AtomicU64::new(0)),
        evictions: Arc::new(AtomicU64::new(0)),
        total_bytes: Arc::new(AtomicUsize::new(0)),
    };

    assert_eq!(empty_stats.hit_rate(), 0.0);

    // Test with only hits
    let hits_only_stats = CacheStats {
        hits: Arc::new(AtomicU64::new(100)),
        misses: Arc::new(AtomicU64::new(0)),
        evictions: Arc::new(AtomicU64::new(0)),
        total_bytes: Arc::new(AtomicUsize::new(0)),
    };

    assert_eq!(hits_only_stats.hit_rate(), 1.0);

    // Test with only misses
    let misses_only_stats = CacheStats {
        hits: Arc::new(AtomicU64::new(0)),
        misses: Arc::new(AtomicU64::new(50)),
        evictions: Arc::new(AtomicU64::new(0)),
        total_bytes: Arc::new(AtomicUsize::new(0)),
    };

    assert_eq!(misses_only_stats.hit_rate(), 0.0);
}

#[test]
fn test_cache_config_ttl_values() {
    let config = CacheConfig::default();

    // Test TTL values are reasonable
    assert!(config.ast_ttl_secs > 0);
    assert!(config.template_ttl_secs > 0);
    assert!(config.dag_ttl_secs > 0);
    assert!(config.churn_ttl_secs > 0);
    assert!(config.git_stats_ttl_secs > 0);

    // Test TTL ordering (longer-lived caches have higher TTL)
    assert!(config.churn_ttl_secs >= config.git_stats_ttl_secs);
    assert!(config.template_ttl_secs >= config.ast_ttl_secs);
}

#[test]
fn test_cache_config_memory_settings() {
    let config = CacheConfig::default();

    assert!(config.max_memory_mb > 0);
    assert!(config.eviction_batch_size > 0);
    assert!(config.parallel_warmup_threads > 0);
}

#[test]
fn test_cache_config_git_settings() {
    let config = CacheConfig::default();

    assert!(config.git_cache_max_age_days > 0);
    // git_cache_by_branch can be true or false, both are valid
}

#[test]
fn test_cache_config_warmup_patterns() {
    let config = CacheConfig::default();

    assert!(!config.warmup_patterns.is_empty());

    // Should contain common file patterns
    let has_rust = config.warmup_patterns.iter().any(|p| p.contains("*.rs"));
    let has_typescript = config.warmup_patterns.iter().any(|p| p.contains("*.ts"));

    assert!(has_rust || has_typescript); // Should have at least one common pattern
}

#[test]
fn test_cache_effectiveness_empty_caches() {
    let effectiveness = CacheEffectiveness {
        overall_hit_rate: 0.0,
        memory_efficiency: 0.0,
        time_saved_ms: 0,
        most_valuable_caches: vec![],
    };

    assert_eq!(effectiveness.overall_hit_rate, 0.0);
    assert_eq!(effectiveness.memory_efficiency, 0.0);
    assert_eq!(effectiveness.time_saved_ms, 0);
    assert!(effectiveness.most_valuable_caches.is_empty());
}

#[test]
fn test_cache_diagnostics_empty_collections() {
    let diagnostics = CacheDiagnostics {
        session_id: Uuid::new_v4(),
        uptime: Duration::from_secs(0),
        memory_usage_mb: 0.0,
        memory_pressure: 0.0,
        cache_stats: vec![],
        hot_paths: vec![],
        effectiveness: CacheEffectiveness {
            overall_hit_rate: 0.0,
            memory_efficiency: 0.0,
            time_saved_ms: 0,
            most_valuable_caches: vec![],
        },
    };

    assert!(diagnostics.cache_stats.is_empty());
    assert!(diagnostics.hot_paths.is_empty());
    assert!(diagnostics.effectiveness.most_valuable_caches.is_empty());
    assert_eq!(diagnostics.uptime, Duration::from_secs(0));
    assert_eq!(diagnostics.memory_usage_mb, 0.0);
    assert_eq!(diagnostics.memory_pressure, 0.0);
}
