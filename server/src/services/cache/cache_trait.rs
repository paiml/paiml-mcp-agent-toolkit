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
