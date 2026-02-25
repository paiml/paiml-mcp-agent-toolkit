impl AdaptiveAllocator {
    pub fn new(config: AllocatorConfig) -> Self {
        Self {
            history: Arc::new(RwLock::new(ResourceHistory {
                samples: VecDeque::new(),
                max_samples: 1000,
            })),
            predictor: Arc::new(RwLock::new(ResourcePredictor {
                cpu_trend: 0.0,
                memory_trend: 0.0,
                network_trend: 0.0,
                io_trend: 0.0,
            })),
            config,
        }
    }

    pub fn record_usage(
        &self,
        usage: ResourceUsage,
        _limits: ResourceLimits,
        performance_score: f32,
    ) {
        let mut history = self.history.write();

        history.samples.push_back(ResourceSample {
            timestamp: Instant::now(),
            usage,
            _limits,
            performance_score,
        });

        // Remove old samples
        let cutoff = Instant::now() - self.config.history_window;
        while let Some(front) = history.samples.front() {
            if front.timestamp < cutoff {
                history.samples.pop_front();
            } else {
                break;
            }
        }

        // Limit total samples
        while history.samples.len() > history.max_samples {
            history.samples.pop_front();
        }

        // Update predictions
        self.update_predictions(&history);
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_predictions(&self, history: &ResourceHistory) {
        if history.samples.len() < 10 {
            return; // Not enough data
        }

        let mut predictor = self.predictor.write();

        // Simple linear regression for trends
        let recent: Vec<_> = history.samples.iter().rev().take(20).collect();

        if recent.len() >= 2 {
            // CPU trend
            let cpu_start = recent
                .last()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .cpu_percent;
            let cpu_end = recent
                .first()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .cpu_percent;
            predictor.cpu_trend = (cpu_end - cpu_start) / cpu_start.max(0.1);

            // Memory trend
            let mem_start = recent
                .last()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .memory_bytes as f32;
            let mem_end = recent
                .first()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .memory_bytes as f32;
            predictor.memory_trend = (mem_end - mem_start) / mem_start.max(1.0);

            // Network trend
            let net_start = (recent
                .last()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .network_ingress_bytes
                + recent
                    .last()
                    .expect("recent has >=2 elements (checked above)")
                    .usage
                    .network_egress_bytes) as f32;
            let net_end = (recent
                .first()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .network_ingress_bytes
                + recent
                    .first()
                    .expect("recent has >=2 elements (checked above)")
                    .usage
                    .network_egress_bytes) as f32;
            predictor.network_trend = (net_end - net_start) / net_start.max(1.0);

            // I/O trend
            let io_start = (recent
                .last()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .disk_read_bytes
                + recent
                    .last()
                    .expect("recent has >=2 elements (checked above)")
                    .usage
                    .disk_write_bytes) as f32;
            let io_end = (recent
                .first()
                .expect("recent has >=2 elements (checked above)")
                .usage
                .disk_read_bytes
                + recent
                    .first()
                    .expect("recent has >=2 elements (checked above)")
                    .usage
                    .disk_write_bytes) as f32;
            predictor.io_trend = (io_end - io_start) / io_start.max(1.0);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn suggest_adjustment(&self, current_limits: &ResourceLimits) -> Option<ResourceLimits> {
        let predictor = self.predictor.read();
        let mut adjusted = current_limits.clone();
        let mut needs_adjustment = false;

        // Adjust CPU
        if predictor.cpu_trend.abs() > self.config.adjustment_threshold {
            let factor = 1.0 + predictor.cpu_trend.clamp(-0.2, 0.5);
            adjusted.cpu.cores = (adjusted.cpu.cores * factor)
                .max(adjusted.cpu.cores * self.config.min_adjustment)
                .min(adjusted.cpu.cores * self.config.max_adjustment);
            needs_adjustment = true;
        }

        // Adjust memory
        if predictor.memory_trend.abs() > self.config.adjustment_threshold {
            let factor = 1.0 + predictor.memory_trend.clamp(-0.2, 0.5);
            adjusted.memory.max_bytes = ((adjusted.memory.max_bytes as f32) * factor)
                .max((adjusted.memory.max_bytes as f32) * self.config.min_adjustment)
                .min((adjusted.memory.max_bytes as f32) * self.config.max_adjustment)
                as usize;
            needs_adjustment = true;
        }

        // Adjust network
        if predictor.network_trend.abs() > self.config.adjustment_threshold {
            let factor = 1.0 + predictor.network_trend.clamp(-0.2, 0.5);
            adjusted.network.ingress_bytes_per_sec =
                ((adjusted.network.ingress_bytes_per_sec as f32) * factor) as u64;
            adjusted.network.egress_bytes_per_sec =
                ((adjusted.network.egress_bytes_per_sec as f32) * factor) as u64;
            needs_adjustment = true;
        }

        // Adjust I/O
        if predictor.io_trend.abs() > self.config.adjustment_threshold {
            let factor = 1.0 + predictor.io_trend.clamp(-0.2, 0.5);
            adjusted.disk_io.read_bytes_per_sec =
                ((adjusted.disk_io.read_bytes_per_sec as f32) * factor) as u64;
            adjusted.disk_io.write_bytes_per_sec =
                ((adjusted.disk_io.write_bytes_per_sec as f32) * factor) as u64;
            needs_adjustment = true;
        }

        if needs_adjustment {
            Some(adjusted)
        } else {
            None
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let history = self.history.read();

        if history.samples.is_empty() {
            return PerformanceStats::default();
        }

        let avg_score = history
            .samples
            .iter()
            .map(|s| s.performance_score)
            .sum::<f32>()
            / history.samples.len() as f32;

        let avg_cpu = history
            .samples
            .iter()
            .map(|s| s.usage.cpu_percent)
            .sum::<f32>()
            / history.samples.len() as f32;

        let avg_memory = history
            .samples
            .iter()
            .map(|s| s.usage.memory_bytes)
            .sum::<usize>()
            / history.samples.len();

        let predictor = self.predictor.read();

        PerformanceStats {
            average_performance_score: avg_score,
            average_cpu_usage: avg_cpu,
            average_memory_usage: avg_memory,
            cpu_trend: predictor.cpu_trend,
            memory_trend: predictor.memory_trend,
            sample_count: history.samples.len(),
        }
    }
}
