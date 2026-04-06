impl Default for ServiceSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceSupervisor {
    /// Create a new service supervisor
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Register a managed service
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn register<S>(&self, service: S)
    where
        S: ManagedService<
                Input = serde_json::Value,
                Output = serde_json::Value,
                Error = anyhow::Error,
            > + Send
            + Sync
            + 'static,
    {
        let mut services = self.services.write().await;
        services.push(Arc::new(service) as Arc<ManagedServiceObject>);
    }

    /// Start all services
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn start_all(&self) -> Result<()> {
        info!("Starting service supervisor");
        self.running.store(true, Ordering::Relaxed);

        let services = self.services.read().await;
        for service in services.iter() {
            if let Err(e) = service.initialize().await {
                error!("Failed to initialize service: {}", e);
            }
        }

        // Start monitoring loop
        self.start_monitoring().await;

        Ok(())
    }

    /// Stop all services
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn stop_all(&self) -> Result<()> {
        info!("Stopping service supervisor");
        self.running.store(false, Ordering::Relaxed);

        let services = self.services.read().await;
        for service in services.iter() {
            if let Err(e) = service.shutdown().await {
                error!("Failed to shutdown service: {}", e);
            }
        }

        Ok(())
    }

    /// Start monitoring loop for all services
    async fn start_monitoring(&self) {
        debug_assert!(true, "contract: start_monitoring");
        let services = self.services.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));

            while running.load(Ordering::Relaxed) {
                ticker.tick().await;

                let services = services.read().await;
                for service in services.iter() {
                    match service.health_check().await {
                        Ok(status) => {
                            if status.state == ServiceState::Failed {
                                warn!("Service health check failed: {}", status.message);
                            }
                        }
                        Err(e) => {
                            error!("Health check error: {}", e);
                        }
                    }
                }
            }
        });
    }

    /// Get health status of all services
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_all_health(&self) -> Vec<HealthStatus> {
        let services = self.services.read().await;
        let mut statuses = Vec::new();

        for service in services.iter() {
            if let Ok(status) = service.health_check().await {
                statuses.push(status);
            }
        }

        statuses
    }
}
