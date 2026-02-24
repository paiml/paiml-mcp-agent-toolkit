impl AdaptiveThresholdManager {
    /// Create new adaptive threshold manager
    #[must_use]
    pub fn new(config: AdaptiveConfig) -> Self {
        Self {
            config,
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            current_thresholds: Arc::new(RwLock::new(CurrentThresholds::default())),
            adjustment_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Record performance sample for adaptation
    pub async fn record_sample(&self, sample: PerformanceSample) -> Result<()> {
        let mut history = self.performance_history.write().await;

        // Add new sample
        history.push_back(sample);

        // Maintain window size
        while history.len() > self.config.sample_window_size {
            history.pop_front();
        }

        // Check if adaptation is needed
        if history.len() >= 10 {
            // Minimum samples for reliable adjustment
            if let Some(adjustment) = self.calculate_adjustment(&history).await? {
                self.apply_adjustment(adjustment).await?;
            }
        }

        Ok(())
    }

    /// Calculate recommended threshold adjustment based on performance
    #[allow(clippy::cast_possible_truncation)]
    async fn calculate_adjustment(
        &self,
        history: &VecDeque<PerformanceSample>,
    ) -> Result<Option<ThresholdAdjustment>> {
        if history.len() < 5 {
            return Ok(None);
        }

        // Calculate performance metrics
        let avg_duration = history.iter().map(|s| s.analysis_duration_ms).sum::<u64>() as f32
            / history.len() as f32;

        let avg_cache_hit =
            history.iter().map(|s| s.cache_hit_ratio).sum::<f32>() / history.len() as f32;

        let avg_memory =
            history.iter().map(|s| s.memory_usage_mb).sum::<f32>() / history.len() as f32;

        let avg_cpu = history.iter().map(|s| s.cpu_utilization).sum::<f32>() / history.len() as f32;

        let avg_queue_depth =
            history.iter().map(|s| s.queue_depth).sum::<usize>() as f32 / history.len() as f32;

        // Performance below threshold
        if avg_duration > self.config.target_analysis_time_ms as f32 * 1.5 {
            if avg_cache_hit < self.config.min_cache_hit_ratio {
                // Low cache hit ratio - scale up cache
                return Ok(Some(ThresholdAdjustment::ScaleUp {
                    cache_factor: 1.0 + self.config.adjustment_sensitivity,
                    permit_factor: 1.0,
                }));
            } else if avg_queue_depth > 5.0 {
                // High queue depth - increase permits
                return Ok(Some(ThresholdAdjustment::ScaleUp {
                    cache_factor: 1.0,
                    permit_factor: 1.0 + self.config.adjustment_sensitivity,
                }));
            }
            // Reduce compression for speed
            return Ok(Some(ThresholdAdjustment::LessCompression {
                compression_level: 1,
            }));
        }

        // Resource usage too high
        if avg_memory > self.config.max_memory_mb || avg_cpu > self.config.max_cpu_utilization {
            if avg_cache_hit > 0.9
                && avg_duration < self.config.target_analysis_time_ms as f32 * 0.8
            {
                // High cache hit and fast performance - can reduce cache
                return Ok(Some(ThresholdAdjustment::ScaleDown {
                    cache_factor: 1.0 - self.config.adjustment_sensitivity,
                    permit_factor: 1.0,
                }));
            }
            // Increase compression to save memory
            return Ok(Some(ThresholdAdjustment::MoreCompression {
                compression_level: 6,
            }));
        }

        // Performance is excellent - maintain current settings
        if avg_duration < self.config.target_analysis_time_ms as f32 * 0.5
            && avg_cache_hit > 0.8
            && avg_memory < self.config.max_memory_mb * 0.5
        {
            return Ok(Some(ThresholdAdjustment::Maintain));
        }

        Ok(None)
    }

    /// Apply threshold adjustment to current configuration
    #[allow(clippy::cast_possible_truncation)]
    async fn apply_adjustment(&self, adjustment: ThresholdAdjustment) -> Result<()> {
        let mut thresholds = self.current_thresholds.write().await;
        let mut adjustments = self.adjustment_history.write().await;

        match adjustment.clone() {
            ThresholdAdjustment::ScaleUp {
                cache_factor,
                permit_factor,
            } => {
                thresholds.hot_cache_size =
                    ((thresholds.hot_cache_size as f32 * cache_factor) as usize).min(10000);
                thresholds.high_priority_permits =
                    ((thresholds.high_priority_permits as f32 * permit_factor) as usize).min(50);
                thresholds.low_priority_permits =
                    ((thresholds.low_priority_permits as f32 * permit_factor) as usize).min(20);
            }
            ThresholdAdjustment::ScaleDown {
                cache_factor,
                permit_factor,
            } => {
                thresholds.hot_cache_size =
                    ((thresholds.hot_cache_size as f32 * cache_factor) as usize).max(100);
                thresholds.high_priority_permits =
                    ((thresholds.high_priority_permits as f32 * permit_factor) as usize).max(2);
                thresholds.low_priority_permits =
                    ((thresholds.low_priority_permits as f32 * permit_factor) as usize).max(1);
            }
            ThresholdAdjustment::MoreCompression { compression_level } => {
                thresholds.compression_level = compression_level.min(9);
            }
            ThresholdAdjustment::LessCompression { compression_level } => {
                thresholds.compression_level = compression_level.max(1);
            }
            ThresholdAdjustment::Maintain => {
                // No changes needed
            }
        }

        // Record adjustment
        adjustments.push_back(adjustment);

        // Maintain adjustment history size
        while adjustments.len() > 100 {
            adjustments.pop_front();
        }

        Ok(())
    }

    /// Get current threshold configuration
    pub async fn get_current_thresholds(&self) -> CurrentThresholds {
        self.current_thresholds.read().await.clone()
    }

    /// Get recent performance statistics
    #[allow(clippy::cast_possible_truncation)]
    pub async fn get_performance_stats(&self) -> PerformanceStatistics {
        let history = self.performance_history.read().await;
        let adjustments = self.adjustment_history.read().await;

        if history.is_empty() {
            return PerformanceStatistics::default();
        }

        let recent = history.iter().rev().take(10).collect::<Vec<_>>();

        let avg_duration =
            recent.iter().map(|s| s.analysis_duration_ms).sum::<u64>() as f32 / recent.len() as f32;

        let avg_cache_hit =
            recent.iter().map(|s| s.cache_hit_ratio).sum::<f32>() / recent.len() as f32;

        let avg_memory =
            recent.iter().map(|s| s.memory_usage_mb).sum::<f32>() / recent.len() as f32;

        let avg_cpu = recent.iter().map(|s| s.cpu_utilization).sum::<f32>() / recent.len() as f32;

        let recent_adjustments = adjustments.len().min(10); // Simplified recent count

        PerformanceStatistics {
            avg_analysis_duration_ms: avg_duration,
            avg_cache_hit_ratio: avg_cache_hit,
            avg_memory_usage_mb: avg_memory,
            avg_cpu_utilization: avg_cpu,
            total_samples: history.len(),
            recent_adjustments_count: recent_adjustments,
            performance_trend: self.calculate_trend(&history),
        }
    }

    /// Calculate performance trend from recent samples
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_trend(&self, history: &VecDeque<PerformanceSample>) -> PerformanceTrend {
        if history.len() < 10 {
            return PerformanceTrend::Stable;
        }

        let history_vec: Vec<_> = history.iter().collect();
        let mid_point = history_vec.len() / 2;
        let recent_half = &history_vec[mid_point..];
        let older_half = &history_vec[..mid_point];

        let recent_avg = recent_half
            .iter()
            .map(|s| s.analysis_duration_ms)
            .sum::<u64>() as f32
            / recent_half.len() as f32;

        let older_avg = older_half
            .iter()
            .map(|s| s.analysis_duration_ms)
            .sum::<u64>() as f32
            / older_half.len() as f32;

        let change_ratio = (recent_avg - older_avg) / older_avg;

        if change_ratio > 0.2 {
            PerformanceTrend::Degrading
        } else if change_ratio < -0.2 {
            PerformanceTrend::Improving
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Create performance sample from current system state
    pub async fn create_sample(
        &self,
        duration: Duration,
        cache_hit: bool,
        queue_depth: usize,
    ) -> PerformanceSample {
        PerformanceSample {
            timestamp: Instant::now(),
            analysis_duration_ms: duration.as_millis() as u64,
            cache_hit_ratio: if cache_hit { 1.0 } else { 0.0 },
            memory_usage_mb: self.get_memory_usage().await,
            cpu_utilization: self.get_cpu_usage().await,
            queue_depth,
        }
    }

    /// Get current memory usage (simplified implementation)
    #[allow(clippy::cast_possible_truncation)]
    async fn get_memory_usage(&self) -> f32 {
        // In a full implementation, this would use system APIs
        // For now, estimate based on cache size and active operations
        let thresholds = self.current_thresholds.read().await;
        let estimated_cache_mb = (thresholds.hot_cache_size * 1024) as f32 / (1024.0 * 1024.0);
        estimated_cache_mb + 50.0 // Base memory usage
    }

    /// Get current CPU utilization (simplified implementation)
    #[allow(clippy::cast_possible_truncation)]
    async fn get_cpu_usage(&self) -> f32 {
        // In a full implementation, this would use system APIs
        // For now, estimate based on active operations
        let history = self.performance_history.read().await;
        let recent_activity = history
            .iter()
            .rev()
            .take(5)
            .filter(|s| s.timestamp.elapsed() < Duration::from_secs(10))
            .count();

        (recent_activity as f32 * 0.1).min(1.0) // Estimate 10% CPU per recent operation
    }

    /// Reset thresholds to default configuration
    pub async fn reset_to_defaults(&self) -> Result<()> {
        let mut thresholds = self.current_thresholds.write().await;
        *thresholds = CurrentThresholds::default();

        let mut adjustments = self.adjustment_history.write().await;
        adjustments.push_back(ThresholdAdjustment::Maintain);

        Ok(())
    }
}
