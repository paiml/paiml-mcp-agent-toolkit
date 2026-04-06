#![cfg_attr(coverage_nightly, coverage(off))]
//! Core cache management operations for the O(1) Hooks Cache Manager.
//!
//! Implements init, check, update, clear, metrics recording, and health checking.

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;

use super::types::{
    CacheCheckResult, CacheMissReason, CacheResult, GateCacheEntry, HooksCacheMetrics,
    TreeHashCache,
};
use super::HooksCacheManager;
use super::MAX_CACHE_AGE_HOURS;

impl HooksCacheManager {
    /// Initialize cache directory structure
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(self.cache_dir.join("gates"))?;
        fs::create_dir_all(self.cache_dir.join("files"))?;

        // Create initial metrics file
        let metrics = HooksCacheMetrics::default();
        self.save_metrics(&metrics)?;

        Ok(())
    }

    /// Check cache for O(1) decision
    ///
    /// Returns immediately if cache hit (5ms target)
    pub fn check(&self) -> Result<CacheCheckResult> {
        let cache_path = self.cache_dir.join("tree-hash.json");

        // Check if cache file exists
        if !cache_path.exists() {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::NoCacheFile,
            });
        }

        // Load cache
        let cache: TreeHashCache = match fs::read_to_string(&cache_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    return Ok(CacheCheckResult::Miss {
                        reason: CacheMissReason::CacheCorrupted(e.to_string()),
                    });
                }
            },
            Err(e) => {
                return Ok(CacheCheckResult::Miss {
                    reason: CacheMissReason::CacheCorrupted(e.to_string()),
                })
            }
        };

        // Check tree hash
        let current_hash = self.get_tree_hash()?;
        if cache.tree_hash != current_hash {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::TreeHashChanged {
                    old: cache.tree_hash,
                    new: current_hash,
                },
            });
        }

        // Check config hash
        let current_config_hash = self.get_config_hash()?;
        if cache.config_hash != current_config_hash {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::ConfigHashChanged,
            });
        }

        // Check PMAT version
        let current_version = env!("CARGO_PKG_VERSION");
        if cache.pmat_version != current_version {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::VersionChanged {
                    old: cache.pmat_version,
                    new: current_version.to_string(),
                },
            });
        }

        // Check cache staleness
        let age = Utc::now() - cache.timestamp;
        if age.num_hours() > MAX_CACHE_AGE_HOURS {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::CacheStale {
                    age_hours: age.num_hours(),
                },
            });
        }

        // Cache hit!
        Ok(CacheCheckResult::Hit {
            result: cache.result,
            cached_at: cache.timestamp,
        })
    }

    /// Update cache after successful gate run
    pub fn update(
        &self,
        result: CacheResult,
        gates: HashMap<String, GateCacheEntry>,
    ) -> Result<()> {
        let cache = TreeHashCache {
            tree_hash: self.get_tree_hash()?,
            result,
            gates,
            timestamp: Utc::now(),
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash: self.get_config_hash()?,
        };

        let cache_path = self.cache_dir.join("tree-hash.json");
        let content = serde_json::to_string_pretty(&cache)?;
        fs::write(cache_path, content)?;

        Ok(())
    }

    /// Clear all caches
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        self.init()?;
        Ok(())
    }

    /// Clear specific gate cache
    pub fn clear_gate(&self, gate_name: &str) -> Result<()> {
        debug_assert!(!gate_name.is_empty(), "gate_name must not be empty");
        let gate_path = self
            .cache_dir
            .join("gates")
            .join(format!("{}.json", gate_name));
        if gate_path.exists() {
            fs::remove_file(gate_path)?;
        }
        Ok(())
    }

    /// Get cache metrics
    pub fn get_metrics(&self) -> Result<HooksCacheMetrics> {
        let metrics_path = self.cache_dir.join("metrics.json");
        if !metrics_path.exists() {
            return Ok(HooksCacheMetrics::default());
        }

        let content = fs::read_to_string(metrics_path)?;
        let metrics: HooksCacheMetrics = serde_json::from_str(&content)?;
        Ok(metrics)
    }

    /// Update metrics after hook run
    pub fn record_run(&self, hit: bool, duration_ms: u64) -> Result<()> {
        let mut metrics = self.get_metrics()?;

        metrics.total_runs += 1;
        if hit {
            metrics.cache_hits += 1;
            // Update rolling average for hits
            let n = metrics.cache_hits as f64;
            metrics.avg_cache_hit_time_ms =
                ((n - 1.0) * metrics.avg_cache_hit_time_ms + duration_ms as f64) / n;
        } else {
            metrics.cache_misses += 1;
            metrics.last_full_rebuild = Some(Utc::now());
            // Update rolling average for misses
            let n = metrics.cache_misses as f64;
            metrics.avg_cache_miss_time_ms =
                ((n - 1.0) * metrics.avg_cache_miss_time_ms + duration_ms as f64) / n;
        }

        // Update cache size
        metrics.cache_size_bytes = self.calculate_cache_size()?;

        self.save_metrics(&metrics)?;
        Ok(())
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> Result<f64> {
        let metrics = self.get_metrics()?;
        if metrics.total_runs == 0 {
            return Ok(0.0);
        }
        Ok(metrics.cache_hits as f64 / metrics.total_runs as f64)
    }

    /// Check if cache is healthy (CB-021)
    pub fn is_healthy(&self) -> Result<bool> {
        let metrics = self.get_metrics()?;
        // Need at least 10 runs to assess health
        if metrics.total_runs < 10 {
            return Ok(true);
        }
        // Healthy if hit rate > 60%
        Ok(self.hit_rate()? > 0.60)
    }
}
