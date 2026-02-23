#![cfg_attr(coverage_nightly, coverage(off))]
//! Eviction strategy implementations for the adaptive cache
//!
//! Provides LRU, LFU, TTL, FIFO, Random, and Adaptive eviction policies.

use anyhow::Result;
use chrono::Utc;
use rustc_hash::FxHashMap;
use std::sync::atomic::Ordering;

use super::adaptive_cache::AdaptiveCache;
use super::types::{AccessPattern, AdaptiveCacheEntry, CacheTier, EvictionPolicy};

/// Trait grouping all eviction methods for AdaptiveCache
pub(crate) trait EvictionMethods<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn evict_from_tier(
        &self,
        cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>,
        tier: CacheTier,
    ) -> Result<()>;
    fn evict_lru(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>);
    fn evict_lfu(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>);
    fn evict_ttl(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>);
    fn evict_fifo(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>);
    fn evict_random(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>);
    fn evict_adaptive(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>);
    fn calculate_eviction_score(&self, pattern: &AccessPattern) -> f64;
}

impl<K, V> EvictionMethods<K, V> for AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn evict_from_tier(
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

    fn evict_lru(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.pattern.last_access)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    fn evict_lfu(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(least_used_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.pattern.access_count)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&least_used_key);
        }
    }

    fn evict_ttl(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
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

    fn evict_fifo(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    fn evict_random(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
    }

    fn evict_adaptive(&self, cache: &mut FxHashMap<K, AdaptiveCacheEntry<V>>) {
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

    fn calculate_eviction_score(&self, pattern: &AccessPattern) -> f64 {
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
