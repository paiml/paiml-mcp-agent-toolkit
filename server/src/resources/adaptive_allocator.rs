use super::*;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

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
            history_window: Duration::from_secs(300), // 5 minutes
            prediction_horizon: Duration::from_secs(60), // 1 minute
            adjustment_threshold: 0.1, // 10% change triggers adjustment
            min_adjustment: 0.8, // Min 80% of current
            max_adjustment: 1.5, // Max 150% of current
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
    limits: ResourceLimits,
    performance_score: f32,
}

struct ResourcePredictor {
    cpu_trend: f32,
    memory_trend: f32,
    network_trend: f32,
    io_trend: f32,
}

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
        limits: ResourceLimits,
        performance_score: f32,
    ) {
        let mut history = self.history.write();
        
        history.samples.push_back(ResourceSample {
            timestamp: Instant::now(),
            usage,
            limits,
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
    
    fn update_predictions(&self, history: &ResourceHistory) {
        if history.samples.len() < 10 {
            return; // Not enough data
        }
        
        let mut predictor = self.predictor.write();
        
        // Simple linear regression for trends
        let recent: Vec<_> = history.samples.iter()
            .rev()
            .take(20)
            .collect();
        
        if recent.len() >= 2 {
            // CPU trend
            let cpu_start = recent.last().unwrap().usage.cpu_percent;
            let cpu_end = recent.first().unwrap().usage.cpu_percent;
            predictor.cpu_trend = (cpu_end - cpu_start) / cpu_start.max(0.1);
            
            // Memory trend
            let mem_start = recent.last().unwrap().usage.memory_bytes as f32;
            let mem_end = recent.first().unwrap().usage.memory_bytes as f32;
            predictor.memory_trend = (mem_end - mem_start) / mem_start.max(1.0);
            
            // Network trend
            let net_start = (recent.last().unwrap().usage.network_ingress_bytes + 
                            recent.last().unwrap().usage.network_egress_bytes) as f32;
            let net_end = (recent.first().unwrap().usage.network_ingress_bytes + 
                          recent.first().unwrap().usage.network_egress_bytes) as f32;
            predictor.network_trend = (net_end - net_start) / net_start.max(1.0);
            
            // I/O trend
            let io_start = (recent.last().unwrap().usage.disk_read_bytes + 
                           recent.last().unwrap().usage.disk_write_bytes) as f32;
            let io_end = (recent.first().unwrap().usage.disk_read_bytes + 
                         recent.first().unwrap().usage.disk_write_bytes) as f32;
            predictor.io_trend = (io_end - io_start) / io_start.max(1.0);
        }
    }
    
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
                .min((adjusted.memory.max_bytes as f32) * self.config.max_adjustment) as usize;
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
    
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let history = self.history.read();
        
        if history.samples.is_empty() {
            return PerformanceStats::default();
        }
        
        let avg_score = history.samples.iter()
            .map(|s| s.performance_score)
            .sum::<f32>() / history.samples.len() as f32;
        
        let avg_cpu = history.samples.iter()
            .map(|s| s.usage.cpu_percent)
            .sum::<f32>() / history.samples.len() as f32;
        
        let avg_memory = history.samples.iter()
            .map(|s| s.usage.memory_bytes)
            .sum::<usize>() / history.samples.len();
        
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

#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub average_performance_score: f32,
    pub average_cpu_usage: f32,
    pub average_memory_usage: usize,
    pub cpu_trend: f32,
    pub memory_trend: f32,
    pub sample_count: usize,
}

// Auto-scaling manager
pub struct AutoScaler {
    allocator: Arc<AdaptiveAllocator>,
    manager: Arc<ResourceManager>,
    config: AutoScalerConfig,
    last_adjustment: Arc<RwLock<Option<Instant>>>,
}

#[derive(Clone)]
pub struct AutoScalerConfig {
    pub enabled: bool,
    pub check_interval: Duration,
    pub cooldown_period: Duration,
    pub scale_up_threshold: f32,
    pub scale_down_threshold: f32,
}

impl Default for AutoScalerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(10),
            cooldown_period: Duration::from_secs(60),
            scale_up_threshold: 0.8, // Scale up at 80% usage
            scale_down_threshold: 0.3, // Scale down at 30% usage
        }
    }
}

impl AutoScaler {
    pub fn new(
        allocator: Arc<AdaptiveAllocator>,
        manager: Arc<ResourceManager>,
        config: AutoScalerConfig,
    ) -> Self {
        Self {
            allocator,
            manager,
            config,
            last_adjustment: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn run(&self) {
        if !self.config.enabled {
            return;
        }
        
        loop {
            tokio::time::sleep(self.config.check_interval).await;
            
            // Check cooldown
            if let Some(last) = *self.last_adjustment.read() {
                if last.elapsed() < self.config.cooldown_period {
                    continue;
                }
            }
            
            // Get current usage and limits
            let usage = match self.manager.get_current_usage() {
                Ok(u) => u,
                Err(_) => continue,
            };
            
            let limits = self.manager.limits.read().clone();
            
            // Calculate utilization
            let cpu_util = usage.cpu_percent / 100.0;
            let mem_util = usage.memory_bytes as f32 / limits.memory.max_bytes as f32;
            let overall_util = (cpu_util + mem_util) / 2.0;
            
            // Record for learning
            self.allocator.record_usage(usage, limits.clone(), 1.0 - overall_util);
            
            // Check if adjustment is needed
            if overall_util > self.config.scale_up_threshold ||
               overall_util < self.config.scale_down_threshold {
                
                if let Some(new_limits) = self.allocator.suggest_adjustment(&limits) {
                    if let Ok(()) = self.manager.update_limits(new_limits) {
                        *self.last_adjustment.write() = Some(Instant::now());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_allocator() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());
        
        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 512 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 1000,
            network_egress_bytes: 1000,
            disk_read_bytes: 5000,
            disk_write_bytes: 5000,
            timestamp: std::time::SystemTime::now(),
        };
        
        let limits = ResourceLimits::default();
        
        // Record some usage
        for i in 0..20 {
            let mut u = usage.clone();
            u.cpu_percent = 50.0 + i as f32;
            allocator.record_usage(u, limits.clone(), 0.7);
        }
        
        // Should suggest scaling up due to increasing CPU trend
        let suggestion = allocator.suggest_adjustment(&limits);
        assert!(suggestion.is_some());
    }
    
    #[test]
    fn test_performance_stats() {
        let allocator = AdaptiveAllocator::new(AllocatorConfig::default());
        
        let usage = ResourceUsage {
            cpu_percent: 60.0,
            memory_bytes: 600 * 1024 * 1024,
            gpu_memory_bytes: None,
            gpu_compute_percent: None,
            network_ingress_bytes: 0,
            network_egress_bytes: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            timestamp: std::time::SystemTime::now(),
        };
        
        let limits = ResourceLimits::default();
        
        allocator.record_usage(usage, limits, 0.8);
        
        let stats = allocator.get_performance_stats();
        assert_eq!(stats.sample_count, 1);
        assert_eq!(stats.average_cpu_usage, 60.0);
        assert_eq!(stats.average_performance_score, 0.8);
    }
}