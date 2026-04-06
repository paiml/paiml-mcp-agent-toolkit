// profiler_operations.rs — PerformanceProfiler impl, ProfileHandle, ProfilingSummary
// Included from profiler.rs — shares parent module scope (no use imports here)

impl PerformanceProfiler {
    #[must_use]
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
                profile
                    .end_time
                    .expect("internal error")
                    .duration_since(profile.start_time)
                    .as_secs_f64()
                    * 1000.0,
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
                    recommendation: format!("High memory allocation detected: {mem_growth:.1} MB"),
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
        debug_assert!(limit > 0, "limit must be positive");
        let bottlenecks = self.bottlenecks.read().await;
        let mut sorted: Vec<_> = bottlenecks.clone();
        sorted.sort_by(|a, b| {
            b.impact_ms
                .partial_cmp(&a.impact_ms)
                .expect("internal error")
        });
        sorted.into_iter().take(limit).collect()
    }

    /// Get profiling summary
    pub async fn get_summary(&self) -> ProfilingSummary {
        let completed = self.completed_profiles.read().await;
        let active = self.active_profiles.read().await;
        let bottlenecks = self.bottlenecks.read().await;
        let memory_samples = self.memory_samples.read().await;

        let total_operations = completed.len() + active.len();
        let avg_duration = if completed.is_empty() {
            0.0
        } else {
            completed.iter().filter_map(|p| p.duration_ms).sum::<f64>() / completed.len() as f64
        };

        let memory_usage = memory_samples.last().map_or(0.0, |s| s.heap_used_mb);

        ProfilingSummary {
            total_operations,
            completed_operations: completed.len(),
            active_operations: active.len(),
            avg_operation_duration_ms: avg_duration,
            total_bottlenecks: bottlenecks.len(),
            critical_bottlenecks: bottlenecks
                .iter()
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
