#![cfg_attr(coverage_nightly, coverage(off))]
//! Advanced caching strategies for PMAT
//!
//! This module implements sophisticated caching strategies that go beyond the basic
//! file-based caching to provide intelligent, adaptive caching for various workloads.
//!
//! # Strategy Types
//!
//! - **Adaptive Cache Strategy**: Automatically adjusts based on access patterns
//! - **Multi-Tier Cache Strategy**: L1/L2/L3 hierarchical caching
//! - **Predictive Cache Strategy**: Pre-loads likely-to-be-needed data
//! - **Collaborative Cache Strategy**: Shares cache data across similar projects
//! - **Time-Series Cache Strategy**: Optimized for temporal data patterns
//!
//! # Design Principles
//!
//! - **Performance**: Sub-millisecond access for L1 cache
//! - **Intelligence**: ML-driven cache warming and eviction
//! - **Scalability**: Handles massive codebases (10M+ LOC)
//! - **Efficiency**: Optimal memory utilization with smart compression
//! - **Reliability**: Graceful degradation and cache corruption recovery

pub mod adaptive_cache;
pub(crate) mod eviction;
pub mod predictor;
#[cfg(test)]
mod tests;
pub mod types;

pub use adaptive_cache::AdaptiveCache;
pub use predictor::CachePredictor;
pub use types::{
    AccessPattern, AdaptiveCacheStats, AdvancedCacheConfig, CacheTier, CacheWarmingConfig,
    EvictionPolicy, PatternStats, PerformanceConfig, PerformanceStats, TierStats, WarmingStats,
};
