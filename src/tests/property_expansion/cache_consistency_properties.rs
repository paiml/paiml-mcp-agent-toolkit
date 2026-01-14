//! Property-based tests for cache consistency
//! 
//! This module verifies that the content-addressed cache maintains
//! consistency across all operations and hierarchical levels.

use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache categories for different types of data
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CacheCategory {
    Ast,
    Complexity,
    DeadCode,
    Satd,
    Refactoring,
}

/// Content-addressed cache key
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub category: CacheCategory,
    pub content_hash: u64,
    pub version: u32,
}

impl CacheKey {
    /// Create a cache key from content
    pub fn from_content(category: CacheCategory, content: &[u8]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        
        Self {
            category,
            content_hash: hasher.finish(),
            version: 1,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: CacheKey,
    pub value: Vec<u8>,
    pub access_count: u32,
    pub last_access: std::time::SystemTime,
}

/// Cache operation for testing
#[derive(Debug, Clone)]
pub enum CacheOp {
    Put(CacheKey, Vec<u8>),
    Get(CacheKey),
    Evict(CacheKey),
    Clear(CacheCategory),
}

/// Hierarchical cache with L1, L2, and L3 levels
pub struct CacheHierarchy {
    pub l1: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,  // Hot cache
    pub l2: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,  // Warm cache
    pub l3: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,  // Cold cache
    pub stats: Arc<RwLock<CacheStats>>,
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub puts: u64,
}

impl CacheHierarchy {
    /// Create a new cache hierarchy
    pub fn new() -> Self {
        Self {
            l1: Arc::new(RwLock::new(HashMap::new())),
            l2: Arc::new(RwLock::new(HashMap::new())),
            l3: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// Put a value in the cache
    pub async fn put(&self, key: &CacheKey, value: Vec<u8>) -> Result<(), String> {
        let entry = CacheEntry {
            key: key.clone(),
            value: value.clone(),
            access_count: 0,
            last_access: std::time::SystemTime::now(),
        };
        
        // Always put in L1 (hot cache)
        let mut l1 = self.l1.write().await;
        l1.insert(key.clone(), entry.clone());
        
        // Also put in L3 for persistence
        let mut l3 = self.l3.write().await;
        l3.insert(key.clone(), entry);
        
        let mut stats = self.stats.write().await;
        stats.puts += 1;
        
        Ok(())
    }
    
    /// Get a value from the cache
    pub async fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        // Check L1 first
        {
            let mut l1 = self.l1.write().await;
            if let Some(entry) = l1.get_mut(key) {
                entry.access_count += 1;
                entry.last_access = std::time::SystemTime::now();
                
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                
                return Some(entry.value.clone());
            }
        }
        
        // Check L2
        {
            let mut l2 = self.l2.write().await;
            if let Some(entry) = l2.get_mut(key) {
                entry.access_count += 1;
                entry.last_access = std::time::SystemTime::now();
                
                // Promote to L1
                let mut l1 = self.l1.write().await;
                l1.insert(key.clone(), entry.clone());
                
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                
                return Some(entry.value.clone());
            }
        }
        
        // Check L3
        {
            let mut l3 = self.l3.write().await;
            if let Some(entry) = l3.get_mut(key) {
                entry.access_count += 1;
                entry.last_access = std::time::SystemTime::now();
                
                // Promote to L1
                let mut l1 = self.l1.write().await;
                l1.insert(key.clone(), entry.clone());
                
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                
                return Some(entry.value.clone());
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        
        None
    }
    
    /// Evict an entry from L1 cache
    pub async fn evict(&self, key: &CacheKey) {
        let mut l1 = self.l1.write().await;
        if let Some(entry) = l1.remove(key) {
            // Move to L2
            let mut l2 = self.l2.write().await;
            l2.insert(key.clone(), entry);
            
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
        }
    }
    
    /// Check if a key has been stored
    pub async fn has_been_stored(&self, key: &CacheKey) -> bool {
        let l1 = self.l1.read().await;
        let l2 = self.l2.read().await;
        let l3 = self.l3.read().await;
        
        l1.contains_key(key) || l2.contains_key(key) || l3.contains_key(key)
    }
    
    /// Clear all entries of a specific category
    pub async fn clear_category(&self, category: CacheCategory) {
        let mut l1 = self.l1.write().await;
        let mut l2 = self.l2.write().await;
        let mut l3 = self.l3.write().await;
        
        l1.retain(|k, _| k.category != category);
        l2.retain(|k, _| k.category != category);
        l3.retain(|k, _| k.category != category);
    }
}

/// Cache metadata for testing
#[derive(Debug, Clone)]
pub struct CacheMetadata {
    pub ttl_seconds: u64,
    pub max_size_bytes: usize,
    pub eviction_policy: String,
}

// Arbitrary implementations for property testing
impl Arbitrary for CacheCategory {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(CacheCategory::Ast),
            Just(CacheCategory::Complexity),
            Just(CacheCategory::DeadCode),
            Just(CacheCategory::Satd),
            Just(CacheCategory::Refactoring),
        ].boxed()
    }
}

impl Arbitrary for CacheOp {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            (any::<CacheCategory>(), prop::collection::vec(any::<u8>(), 1..1000))
                .prop_map(|(cat, content)| {
                    let key = CacheKey::from_content(cat, &content);
                    CacheOp::Put(key, content)
                }),
            (any::<CacheCategory>(), prop::collection::vec(any::<u8>(), 1..1000))
                .prop_map(|(cat, content)| {
                    let key = CacheKey::from_content(cat, &content);
                    CacheOp::Get(key)
                }),
            (any::<CacheCategory>(), prop::collection::vec(any::<u8>(), 1..1000))
                .prop_map(|(cat, content)| {
                    let key = CacheKey::from_content(cat, &content);
                    CacheOp::Evict(key)
                }),
            any::<CacheCategory>().prop_map(CacheOp::Clear),
        ].boxed()
    }
}

impl Arbitrary for CacheMetadata {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            60u64..3600,  // TTL between 1 minute and 1 hour
            1024usize..1048576,  // Size between 1KB and 1MB
            prop::sample::select(vec!["LRU", "LFU", "FIFO"]),
        ).prop_map(|(ttl_seconds, max_size_bytes, eviction_policy)| {
            CacheMetadata {
                ttl_seconds,
                max_size_bytes,
                eviction_policy,
            }
        }).boxed()
    }
}

proptest! {
    /// Property: Identical content produces identical keys
    #[test]
    fn cache_key_determinism(
        content in prop::collection::vec(any::<u8>(), 1..10_000),
        category in any::<CacheCategory>()
    ) {
        let key1 = CacheKey::from_content(category, &content);
        let key2 = CacheKey::from_content(category, &content);
        
        // Property: Identical content produces identical keys
        prop_assert_eq!(key1.content_hash, key2.content_hash);
        prop_assert_eq!(key1.category, key2.category);
    }
    
    /// Property: Different content produces different keys
    #[test]
    fn cache_key_collision_resistance(
        content in prop::collection::vec(any::<u8>(), 1..10_000),
        category in any::<CacheCategory>()
    ) {
        let key1 = CacheKey::from_content(category, &content);
        
        // Modify content slightly
        let mut modified = content.clone();
        if let Some(byte) = modified.get_mut(0) {
            *byte = byte.wrapping_add(1);
        } else {
            modified.push(1);
        }
        
        let key2 = CacheKey::from_content(category, &modified);
        
        // Property: Different content produces different keys
        prop_assert_ne!(
            key1.content_hash, key2.content_hash,
            "Hash collision detected for different content"
        );
    }
    
    /// Property: Put followed by get returns same value
    #[tokio::test]
    async fn cache_put_get_consistency(
        content in prop::collection::vec(any::<u8>(), 1..1000),
        category in any::<CacheCategory>()
    ) {
        let cache = CacheHierarchy::new();
        let key = CacheKey::from_content(category, &content);
        
        // Put value
        cache.put(&key, content.clone()).await.unwrap();
        
        // Get should return same value
        let retrieved = cache.get(&key).await;
        prop_assert_eq!(Some(content), retrieved);
    }
    
    /// Property: Multiple puts of same key keep latest value
    #[tokio::test]
    async fn cache_put_overwrites(
        contents in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..100),
            2..5
        ),
        category in any::<CacheCategory>()
    ) {
        let cache = CacheHierarchy::new();
        
        // Use same key for all contents
        let key = CacheKey::from_content(category, b"fixed_key");
        
        // Put all values
        for content in &contents {
            cache.put(&key, content.clone()).await.unwrap();
        }
        
        // Should get the last value
        let retrieved = cache.get(&key).await;
        prop_assert_eq!(contents.last().cloned(), retrieved);
    }
    
    /// Property: Eviction moves from L1 to L2, not deleting
    #[tokio::test]
    async fn cache_eviction_preserves_data(
        content in prop::collection::vec(any::<u8>(), 1..1000),
        category in any::<CacheCategory>()
    ) {
        let cache = CacheHierarchy::new();
        let key = CacheKey::from_content(category, &content);
        
        // Put value
        cache.put(&key, content.clone()).await.unwrap();
        
        // Evict from L1
        cache.evict(&key).await;
        
        // Should still be accessible (from L2 or L3)
        let retrieved = cache.get(&key).await;
        prop_assert_eq!(Some(content), retrieved);
    }
    
    /// Property: Clear category removes all entries of that category
    #[tokio::test]
    async fn cache_clear_category_selective(
        contents in prop::collection::vec(
            (any::<CacheCategory>(), prop::collection::vec(any::<u8>(), 1..100)),
            5..10
        )
    ) {
        let cache = CacheHierarchy::new();
        let mut keys_by_category: HashMap<CacheCategory, Vec<CacheKey>> = HashMap::new();
        
        // Put all values
        for (category, content) in &contents {
            let key = CacheKey::from_content(*category, content);
            cache.put(&key, content.clone()).await.unwrap();
            keys_by_category.entry(*category).or_default().push(key);
        }
        
        // Pick a category to clear
        if let Some((&category_to_clear, _)) = keys_by_category.iter().next() {
            cache.clear_category(category_to_clear).await;
            
            // Check all keys
            for (&cat, keys) in &keys_by_category {
                for key in keys {
                    let retrieved = cache.get(key).await;
                    if cat == category_to_clear {
                        prop_assert!(retrieved.is_none(), "Category should be cleared");
                    } else {
                        prop_assert!(retrieved.is_some(), "Other categories should remain");
                    }
                }
            }
        }
    }
    
    /// Property: Cache statistics are accurate
    #[tokio::test]
    async fn cache_statistics_accuracy(
        operations in prop::collection::vec(any::<CacheOp>(), 1..50)
    ) {
        let cache = CacheHierarchy::new();
        let mut expected_puts = 0u64;
        let mut stored_keys = std::collections::HashSet::new();
        
        for op in operations {
            match op {
                CacheOp::Put(key, value) => {
                    cache.put(&key, value).await.unwrap();
                    expected_puts += 1;
                    stored_keys.insert(key);
                }
                CacheOp::Get(key) => {
                    let _ = cache.get(&key).await;
                }
                CacheOp::Evict(key) => {
                    cache.evict(&key).await;
                }
                CacheOp::Clear(category) => {
                    cache.clear_category(category).await;
                    stored_keys.retain(|k| k.category != category);
                }
            }
        }
        
        let stats = cache.stats.read().await;
        prop_assert_eq!(stats.puts, expected_puts, "Put count mismatch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = CacheHierarchy::new();
        let key = CacheKey::from_content(CacheCategory::Ast, b"test");
        let value = vec![1, 2, 3, 4, 5];
        
        // Put and get
        cache.put(&key, value.clone()).await.unwrap();
        let retrieved = cache.get(&key).await;
        assert_eq!(Some(value), retrieved);
        
        // Check stats
        let stats = cache.stats.read().await;
        assert_eq!(stats.puts, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }
    
    #[test]
    fn test_cache_key_generation() {
        let content1 = b"hello world";
        let content2 = b"hello world!";
        
        let key1 = CacheKey::from_content(CacheCategory::Ast, content1);
        let key2 = CacheKey::from_content(CacheCategory::Ast, content2);
        
        // Different content should produce different hashes
        assert_ne!(key1.content_hash, key2.content_hash);
        
        // Same content should produce same hash
        let key3 = CacheKey::from_content(CacheCategory::Ast, content1);
        assert_eq!(key1.content_hash, key3.content_hash);
    }
}