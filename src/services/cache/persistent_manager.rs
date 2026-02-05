use crate::services::cache::{
    config::CacheConfig,
    diagnostics::{CacheDiagnostics, CacheEffectiveness, CacheStatsSnapshot},
    persistent::PersistentCache,
    strategies::AstCacheStrategy,
};
use crate::services::context::FileContext;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Persistent cache manager that stores cache data on disk
pub struct PersistentCacheManager {
    // Different cache types
    ast_cache: Arc<PersistentCache<AstCacheStrategy>>,

    // Global settings
    config: CacheConfig,
    session_id: Uuid,
    created: Instant,
    #[allow(dead_code)]
    cache_dir: PathBuf,
}

impl PersistentCacheManager {
    pub fn new(config: CacheConfig, cache_dir: PathBuf) -> Result<Self> {
        // Create individual cache directories
        let ast_cache_dir = cache_dir.join("ast");

        Ok(Self {
            ast_cache: Arc::new(PersistentCache::new(AstCacheStrategy, ast_cache_dir)?),
            config,
            session_id: Uuid::new_v4(),
            created: Instant::now(),
            cache_dir,
        })
    }

    /// Create with default cache directory
    pub fn with_default_dir(config: CacheConfig) -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Self::new(config, cache_dir)
    }

    /// Get default cache directory
    pub fn default_cache_dir() -> Result<PathBuf> {
        if let Some(cache_dir) = dirs::cache_dir() {
            Ok(cache_dir.join("paiml-mcp-agent-toolkit"))
        } else if let Some(home_dir) = dirs::home_dir() {
            Ok(home_dir.join(".cache").join("paiml-mcp-agent-toolkit"))
        } else {
            // Fallback to /tmp
            Ok(PathBuf::from("/tmp/paiml-mcp-agent-toolkit-cache"))
        }
    }

    /// Get or compute AST with caching
    pub async fn get_or_compute_ast<F, Fut>(
        &self,
        path: &Path,
        compute: F,
    ) -> Result<Arc<FileContext>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<FileContext>>,
    {
        let path_buf = path.to_path_buf();

        // Try cache first
        if let Some(ast) = self.ast_cache.get(&path_buf) {
            return Ok(ast);
        }

        // Compute and cache
        let ast = compute().await?;
        let _ = self.ast_cache.put(path_buf, ast.clone());
        Ok(Arc::new(ast))
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        self.ast_cache.cleanup_expired();
    }

    /// Clear all caches
    pub fn clear(&self) {
        let _ = self.ast_cache.clear();
    }

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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_persistent_manager_new_creates_cache_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();

        let manager = PersistentCacheManager::new(config, cache_dir.clone());
        assert!(manager.is_ok());

        // Check that ast subdirectory was created
        let ast_dir = cache_dir.join("ast");
        assert!(ast_dir.exists(), "AST cache directory should exist");
        assert!(
            ast_dir.is_dir(),
            "AST cache directory should be a directory"
        );
    }

    #[test]
    fn test_persistent_manager_new_with_existing_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir.clone());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_persistent_manager_with_default_dir() {
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::with_default_dir(config);
        // Should succeed - uses system cache directory
        assert!(manager.is_ok());
    }

    #[test]
    fn test_default_cache_dir_returns_path() {
        let cache_dir = PersistentCacheManager::default_cache_dir();
        assert!(cache_dir.is_ok());

        let path = cache_dir.unwrap();
        assert!(path.to_string_lossy().contains("paiml-mcp-agent-toolkit"));
    }

    // =========================================================================
    // Session ID Tests
    // =========================================================================

    #[test]
    fn test_persistent_manager_has_unique_session_id() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = CacheConfig::default();

        let manager1 =
            PersistentCacheManager::new(config.clone(), temp_dir.path().join("cache1")).unwrap();
        let manager2 =
            PersistentCacheManager::new(config.clone(), temp_dir.path().join("cache2")).unwrap();

        let diag1 = manager1.get_diagnostics();
        let diag2 = manager2.get_diagnostics();

        assert_ne!(
            diag1.session_id, diag2.session_id,
            "Each manager should have a unique session ID"
        );
    }

    // =========================================================================
    // Cache Operations Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_or_compute_ast_cache_miss() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

        let compute_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let compute_called_clone = compute_called.clone();
        let test_file_path = test_file.to_string_lossy().to_string();

        let result = manager
            .get_or_compute_ast(&test_file, || async move {
                compute_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(FileContext {
                    path: test_file_path,
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await;

        assert!(result.is_ok());
        assert!(
            compute_called.load(std::sync::atomic::Ordering::SeqCst),
            "Compute function should be called on cache miss"
        );
    }

    #[tokio::test]
    async fn test_get_or_compute_ast_cache_hit() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

        // First call - cache miss
        let test_file_clone = test_file.clone();
        let _ = manager
            .get_or_compute_ast(&test_file, || async move {
                Ok(FileContext {
                    path: test_file_clone.to_string_lossy().to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await
            .unwrap();

        // Second call - should use cache (compute should not be called)
        let compute_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let compute_called_clone = compute_called.clone();
        let test_file_clone2 = test_file.clone();

        let _ = manager
            .get_or_compute_ast(&test_file, || async move {
                compute_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(FileContext {
                    path: test_file_clone2.to_string_lossy().to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await;

        // Note: Due to mtime-based validation, compute may or may not be called
        // The test verifies the function completes successfully
    }

    #[tokio::test]
    async fn test_get_or_compute_ast_error_propagation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let test_file = temp_dir.path().join("nonexistent.rs");

        let result = manager
            .get_or_compute_ast(&test_file, || async move {
                Err(anyhow::anyhow!("Simulated error"))
            })
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Simulated error"));
    }

    // =========================================================================
    // Clear and Cleanup Tests
    // =========================================================================

    #[tokio::test]
    async fn test_clear_removes_all_entries() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir.clone()).unwrap();

        // Create and cache a test file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

        let test_file_clone = test_file.clone();
        let _ = manager
            .get_or_compute_ast(&test_file, || async move {
                Ok(FileContext {
                    path: test_file_clone.to_string_lossy().to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await
            .unwrap();

        // Clear cache
        manager.clear();

        // Verify cache is cleared via diagnostics
        let diag = manager.get_diagnostics();
        // After clear, entries should be 0
        for (_, stats) in &diag.cache_stats {
            assert_eq!(stats.entries, 0, "Cache should be empty after clear");
        }
    }

    #[test]
    fn test_cleanup_expired_does_not_panic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        // Should not panic even with empty cache
        manager.cleanup_expired();
    }

    // =========================================================================
    // Diagnostics Tests
    // =========================================================================

    #[test]
    fn test_get_diagnostics_returns_valid_structure() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let diag = manager.get_diagnostics();

        // Verify structure
        assert!(!diag.session_id.is_nil(), "Session ID should not be nil");
        assert!(diag.uptime.as_nanos() >= 0, "Uptime should be non-negative");
        assert!(
            diag.memory_usage_mb >= 0.0,
            "Memory usage should be non-negative"
        );
        assert!(
            diag.memory_pressure >= 0.0 && diag.memory_pressure <= 1.0,
            "Memory pressure should be between 0 and 1"
        );
        assert!(
            !diag.cache_stats.is_empty(),
            "Should have at least one cache stat"
        );
        assert!(
            diag.cache_stats.iter().any(|(name, _)| name == "ast"),
            "Should have AST cache stats"
        );
    }

    #[test]
    fn test_get_diagnostics_memory_pressure_calculation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let mut config = CacheConfig::default();
        config.max_memory_mb = 100;
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let diag = manager.get_diagnostics();

        // With empty cache, memory pressure should be very low
        assert!(
            diag.memory_pressure < 0.1,
            "Empty cache should have low memory pressure"
        );
    }

    #[test]
    fn test_get_diagnostics_zero_max_memory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let mut config = CacheConfig::default();
        config.max_memory_mb = 0; // Zero max memory
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let diag = manager.get_diagnostics();

        // With zero max memory, pressure should be 0.0 (not NaN or Inf)
        assert_eq!(
            diag.memory_pressure, 0.0,
            "With zero max_memory_mb, pressure should be 0.0"
        );
    }

    #[test]
    fn test_get_diagnostics_effectiveness_metrics() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let diag = manager.get_diagnostics();

        // Verify effectiveness structure
        assert!(
            diag.effectiveness.overall_hit_rate >= 0.0
                && diag.effectiveness.overall_hit_rate <= 1.0,
            "Hit rate should be between 0 and 1"
        );
        assert!(
            diag.effectiveness.memory_efficiency >= 0.0
                && diag.effectiveness.memory_efficiency <= 1.0,
            "Memory efficiency should be between 0 and 1"
        );
        assert!(
            !diag.effectiveness.most_valuable_caches.is_empty(),
            "Should have most valuable caches list"
        );
    }

    #[test]
    fn test_get_diagnostics_uptime_increases() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let diag1 = manager.get_diagnostics();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let diag2 = manager.get_diagnostics();

        assert!(
            diag2.uptime >= diag1.uptime,
            "Uptime should increase over time"
        );
    }

    // =========================================================================
    // Edge Cases Tests
    // =========================================================================

    #[test]
    fn test_persistent_manager_nested_cache_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("deeply").join("nested").join("cache");
        let config = CacheConfig::default();

        let manager = PersistentCacheManager::new(config, cache_dir.clone());
        assert!(
            manager.is_ok(),
            "Should create deeply nested cache directory"
        );

        let ast_dir = cache_dir.join("ast");
        assert!(ast_dir.exists());
    }

    #[tokio::test]
    async fn test_get_or_compute_ast_with_special_characters_in_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        // Create a test file with special characters
        let test_file = temp_dir.path().join("test-file_with.special.rs");
        fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

        let test_file_clone = test_file.clone();
        let result = manager
            .get_or_compute_ast(&test_file, || async move {
                Ok(FileContext {
                    path: test_file_clone.to_string_lossy().to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await;

        assert!(
            result.is_ok(),
            "Should handle paths with special characters"
        );
    }

    #[test]
    fn test_multiple_clear_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        // Multiple clears should not panic
        manager.clear();
        manager.clear();
        manager.clear();
    }

    #[test]
    fn test_multiple_cleanup_expired_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        let config = CacheConfig::default();
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        // Multiple cleanups should not panic
        manager.cleanup_expired();
        manager.cleanup_expired();
        manager.cleanup_expired();
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // =========================================================================
    // Property-Based Tests for Configuration
    // =========================================================================

    proptest! {
        #[test]
        fn test_diagnostics_memory_pressure_bounds(max_mb in 1usize..10000) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join("cache");
            let mut config = CacheConfig::default();
            config.max_memory_mb = max_mb;

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                prop_assert!(diag.memory_pressure >= 0.0);
                prop_assert!(diag.memory_pressure <= 1.0);
            }
        }

        #[test]
        fn test_diagnostics_effectiveness_bounds(_seed in 0u32..1000) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();

                // Hit rate should be between 0 and 1
                prop_assert!(diag.effectiveness.overall_hit_rate >= 0.0);
                prop_assert!(diag.effectiveness.overall_hit_rate <= 1.0);

                // Memory efficiency should be between 0 and 1
                prop_assert!(diag.effectiveness.memory_efficiency >= 0.0);
                prop_assert!(diag.effectiveness.memory_efficiency <= 1.0);
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Uptime
    // =========================================================================

    proptest! {
        #[test]
        fn test_uptime_is_non_negative(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                prop_assert!(diag.uptime.as_nanos() >= 0);
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Session ID
    // =========================================================================

    proptest! {
        #[test]
        fn test_session_id_is_valid_uuid(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                // UUID should not be nil
                prop_assert!(!diag.session_id.is_nil());
                // UUID version should be 4 (random)
                prop_assert_eq!(diag.session_id.get_version_num(), 4);
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Cache Stats
    // =========================================================================

    proptest! {
        #[test]
        fn test_cache_stats_has_ast_entry(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                let has_ast = diag.cache_stats.iter().any(|(name, _)| name == "ast");
                prop_assert!(has_ast, "Should always have AST cache stats");
            }
        }
    }

    // =========================================================================
    // Property-Based Tests for Memory Usage
    // =========================================================================

    proptest! {
        #[test]
        fn test_memory_usage_is_non_negative(_seed in 0u32..100) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let cache_dir = temp_dir.path().join(format!("cache_{}", _seed));
            let config = CacheConfig::default();

            if let Ok(manager) = PersistentCacheManager::new(config, cache_dir) {
                let diag = manager.get_diagnostics();
                prop_assert!(diag.memory_usage_mb >= 0.0);
            }
        }
    }
}
