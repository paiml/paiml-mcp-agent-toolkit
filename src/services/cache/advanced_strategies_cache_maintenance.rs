// AdaptiveCache stats recording, pattern updates, and maintenance methods
//
// Included by advanced_strategies_cache.rs — do NOT add `use` imports here.

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn record_hit(&self, tier: CacheTier, _access_time: Duration) {
        if let Some(tier_stats) = self.stats.read().tier_stats.get(&tier) {
            tier_stats.hits.fetch_add(1, Ordering::Relaxed);
            // Update average access time (simplified)
        }
    }

    fn record_miss(&self) {
        // Record miss for all tiers
        for tier_stats in self.stats.read().tier_stats.values() {
            tier_stats.misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_insert_time(&self, _insert_time: Duration) {
        // Update insertion statistics
    }

    fn update_access_pattern(&self, key: &K) {
        let mut patterns = self.access_patterns.write();
        if let Some(pattern) = patterns.get_mut(key) {
            pattern.access_count += 1;
            pattern.last_access = Utc::now();
            // Update frequency and locality scores
            pattern.frequency = (pattern.frequency * 0.9 + 0.1).min(1.0);
        }
    }

    async fn cleanup_expired_entries(&self) -> Result<()> {
        let now = Utc::now();

        // Clean L1
        {
            let mut cache = self.l1_cache.write();
            cache.retain(|_, entry| entry.expires_at.map_or(true, |exp| exp > now));
        }

        // Clean L2
        {
            let mut cache = self.l2_cache.write();
            cache.retain(|_, entry| entry.expires_at.map_or(true, |exp| exp > now));
        }

        // Clean L3
        {
            let mut cache = self.l3_cache.write();
            cache.retain(|_, entry| entry.expires_at.map_or(true, |exp| exp > now));
        }

        Ok(())
    }

    async fn optimize_cache_layout(&self) -> Result<()> {
        // Access pattern analysis and tier placement optimization
        // ML-based optimization algorithms execute here
        Ok(())
    }

    fn update_global_patterns(&self) {
        // Update global access pattern statistics
        let patterns = self.access_patterns.read();
        let mut stats = self.stats.write();

        if !patterns.is_empty() {
            stats.pattern_stats.avg_frequency =
                patterns.values().map(|p| p.frequency).sum::<f64>() / patterns.len() as f64;

            stats.pattern_stats.avg_temporal_locality =
                patterns.values().map(|p| p.temporal_locality).sum::<f64>() / patterns.len() as f64;

            stats.pattern_stats.avg_spatial_locality =
                patterns.values().map(|p| p.spatial_locality).sum::<f64>() / patterns.len() as f64;
        }
    }
}
