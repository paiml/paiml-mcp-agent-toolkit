// Cache operations - TwoTierCache method implementations
// Included by cache.rs via include!()

impl TwoTierCache {
    pub fn new() -> Self {
        Self {
            l1: Arc::new(RwLock::new(HashMap::new())),
            l2: Arc::new(RwLock::new(HashMap::new())),
            l1_ttl: Duration::from_millis(10),
            l2_ttl: Duration::from_secs(60),
            l1_hits: AtomicU64::new(0),
            l1_misses: AtomicU64::new(0),
            l2_hits: AtomicU64::new(0),
            l2_misses: AtomicU64::new(0),
        }
    }

    /// Get value from cache or load it
    pub async fn get_with_loader<F, Fut>(&self, key: &str, loader: F) -> Arc<AnalysisResult>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AnalysisResult>,
    {
        let hash = self.hash_key(key);

        // L1 lookup - ~100ns
        {
            let l1_guard = self.l1.read().await;
            if let Some(entry) = l1_guard.get(&hash) {
                if !entry.is_expired() {
                    self.l1_hits.fetch_add(1, Ordering::Relaxed);
                    return Arc::clone(&entry.value);
                }
            }
        }
        self.l1_misses.fetch_add(1, Ordering::Relaxed);

        // L2 lookup - ~1μs
        {
            let l2_guard = self.l2.read().await;
            if let Some(entry) = l2_guard.get(&hash) {
                if !entry.is_expired() {
                    self.l2_hits.fetch_add(1, Ordering::Relaxed);
                    let value = Arc::clone(&entry.value);

                    // Promote to L1
                    drop(l2_guard);
                    let mut l1_guard = self.l1.write().await;
                    l1_guard.insert(hash, CacheEntry::new((*value).clone(), self.l1_ttl));

                    return value;
                }
            }
        }
        self.l2_misses.fetch_add(1, Ordering::Relaxed);

        // Load from source - ~15ms
        let result = Arc::new(loader().await);

        // Populate both caches
        {
            let mut l1_guard = self.l1.write().await;
            l1_guard.insert(hash, CacheEntry::new((*result).clone(), self.l1_ttl));
        }

        {
            let mut l2_guard = self.l2.write().await;
            l2_guard.insert(hash, CacheEntry::new((*result).clone(), self.l2_ttl));
        }

        result
    }

    /// Hash key using FNV-1a
    #[inline(always)]
    pub fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = FnvHasher::default();
        hasher.write(key.as_bytes());
        hasher.finish()
    }

    /// Get cache hit rate metrics
    pub fn hit_rate(&self) -> CacheMetrics {
        let l1_total =
            self.l1_hits.load(Ordering::Relaxed) + self.l1_misses.load(Ordering::Relaxed);
        let l2_total =
            self.l2_hits.load(Ordering::Relaxed) + self.l2_misses.load(Ordering::Relaxed);

        let l1_hit_rate = if l1_total > 0 {
            self.l1_hits.load(Ordering::Relaxed) as f64 / l1_total as f64
        } else {
            0.0
        };

        let l2_hit_rate = if l2_total > 0 {
            self.l2_hits.load(Ordering::Relaxed) as f64 / l2_total as f64
        } else {
            0.0
        };

        let effective_total = l1_total;
        let effective_hits =
            self.l1_hits.load(Ordering::Relaxed) + self.l2_hits.load(Ordering::Relaxed);
        let effective_hit_rate = if effective_total > 0 {
            effective_hits as f64 / effective_total as f64
        } else {
            0.0
        };

        CacheMetrics {
            l1_hit_rate,
            l2_hit_rate,
            effective_hit_rate,
            l1_size: 0, // Would need to track size
            l2_size: 0,
        }
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.l1.write().await.clear();
        self.l2.write().await.clear();
    }

    /// Evict expired entries
    pub async fn evict_expired(&self) {
        {
            let mut l1_guard = self.l1.write().await;
            l1_guard.retain(|_, entry| !entry.is_expired());
        }

        {
            let mut l2_guard = self.l2.write().await;
            l2_guard.retain(|_, entry| !entry.is_expired());
        }
    }
}
