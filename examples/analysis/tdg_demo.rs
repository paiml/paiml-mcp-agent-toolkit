use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cache manager with high complexity due to multiple responsibilities
pub struct CacheManager {
    storage: Arc<Mutex<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStats>>,
}

struct CacheEntry {
    key: String,
    value: Vec<u8>,
    created_at: Instant,
    accessed_at: Instant,
    access_count: u64,
    ttl: Duration,
    size: usize,
}

struct CacheConfig {
    max_size: usize,
    max_entries: usize,
    default_ttl: Duration,
    eviction_policy: EvictionPolicy,
}

struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
    total_size: usize,
}

#[derive(Clone)]
enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
    Random,
}

impl CacheManager {
    pub fn new(max_size: usize, max_entries: usize) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            config: CacheConfig {
                max_size,
                max_entries,
                default_ttl: Duration::from_secs(3600),
                eviction_policy: EvictionPolicy::LRU,
            },
            stats: Arc::new(Mutex::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                total_size: 0,
            })),
        }
    }

    /// Get value from cache with complex validation and update logic
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut storage = self.storage.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        if let Some(entry) = storage.get_mut(key) {
            // Check if entry is expired
            if entry.created_at.elapsed() > entry.ttl {
                // Remove expired entry
                stats.total_size -= entry.size;
                storage.remove(key);
                stats.misses += 1;
                return None;
            }
            
            // Update access statistics
            entry.accessed_at = Instant::now();
            entry.access_count += 1;
            stats.hits += 1;
            
            // Clone value for return
            let value = entry.value.clone();
            
            // Special handling for frequently accessed items
            if entry.access_count > 100 {
                if entry.access_count % 10 == 0 {
                    // Extend TTL for hot items
                    entry.ttl = entry.ttl * 2;
                    if entry.ttl > Duration::from_secs(86400) {
                        entry.ttl = Duration::from_secs(86400);
                    }
                }
            }
            
            Some(value)
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Set value in cache with complex eviction logic
    pub fn set(&self, key: String, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), String> {
        let entry_size = key.len() + value.len() + 64; // Overhead estimate
        
        // Validate entry size
        if entry_size > self.config.max_size {
            return Err("Entry too large for cache".to_string());
        }
        
        let mut storage = self.storage.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Check if we need to evict entries
        if storage.len() >= self.config.max_entries || stats.total_size + entry_size > self.config.max_size {
            // Complex eviction logic based on policy
            match self.config.eviction_policy {
                EvictionPolicy::LRU => {
                    // Find least recently used entry
                    let mut oldest_key = None;
                    let mut oldest_time = Instant::now();
                    
                    for (k, entry) in storage.iter() {
                        if entry.accessed_at < oldest_time {
                            oldest_time = entry.accessed_at;
                            oldest_key = Some(k.clone());
                        }
                    }
                    
                    if let Some(key_to_remove) = oldest_key {
                        if let Some(removed) = storage.remove(&key_to_remove) {
                            stats.total_size -= removed.size;
                            stats.evictions += 1;
                        }
                    }
                }
                EvictionPolicy::LFU => {
                    // Find least frequently used entry
                    let mut least_used_key = None;
                    let mut min_count = u64::MAX;
                    
                    for (k, entry) in storage.iter() {
                        if entry.access_count < min_count {
                            min_count = entry.access_count;
                            least_used_key = Some(k.clone());
                        }
                    }
                    
                    if let Some(key_to_remove) = least_used_key {
                        if let Some(removed) = storage.remove(&key_to_remove) {
                            stats.total_size -= removed.size;
                            stats.evictions += 1;
                        }
                    }
                }
                EvictionPolicy::FIFO => {
                    // Find oldest entry
                    let mut oldest_key = None;
                    let mut oldest_created = Instant::now();
                    
                    for (k, entry) in storage.iter() {
                        if entry.created_at < oldest_created {
                            oldest_created = entry.created_at;
                            oldest_key = Some(k.clone());
                        }
                    }
                    
                    if let Some(key_to_remove) = oldest_key {
                        if let Some(removed) = storage.remove(&key_to_remove) {
                            stats.total_size -= removed.size;
                            stats.evictions += 1;
                        }
                    }
                }
                EvictionPolicy::Random => {
                    // Remove a random entry
                    if let Some(key_to_remove) = storage.keys().next().cloned() {
                        if let Some(removed) = storage.remove(&key_to_remove) {
                            stats.total_size -= removed.size;
                            stats.evictions += 1;
                        }
                    }
                }
            }
            
            // Check again if we have space
            if storage.len() >= self.config.max_entries || stats.total_size + entry_size > self.config.max_size {
                // Still no space after eviction
                if stats.total_size + entry_size > self.config.max_size {
                    return Err("Cache full, unable to evict enough space".to_string());
                }
                if storage.len() >= self.config.max_entries {
                    return Err("Cache at maximum entry count".to_string());
                }
            }
        }
        
        // Remove old entry if it exists
        if let Some(old_entry) = storage.remove(&key) {
            stats.total_size -= old_entry.size;
        }
        
        // Create new entry
        let entry = CacheEntry {
            key: key.clone(),
            value,
            created_at: Instant::now(),
            accessed_at: Instant::now(),
            access_count: 0,
            ttl: ttl.unwrap_or(self.config.default_ttl),
            size: entry_size,
        };
        
        stats.total_size += entry_size;
        storage.insert(key, entry);
        
        Ok(())
    }

    /// Clear expired entries with complex cleanup logic
    pub fn cleanup(&self) -> usize {
        let mut storage = self.storage.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        let mut removed = 0;
        
        let keys_to_remove: Vec<String> = storage
            .iter()
            .filter(|(_, entry)| {
                let expired = entry.created_at.elapsed() > entry.ttl;
                let stale = entry.accessed_at.elapsed() > Duration::from_secs(7200)
                    && entry.access_count < 5;
                expired || stale
            })
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in keys_to_remove {
            if let Some(entry) = storage.remove(&key) {
                stats.total_size -= entry.size;
                removed += 1;
            }
        }
        
        removed
    }

    /// Get cache statistics with derived metrics
    pub fn get_stats(&self) -> CacheStatsReport {
        let stats = self.stats.lock().unwrap();
        let storage = self.storage.lock().unwrap();
        
        let total_requests = stats.hits + stats.misses;
        let hit_rate = if total_requests > 0 {
            (stats.hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_entry_size = if storage.len() > 0 {
            stats.total_size / storage.len()
        } else {
            0
        };
        
        CacheStatsReport {
            hits: stats.hits,
            misses: stats.misses,
            evictions: stats.evictions,
            total_size: stats.total_size,
            entry_count: storage.len(),
            hit_rate,
            avg_entry_size,
            capacity_used: (stats.total_size as f64 / self.config.max_size as f64) * 100.0,
        }
    }
}

pub struct CacheStatsReport {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_size: usize,
    pub entry_count: usize,
    pub hit_rate: f64,
    pub avg_entry_size: usize,
    pub capacity_used: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = CacheManager::new(1024, 10);
        
        // Test set and get
        cache.set("key1".to_string(), vec![1, 2, 3], None).unwrap();
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        
        // Test miss
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = CacheManager::new(1024, 3);
        
        // Fill cache
        cache.set("key1".to_string(), vec![1; 100], None).unwrap();
        cache.set("key2".to_string(), vec![2; 100], None).unwrap();
        cache.set("key3".to_string(), vec![3; 100], None).unwrap();
        
        // This should trigger eviction
        cache.set("key4".to_string(), vec![4; 100], None).unwrap();
        
        let stats = cache.get_stats();
        assert_eq!(stats.evictions, 1);
        assert_eq!(stats.entry_count, 3);
    }
}

fn main() {
    println!("TDG Demo: Complex cache implementation with high technical debt");
    
    let cache = CacheManager::new(1024 * 1024, 1000);
    
    // Simulate usage
    for i in 0..100 {
        let key = format!("item_{}", i);
        let value = vec![i as u8; 100];
        cache.set(key.clone(), value, None).unwrap();
        
        // Access some items multiple times
        if i % 3 == 0 {
            for _ in 0..5 {
                cache.get(&key);
            }
        }
    }
    
    let stats = cache.get_stats();
    println!("Cache Statistics:");
    println!("  Hit Rate: {:.2}%", stats.hit_rate);
    println!("  Capacity Used: {:.2}%", stats.capacity_used);
    println!("  Evictions: {}", stats.evictions);
}