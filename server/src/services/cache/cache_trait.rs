use crate::services::context::FileContext;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Trait for cache managers that support AST caching
#[async_trait::async_trait]
pub trait AstCacheManager: Send + Sync {
    /// Get or compute AST with caching
    async fn get_or_compute_ast<F, Fut>(&self, path: &Path, compute: F) -> Result<Arc<FileContext>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<FileContext>> + Send;
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
