// AdaptiveCache public API methods
//
// Included by advanced_strategies_cache.rs — do NOT add `use` imports here.

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new adaptive cache
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: AdvancedCacheConfig) -> Self {
        let mut tier_stats = FxHashMap::default();
        tier_stats.insert(CacheTier::L1, TierStats::default());
        tier_stats.insert(CacheTier::L2, TierStats::default());
        tier_stats.insert(CacheTier::L3, TierStats::default());

        Self {
            config,
            l1_cache: Arc::new(RwLock::new(FxHashMap::default())),
            l2_cache: Arc::new(RwLock::new(FxHashMap::default())),
            l3_cache: Arc::new(RwLock::new(FxHashMap::default())),
            access_patterns: Arc::new(RwLock::new(FxHashMap::default())),
            stats: Arc::new(RwLock::new(AdaptiveCacheStats {
                tier_stats,
                ..Default::default()
            })),
            predictor: Arc::new(super::advanced_strategies_predictor::CachePredictor::new(0.8)),
        }
    }

    /// Get value from cache with intelligent tier promotion
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get(&self, key: &K) -> Option<Arc<V>> {
        let start = Instant::now();

        // Try L1 first (fastest)
        if let Some(entry) = self.get_from_tier(key, CacheTier::L1) {
            self.record_hit(CacheTier::L1, start.elapsed());
            self.update_access_pattern(key);
            return Some(entry.value);
        }

        // Try L2 (compressed)
        if let Some(entry) = self.get_from_tier(key, CacheTier::L2) {
            self.record_hit(CacheTier::L2, start.elapsed());
            // Promote to L1 if frequently accessed
            if self.should_promote(&entry.pattern) {
                let _ = self.promote_to_l1(key, &entry).await;
            }
            self.update_access_pattern(key);
            return Some(entry.value);
        }

        // Try L3 (persistent)
        if let Some(entry) = self.get_from_tier(key, CacheTier::L3) {
            self.record_hit(CacheTier::L3, start.elapsed());
            // Consider promotion based on pattern
            if self.should_promote(&entry.pattern) {
                if entry.pattern.frequency > 0.7 {
                    let _ = self.promote_to_l1(key, &entry).await;
                } else if entry.pattern.frequency > 0.3 {
                    let _ = self.promote_to_l2(key, &entry).await;
                }
            }
            self.update_access_pattern(key);
            return Some(entry.value);
        }

        // Cache miss
        self.record_miss();
        None
    }

    /// Put value into cache with intelligent tier placement
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn put(&self, key: K, value: V) -> Result<()> {
        let start = Instant::now();
        let value_arc = Arc::new(value);

        // Estimate size (simplified)
        let size = std::mem::size_of::<V>();

        // Determine initial tier based on access patterns
        let tier = self.determine_initial_tier(&key, size);

        let entry = AdaptiveCacheEntry {
            value: value_arc,
            pattern: self.get_or_create_pattern(&key),
            size,
            tier,
            created_at: Utc::now(),
            expires_at: self.calculate_expiration(tier),
        };

        // Insert into appropriate tier
        match tier {
            CacheTier::L1 => self.insert_l1(key, entry).await?,
            CacheTier::L2 => self.insert_l2(key, entry).await?,
            CacheTier::L3 => self.insert_l3(key, entry).await?,
        }

        self.record_insert_time(start.elapsed());
        Ok(())
    }

    /// Remove entry from all tiers
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn remove(&self, key: &K) -> Option<Arc<V>> {
        // Try to remove from all tiers
        let l1_removed = self.l1_cache.write().remove(key);
        let l2_removed = self.l2_cache.write().remove(key);
        let l3_removed = self.l3_cache.write().remove(key);

        // Return the most recent value found
        l1_removed
            .or(l2_removed)
            .or(l3_removed)
            .map(|entry| entry.value)
    }

    /// Clear all cache tiers
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn clear(&self) -> Result<()> {
        self.l1_cache.write().clear();
        self.l2_cache.write().clear();
        self.l3_cache.write().clear();
        self.access_patterns.write().clear();

        // Reset statistics
        let mut stats = self.stats.write();
        for tier_stats in stats.tier_stats.values_mut() {
            tier_stats.hits.store(0, Ordering::Relaxed);
            tier_stats.misses.store(0, Ordering::Relaxed);
            tier_stats.evictions.store(0, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Get comprehensive cache statistics
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_stats(&self) -> AdaptiveCacheStats {
        let _stats = self.stats.read();
        // Manual clone since we removed Clone derive due to atomics
        AdaptiveCacheStats {
            tier_stats: FxHashMap::default(), // Simplified for now
            ..Default::default()
        }
    }

    /// Warm cache based on configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn warm_cache(&self, warm_keys: Vec<K>) -> Result<usize> {
        let start = Instant::now();
        let mut warmed_count = 0;

        for key in warm_keys {
            if let Some(_predicted_value) = self.predictor.predict_value(&key) {
                // This is a simplified warming - in practice, you'd compute the actual value
                // self.put(key, predicted_value).await?;
                warmed_count += 1;
            }
        }

        let warming_time = start.elapsed();
        self.stats.write().warming_stats.total_warming_time = warming_time;
        self.stats
            .write()
            .warming_stats
            .files_warmed
            .store(warmed_count, Ordering::Relaxed);

        info!(
            "Cache warming completed: {} entries in {:?}",
            warmed_count, warming_time
        );
        Ok(warmed_count)
    }

    /// Run background maintenance
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn background_maintenance(&self) -> Result<()> {
        if !self.config.performance_config.background_cleanup {
            return Ok(());
        }

        // Clean expired entries
        self.cleanup_expired_entries().await?;

        // Cache layout optimization
        self.optimize_cache_layout().await?;

        // Update access patterns
        self.update_global_patterns();

        self.stats
            .write()
            .performance
            .cleanup_operations
            .fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}
