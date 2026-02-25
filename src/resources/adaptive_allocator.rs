#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Adaptive resource allocator that learns optimal allocations
pub struct AdaptiveAllocator {
    history: Arc<RwLock<ResourceHistory>>,
    predictor: Arc<RwLock<ResourcePredictor>>,
    config: AllocatorConfig,
}

#[derive(Clone)]
pub struct AllocatorConfig {
    pub history_window: Duration,
    pub prediction_horizon: Duration,
    pub adjustment_threshold: f32,
    pub min_adjustment: f32,
    pub max_adjustment: f32,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            history_window: Duration::from_secs(300),    // 5 minutes
            prediction_horizon: Duration::from_secs(60), // 1 minute
            adjustment_threshold: 0.1,                   // 10% change triggers adjustment
            min_adjustment: 0.8,                         // Min 80% of current
            max_adjustment: 1.5,                         // Max 150% of current
        }
    }
}

struct ResourceHistory {
    samples: VecDeque<ResourceSample>,
    max_samples: usize,
}

struct ResourceSample {
    timestamp: Instant,
    usage: ResourceUsage,
    _limits: ResourceLimits,
    performance_score: f32,
}

struct ResourcePredictor {
    cpu_trend: f32,
    memory_trend: f32,
    network_trend: f32,
    io_trend: f32,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub average_performance_score: f32,
    pub average_cpu_usage: f32,
    pub average_memory_usage: usize,
    pub cpu_trend: f32,
    pub memory_trend: f32,
    pub sample_count: usize,
}

// --- Include submodules ---
include!("adaptive_allocator_engine.rs");
include!("adaptive_allocator_autoscaler.rs");
include!("adaptive_allocator_tests.rs");
