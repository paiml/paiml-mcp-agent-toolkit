// Tests for memory integration types
// Included from memory_integration.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
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

