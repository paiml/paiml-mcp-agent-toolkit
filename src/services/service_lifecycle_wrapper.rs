impl<S: Service> LifecycleWrapper<S> {
    /// Create a new lifecycle wrapper for a service
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(service: S, health_check_interval: Duration) -> Self {
        Self {
            service: Arc::new(RwLock::new(service)),
            state: Arc::new(RwLock::new(ServiceState::Uninitialized)),
            running: Arc::new(AtomicBool::new(false)),
            start_time: std::time::SystemTime::now(),
            health_check_interval,
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
        }
    }

    /// Start the service with lifecycle management
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ServiceState::Starting;
        drop(state);

        info!("Starting service lifecycle");

        // Mark as running
        self.running.store(true, Ordering::Relaxed);

        // Update state to running
        let mut state = self.state.write().await;
        *state = ServiceState::Running;
        drop(state);

        // Start health check loop
        self.start_health_check_loop().await;

        info!("Service lifecycle started");
        Ok(())
    }

    /// Stop the service gracefully
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ServiceState::Stopping;
        drop(state);

        info!("Stopping service lifecycle");

        // Signal to stop
        self.running.store(false, Ordering::Relaxed);

        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Update state
        let mut state = self.state.write().await;
        *state = ServiceState::Stopped;
        drop(state);

        info!("Service lifecycle stopped");
        Ok(())
    }

    /// Get current service state
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_state(&self) -> ServiceState {
        *self.state.read().await
    }

    /// Get health status
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_health(&self) -> HealthStatus {
        let state = *self.state.read().await;
        let metrics = self.metrics.read().await.clone();
        let uptime = self.start_time.elapsed().unwrap_or_default().as_secs();

        HealthStatus {
            state,
            message: format!("Service in {state:?} state"),
            last_check: std::time::SystemTime::now(),
            uptime_seconds: uptime,
            metrics,
        }
    }

    /// Start the health check loop
    async fn start_health_check_loop(&self) {
        let running = self.running.clone();
        let state = self.state.clone();
        let metrics = self.metrics.clone();
        let interval_duration = self.health_check_interval;

        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);

            while running.load(Ordering::Relaxed) {
                ticker.tick().await;

                // Perform health check
                let current_state = *state.read().await;

                // Simple health check based on metrics
                let current_metrics = metrics.read().await.clone();
                let success_rate = current_metrics.success_rate();

                let new_state = if success_rate < 0.5 && current_metrics.request_count > 10 {
                    ServiceState::Failed
                } else if success_rate < 0.8 && current_metrics.request_count > 10 {
                    ServiceState::Degraded
                } else if current_state == ServiceState::Running
                    || current_state == ServiceState::Degraded
                {
                    ServiceState::Running
                } else {
                    current_state
                };

                if new_state != current_state {
                    let mut state_guard = state.write().await;
                    *state_guard = new_state;
                    info!(
                        "Service state changed: {:?} -> {:?}",
                        current_state, new_state
                    );
                }
            }
        });
    }
}

#[async_trait]
impl<S> Service for LifecycleWrapper<S>
where
    S: Service + Send + Sync,
    S::Input: Send + Sync + 'static,
    S::Output: Send + Sync + 'static,
    S::Error: Send + Sync + 'static,
{
    type Input = S::Input;
    type Output = S::Output;
    type Error = S::Error;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let start = std::time::Instant::now();

        // Check if service is in a runnable state
        let current_state = *self.state.read().await;
        if current_state != ServiceState::Running && current_state != ServiceState::Degraded {
            warn!("Service called while in {:?} state", current_state);
        }

        // Process through wrapped service
        let service = self.service.read().await;
        let result = service.process(input).await;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.record_request(start.elapsed(), result.is_ok());

        result
    }
}
