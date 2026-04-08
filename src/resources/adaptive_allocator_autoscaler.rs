// Auto-scaling manager
/// Auto scaler.
pub struct AutoScaler {
    allocator: Arc<AdaptiveAllocator>,
    manager: Arc<ResourceManager>,
    config: AutoScalerConfig,
    last_adjustment: Arc<RwLock<Option<Instant>>>,
}

#[derive(Clone)]
/// Configuration for auto scaler.
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
            scale_up_threshold: 0.8,   // Scale up at 80% usage
            scale_down_threshold: 0.3, // Scale down at 30% usage
        }
    }
}

impl AutoScaler {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    #[allow(clippy::cast_possible_truncation)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            self.allocator
                .record_usage(usage, limits.clone(), 1.0 - overall_util);

            // Check if adjustment is needed
            if overall_util > self.config.scale_up_threshold
                || overall_util < self.config.scale_down_threshold
            {
                if let Some(new_limits) = self.allocator.suggest_adjustment(&limits) {
                    if let Ok(()) = self.manager.update_limits(new_limits) {
                        *self.last_adjustment.write() = Some(Instant::now());
                    }
                }
            }
        }
    }
}
