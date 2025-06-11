//! High-level caching for parsed file results
//!
//! This module provides caching for FileContext and FileComplexityMetrics
//! to avoid re-parsing the same files multiple times.

use crate::services::complexity::FileComplexityMetrics;
use crate::services::context::FileContext;
use anyhow::Result;
use blake3::Hasher;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Cache key for parsed file entries
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ParsedFileCacheKey {
    /// Canonical file path
    path: PathBuf,
    /// Blake3 hash of file contents
    content_hash: [u8; 32],
    /// Cache type (context or complexity)
    cache_type: CacheType,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum CacheType {
    Context,
    Complexity,
}

/// Cached parsed file data
#[derive(Debug, Clone)]
enum CachedData {
    Context(Arc<FileContext>),
    Complexity(Arc<FileComplexityMetrics>),
}

/// Cached entry with metadata
#[derive(Debug)]
struct CachedEntry {
    /// The cached data
    data: CachedData,
    /// When this entry was created
    created_at: SystemTime,
    /// Size of the original source file
    #[allow(dead_code)]
    source_size: usize,
}

/// Thread-safe cache for parsed file results
pub struct ParsedFileCache {
    /// The actual cache storage
    cache: Arc<DashMap<ParsedFileCacheKey, CachedEntry>>,
    /// Maximum number of entries
    max_entries: usize,
    /// Time-to-live for cache entries
    ttl: Duration,
}

impl ParsedFileCache {
    /// Create a new parsed file cache
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_entries,
            ttl,
        }
    }

    /// Get or compute FileContext with memoization
    pub async fn get_or_compute_context<F, Fut>(
        &self,
        path: &Path,
        content: &str,
        compute: F,
    ) -> Result<Arc<FileContext>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<FileContext>>,
    {
        let key = self.compute_key(path, content, CacheType::Context)?;

        // Check cache first
        if let Some(entry) = self.cache.get(&key) {
            if entry.created_at.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                if let CachedData::Context(context) = &entry.data {
                    return Ok(context.clone());
                }
            }
        }

        // Compute the result
        let context = compute().await?;
        let context = Arc::new(context);

        // Store in cache
        let entry = CachedEntry {
            data: CachedData::Context(context.clone()),
            created_at: SystemTime::now(),
            source_size: content.len(),
        };

        self.cache.insert(key, entry);
        self.perform_maintenance();

        Ok(context)
    }

    /// Get or compute FileComplexityMetrics with memoization
    pub async fn get_or_compute_complexity<F, Fut>(
        &self,
        path: &Path,
        content: &str,
        compute: F,
    ) -> Result<Arc<FileComplexityMetrics>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<FileComplexityMetrics>>,
    {
        let key = self.compute_key(path, content, CacheType::Complexity)?;

        // Check cache first
        if let Some(entry) = self.cache.get(&key) {
            if entry.created_at.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                if let CachedData::Complexity(complexity) = &entry.data {
                    return Ok(complexity.clone());
                }
            }
        }

        // Compute the result
        let complexity = compute().await?;
        let complexity = Arc::new(complexity);

        // Store in cache
        let entry = CachedEntry {
            data: CachedData::Complexity(complexity.clone()),
            created_at: SystemTime::now(),
            source_size: content.len(),
        };

        self.cache.insert(key, entry);
        self.perform_maintenance();

        Ok(complexity)
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let (context_count, complexity_count) =
            self.cache
                .iter()
                .fold((0, 0), |(ctx, cplx), entry| match entry.key().cache_type {
                    CacheType::Context => (ctx + 1, cplx),
                    CacheType::Complexity => (ctx, cplx + 1),
                });

        CacheStats {
            total_entries,
            context_entries: context_count,
            complexity_entries: complexity_count,
            max_entries: self.max_entries,
        }
    }

    /// Compute cache key from file path and contents
    fn compute_key(
        &self,
        path: &Path,
        content: &str,
        cache_type: CacheType,
    ) -> Result<ParsedFileCacheKey> {
        // Compute content hash
        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        let content_hash = hasher.finalize().into();

        Ok(ParsedFileCacheKey {
            path: path.to_path_buf(),
            content_hash,
            cache_type,
        })
    }

    /// Perform cache maintenance (evict old entries)
    fn perform_maintenance(&self) {
        if self.cache.len() <= self.max_entries {
            return;
        }

        let _now = SystemTime::now();

        // Remove expired entries
        let expired_keys: Vec<_> = self
            .cache
            .iter()
            .filter(|entry| entry.created_at.elapsed().unwrap_or(Duration::MAX) > self.ttl)
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_keys {
            self.cache.remove(&key);
        }

        // If still over capacity, remove oldest entries
        if self.cache.len() > self.max_entries {
            let mut entries: Vec<_> = self
                .cache
                .iter()
                .map(|entry| (entry.key().clone(), entry.created_at))
                .collect();

            entries.sort_by_key(|(_, created_at)| *created_at);

            let to_remove = entries.len() - self.max_entries;
            for (key, _) in entries.into_iter().take(to_remove) {
                self.cache.remove(&key);
            }
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries in cache
    pub total_entries: usize,
    /// Number of FileContext entries
    pub context_entries: usize,
    /// Number of FileComplexityMetrics entries
    pub complexity_entries: usize,
    /// Maximum allowed entries
    pub max_entries: usize,
}

lazy_static::lazy_static! {
    /// Global parsed file cache
    pub static ref PARSED_FILE_CACHE: ParsedFileCache = ParsedFileCache::new(
        1000,  // Max 1000 entries
        Duration::from_secs(300)  // 5 minute TTL
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parsed_file_cache_context() {
        let cache = ParsedFileCache::new(10, Duration::from_secs(60));

        let path = Path::new("test.rs");
        let content = "fn main() {}";

        let result = cache
            .get_or_compute_context(path, content, || async {
                Ok(FileContext {
                    path: "test.rs".to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await
            .unwrap();

        assert_eq!(result.path, "test.rs");

        // Second call should use cache
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        let result2 = cache
            .get_or_compute_context(path, content, || async move {
                called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(FileContext {
                    path: "should_not_be_called".to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            })
            .await
            .unwrap();

        assert!(
            !called.load(std::sync::atomic::Ordering::SeqCst),
            "Cache should have been used"
        );
        assert_eq!(result2.path, "test.rs");
    }
}
