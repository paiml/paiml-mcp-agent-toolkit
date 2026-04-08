#![cfg_attr(coverage_nightly, coverage(off))]
// Two-tier cache implementation for Claude integration
// L1: In-memory cache with short TTL, L2: Persistent cache

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Analysis result that can be cached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub complexity: u32,
    pub cognitive_complexity: u32,
    pub satd_count: usize,
    pub timestamp: SystemTime,
    pub content_hash: u64,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            complexity: 0,
            cognitive_complexity: 0,
            satd_count: 0,
            timestamp: SystemTime::now(),
            content_hash: 0,
        }
    }
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: Arc<T>,
    expires_at: SystemTime,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value: Arc::new(value),
            expires_at: SystemTime::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// Two-tier cache with L1 (memory) and L2 (persistent)
pub struct TwoTierCache {
    /// L1: In-process cache with 10ms TTL
    l1: Arc<RwLock<HashMap<u64, CacheEntry<AnalysisResult>>>>,

    /// L2: Memory-mapped cache with 60s TTL
    l2: Arc<RwLock<HashMap<u64, CacheEntry<AnalysisResult>>>>,

    /// L1 cache TTL
    l1_ttl: Duration,

    /// L2 cache TTL
    l2_ttl: Duration,

    /// Metrics
    l1_hits: AtomicU64,
    l1_misses: AtomicU64,
    l2_hits: AtomicU64,
    l2_misses: AtomicU64,
}

impl Default for TwoTierCache {
    fn default() -> Self {
        Self::new()
    }
}

/// FNV-1a hasher for fast hashing
#[derive(Default)]
struct FnvHasher {
    state: u64,
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        const FNV_PRIME: u64 = 0x100000001b3;
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;

        self.state = FNV_OFFSET;
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Cache metrics.
pub struct CacheMetrics {
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub effective_hit_rate: f64,
    pub l1_size: usize,
    pub l2_size: usize,
}

// TwoTierCache method implementations
include!("cache_operations.rs");

// Tests
include!("cache_tests.rs");
