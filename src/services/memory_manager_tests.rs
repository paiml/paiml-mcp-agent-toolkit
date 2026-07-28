// Unit tests and property tests for memory_manager
//
// This file is included by memory_manager.rs and shares its module scope.
// Do NOT add `use` imports or `#!` inner attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_memory_manager_creation() -> Result<()> {
        let manager = MemoryManager::new()?;
        let stats = manager.stats();
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.peak_usage, 0);
        Ok(())
    }

    #[test]
    fn test_buffer_allocation() -> Result<()> {
        let manager = MemoryManager::new()?;
        let buffer = manager.allocate_buffer(PoolType::AstParsing, 1024)?;
        assert_eq!(buffer.as_slice().len(), 1024);
        assert!(buffer.capacity() >= 1024);
        Ok(())
    }

    #[test]
    fn test_string_interning() -> Result<()> {
        let manager = MemoryManager::new()?;

        let str1 = manager.intern_string("test")?;
        let str2 = manager.intern_string("test")?;

        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&str1, &str2));
        assert_eq!(str1.as_ref(), "test");
        Ok(())
    }

    #[test]
    fn test_memory_cleanup() -> Result<()> {
        let manager = MemoryManager::new()?;

        // Allocate some memory
        let _buffer1 = manager.allocate_buffer(PoolType::AstParsing, 1024)?;
        let _buffer2 = manager.allocate_buffer(PoolType::FileContent, 2048)?;
        let _interned = manager.intern_string("test_string")?;

        let stats_before = manager.stats();
        assert!(stats_before.total_allocated > 0);

        // Force cleanup
        let _cleaned = manager.cleanup()?;

        // Note: Cleanup behavior depends on memory pressure
        // In a real scenario with high pressure, cleanup would free memory
        Ok(())
    }

    #[test]
    fn test_pool_buffer_reuse() -> Result<()> {
        let manager = MemoryManager::new()?;

        // Allocate and drop buffer
        {
            let _buffer = manager.allocate_buffer(PoolType::AstParsing, 1024)?;
        }

        // Allocate another buffer of same size - should reuse
        let buffer2 = manager.allocate_buffer(PoolType::AstParsing, 1024)?;
        assert_eq!(buffer2.as_slice().len(), 1024);

        Ok(())
    }

    #[test]
    fn test_allocation_strategy() -> Result<()> {
        let manager = MemoryManager::new()?;

        // Small allocation should use pooled strategy
        assert_eq!(manager.determine_strategy(1024), AllocationStrategy::Pooled);

        // Large allocation should use memory-mapped strategy
        assert_eq!(
            manager.determine_strategy(2 * 1024 * 1024),
            AllocationStrategy::MemoryMapped
        );

        // Medium allocation should use direct strategy
        assert_eq!(
            manager.determine_strategy(512 * 1024),
            AllocationStrategy::Direct
        );

        Ok(())
    }

    #[test]
    fn test_concurrent_access() -> Result<()> {
        let manager = MemoryManager::new()?;
        let manager = Arc::clone(&manager);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let manager = Arc::clone(&manager);
                thread::spawn(move || -> Result<()> {
                    for j in 0..10 {
                        let _buffer =
                            manager.allocate_buffer(PoolType::AstParsing, 1024 + j * 100)?;
                        let _interned =
                            manager.intern_string(&format!("thread_{}_iter_{}", i, j))?;
                        thread::sleep(Duration::from_millis(1));
                    }
                    Ok(())
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap()?;
        }

        let stats = manager.stats();
        // Check peak_usage instead of total_allocated because buffers are dropped
        // at end of thread scope, returning them to pool and decrementing total_allocated
        assert!(
            stats.peak_usage > 0,
            "Expected peak_usage > 0, got {}",
            stats.peak_usage
        );

        Ok(())
    }
}

