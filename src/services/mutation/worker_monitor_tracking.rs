impl WorkerMetrics {
    /// Create new worker metrics
    pub fn new(id: usize) -> Self {
        Self {
            id,
            state: WorkerState::Idle,
            processed_count: 0,
            failed_count: 0,
            last_heartbeat: Instant::now(),
            recent_errors: Vec::new(),
            avg_processing_time_ms: 0.0,
            heartbeat_count: 0,
            custom_metrics: HashMap::new(),
        }
    }

    /// Record a successful task completion
    pub fn record_success(&mut self, processing_time_ms: u64) {
        self.processed_count += 1;
        self.state = WorkerState::Idle;

        // Update average processing time
        self.avg_processing_time_ms = (self.avg_processing_time_ms
            * (self.processed_count as f64 - 1.0)
            + processing_time_ms as f64)
            / self.processed_count as f64;
    }

    /// Record a task failure
    pub fn record_failure(&mut self, error: &str) {
        self.failed_count += 1;
        self.state = WorkerState::Idle;

        // Add error to recent errors (keep only most recent 5)
        self.recent_errors.push(error.to_string());
        if self.recent_errors.len() > 5 {
            self.recent_errors.remove(0);
        }
    }

    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.heartbeat_count += 1;
    }

    /// Check if worker appears to be stalled
    pub fn is_stalled(&self, timeout: Duration) -> bool {
        if self.state == WorkerState::Processing {
            self.last_heartbeat.elapsed() > timeout
        } else {
            false
        }
    }

    /// Set worker state
    pub fn set_state(&mut self, state: WorkerState) {
        self.state = state;
        self.update_heartbeat();
    }

    /// Add or update a custom metric
    pub fn set_custom_metric(&mut self, key: &str, value: &str) {
        self.custom_metrics
            .insert(key.to_string(), value.to_string());
    }

    /// Get time since last heartbeat
    pub fn time_since_heartbeat(&self) -> Duration {
        self.last_heartbeat.elapsed()
    }
}
