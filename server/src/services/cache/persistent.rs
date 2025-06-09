use crate::services::cache::base::{CacheEntry, CacheStats, CacheStrategy};
use anyhow::{Context, Result};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Persistent cache entry for serialization
#[derive(Serialize, Deserialize, Clone)]
struct PersistentCacheEntry<V> {
    value: V,
    created_timestamp: u64, // Unix timestamp in seconds
    size_bytes: usize,
}

impl<V> PersistentCacheEntry<V> {
    fn new(value: V, size_bytes: usize) -> Self {
        let created_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            value,
            created_timestamp,
            size_bytes,
        }
    }

    fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Duration::from_secs(now.saturating_sub(self.created_timestamp))
    }

    fn into_cache_entry(self) -> CacheEntry<V> {
        let age = self.age();
        let created = Instant::now() - age;

        CacheEntry {
            value: Arc::new(self.value),
            created,
            access_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            size_bytes: self.size_bytes,
            last_accessed: Arc::new(parking_lot::Mutex::new(Instant::now())),
        }
    }
}

/// Persistent file-based cache with TTL support
pub struct PersistentCache<T: CacheStrategy> {
    /// In-memory cache for fast access
    memory_cache: Arc<RwLock<FxHashMap<String, CacheEntry<T::Value>>>>,

    /// Cache directory
    cache_dir: PathBuf,

    /// Strategy
    strategy: Arc<T>,

    /// Statistics
    pub stats: CacheStats,
}

impl<T: CacheStrategy> PersistentCache<T>
where
    T::Value: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(strategy: T, cache_dir: PathBuf) -> Result<Self> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).with_context(|| {
            format!("Failed to create cache directory: {}", cache_dir.display())
        })?;

        let mut cache = Self {
            memory_cache: Arc::new(RwLock::new(FxHashMap::default())),
            cache_dir,
            strategy: Arc::new(strategy),
            stats: CacheStats::new(),
        };

        // Load existing cache entries
        cache.load_from_disk()?;

        Ok(cache)
    }

    /// Get cache file path for a key
    fn cache_file_path(&self, cache_key: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        cache_key.hash(&mut hasher);
        let hash = hasher.finish();

        self.cache_dir.join(format!("{hash:016x}.json"))
    }

    /// Load cache entries from disk
    fn load_from_disk(&mut self) -> Result<()> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.cache_dir).with_context(|| {
            format!(
                "Failed to read cache directory: {}",
                self.cache_dir.display()
            )
        })?;

        let mut loaded = 0;
        let mut expired = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_cache_file(&path) {
                    Ok(true) => loaded += 1,
                    Ok(false) => expired += 1,
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to load cache file {}: {}",
                            path.display(),
                            e
                        );
                        // Remove corrupted cache file
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }

        eprintln!("Loaded {loaded} cache entries, expired {expired} entries");
        Ok(())
    }

    /// Load a single cache file
    fn load_cache_file(&self, path: &Path) -> Result<bool> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read cache file: {}", path.display()))?;

        // Try to deserialize as different value types
        // This is a bit hacky but works for our use case
        if let Ok(entry) = serde_json::from_str::<PersistentCacheEntry<T::Value>>(&content) {
            // Check if expired
            if let Some(ttl) = self.strategy.ttl() {
                if entry.age() > ttl {
                    // Expired, remove file
                    let _ = fs::remove_file(path);
                    return Ok(false);
                }
            }

            // Extract cache key from filename (reverse lookup)
            let size_bytes = entry.size_bytes;
            let cache_entry = entry.into_cache_entry();
            let filename = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            self.memory_cache
                .write()
                .insert(filename.to_string(), cache_entry);
            self.stats.add_bytes(size_bytes);
            return Ok(true);
        }

        Ok(false)
    }

    /// Get a value from the cache
    pub fn get(&self, key: &T::Key) -> Option<Arc<T::Value>> {
        let cache_key = self.strategy.cache_key(key);

        // First check memory cache
        {
            let mut memory = self.memory_cache.write();
            if let Some(entry) = memory.get_mut(&cache_key) {
                // Check TTL
                if let Some(ttl) = self.strategy.ttl() {
                    if entry.age() > ttl {
                        // Expired, remove from memory and disk
                        self.stats.remove_bytes(entry.size_bytes);
                        memory.remove(&cache_key);
                        let _ = fs::remove_file(self.cache_file_path(&cache_key));
                        self.stats.record_miss();
                        return None;
                    }
                }

                // Validate
                if self.strategy.validate(key, &entry.value) {
                    entry.access();
                    self.stats.record_hit();
                    return Some(entry.value.clone());
                } else {
                    // Invalid, remove
                    self.stats.remove_bytes(entry.size_bytes);
                    memory.remove(&cache_key);
                    let _ = fs::remove_file(self.cache_file_path(&cache_key));
                    self.stats.record_miss();
                    return None;
                }
            }
        }

        // Not in memory, try loading from disk
        let cache_file = self.cache_file_path(&cache_key);
        if cache_file.exists() {
            if let Ok(content) = fs::read_to_string(&cache_file) {
                if let Ok(persistent_entry) =
                    serde_json::from_str::<PersistentCacheEntry<T::Value>>(&content)
                {
                    // Check TTL
                    if let Some(ttl) = self.strategy.ttl() {
                        if persistent_entry.age() > ttl {
                            // Expired, remove file
                            let _ = fs::remove_file(&cache_file);
                            self.stats.record_miss();
                            return None;
                        }
                    }

                    // Load into memory cache
                    let size_bytes = persistent_entry.size_bytes;
                    let cache_entry = persistent_entry.into_cache_entry();

                    // Validate
                    if self.strategy.validate(key, &cache_entry.value) {
                        cache_entry.access();
                        let value = cache_entry.value.clone();

                        self.memory_cache.write().insert(cache_key, cache_entry);
                        self.stats.add_bytes(size_bytes);
                        self.stats.record_hit();
                        return Some(value);
                    } else {
                        // Invalid, remove file
                        let _ = fs::remove_file(&cache_file);
                        self.stats.record_miss();
                        return None;
                    }
                }
            }

            // Failed to load, remove corrupted file
            let _ = fs::remove_file(&cache_file);
        }

        self.stats.record_miss();
        None
    }

    /// Put a value into the cache
    pub fn put(&self, key: T::Key, value: T::Value) -> Result<()> {
        let cache_key = self.strategy.cache_key(&key);
        let size_bytes = self.estimate_size(&value);

        // Create persistent entry
        let persistent_entry = PersistentCacheEntry::new(value.clone(), size_bytes);

        // Save to disk
        let cache_file = self.cache_file_path(&cache_key);
        if let Ok(content) = serde_json::to_string(&persistent_entry) {
            if let Err(e) = fs::write(&cache_file, content) {
                eprintln!(
                    "Warning: Failed to write cache file {}: {}",
                    cache_file.display(),
                    e
                );
            }
        }

        // Add to memory cache
        let cache_entry = CacheEntry::new(value, size_bytes);
        self.memory_cache.write().insert(cache_key, cache_entry);
        self.stats.add_bytes(size_bytes);
        Ok(())
    }

    /// Estimate size of a value (simplified)
    fn estimate_size(&self, _value: &T::Value) -> usize {
        // Simplified size estimation
        // In practice, you might want to implement proper size calculation
        std::mem::size_of::<T::Value>()
    }

    /// Clear expired entries
    pub fn cleanup_expired(&self) {
        let mut to_remove = Vec::new();

        // Check memory cache
        {
            let memory = self.memory_cache.read();
            for (key, entry) in memory.iter() {
                if let Some(ttl) = self.strategy.ttl() {
                    if entry.age() > ttl {
                        to_remove.push(key.clone());
                    }
                }
            }
        }

        // Remove expired entries
        {
            let mut memory = self.memory_cache.write();
            for key in &to_remove {
                if let Some(entry) = memory.remove(key) {
                    self.stats.remove_bytes(entry.size_bytes);
                    let _ = fs::remove_file(self.cache_file_path(key));
                }
            }
        }

        // Also clean up any expired files on disk
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(persistent_entry) =
                            serde_json::from_str::<PersistentCacheEntry<T::Value>>(&content)
                        {
                            if let Some(ttl) = self.strategy.ttl() {
                                if persistent_entry.age() > ttl {
                                    let _ = fs::remove_file(&path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get cache statistics
    pub fn len(&self) -> usize {
        self.memory_cache.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.memory_cache.read().is_empty()
    }

    /// Remove a specific entry from the cache
    pub fn remove(&self, key: &T::Key) -> Option<Arc<T::Value>> {
        let cache_key = self.strategy.cache_key(key);

        // Remove from memory cache
        let value = self.memory_cache.write().remove(&cache_key).map(|entry| {
            self.stats.remove_bytes(entry.size_bytes);
            entry.value.clone()
        });

        // Remove from disk
        let cache_file = self.cache_file_path(&cache_key);
        let _ = fs::remove_file(&cache_file);

        value
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<()> {
        // Clear memory cache and update stats
        {
            let mut memory = self.memory_cache.write();
            for (_, entry) in memory.iter() {
                self.stats.remove_bytes(entry.size_bytes);
            }
            memory.clear();
        }

        // Clear disk cache
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let _ = fs::remove_file(&path);
                }
            }
        }
        Ok(())
    }

    /// Evict entries if needed based on memory pressure
    pub fn evict_if_needed(&self) {
        // Simple eviction strategy: remove oldest entries if we exceed max_size
        let max_size = self.strategy.max_size();

        while self.len() > max_size {
            // Find the oldest entry
            let oldest_key = {
                let memory = self.memory_cache.read();
                memory
                    .iter()
                    .min_by_key(|(_, entry)| entry.created)
                    .map(|(key, _)| key.clone())
            };

            if let Some(key) = oldest_key {
                let mut memory = self.memory_cache.write();
                if let Some(entry) = memory.remove(&key) {
                    self.stats.remove_bytes(entry.size_bytes);
                    self.stats.record_eviction();
                    let _ = fs::remove_file(self.cache_file_path(&key));
                }
            } else {
                break;
            }
        }
    }
}
