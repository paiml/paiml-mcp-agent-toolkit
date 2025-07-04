pub mod adapters;
pub mod base;
#[cfg(test)]
pub mod cache_property_tests;
pub mod cache_trait;
pub mod config;
pub mod content_cache;
pub mod diagnostics;
pub mod manager;
pub mod persistent;
pub mod persistent_manager;
pub mod strategies;
pub mod unified;
pub mod unified_manager;

pub use base::{CacheEntry, CacheStats, CacheStrategy};
pub use cache_trait::AstCacheManager;
pub use config::CacheConfig;
pub use content_cache::ContentCache;
pub use diagnostics::{CacheDiagnostics, CacheEffectiveness};
pub use manager::SessionCacheManager;
pub use persistent_manager::PersistentCacheManager;
pub use strategies::{
    AstCacheStrategy, ChurnCacheStrategy, DagCacheStrategy, TemplateCacheStrategy,
};
pub use unified::{LayeredCache, UnifiedCache, UnifiedCacheConfig, VectorizedCacheKey};
pub use unified_manager::{UnifiedCacheDiagnostics, UnifiedCacheManager};
