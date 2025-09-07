//! Compatibility stub for unified cache module
//!
//! This module provides type aliases and traits to fix compilation after cleanup

use super::base::CacheStats;
use super::config::CacheConfig;
use anyhow::Result;
use async_trait::async_trait;
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

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // Optional methods that implementations might have
    async fn evict_if_needed(&self) -> Result<()> {
        // Default no-op implementation
        Ok(())
    }
}

/// Stub types that might be referenced
pub struct LayeredCache;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VectorizedCacheKey {
    data: Vec<u8>,
    pub hash_low: u64,
    pub hash_high: u64,
}

impl VectorizedCacheKey {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let hash = blake3::hash(bytes);
        let hash_bytes = hash.as_bytes();
        let hash_low = u64::from_le_bytes([
            hash_bytes[0],
            hash_bytes[1],
            hash_bytes[2],
            hash_bytes[3],
            hash_bytes[4],
            hash_bytes[5],
            hash_bytes[6],
            hash_bytes[7],
        ]);
        let hash_high = u64::from_le_bytes([
            hash_bytes[8],
            hash_bytes[9],
            hash_bytes[10],
            hash_bytes[11],
            hash_bytes[12],
            hash_bytes[13],
            hash_bytes[14],
            hash_bytes[15],
        ]);

        Self {
            data: bytes.to_vec(),
            hash_low,
            hash_high,
        }
    }

    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            hash_low: 0,
            hash_high: 0,
        }
    }
}

impl Default for VectorizedCacheKey {
    fn default() -> Self {
        Self::new()
    }
}
