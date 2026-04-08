#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use crate::services::cache::{
    config::CacheConfig,
    diagnostics::{CacheDiagnostics, CacheEffectiveness, CacheStatsSnapshot},
    persistent::PersistentCache,
    strategies::AstCacheStrategy,
};
use crate::services::context::FileContext;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Persistent cache manager that stores cache data on disk
pub struct PersistentCacheManager {
    // Different cache types
    ast_cache: Arc<PersistentCache<AstCacheStrategy>>,

    // Global settings
    config: CacheConfig,
    session_id: Uuid,
    created: Instant,

    cache_dir: PathBuf,
}

// Core construction and cache operations
include!("persistent_manager_core.rs");

// Diagnostics and metrics
include!("persistent_manager_diagnostics.rs");

// Unit tests
include!("persistent_manager_tests.rs");

// Property-based tests
include!("persistent_manager_proptests.rs");
