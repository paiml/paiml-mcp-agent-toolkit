// CacheOrchestrator: Default impls, constructor, and public methods

impl Default for WorkloadProfile {
    fn default() -> Self {
        Self {
            request_rate: 100.0,
            working_set_size: 100 * 1024 * 1024, // 100MB
            temporal_locality: 0.7,
            spatial_locality: 0.5,
            read_write_ratio: 0.8, // 80% reads
            target_hit_rate: 0.9,
            latency_sensitivity: 0.8,
        }
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            auto_strategy_switching: true,
            evaluation_interval: Duration::from_secs(300), // 5 minutes
            min_improvement_threshold: 0.05,               // 5% improvement
            evaluation_window: 10,
            enable_prediction: true,
        }
    }
}

impl CacheOrchestrator {
    /// Create a new cache orchestrator
    #[must_use]
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            workload_profile: RwLock::new(WorkloadProfile::default()),
            strategies: RwLock::new(FxHashMap::default()),
            metrics: RwLock::new(PerformanceMetrics::default()),
            evaluation_history: RwLock::new(Vec::new()),
            config,
            counters: PerformanceCounters {
                strategy_switches: AtomicU64::new(0),
                evaluations_performed: AtomicU64::new(0),
                recommendations_generated: AtomicU64::new(0),
                performance_improvements: AtomicU64::new(0),
            },
        }
    }

    /// Register a cache strategy
    pub fn register_strategy(&self, strategy: Box<dyn CacheStrategy + Send + Sync>) -> Result<()> {
        let strategy_id = strategy.strategy_id().to_string();
        self.strategies
            .write()
            .insert(strategy_id.clone(), strategy);
        info!("Registered cache strategy: {}", strategy_id);
        Ok(())
    }

    /// Analyze workload and recommend optimal strategy
    pub async fn recommend_strategy(&self) -> Result<StrategyRecommendation> {
        let workload = self.workload_profile.read().clone();
        let recommendation = self.analyze_workload_and_recommend(&workload).await?;

        self.counters
            .recommendations_generated
            .fetch_add(1, Ordering::Relaxed);

        info!(
            "Generated strategy recommendation: {:?}",
            recommendation.eviction_policy
        );
        Ok(recommendation)
    }

    /// Update workload profile based on current metrics
    pub async fn update_workload_profile(&self, new_profile: WorkloadProfile) -> Result<()> {
        {
            let mut current_profile = self.workload_profile.write();
            *current_profile = new_profile;
        } // Drop lock before await

        // If auto-switching is enabled, evaluate if we need a new strategy
        if self.config.auto_strategy_switching {
            self.evaluate_and_switch_if_needed().await?;
        }

        Ok(())
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().clone()
    }

    /// Run continuous optimization
    pub async fn run_optimization_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(self.config.evaluation_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.perform_optimization_cycle().await {
                warn!("Optimization cycle failed: {}", e);
            }
        }
    }

    /// Force strategy evaluation and potential switch
    pub async fn evaluate_and_switch_if_needed(&self) -> Result<()> {
        let recommendation = self.recommend_strategy().await?;
        let _current_metrics = self.get_performance_metrics();

        // Check if switching would provide significant improvement
        if recommendation.expected_improvement > self.config.min_improvement_threshold
            && recommendation.confidence > 0.7
        {
            info!(
                "Switching cache strategy due to expected {:.2}% improvement",
                recommendation.expected_improvement * 100.0
            );

            self.switch_to_recommended_strategy(recommendation).await?;
            self.counters
                .strategy_switches
                .fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Get orchestrator statistics
    pub fn get_orchestrator_stats(&self) -> OrchestratorStats {
        OrchestratorStats {
            strategy_switches: self.counters.strategy_switches.load(Ordering::Relaxed),
            evaluations_performed: self.counters.evaluations_performed.load(Ordering::Relaxed),
            recommendations_generated: self
                .counters
                .recommendations_generated
                .load(Ordering::Relaxed),
            performance_improvements: self
                .counters
                .performance_improvements
                .load(Ordering::Relaxed),
            current_metrics: self.get_performance_metrics(),
            workload_profile: self.workload_profile.read().clone(),
        }
    }
}

/// Statistics from the cache orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorStats {
    pub strategy_switches: u64,
    pub evaluations_performed: u64,
    pub recommendations_generated: u64,
    pub performance_improvements: u64,
    pub current_metrics: PerformanceMetrics,
    pub workload_profile: WorkloadProfile,
}
