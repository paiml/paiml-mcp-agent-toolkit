/// Performance sample for adaptive threshold calculation
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: Instant,
    pub analysis_duration_ms: u64,
    pub cache_hit_ratio: f32,
    pub memory_usage_mb: f32,
    pub cpu_utilization: f32,
    pub queue_depth: usize,
}

/// Adaptive threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Target performance threshold (ms)
    pub target_analysis_time_ms: u64,
    /// Minimum cache hit ratio before adjustment
    pub min_cache_hit_ratio: f32,
    /// Maximum memory usage before scaling back
    pub max_memory_mb: f32,
    /// CPU utilization threshold for backpressure
    pub max_cpu_utilization: f32,
    /// Sample window size for averaging
    pub sample_window_size: usize,
    /// Adjustment sensitivity (0.0 - 1.0)
    pub adjustment_sensitivity: f32,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            target_analysis_time_ms: 100, // 100ms target
            min_cache_hit_ratio: 0.6,     // 60% cache hits
            max_memory_mb: 512.0,         // 512MB limit
            max_cpu_utilization: 0.8,     // 80% CPU max
            sample_window_size: 50,       // 50 sample rolling window
            adjustment_sensitivity: 0.1,  // 10% adjustment steps
        }
    }
}

/// Threshold adjustment recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdAdjustment {
    /// Increase cache size and permits
    ScaleUp {
        cache_factor: f32,
        permit_factor: f32,
    },
    /// Decrease cache size and permits
    ScaleDown {
        cache_factor: f32,
        permit_factor: f32,
    },
    /// Increase compression ratio
    MoreCompression { compression_level: u8 },
    /// Reduce compression for speed
    LessCompression { compression_level: u8 },
    /// No adjustment needed
    Maintain,
}

/// Performance monitoring and adaptive threshold manager
pub struct AdaptiveThresholdManager {
    config: AdaptiveConfig,
    performance_history: Arc<RwLock<VecDeque<PerformanceSample>>>,
    current_thresholds: Arc<RwLock<CurrentThresholds>>,
    adjustment_history: Arc<RwLock<VecDeque<ThresholdAdjustment>>>,
}

/// Current active thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentThresholds {
    pub hot_cache_size: usize,
    pub high_priority_permits: usize,
    pub low_priority_permits: usize,
    pub compression_level: u8,
    pub archive_after_hours: u32,
    pub cleanup_interval_minutes: u32,
}

impl Default for CurrentThresholds {
    fn default() -> Self {
        Self {
            hot_cache_size: 1000,
            high_priority_permits: 10,
            low_priority_permits: 2,
            compression_level: 4,         // Balanced LZ4 level
            archive_after_hours: 24 * 30, // 30 days
            cleanup_interval_minutes: 60, // 1 hour
        }
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub avg_analysis_duration_ms: f32,
    pub avg_cache_hit_ratio: f32,
    pub avg_memory_usage_mb: f32,
    pub avg_cpu_utilization: f32,
    pub total_samples: usize,
    pub recent_adjustments_count: usize,
    pub performance_trend: PerformanceTrend,
}

impl Default for PerformanceStatistics {
    fn default() -> Self {
        Self {
            avg_analysis_duration_ms: 0.0,
            avg_cache_hit_ratio: 0.0,
            avg_memory_usage_mb: 0.0,
            avg_cpu_utilization: 0.0,
            total_samples: 0,
            recent_adjustments_count: 0,
            performance_trend: PerformanceTrend::Stable,
        }
    }
}

impl PerformanceStatistics {
    /// Format statistics for diagnostic display
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_diagnostic(&self) -> String {
        let trend_indicator = match self.performance_trend {
            PerformanceTrend::Improving => "📈 IMPROVING",
            PerformanceTrend::Stable => "➡️ STABLE",
            PerformanceTrend::Degrading => "📉 DEGRADING",
        };

        format!(
            "Adaptive Thresholds:\n\
             - Performance: {}\n\
             - Avg duration: {:.1}ms\n\
             - Cache hit ratio: {:.1}%\n\
             - Memory usage: {:.1}MB\n\
             - CPU utilization: {:.1}%\n\
             - Total samples: {}\n\
             - Recent adjustments: {}",
            trend_indicator,
            self.avg_analysis_duration_ms,
            self.avg_cache_hit_ratio * 100.0,
            self.avg_memory_usage_mb,
            self.avg_cpu_utilization * 100.0,
            self.total_samples,
            self.recent_adjustments_count
        )
    }
}
