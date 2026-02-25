// Utility functions for memory optimization
// Included from memory_integration.rs - shares parent module scope

/// Utility functions for memory optimization
pub mod utils {
    use super::{global_memory_manager, MemoryVec, PoolType, Result};

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
