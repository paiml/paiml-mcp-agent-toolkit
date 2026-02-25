#![cfg_attr(coverage_nightly, coverage(off))]
//! Sprint 31 Week 2 - Performance Profiling Tools
//!
//! Advanced profiling capabilities for the TDG system including flame graphs,
//! call traces, memory profiling, and bottleneck detection.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Performance profile for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProfile {
    pub operation_id: String,
    pub operation_type: String,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
    #[serde(skip)]
    pub end_time: Option<Instant>,
    pub duration_ms: Option<f64>,
    pub memory_before_mb: f64,
    pub memory_after_mb: Option<f64>,
    pub cpu_time_ms: f64,
    pub io_wait_ms: f64,
    pub children: Vec<OperationProfile>,
    pub metadata: HashMap<String, String>,
}

/// Flame graph data structure for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphNode {
    pub name: String,
    pub value: f64, // Duration in milliseconds
    pub children: Vec<FlameGraphNode>,
    pub metadata: HashMap<String, String>,
}

/// Call stack frame for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    pub function_name: String,
    pub file_path: String,
    pub line_number: u32,
    pub duration_ms: f64,
    pub self_time_ms: f64,
    pub call_count: u32,
}

/// Memory allocation sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub heap_used_mb: f64,
    pub heap_total_mb: f64,
    pub stack_used_mb: f64,
    pub gc_count: u32,
    pub gc_pause_ms: f64,
}

/// Bottleneck detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub location: String,
    pub bottleneck_type: BottleneckType,
    pub severity: BottleneckSeverity,
    pub impact_ms: f64,
    pub occurrence_count: u32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BottleneckType {
    CpuBound,
    IoBound,
    MemoryBound,
    LockContention,
    NetworkLatency,
    DatabaseQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Advanced performance profiler for the TDG system
pub struct PerformanceProfiler {
    /// Active operation profiles
    active_profiles: Arc<RwLock<HashMap<String, OperationProfile>>>,
    /// Completed profiles
    completed_profiles: Arc<RwLock<Vec<OperationProfile>>>,
    /// Call stack traces
    #[allow(dead_code)]
    call_stacks: Arc<RwLock<Vec<Vec<CallFrame>>>>,
    /// Memory samples
    memory_samples: Arc<RwLock<Vec<MemorySample>>>,
    /// Detected bottlenecks
    bottlenecks: Arc<RwLock<Vec<Bottleneck>>>,
    /// Profiling configuration
    config: ProfilerConfig,
}

#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    pub enable_cpu_profiling: bool,
    pub enable_memory_profiling: bool,
    pub enable_io_profiling: bool,
    pub sample_interval_ms: u64,
    pub max_stack_depth: usize,
    pub max_profiles_retained: usize,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enable_cpu_profiling: true,
            enable_memory_profiling: true,
            enable_io_profiling: true,
            sample_interval_ms: 100,
            max_stack_depth: 50,
            max_profiles_retained: 1000,
        }
    }
}

// --- Included submodules ---

include!("profiler_operations.rs");
include!("profiler_tests.rs");
