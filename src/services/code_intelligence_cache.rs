/// Unified cache for analysis results
pub struct UnifiedCache {
    cache: Arc<RwLock<lru::LruCache<String, AnalysisReport>>>,
}

impl UnifiedCache {
    /// Creates a new unified cache with the specified capacity.
    ///
    /// Uses an LRU (Least Recently Used) eviction policy to maintain bounded memory usage.
    /// When the cache reaches capacity, the least recently accessed items are evicted.
    ///
    /// # Parameters
    ///
    /// * `capacity` - Maximum number of analysis reports to store (must be > 0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::code_intelligence::UnifiedCache;
    ///
    /// // Create cache with capacity for 100 reports
    /// let cache = UnifiedCache::new(100);
    ///
    /// // Verify cache is empty initially
    /// # tokio_test::block_on(async {
    /// assert!(cache.get("any_key").await.is_none());
    /// # });
    /// ```
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        // Default to capacity 1 if 0 is provided (NonZeroUsize requirement)
        let capacity = std::num::NonZeroUsize::new(capacity)
            .unwrap_or(std::num::NonZeroUsize::new(1).expect("1 is non-zero (const)"));

        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(capacity))),
        }
    }

    /// Retrieves an analysis report from the cache without affecting LRU order.
    ///
    /// Uses `peek` instead of `get` to avoid updating the LRU order, making this
    /// operation read-only from the cache's perspective.
    ///
    /// # Parameters
    ///
    /// * `key` - The cache key to look up
    ///
    /// # Returns
    ///
    /// * `Some(AnalysisReport)` - If the key exists in cache
    /// * `None` - If the key is not found or has been evicted
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::code_intelligence::{UnifiedCache, AnalysisReport};
    /// use chrono::Utc;
    ///
    /// # tokio_test::block_on(async {
    /// let cache = UnifiedCache::new(10);
    ///
    /// // Cache miss returns None
    /// assert!(cache.get("nonexistent").await.is_none());
    ///
    /// // Add an item to cache
    /// let report = AnalysisReport {
    ///     duplicates: None,
    ///     dead_code: None,
    ///     complexity_metrics: None,
    ///     dependency_graph: None,
    ///     defect_predictions: None,
    ///     graph_metrics: None,
    ///     timestamp: Utc::now(),
    /// };
    ///
    /// cache.put("test_key".to_string(), report.clone()).await;
    ///
    /// // Cache hit returns the report
    /// let retrieved = cache.get("test_key").await;
    /// assert!(retrieved.is_some());
    /// assert_eq!(retrieved.expect("cache hit").timestamp, report.timestamp);
    /// # });
    /// ```
    pub async fn get(&self, key: &str) -> Option<AnalysisReport> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        self.cache.read().await.peek(key).cloned()
    }

    /// Stores an analysis report in the cache with LRU eviction.
    ///
    /// If the cache is at capacity, the least recently used item will be evicted
    /// to make room for the new report.
    ///
    /// # Parameters
    ///
    /// * `key` - Unique identifier for the report (typically from `AnalysisRequest::cache_key()`)
    /// * `report` - The analysis report to store
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::code_intelligence::{UnifiedCache, AnalysisReport};
    /// use chrono::Utc;
    ///
    /// # tokio_test::block_on(async {
    /// let cache = UnifiedCache::new(2); // Small capacity for testing
    ///
    /// let report1 = AnalysisReport {
    ///     duplicates: None,
    ///     dead_code: None,
    ///     complexity_metrics: None,
    ///     dependency_graph: None,
    ///     defect_predictions: None,
    ///     graph_metrics: None,
    ///     timestamp: Utc::now(),
    /// };
    ///
    /// let report2 = report1.clone();
    /// let report3 = report1.clone();
    ///
    /// // Fill cache to capacity
    /// cache.put("key1".to_string(), report1).await;
    /// cache.put("key2".to_string(), report2).await;
    ///
    /// // Both items should be retrievable
    /// assert!(cache.get("key1").await.is_some());
    /// assert!(cache.get("key2").await.is_some());
    ///
    /// // Adding third item should evict least recently used (key1)
    /// cache.put("key3".to_string(), report3).await;
    /// assert!(cache.get("key1").await.is_none()); // Evicted
    /// assert!(cache.get("key2").await.is_some()); // Still present
    /// assert!(cache.get("key3").await.is_some()); // Newly added
    /// # });
    /// ```
    pub async fn put(&self, key: String, report: AnalysisReport) {
        self.cache.write().await.put(key, report);
    }
}
