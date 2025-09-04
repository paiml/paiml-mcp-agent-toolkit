//! Compatibility stub for unified cache module
//! 
//! This module provides type aliases and traits to fix compilation after cleanup

use super::config::CacheConfig;
use super::base::CacheStats;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

/// Stub type - redirects to standard CacheConfig  
pub type UnifiedCacheConfig = CacheConfig;

/// Stub trait for UnifiedCache
#[async_trait]
pub trait UnifiedCache: Send + Sync {
    type Key;
    type Value;
    
    async fn get(&self, key: &Self::Key) -> Option<Arc<Self::Value>>;
    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<()>;
    async fn remove(&self, key: &Self::Key) -> Option<Arc<Self::Value>>;
    async fn clear(&self) -> Result<()>;
    fn stats(&self) -> Arc<CacheStats>;
    fn size_bytes(&self) -> usize;
    fn len(&self) -> usize;
    
    // Optional methods that implementations might have
    async fn evict_if_needed(&self) -> Result<()> {
        // Default no-op implementation
        Ok(())
    }
}

/// Stub types that might be referenced
pub struct LayeredCache;
pub struct VectorizedCacheKey;