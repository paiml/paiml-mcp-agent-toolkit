// AdaptiveCache eviction, tier insertion, and tier helper methods
//
// Included by advanced_strategies_cache.rs — do NOT add `use` imports here.

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub(crate) fn get_from_tier(&self, key: &K, tier: CacheTier) -> Option<AdaptiveCacheEntry<V>> {
        match tier {
            CacheTier::L1 => self.l1_cache.read().get(key).cloned(),
            CacheTier::L2 => self.l2_cache.read().get(key).cloned(),
            CacheTier::L3 => self.l3_cache.read().get(key).cloned(),
        }
    }

    pub(crate) fn should_promote(&self, pattern: &AccessPattern) -> bool {
        pattern.frequency > 0.5 || pattern.temporal_locality > 0.7
    }

    async fn promote_to_l1(&self, key: &K, entry: &AdaptiveCacheEntry<V>) -> Result<()> {
        let mut promoted_entry = entry.clone();
        promoted_entry.tier = CacheTier::L1;
        self.insert_l1(key.clone(), promoted_entry).await
    }

    async fn promote_to_l2(&self, key: &K, entry: &AdaptiveCacheEntry<V>) -> Result<()> {
        let mut promoted_entry = entry.clone();
        promoted_entry.tier = CacheTier::L2;
        self.insert_l2(key.clone(), promoted_entry).await
    }

    pub(crate) fn determine_initial_tier(&self, _key: &K, size: usize) -> CacheTier {
        debug_assert!(size > 0, "size must be positive");
        // Simple heuristic - could be more sophisticated
        if size < 64 * 1024 {
            // < 64KB
            CacheTier::L1
        } else if size < 1024 * 1024 {
            // < 1MB
            CacheTier::L2
        } else {
            CacheTier::L3
        }
    }

    pub(crate) fn get_or_create_pattern(&self, key: &K) -> AccessPattern {
        self.access_patterns
            .read()
            .get(key)
            .cloned()
            .unwrap_or_else(|| AccessPattern {
                frequency: 0.0,
                temporal_locality: 0.0,
                spatial_locality: 0.0,
                entropy: 0.0,
                last_access: Utc::now(),
                access_count: 0,
            })
    }

    pub(crate) fn calculate_expiration(&self, tier: CacheTier) -> Option<DateTime<Utc>> {
        if matches!(self.config.eviction_policy, EvictionPolicy::TTL) {
            let ttl = match tier {
                CacheTier::L1 => Duration::from_secs(300),  // 5 minutes
                CacheTier::L2 => Duration::from_secs(1800), // 30 minutes
                CacheTier::L3 => Duration::from_secs(3600), // 1 hour
            };
            Some(Utc::now() + chrono::Duration::from_std(ttl).expect("internal error"))
        } else {
            None
        }
    }

    async fn insert_l1(&self, key: K, entry: AdaptiveCacheEntry<V>) -> Result<()> {
        let mut cache = self.l1_cache.write();

        // Check if we need to evict
        let max_size = *self
            .config
            .tier_memory_limits
            .get(&CacheTier::L1)
            .unwrap_or(&(64 * 1024 * 1024));
        if self.calculate_tier_size(&cache) + entry.size > max_size {
            self.evict_from_tier(&mut cache, CacheTier::L1)?;
        }

        cache.insert(key, entry);
        Ok(())
    }

    async fn insert_l2(&self, key: K, entry: AdaptiveCacheEntry<V>) -> Result<()> {
        let mut cache = self.l2_cache.write();

        let max_size = *self
            .config
            .tier_memory_limits
            .get(&CacheTier::L2)
            .unwrap_or(&(256 * 1024 * 1024));
        if self.calculate_tier_size(&cache) + entry.size > max_size {
            self.evict_from_tier(&mut cache, CacheTier::L2)?;
        }

        cache.insert(key, entry);
        Ok(())
    }

    async fn insert_l3(&self, key: K, entry: AdaptiveCacheEntry<V>) -> Result<()> {
        let mut cache = self.l3_cache.write();

        let max_size = *self
            .config
            .tier_memory_limits
            .get(&CacheTier::L3)
            .unwrap_or(&(1024 * 1024 * 1024));
        if self.calculate_tier_size(&cache) + entry.size > max_size {
            self.evict_from_tier(&mut cache, CacheTier::L3)?;
        }

        cache.insert(key, entry);
        Ok(())
    }

    fn calculate_tier_size(&self, cache: &FxHashMap<K, AdaptiveCacheEntry<V>>) -> usize {
        cache.values().map(|entry| entry.size).sum()
    }

    pub(crate) fn evict_from_tier(
        &self,
        cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>,
        tier: CacheTier,
    ) -> Result<()> {
        if cache.is_empty() {
            return Ok(());
        }

        match self.config.eviction_policy {
            EvictionPolicy::LRU => self.evict_lru(cache),
            EvictionPolicy::LFU => self.evict_lfu(cache),
            EvictionPolicy::TTL => self.evict_ttl(cache),
            EvictionPolicy::FIFO => self.evict_fifo(cache),
            EvictionPolicy::Random => self.evict_random(cache),
            EvictionPolicy::Adaptive => self.evict_adaptive(cache),
        }

        // Update eviction stats
        if let Some(tier_stats) = self.stats.read().tier_stats.get(&tier) {
            tier_stats.evictions.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    pub(crate) fn evict_lru(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.pattern.last_access)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    pub(crate) fn evict_lfu(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(least_used_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.pattern.access_count)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&least_used_key);
        }
    }

    pub(crate) fn evict_ttl(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        let now = Utc::now();
        let expired_keys: Vec<_> = cache
            .iter()
            .filter(|(_, entry)| entry.expires_at.is_some_and(|exp| exp < now))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            cache.remove(&key);
        }

        // If no expired entries, fall back to LRU
        if !cache.is_empty() {
            self.evict_lru(cache);
        }
    }

    pub(crate) fn evict_fifo(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    pub(crate) fn evict_random(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
    }

    pub(crate) fn evict_adaptive(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        // Adaptive eviction considers multiple factors
        if let Some(victim_key) = cache
            .iter()
            .min_by(|(_, a), (_, b)| {
                let score_a = self.calculate_eviction_score(&a.pattern);
                let score_b = self.calculate_eviction_score(&b.pattern);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(key, _)| key.clone())
        {
            cache.remove(&victim_key);
        }
    }

    pub(crate) fn calculate_eviction_score(&self, pattern: &AccessPattern) -> f64 {
        // Contract: calculate_eviction_score returns a bounded score
        // Lower score = more likely to evict
        // Combine frequency, recency, and locality
        let recency_weight = 0.4;
        let frequency_weight = 0.4;
        let locality_weight = 0.2;

        let recency_score = {
            let age = Utc::now().signed_duration_since(pattern.last_access);
            1.0 - (age.num_seconds() as f64 / 3600.0).min(1.0) // Normalize to hours
        };

        recency_weight * recency_score
            + frequency_weight * pattern.frequency
            + locality_weight * (pattern.temporal_locality + pattern.spatial_locality) / 2.0
    }
}
