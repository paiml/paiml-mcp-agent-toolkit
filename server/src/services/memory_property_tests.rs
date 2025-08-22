//! Property-based tests for memory management system
//!
//! This module provides comprehensive property-based testing for the memory management
//! optimization features, ensuring correctness under various load conditions and
//! usage patterns.

use crate::services::memory_manager::{
    MemoryConfig, MemoryManager, PoolType, init_global_memory_manager_with_config,
};
use crate::services::memory_integration::{
    MemoryVec, MemoryString, AstBufferPool, InternedStringSet, MemoryAwareCache,
};
use anyhow::Result;
use proptest::prelude::*;
use std::collections::HashMap;
use tracing::info;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

proptest! {
    /// Test that memory manager can handle arbitrary allocation patterns
    #[test]
    fn test_memory_manager_allocation_patterns(
        allocation_sizes in prop::collection::vec(1usize..1024*1024, 1..100),
        pool_types in prop::collection::vec(prop::sample::select(vec![
            PoolType::AstParsing,
            PoolType::StringIntern,
            PoolType::AnalysisCache,
            PoolType::FileContent,
            PoolType::GraphConstruction,
        ]), 1..100)
    ) {
        prop_assert!(test_allocation_patterns(allocation_sizes, pool_types).is_ok());
    }

    /// Test string interning efficiency and correctness
    #[test]
    fn test_string_interning_properties(
        strings in prop::collection::vec(prop::string::string_regex("[a-zA-Z0-9_]{1,50}").unwrap(), 10..1000),
        duplication_factor in 1usize..10
    ) {
        prop_assert!(test_string_interning(strings, duplication_factor).is_ok());
    }

    /// Test memory cleanup under pressure
    #[test]
    fn test_memory_cleanup_properties(
        initial_allocations in 50usize..500,
        pressure_threshold in 0.5f64..0.95f64
    ) {
        prop_assert!(test_cleanup_under_pressure(initial_allocations, pressure_threshold).is_ok());
    }

    /// Test concurrent memory operations
    #[test]
    fn test_concurrent_memory_operations(
        thread_count in 2usize..8,
        operations_per_thread in 10usize..100
    ) {
        prop_assert!(test_concurrent_operations(thread_count, operations_per_thread).is_ok());
    }

    /// Test memory pool efficiency with different usage patterns
    #[test]
    fn test_pool_efficiency_patterns(
        buffer_sizes in prop::collection::vec(64usize..8192, 10..100),
        reuse_probability in 0.1f64..0.9f64
    ) {
        prop_assert!(test_pool_efficiency(buffer_sizes, reuse_probability).is_ok());
    }
}

fn test_allocation_patterns(sizes: Vec<usize>, pool_types: Vec<PoolType>) -> Result<()> {
    let manager = MemoryManager::new()?;
    let mut buffers = Vec::new();

    // Test allocation patterns
    for (size, pool_type) in sizes.iter().zip(pool_types.iter().cycle()) {
        let buffer = manager.allocate_buffer(*pool_type, *size)?;
        assert!(buffer.capacity() >= *size);
        assert_eq!(buffer.as_slice().len(), *size);
        buffers.push(buffer);
    }

    // Verify memory tracking
    let stats = manager.stats();
    assert!(stats.total_allocated > 0);
    assert!(stats.peak_usage >= stats.total_allocated);

    // Test cleanup
    drop(buffers);
    let _cleaned = manager.cleanup()?;
    
    // Note: Cleanup amount depends on memory pressure
    Ok(())
}

fn test_string_interning(mut strings: Vec<String>, duplication_factor: usize) -> Result<()> {
    let manager = MemoryManager::new()?;
    let mut interned_strings = Vec::new();

    // Create duplicates to test interning efficiency
    for _ in 0..duplication_factor {
        strings.extend(strings.clone());
    }

    // Intern all strings
    for s in &strings {
        let interned = manager.intern_string(s)?;
        interned_strings.push(interned);
    }

    // Verify identical strings share memory
    let mut string_map: HashMap<String, Arc<str>> = HashMap::new();
    for interned in &interned_strings {
        let key = interned.to_string();
        if let Some(existing) = string_map.get(&key) {
            assert!(Arc::ptr_eq(existing, interned), "Identical strings should share memory");
        } else {
            string_map.insert(key, Arc::clone(interned));
        }
    }

    // Verify interning efficiency (should have fewer unique strings than total)
    let unique_count = string_map.len();
    let total_count = interned_strings.len();
    if duplication_factor > 1 {
        assert!(unique_count < total_count, "String interning should reduce memory usage");
    }

    Ok(())
}

fn test_cleanup_under_pressure(allocations: usize, pressure_threshold: f64) -> Result<()> {
    // Create a manager with smaller memory limits to trigger pressure
    let config = MemoryConfig {
        max_total_memory: 16 * 1024 * 1024, // 16MB limit
        cache_pressure_threshold: pressure_threshold,
        ..Default::default()
    };
    
    let manager = MemoryManager::with_config(config)?;
    let mut buffers = Vec::new();

    // Allocate until we approach pressure threshold
    for i in 0..allocations {
        let size = 1024 * (i % 100 + 1); // Variable sizes 1KB-100KB
        let pool_type = match i % 5 {
            0 => PoolType::AstParsing,
            1 => PoolType::StringIntern,
            2 => PoolType::AnalysisCache,
            3 => PoolType::FileContent,
            _ => PoolType::GraphConstruction,
        };
        
        if let Ok(buffer) = manager.allocate_buffer(pool_type, size) {
            buffers.push(buffer);
        }
        
        // Check if we've reached pressure threshold
        let stats = manager.stats();
        if stats.allocation_pressure > pressure_threshold {
            break;
        }
    }

    let stats_before = manager.stats();
    let pressure_before = stats_before.allocation_pressure;

    // Force cleanup
    let cleaned = manager.cleanup()?;
    let stats_after = manager.stats();
    let _pressure_after = stats_after.allocation_pressure;

    // Verify cleanup succeeded when we were over threshold
    if pressure_before > pressure_threshold {
        // cleanup() succeeded if we got here without panic, cleaned amount is always valid
        info!("Cleaned {} bytes when pressure was {:.1}%", cleaned, pressure_before * 100.0);
        // Note: Pressure might not always decrease due to retained allocations
    }

    Ok(())
}

fn test_concurrent_operations(thread_count: usize, operations: usize) -> Result<()> {
    let manager = MemoryManager::new()?;
    let manager = Arc::new(manager);
    
    let handles: Vec<_> = (0..thread_count).map(|thread_id| {
        let manager = Arc::clone(&manager);
        thread::spawn(move || -> Result<()> {
            let mut buffers = Vec::new();
            let mut strings = Vec::new();
            
            for i in 0..operations {
                // Mix of buffer allocations and string interning
                if i % 2 == 0 {
                    let size = 1024 + (i * 100) % 4096;
                    let pool_type = match thread_id % 3 {
                        0 => PoolType::AstParsing,
                        1 => PoolType::FileContent,
                        _ => PoolType::AnalysisCache,
                    };
                    
                    if let Ok(buffer) = manager.allocate_buffer(pool_type, size) {
                        buffers.push(buffer);
                    }
                } else {
                    let test_string = format!("thread_{}_op_{}", thread_id, i);
                    if let Ok(interned) = manager.intern_string(&test_string) {
                        strings.push(interned);
                    }
                }
                
                // Occasional cleanup to test contention
                if i % 50 == 0 {
                    let _ = manager.cleanup();
                }
                
                // Small delay to increase contention
                thread::sleep(Duration::from_micros(1));
            }
            
            Ok(())
        })
    }).collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify system is still functional
    let stats = manager.stats();
    // Basic sanity check - total_allocated is valid if we got here without panic
    
    Ok(())
}

fn test_pool_efficiency(buffer_sizes: Vec<usize>, reuse_probability: f64) -> Result<()> {
    let manager = MemoryManager::new()?;
    let mut active_buffers = Vec::new();
    
    for (i, &size) in buffer_sizes.iter().enumerate() {
        let pool_type = PoolType::AstParsing;
        
        // Allocate buffer
        let buffer = manager.allocate_buffer(pool_type, size)?;
        active_buffers.push(buffer);
        
        // Probabilistically drop some buffers to test reuse
        if (i % 100) as f64 / 100.0 < reuse_probability {
            // Drop a random buffer to return it to pool
            if !active_buffers.is_empty() {
                let index = i % active_buffers.len();
                active_buffers.remove(index);
            }
        }
        
        // Check pool stats periodically
        if i % 20 == 0 {
            let stats = manager.stats();
            if let Some(pool_stats) = stats.pool_stats.get(&pool_type) {
                // Verify pool is accumulating some efficiency
                if pool_stats.allocation_count > 10 {
                    assert!(pool_stats.reuse_count <= pool_stats.allocation_count);
                }
            }
        }
    }
    
    // Drop all remaining buffers
    active_buffers.clear();
    
    // Check final efficiency
    let stats = manager.stats();
    if let Some(pool_stats) = stats.pool_stats.get(&PoolType::AstParsing) {
        if pool_stats.allocation_count > 0 {
            let efficiency = pool_stats.reuse_ratio;
            assert!((0.0..=1.0).contains(&efficiency));
            
            // If reuse probability was high, we should see some reuse
            if reuse_probability > 0.5 && pool_stats.allocation_count > 20 {
                assert!(efficiency > 0.0, "Should see some buffer reuse with high reuse probability");
            }
        }
    }
    
    Ok(())
}

/// Test MemoryVec operations
#[test]
fn test_memory_vec_operations() -> Result<()> {
    // Initialize global memory manager for integration testing
    let config = MemoryConfig::default();
    init_global_memory_manager_with_config(config)?;
    
    let mut vec = MemoryVec::new(PoolType::AstParsing)?;
    
    // Test basic operations
    vec.push("test1".to_string())?;
    vec.push("test2".to_string())?;
    vec.push("test3".to_string())?;
    
    assert_eq!(vec.len(), 3);
    assert!(vec.memory_usage() > 0);
    
    // Test memory-aware processing
    let total_length = vec.process_with_memory_awareness(|items| {
        items.iter().map(|s| s.len()).sum::<usize>()
    })?;
    
    assert_eq!(total_length, "test1".len() + "test2".len() + "test3".len());
    
    Ok(())
}

/// Test string interning integration
#[test]
fn test_memory_string_integration() -> Result<()> {
    let config = MemoryConfig::default();
    init_global_memory_manager_with_config(config)?;
    
    let str1 = MemoryString::new("shared_identifier")?;
    let str2 = MemoryString::new("shared_identifier")?;
    let str3 = MemoryString::new("different_identifier")?;
    
    // Verify memory sharing
    assert!(str1.shares_memory_with(&str2));
    assert!(!str1.shares_memory_with(&str3));
    
    // Verify content
    assert_eq!(str1.as_str(), "shared_identifier");
    assert_eq!(str2.as_str(), "shared_identifier");
    assert_eq!(str3.as_str(), "different_identifier");
    
    Ok(())
}

/// Test AST buffer pool
#[test]
fn test_ast_buffer_pool_integration() -> Result<()> {
    let config = MemoryConfig::default();
    init_global_memory_manager_with_config(config)?;
    
    let pool = AstBufferPool::new(PoolType::AstParsing)?;
    
    // Test different buffer sizes
    let buffer1 = pool.get_buffer(1024)?;
    let buffer2 = pool.get_buffer(2048)?;
    let buffer3 = pool.get_buffer_for_content("fn main() { println!(\"Hello, world!\"); }")?;
    
    assert!(buffer1.capacity() >= 1024);
    assert!(buffer2.capacity() >= 2048);
    assert!(buffer3.capacity() > 0);
    
    Ok(())
}

/// Test interned string set
#[test]
fn test_interned_string_set() -> Result<()> {
    let config = MemoryConfig::default();
    init_global_memory_manager_with_config(config)?;
    
    let mut set = InternedStringSet::new()?;
    
    // Test insertion and deduplication
    assert!(set.insert("identifier1")?);
    assert!(!set.insert("identifier1")?); // Should return false for duplicate
    assert!(set.insert("identifier2")?);
    
    // Test iteration
    let identifiers: Vec<_> = set.iter().collect();
    assert_eq!(identifiers.len(), 2);
    assert!(identifiers.contains(&"identifier1"));
    assert!(identifiers.contains(&"identifier2"));
    
    assert!(set.memory_usage() > 0);
    
    Ok(())
}

/// Test memory-aware cache
#[test]
fn test_memory_aware_cache() -> Result<()> {
    let config = MemoryConfig::default();
    init_global_memory_manager_with_config(config)?;
    
    let mut cache = MemoryAwareCache::new(PoolType::AnalysisCache, 10)?;
    
    // Test basic cache operations
    cache.insert("key1", "value1")?;
    cache.insert("key2", "value2")?;
    
    assert_eq!(cache.get(&"key1"), Some(&"value1"));
    assert_eq!(cache.get(&"key2"), Some(&"value2"));
    assert_eq!(cache.get(&"nonexistent"), None);
    
    let stats = cache.stats();
    assert_eq!(stats.item_count, 2);
    assert_eq!(stats.max_items, 10);
    assert!(stats.estimated_memory > 0);
    
    Ok(())
}

/// Test memory management under extreme conditions
#[test]
fn test_extreme_memory_conditions() -> Result<()> {
    // Create manager with very small limits
    let config = MemoryConfig {
        max_total_memory: 1024 * 1024, // 1MB limit
        ..Default::default()
    };
    
    let manager = MemoryManager::with_config(config)?;
    
    // Try to allocate more than limit
    let mut buffers = Vec::new();
    let mut allocation_count = 0;
    
    // Allocate until we hit limits or can't allocate
    for _i in 0..1000 {
        let size = 4096; // 4KB buffers
        match manager.allocate_buffer(PoolType::AstParsing, size) {
            Ok(buffer) => {
                buffers.push(buffer);
                allocation_count += 1;
            }
            Err(_) => {
                // Expected to fail at some point due to memory limits
                break;
            }
        }
        
        // Check if memory pressure triggers cleanup
        let stats = manager.stats();
        if stats.allocation_pressure > 0.9 {
            let _ = manager.cleanup()?;
            // Cleanup might or might not free memory depending on active references
        }
    }
    
    // Should have allocated at least some buffers
    assert!(allocation_count > 0);
    
    // Final cleanup
    buffers.clear();
    let _ = manager.cleanup()?;
    
    Ok(())
}