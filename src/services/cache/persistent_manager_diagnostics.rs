impl PersistentCacheManager {
    /// Get cache diagnostics
    #[must_use]
    pub fn get_diagnostics(&self) -> CacheDiagnostics {
        let uptime = self.created.elapsed();
        let ast_size = self.ast_cache.stats.memory_usage();

        let memory_usage_mb = ast_size as f64 / (1024.0 * 1024.0);
        let memory_pressure = if self.config.max_memory_mb > 0 {
            (memory_usage_mb / self.config.max_memory_mb as f64).min(1.0) as f32
        } else {
            0.0
        };

        // Trigger cleanup if memory pressure is high
        if memory_pressure > 0.8 {
            self.ast_cache.cleanup_expired();
        }

        let cache_stats = vec![(
            "ast".to_string(),
            CacheStatsSnapshot::from((&self.ast_cache.stats, self.ast_cache.len())),
        )];

        // Calculate effectiveness
        let total_operations = cache_stats
            .iter()
            .map(|(_, stats)| stats.hits + stats.misses)
            .sum::<u64>();

        let total_hits = cache_stats.iter().map(|(_, stats)| stats.hits).sum::<u64>();

        let overall_hit_rate = if total_operations > 0 {
            total_hits as f64 / total_operations as f64
        } else {
            0.0
        };

        let memory_efficiency = 1.0 - f64::from(memory_pressure);

        // Estimate time saved (simplified calculation)
        let time_saved_ms = total_hits * 100; // Assume 100ms saved per cache hit

        let most_valuable_caches = vec![("ast".to_string(), total_hits as f64)];

        let effectiveness = CacheEffectiveness {
            overall_hit_rate,
            memory_efficiency,
            time_saved_ms,
            most_valuable_caches,
        };

        CacheDiagnostics {
            session_id: self.session_id,
            uptime,
            memory_usage_mb,
            memory_pressure,
            cache_stats,
            hot_paths: Vec::new(), // TRACKED: Implement hot path tracking
            effectiveness,
        }
    }
}
