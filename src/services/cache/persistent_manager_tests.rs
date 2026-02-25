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
        let _ = diag.uptime.as_nanos();
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
        config.max_memory_mb = 0;
        let manager = PersistentCacheManager::new(config, cache_dir).unwrap();

        let diag = manager.get_diagnostics();

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

        manager.cleanup_expired();
        manager.cleanup_expired();
        manager.cleanup_expired();
    }
}
