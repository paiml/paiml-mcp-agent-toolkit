//! Sprint 31 Week 2 - Performance Profiling Tools
//! 
//! Advanced profiling capabilities for the TDG system including flame graphs,
//! call traces, memory profiling, and bottleneck detection.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;

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

impl PerformanceProfiler {
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            active_profiles: Arc::new(RwLock::new(HashMap::new())),
            completed_profiles: Arc::new(RwLock::new(Vec::new())),
            call_stacks: Arc::new(RwLock::new(Vec::new())),
            memory_samples: Arc::new(RwLock::new(Vec::new())),
            bottlenecks: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Start profiling an operation
    pub async fn start_operation(
        &self,
        operation_id: String,
        operation_type: String,
    ) -> Result<ProfileHandle> {
        let memory_before = self.get_current_memory_usage();
        
        let profile = OperationProfile {
            operation_id: operation_id.clone(),
            operation_type,
            start_time: Instant::now(),
            end_time: None,
            duration_ms: None,
            memory_before_mb: memory_before,
            memory_after_mb: None,
            cpu_time_ms: 0.0,
            io_wait_ms: 0.0,
            children: Vec::new(),
            metadata: HashMap::new(),
        };

        let mut active = self.active_profiles.write().await;
        active.insert(operation_id.clone(), profile);

        Ok(ProfileHandle {
            operation_id,
            profiler: self.clone_arc(),
        })
    }

    /// Complete profiling an operation
    pub async fn complete_operation(&self, operation_id: &str) -> Result<()> {
        let mut active = self.active_profiles.write().await;
        
        if let Some(mut profile) = active.remove(operation_id) {
            profile.end_time = Some(Instant::now());
            profile.duration_ms = Some(
                profile.end_time.unwrap()
                    .duration_since(profile.start_time)
                    .as_secs_f64() * 1000.0
            );
            profile.memory_after_mb = Some(self.get_current_memory_usage());

            // Detect bottlenecks in this operation
            self.detect_bottlenecks(&profile).await?;

            let mut completed = self.completed_profiles.write().await;
            completed.push(profile);

            // Enforce retention limit
            while completed.len() > self.config.max_profiles_retained {
                completed.remove(0);
            }
        }

        Ok(())
    }

    /// Sample memory usage
    pub async fn sample_memory(&self) -> Result<()> {
        if !self.config.enable_memory_profiling {
            return Ok(());
        }

        let sample = MemorySample {
            timestamp: Instant::now(),
            heap_used_mb: self.get_heap_used(),
            heap_total_mb: self.get_heap_total(),
            stack_used_mb: self.get_stack_used(),
            gc_count: self.get_gc_count(),
            gc_pause_ms: self.get_gc_pause_time(),
        };

        let mut samples = self.memory_samples.write().await;
        samples.push(sample);

        // Keep only last hour of samples (assuming 100ms intervals)
        let max_samples = 36000;
        while samples.len() > max_samples {
            samples.remove(0);
        }

        Ok(())
    }

    /// Generate flame graph from completed profiles
    pub async fn generate_flame_graph(&self) -> Result<FlameGraphNode> {
        let completed = self.completed_profiles.read().await;
        
        let mut root = FlameGraphNode {
            name: "root".to_string(),
            value: 0.0,
            children: Vec::new(),
            metadata: HashMap::new(),
        };

        for profile in completed.iter() {
            let node = self.profile_to_flame_node(profile);
            root.children.push(node);
            root.value += profile.duration_ms.unwrap_or(0.0);
        }

        Ok(root)
    }

    /// Convert operation profile to flame graph node
    fn profile_to_flame_node(&self, profile: &OperationProfile) -> FlameGraphNode {
        let mut node = FlameGraphNode {
            name: format!("{} ({})", profile.operation_type, profile.operation_id),
            value: profile.duration_ms.unwrap_or(0.0),
            children: Vec::new(),
            metadata: profile.metadata.clone(),
        };

        for child in &profile.children {
            node.children.push(self.profile_to_flame_node(child));
        }

        node
    }

    /// Detect bottlenecks in an operation
    async fn detect_bottlenecks(&self, profile: &OperationProfile) -> Result<()> {
        let mut detected = Vec::new();
        let duration = profile.duration_ms.unwrap_or(0.0);

        // CPU bound detection
        if profile.cpu_time_ms > duration * 0.8 {
            detected.push(Bottleneck {
                location: profile.operation_type.clone(),
                bottleneck_type: BottleneckType::CpuBound,
                severity: if profile.cpu_time_ms > 5000.0 {
                    BottleneckSeverity::Critical
                } else if profile.cpu_time_ms > 1000.0 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                impact_ms: profile.cpu_time_ms,
                occurrence_count: 1,
                recommendation: "Consider optimizing CPU-intensive operations or parallelizing work".to_string(),
            });
        }

        // IO bound detection
        if profile.io_wait_ms > duration * 0.5 {
            detected.push(Bottleneck {
                location: profile.operation_type.clone(),
                bottleneck_type: BottleneckType::IoBound,
                severity: if profile.io_wait_ms > 3000.0 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                impact_ms: profile.io_wait_ms,
                occurrence_count: 1,
                recommendation: "Consider async I/O, caching, or batching operations".to_string(),
            });
        }

        // Memory bound detection
        if let (Some(mem_after), mem_before) = (profile.memory_after_mb, profile.memory_before_mb) {
            let mem_growth = mem_after - mem_before;
            if mem_growth > 100.0 {
                detected.push(Bottleneck {
                    location: profile.operation_type.clone(),
                    bottleneck_type: BottleneckType::MemoryBound,
                    severity: if mem_growth > 500.0 {
                        BottleneckSeverity::Critical
                    } else {
                        BottleneckSeverity::High
                    },
                    impact_ms: duration,
                    occurrence_count: 1,
                    recommendation: format!("High memory allocation detected: {:.1} MB", mem_growth),
                });
            }
        }

        if !detected.is_empty() {
            let mut bottlenecks = self.bottlenecks.write().await;
            bottlenecks.extend(detected);
        }

        Ok(())
    }

    /// Get top bottlenecks
    pub async fn get_top_bottlenecks(&self, limit: usize) -> Vec<Bottleneck> {
        let bottlenecks = self.bottlenecks.read().await;
        let mut sorted: Vec<_> = bottlenecks.clone();
        sorted.sort_by(|a, b| b.impact_ms.partial_cmp(&a.impact_ms).unwrap());
        sorted.into_iter().take(limit).collect()
    }

    /// Get profiling summary
    pub async fn get_summary(&self) -> ProfilingSummary {
        let completed = self.completed_profiles.read().await;
        let active = self.active_profiles.read().await;
        let bottlenecks = self.bottlenecks.read().await;
        let memory_samples = self.memory_samples.read().await;

        let total_operations = completed.len() + active.len();
        let avg_duration = if !completed.is_empty() {
            completed.iter()
                .filter_map(|p| p.duration_ms)
                .sum::<f64>() / completed.len() as f64
        } else {
            0.0
        };

        let memory_usage = memory_samples.last()
            .map(|s| s.heap_used_mb)
            .unwrap_or(0.0);

        ProfilingSummary {
            total_operations,
            completed_operations: completed.len(),
            active_operations: active.len(),
            avg_operation_duration_ms: avg_duration,
            total_bottlenecks: bottlenecks.len(),
            critical_bottlenecks: bottlenecks.iter()
                .filter(|b| b.severity == BottleneckSeverity::Critical)
                .count(),
            current_memory_mb: memory_usage,
        }
    }

    // Helper methods for system metrics (simplified implementations)
    fn get_current_memory_usage(&self) -> f64 {
        // In production, use sys-info or similar
        100.0 // Placeholder
    }

    fn get_heap_used(&self) -> f64 {
        80.0 // Placeholder
    }

    fn get_heap_total(&self) -> f64 {
        200.0 // Placeholder
    }

    fn get_stack_used(&self) -> f64 {
        20.0 // Placeholder
    }

    fn get_gc_count(&self) -> u32 {
        5 // Placeholder
    }

    fn get_gc_pause_time(&self) -> f64 {
        10.0 // Placeholder
    }

    fn clone_arc(&self) -> Arc<Self> {
        // In production, this would return Arc<Self>
        // For now, create a new instance with same config
        Arc::new(Self::new(self.config.clone()))
    }
}

/// Handle for profiling an operation
pub struct ProfileHandle {
    operation_id: String,
    profiler: Arc<PerformanceProfiler>,
}

impl ProfileHandle {
    pub async fn complete(self) -> Result<()> {
        self.profiler.complete_operation(&self.operation_id).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingSummary {
    pub total_operations: usize,
    pub completed_operations: usize,
    pub active_operations: usize,
    pub avg_operation_duration_ms: f64,
    pub total_bottlenecks: usize,
    pub critical_bottlenecks: usize,
    pub current_memory_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_operation_profiling() {
        let profiler = PerformanceProfiler::new(ProfilerConfig::default());
        
        let handle = profiler.start_operation(
            "test_op_1".to_string(),
            "analysis".to_string(),
        ).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        
        handle.complete().await.unwrap();

        let summary = profiler.get_summary().await;
        assert_eq!(summary.completed_operations, 1);
        assert!(summary.avg_operation_duration_ms >= 100.0);
    }

    #[tokio::test]
    async fn test_bottleneck_detection() {
        let profiler = PerformanceProfiler::new(ProfilerConfig::default());
        
        let mut profile = OperationProfile {
            operation_id: "slow_op".to_string(),
            operation_type: "compute".to_string(),
            start_time: Instant::now(),
            end_time: Some(Instant::now() + Duration::from_secs(10)),
            duration_ms: Some(10000.0),
            memory_before_mb: 100.0,
            memory_after_mb: Some(700.0),
            cpu_time_ms: 9000.0,
            io_wait_ms: 500.0,
            children: Vec::new(),
            metadata: HashMap::new(),
        };

        profiler.detect_bottlenecks(&profile).await.unwrap();
        
        let bottlenecks = profiler.get_top_bottlenecks(10).await;
        assert!(!bottlenecks.is_empty());
        assert!(bottlenecks.iter().any(|b| b.bottleneck_type == BottleneckType::CpuBound));
        assert!(bottlenecks.iter().any(|b| b.bottleneck_type == BottleneckType::MemoryBound));
    }

    #[tokio::test]
    async fn test_flame_graph_generation() {
        let profiler = PerformanceProfiler::new(ProfilerConfig::default());
        
        // Create some operations
        for i in 0..3 {
            let handle = profiler.start_operation(
                format!("op_{}", i),
                "test".to_string(),
            ).await.unwrap();
            
            tokio::time::sleep(Duration::from_millis(50)).await;
            handle.complete().await.unwrap();
        }

        let flame_graph = profiler.generate_flame_graph().await.unwrap();
        assert_eq!(flame_graph.name, "root");
        assert_eq!(flame_graph.children.len(), 3);
    }
}