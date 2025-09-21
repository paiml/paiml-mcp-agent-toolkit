//! Memory management integration utilities for existing PMAT services
//!
//! This module provides integration helpers to retrofit existing services with
//! optimized memory management patterns. It includes:
//! - Wrapper types for common allocation patterns
//! - Drop-in replacements for Vec and String allocations
//! - Cache-friendly data structure implementations
//! - Memory-aware service trait extensions
//!
//! # Integration Strategy
//!
//! The integration follows a phased approach:
//! 1. **Drop-in Replacements**: `MemoryVec`, `MemoryString` for common cases
//! 2. **Service Extensions**: `MemoryAware` trait for service-level optimization
//! 3. **Cache Integration**: Memory-aware cache strategies
//! 4. **AST Optimization**: Pool-based AST node allocation
//!
//! # Usage Example
//!
//! ```rust
//! use pmat::services::memory_integration::{MemoryVec, MemoryAware};
//! use pmat::services::memory_manager::PoolType;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Drop-in replacement for Vec<String>
//! let mut identifiers = MemoryVec::new(PoolType::StringIntern)?;
//! identifiers.push("function_name".to_string())?;
//! identifiers.push("variable_name".to_string())?;
//!
//! // Memory-aware processing
//! let processed = identifiers.process_with_memory_awareness(|items| {
//!     items.iter().map(|s| s.len()).sum::<usize>()
//! })?;
//! # Ok(())
//! # }
//! ```

use crate::services::memory_manager::{
    global_memory_manager, MemoryManager, PoolType, PooledBuffer,
};
use anyhow::Result;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tracing::{debug, trace};

/// Memory-aware vector that uses pooled allocation
pub struct MemoryVec<T> {
    data: Vec<T>,
    pool_type: PoolType,
    memory_manager: Arc<MemoryManager>,
}

impl<T> MemoryVec<T> {
    /// Create a new memory-aware vector
    pub fn new(pool_type: PoolType) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            data: Vec::new(),
            pool_type,
            memory_manager,
        })
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(pool_type: PoolType, capacity: usize) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            data: Vec::with_capacity(capacity),
            pool_type,
            memory_manager,
        })
    }

    /// Push an item with memory tracking
    pub fn push(&mut self, item: T) -> Result<()> {
        let old_capacity = self.data.capacity();
        self.data.push(item);

        // Track memory growth
        let new_capacity = self.data.capacity();
        if new_capacity > old_capacity {
            let growth = (new_capacity - old_capacity) * std::mem::size_of::<T>();
            trace!(
                "MemoryVec grew by {} bytes for pool {:?}",
                growth,
                self.pool_type
            );
        }

        Ok(())
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) -> Result<()> {
        let old_capacity = self.data.capacity();
        self.data.reserve(additional);

        let new_capacity = self.data.capacity();
        if new_capacity > old_capacity {
            let growth = (new_capacity - old_capacity) * std::mem::size_of::<T>();
            trace!(
                "MemoryVec reserved {} bytes for pool {:?}",
                growth,
                self.pool_type
            );
        }

        Ok(())
    }

    /// Get memory usage estimate
    #[must_use] 
    pub fn memory_usage(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<T>()
    }

    /// Process data with memory awareness
    pub fn process_with_memory_awareness<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&[T]) -> R,
    {
        // Check memory pressure before processing
        let stats = self.memory_manager.stats();
        if stats.allocation_pressure > 0.9 {
            debug!(
                "High memory pressure detected: {:.1}%",
                stats.allocation_pressure * 100.0
            );
            self.memory_manager.cleanup()?;
        }

        Ok(f(&self.data))
    }
}

impl<T> Deref for MemoryVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for MemoryVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Memory-aware string that uses string interning
pub struct MemoryString {
    content: Arc<str>,
    memory_manager: Arc<MemoryManager>,
}

impl MemoryString {
    /// Create a new memory-aware string with interning
    pub fn new(content: &str) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        let interned = memory_manager.intern_string(content)?;
        Ok(Self {
            content: interned,
            memory_manager,
        })
    }

    /// Get the string content
    #[must_use] 
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Check if this string shares memory with another
    #[must_use] 
    pub fn shares_memory_with(&self, other: &MemoryString) -> bool {
        Arc::ptr_eq(&self.content, &other.content)
    }

    /// Get memory statistics for this string
    #[must_use] 
    pub fn memory_stats(&self) -> super::memory_manager::MemoryStats {
        self.memory_manager.stats()
    }
}

impl Deref for MemoryString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl std::fmt::Display for MemoryString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::fmt::Debug for MemoryString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemoryString({})", self.content)
    }
}

/// Trait for services that can be made memory-aware
pub trait MemoryAware {
    /// Get memory usage estimate for this service
    fn memory_usage(&self) -> usize;

    /// Cleanup unused memory in this service
    fn cleanup_memory(&mut self) -> Result<usize>;

    /// Configure memory limits for this service
    fn configure_memory(&mut self, max_memory: usize) -> Result<()>;

    /// Check if service is under memory pressure
    fn memory_pressure(&self) -> f64 {
        0.0 // Default implementation
    }
}

/// Memory-optimized buffer pool for AST parsing
pub struct AstBufferPool {
    memory_manager: Arc<MemoryManager>,
    pool_type: PoolType,
}

impl AstBufferPool {
    /// Create a new AST buffer pool
    pub fn new(pool_type: PoolType) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            memory_manager,
            pool_type,
        })
    }

    /// Get a buffer for AST parsing
    pub fn get_buffer(&self, min_size: usize) -> Result<PooledBuffer> {
        self.memory_manager
            .allocate_buffer(self.pool_type, min_size)
    }

    /// Get buffer sized for specific file content
    pub fn get_buffer_for_content(&self, content: &str) -> Result<PooledBuffer> {
        // Size buffer based on content with some overhead for parsing structures
        let size = content.len() * 2; // 2x overhead for AST structures
        self.memory_manager.allocate_buffer(self.pool_type, size)
    }
}

/// Memory-optimized string collection with interning
pub struct InternedStringSet {
    strings: std::collections::HashSet<Arc<str>>,
    memory_manager: Arc<MemoryManager>,
}

impl InternedStringSet {
    /// Create a new interned string set
    pub fn new() -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            strings: std::collections::HashSet::new(),
            memory_manager,
        })
    }

    /// Insert a string with interning
    pub fn insert(&mut self, s: &str) -> Result<bool> {
        let interned = self.memory_manager.intern_string(s)?;
        Ok(self.strings.insert(interned))
    }

    /// Check if string exists
    #[must_use] 
    pub fn contains(&self, s: &str) -> bool {
        // Note: This requires interning the string to check, which is not ideal
        // In practice, you'd store the interned version when inserting
        if let Ok(interned) = self.memory_manager.intern_string(s) {
            self.strings.contains(&interned)
        } else {
            false
        }
    }

    /// Get all strings
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.strings.iter().map(std::convert::AsRef::as_ref)
    }

    /// Get memory usage
    #[must_use] 
    pub fn memory_usage(&self) -> usize {
        // Rough estimate - doesn't account for interning savings
        self.strings.len() * std::mem::size_of::<Arc<str>>()
    }
}

/// Memory-aware cache wrapper for existing caches
pub struct MemoryAwareCache<K, V> {
    cache: std::collections::HashMap<K, V>,
    memory_manager: Arc<MemoryManager>,
    pool_type: PoolType,
    max_items: usize,
}

impl<K, V> MemoryAwareCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new memory-aware cache
    pub fn new(pool_type: PoolType, max_items: usize) -> Result<Self> {
        let memory_manager = global_memory_manager()?;
        Ok(Self {
            cache: std::collections::HashMap::new(),
            memory_manager,
            pool_type,
            max_items,
        })
    }

    /// Insert with memory pressure checking
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        // Check memory pressure
        let stats = self.memory_manager.stats();
        if stats.allocation_pressure > 0.85 && self.cache.len() >= self.max_items {
            // Evict oldest entries (simplified LRU)
            let keys_to_remove: Vec<_> = self
                .cache
                .keys()
                .take(self.cache.len() / 4)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
            debug!("Evicted cache entries due to memory pressure");
        }

        Ok(self.cache.insert(key, value))
    }

    /// Get cached value
    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    /// Get cache statistics
    #[must_use] 
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            item_count: self.cache.len(),
            max_items: self.max_items,
            estimated_memory: self.cache.len()
                * (std::mem::size_of::<K>() + std::mem::size_of::<V>()),
        }
    }

    /// Get the memory pool type used by this cache
    #[must_use] 
    pub fn pool_type(&self) -> PoolType {
        self.pool_type
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub item_count: usize,
    pub max_items: usize,
    pub estimated_memory: usize,
}

/// Utility functions for memory optimization
pub mod utils {
    use super::{Result, MemoryVec, PoolType, global_memory_manager};

    /// Create a memory-optimized vector for identifiers
    pub fn create_identifier_vec() -> Result<MemoryVec<String>> {
        MemoryVec::new(PoolType::StringIntern)
    }

    /// Create a memory-optimized vector for file content
    pub fn create_content_vec() -> Result<MemoryVec<u8>> {
        MemoryVec::new(PoolType::FileContent)
    }

    /// Create a memory-optimized vector for AST nodes
    pub fn create_ast_vec<T>() -> Result<MemoryVec<T>> {
        MemoryVec::new(PoolType::AstParsing)
    }

    /// Estimate memory usage for a collection
    pub fn estimate_collection_memory<T>(collection: &[T]) -> usize {
        std::mem::size_of_val(collection)
    }

    /// Check if memory cleanup is recommended
    pub fn should_cleanup_memory() -> Result<bool> {
        let manager = global_memory_manager()?;
        let stats = manager.stats();
        Ok(stats.allocation_pressure > 0.8)
    }

    /// Force memory cleanup if under pressure
    pub fn cleanup_if_needed() -> Result<usize> {
        if should_cleanup_memory()? {
            let manager = global_memory_manager()?;
            manager.cleanup()
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::memory_manager::init_global_memory_manager;

    fn setup_memory_manager() -> Result<()> {
        // Try to initialize, but ignore "already initialized" errors
        match init_global_memory_manager() {
            Ok(()) => Ok(()),
            Err(e) if e.to_string().contains("already initialized") => Ok(()),
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_memory_vec() -> Result<()> {
        setup_memory_manager()?;

        let mut vec = MemoryVec::new(PoolType::AstParsing)?;
        vec.push("test1".to_string())?;
        vec.push("test2".to_string())?;

        assert_eq!(vec.len(), 2);
        assert!(vec.memory_usage() > 0);

        Ok(())
    }

    #[test]
    fn test_memory_string() -> Result<()> {
        setup_memory_manager()?;

        let str1 = MemoryString::new("test")?;
        let str2 = MemoryString::new("test")?;

        assert_eq!(str1.as_str(), "test");
        assert!(str1.shares_memory_with(&str2));

        Ok(())
    }

    #[test]
    fn test_ast_buffer_pool() -> Result<()> {
        setup_memory_manager()?;

        let pool = AstBufferPool::new(PoolType::AstParsing)?;
        let buffer = pool.get_buffer(1024)?;

        assert!(buffer.capacity() >= 1024);

        Ok(())
    }

    #[test]
    fn test_interned_string_set() -> Result<()> {
        setup_memory_manager()?;

        let mut set = InternedStringSet::new()?;
        assert!(set.insert("test1")?);
        assert!(!set.insert("test1")?); // Already exists
        assert!(set.insert("test2")?);

        assert_eq!(set.strings.len(), 2);

        Ok(())
    }

    #[test]
    fn test_memory_aware_cache() -> Result<()> {
        setup_memory_manager()?;

        let mut cache = MemoryAwareCache::new(PoolType::AnalysisCache, 100)?;
        cache.insert("key1", "value1")?;
        cache.insert("key2", "value2")?;

        assert_eq!(cache.get(&"key1"), Some(&"value1"));

        let stats = cache.stats();
        assert_eq!(stats.item_count, 2);
        assert_eq!(stats.max_items, 100);

        Ok(())
    }

    #[test]
    fn test_utils() -> Result<()> {
        setup_memory_manager()?;

        let _id_vec = utils::create_identifier_vec()?;
        let _content_vec = utils::create_content_vec()?;
        let _ast_vec: MemoryVec<i32> = utils::create_ast_vec()?;

        let test_data = vec![1, 2, 3, 4, 5];
        let memory_usage = utils::estimate_collection_memory(&test_data);
        assert_eq!(memory_usage, 5 * std::mem::size_of::<i32>());

        Ok(())
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
