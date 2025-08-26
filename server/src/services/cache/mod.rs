pub mod adapters;
pub mod advanced_strategies;
pub mod base;
#[cfg(test)]
pub mod cache_property_tests;
#[cfg(test)]
pub mod cache_property_tests_fast;
pub mod cache_trait;
pub mod config;
pub mod content_cache;
pub mod diagnostics;
pub mod manager;
pub mod orchestrator;
pub mod persistent;
pub mod persistent_manager;
pub mod strategies;
pub mod unified;
pub mod unified_manager;

pub use advanced_strategies::{
    AdaptiveCache, AdaptiveCacheStats, AdvancedCacheConfig, CacheTier, CacheWarmingConfig,
    EvictionPolicy, PerformanceConfig,
};
pub use base::{CacheEntry, CacheStats, CacheStrategy};
pub use cache_trait::AstCacheManager;
pub use config::CacheConfig;
pub use content_cache::ContentCache;
pub use diagnostics::{CacheDiagnostics, CacheEffectiveness};
pub use manager::SessionCacheManager;
pub use orchestrator::{
    CacheOrchestrator, OrchestratorConfig, OrchestratorStats, PerformanceMetrics,
    StrategyRecommendation, WorkloadProfile,
};
pub use persistent_manager::PersistentCacheManager;
pub use strategies::{
    AstCacheStrategy, ChurnCacheStrategy, DagCacheStrategy, GitStats, GitStatsCacheStrategy,
    TemplateCacheStrategy,
};
pub use unified::{LayeredCache, UnifiedCache, UnifiedCacheConfig, VectorizedCacheKey};
pub use unified_manager::{UnifiedCacheDiagnostics, UnifiedCacheManager};
