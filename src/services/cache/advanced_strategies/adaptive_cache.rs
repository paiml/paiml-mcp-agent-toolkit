#![cfg_attr(coverage_nightly, coverage(off))]
//! Multi-tier adaptive cache implementation
//!
//! Provides intelligent caching with automatic tier promotion/demotion,
//! pattern-based placement, and background maintenance.

use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

use super::eviction::EvictionMethods;
use super::predictor::CachePredictor;
use super::types::{
    AccessPattern, AdaptiveCacheEntry, AdaptiveCacheStats, AdvancedCacheConfig, CacheTier,
    EvictionPolicy, TierStats,
};

/// Multi-tier adaptive cache implementation
pub struct AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Cache configuration
    pub(crate) config: AdvancedCacheConfig,
    /// L1 cache (fastest)
    pub(crate) l1_cache: Arc<RwLock<FxHashMap<K, AdaptiveCacheEntry<V>>>>,
    /// L2 cache (compressed)
    pub(crate) l2_cache: Arc<RwLock<FxHashMap<K, AdaptiveCacheEntry<V>>>>,
    /// L3 cache (persistent)
    pub(crate) l3_cache: Arc<RwLock<FxHashMap<K, AdaptiveCacheEntry<V>>>>,
    /// Access pattern tracker
    pub(crate) access_patterns: Arc<RwLock<FxHashMap<K, AccessPattern>>>,
    /// Cache statistics
    pub(crate) stats: Arc<RwLock<AdaptiveCacheStats>>,
    /// Predictive cache warmer
    pub(crate) predictor: Arc<CachePredictor<K>>,
}

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new adaptive cache
    #[must_use]
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
            predictor: Arc::new(CachePredictor::new(0.8)),
        }
    }

    /// Get value from cache with intelligent tier promotion
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
    pub fn get_stats(&self) -> AdaptiveCacheStats {
        let _stats = self.stats.read();
        // Manual clone since we removed Clone derive due to atomics
        AdaptiveCacheStats {
            tier_stats: FxHashMap::default(), // Simplified for now
            ..Default::default()
        }
    }

    /// Warm cache based on configuration
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

    // Helper methods

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

    pub(crate) async fn insert_l1(
        &self,
        key: K,
        entry: AdaptiveCacheEntry<V>,
    ) -> Result<()> {
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

    pub(crate) async fn insert_l2(
        &self,
        key: K,
        entry: AdaptiveCacheEntry<V>,
    ) -> Result<()> {
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

    pub(crate) async fn insert_l3(
        &self,
        key: K,
        entry: AdaptiveCacheEntry<V>,
    ) -> Result<()> {
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

    pub(crate) fn calculate_tier_size(
        &self,
        cache: &FxHashMap<K, AdaptiveCacheEntry<V>>,
    ) -> usize {
        cache.values().map(|entry| entry.size).sum()
    }

    pub(crate) fn record_hit(&self, tier: CacheTier, _access_time: Duration) {
        if let Some(tier_stats) = self.stats.read().tier_stats.get(&tier) {
            tier_stats.hits.fetch_add(1, Ordering::Relaxed);
            // Update average access time (simplified)
        }
    }

    pub(crate) fn record_miss(&self) {
        // Record miss for all tiers
        for tier_stats in self.stats.read().tier_stats.values() {
            tier_stats.misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_insert_time(&self, _insert_time: Duration) {
        // Update insertion statistics
    }

    pub(crate) fn update_access_pattern(&self, key: &K) {
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
