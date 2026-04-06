impl WorkerMonitor {
    /// Create a new worker monitor
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(worker_count: usize, stall_timeout: Duration) -> Self {
        Self {
            workers: RwLock::new(HashMap::new()),
            stall_timeout,
            worker_count,
        }
    }

    /// Initialize all workers
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn initialize_workers(&self) {
        let mut workers = self.workers.write().await;

        for id in 0..self.worker_count {
            workers.insert(id, WorkerMetrics::new(id));
        }
    }

    /// Record heartbeat from a worker
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn record_heartbeat(&self, worker_id: usize) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.update_heartbeat();
        }
    }

    /// Record worker starting to process a task
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn record_start_processing(&self, worker_id: usize) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.set_state(WorkerState::Processing);
        }
    }

    /// Record successful task completion
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn record_success(&self, worker_id: usize, processing_time_ms: u64) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.record_success(processing_time_ms);
        }
    }

    /// Record task failure
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn record_failure(&self, worker_id: usize, error: &str) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.record_failure(error);
        }
    }

    /// Mark worker as failed
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn mark_failed(&self, worker_id: usize, reason: &str) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.record_failure(reason);
            worker.set_state(WorkerState::Failed);
        }
    }

    /// Mark worker as terminated
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn mark_terminated(&self, worker_id: usize) {
        let mut workers = self.workers.write().await;

        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.set_state(WorkerState::Terminated);
        }
    }

    /// Get metrics for a specific worker
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_worker_metrics(&self, worker_id: usize) -> Option<WorkerMetrics> {
        let workers = self.workers.read().await;
        workers.get(&worker_id).cloned()
    }

    /// Get metrics for all workers
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_all_metrics(&self) -> Vec<WorkerMetrics> {
        let workers = self.workers.read().await;
        workers.values().cloned().collect()
    }

    /// Get IDs of stalled workers
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_stalled_workers(&self) -> Vec<usize> {
        let workers = self.workers.read().await;

        workers
            .values()
            .filter(|w| w.is_stalled(self.stall_timeout))
            .map(|w| w.id)
            .collect()
    }

    /// Get count of workers in each state
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_state_counts(&self) -> HashMap<WorkerState, usize> {
        let workers = self.workers.read().await;
        let mut counts = HashMap::new();

        for worker in workers.values() {
            *counts.entry(worker.state).or_insert(0) += 1;
        }

        counts
    }

    /// Calculate overall health score (0-100)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub async fn calculate_health_score(&self) -> f64 {
        let workers = self.workers.read().await;
        let total = workers.len();

        if total == 0 {
            return 0.0;
        }

        let healthy_count = workers
            .values()
            .filter(|w| !w.is_stalled(self.stall_timeout) && w.state != WorkerState::Failed)
            .count();

        (healthy_count as f64 / total as f64) * 100.0
    }

    /// Run monitoring task periodically
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run_monitoring_task(
        monitor: Arc<Self>,
        interval: Duration,
        on_stalled: impl Fn(usize) + Send + Sync + 'static,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);

            loop {
                timer.tick().await;

                // Check for stalled workers
                let stalled = monitor.get_stalled_workers().await;
                for worker_id in stalled {
                    on_stalled(worker_id);
                }

                // Calculate and log health metrics
                let health_score = monitor.calculate_health_score().await;
                let state_counts = monitor.get_state_counts().await;

                let idle_count = *state_counts.get(&WorkerState::Idle).unwrap_or(&0);
                let processing_count = *state_counts.get(&WorkerState::Processing).unwrap_or(&0);
                let failed_count = *state_counts.get(&WorkerState::Failed).unwrap_or(&0);

                if health_score < 80.0 || failed_count > 0 {
                    eprintln!(
                        "⚠️ Worker health: {:.1}% (Idle: {}, Processing: {}, Failed: {})",
                        health_score, idle_count, processing_count, failed_count
                    );
                }
            }
        })
    }
}
