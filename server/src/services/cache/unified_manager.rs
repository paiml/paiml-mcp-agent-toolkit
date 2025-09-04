//! Compatibility stub for UnifiedCacheManager
//!
//! This module provides a compatibility wrapper around SessionCacheManager
//! to fix compilation errors after the unified cache module was removed.

use super::config::CacheConfig;
use super::manager::SessionCacheManager;
use anyhow::Result;

/// Stub for UnifiedCacheConfig - redirects to CacheConfig
pub type UnifiedCacheConfig = CacheConfig;

/// Stub for UnifiedCacheManager - wraps SessionCacheManager
pub struct UnifiedCacheManager {
    inner: SessionCacheManager,
}

impl UnifiedCacheManager {
    pub fn new(config: UnifiedCacheConfig) -> Result<Self> {
        Ok(Self {
            inner: SessionCacheManager::new(config),
        })
    }

    // Add any other methods that are needed by refactor_engine
    pub fn clear_all(&self) {
        self.inner.clear_all();
    }
}

/// Stub for UnifiedCacheDiagnostics (if needed)
pub type UnifiedCacheDiagnostics = super::diagnostics::CacheDiagnostics;
