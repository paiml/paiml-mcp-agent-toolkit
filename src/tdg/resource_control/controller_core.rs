impl PlatformResourceController {
    /// Create new resource controller
    #[must_use]
    pub fn new(limits: ResourceLimits) -> Self {
        let semaphore = Arc::new(Semaphore::new(limits.max_concurrent_ops));

        let initial_usage = ResourceUsage {
            timestamp: Instant::now(),
            memory_mb: 0.0,
            cpu_utilization: 0.0,
            active_operations: 0,
            memory_pressure: ResourcePressure::Low,
            cpu_pressure: ResourcePressure::Low,
        };

        Self {
            limits,
            current_usage: Arc::new(RwLock::new(initial_usage)),
            operation_semaphore: semaphore,
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            enforcement_history: Arc::new(RwLock::new(Vec::new())),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start resource monitoring background task
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut monitoring_guard = self.monitoring_active.write().await;
        if *monitoring_guard {
            return Ok(()); // Already monitoring
        }
        *monitoring_guard = true;
        drop(monitoring_guard);

        let usage_arc = self.current_usage.clone();
        let limits = self.limits.clone();
        let monitoring_flag = self.monitoring_active.clone();
        let active_ops = self.active_operations.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(limits.check_interval_secs));

            loop {
                interval.tick().await;

                // Check if monitoring should continue
                {
                    let monitoring = monitoring_flag.read().await;
                    if !*monitoring {
                        break;
                    }
                }

                // Update resource usage
                let new_usage = Self::measure_current_usage(&limits, &active_ops).await;

                {
                    let mut usage = usage_arc.write().await;
                    *usage = new_usage;
                }
            }
        });

        Ok(())
    }

    /// Stop resource monitoring
    pub async fn stop_monitoring(&self) {
        let mut monitoring_guard = self.monitoring_active.write().await;
        *monitoring_guard = false;
    }

    /// Request resource allocation for operation
    pub async fn request_resources(
        &self,
        operation_id: String,
        op_type: OperationType,
        priority: OperationPriority,
        estimated_memory_mb: f64,
    ) -> Result<ResourceAllocation> {
        let current_usage = self.current_usage.read().await.clone();

        // Check if operation should be allowed
        let action = self
            .evaluate_resource_request(&current_usage, &op_type, &priority, estimated_memory_mb)
            .await?;

        match action.clone() {
            ResourceAction::Allow => {
                // Acquire semaphore permit
                let permit = Arc::clone(&self.operation_semaphore)
                    .acquire_owned()
                    .await?;

                // Register operation
                let context = OperationContext {
                    id: operation_id.clone(),
                    operation_type: op_type,
                    started_at: Instant::now(),
                    estimated_memory_mb,
                    priority,
                };

                {
                    let mut active_ops = self.active_operations.write().await;
                    active_ops.insert(operation_id.clone(), context);
                }

                // Log enforcement event
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    "Operation allowed".to_string(),
                )
                .await;

                Ok(ResourceAllocation::new(
                    operation_id,
                    permit,
                    self.active_operations.clone(),
                ))
            }
            ResourceAction::Throttle { delay_ms } => {
                // Add delay before allowing
                sleep(Duration::from_millis(delay_ms)).await;

                // Retry allocation after throttling
                Box::pin(self.request_resources(
                    operation_id,
                    op_type,
                    priority,
                    estimated_memory_mb,
                ))
                .await
            }
            ResourceAction::Queue { estimated_wait_ms } => {
                // Log queuing event
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    format!("Operation queued, estimated wait: {estimated_wait_ms}ms"),
                )
                .await;

                // Wait and retry
                sleep(Duration::from_millis(estimated_wait_ms)).await;
                Box::pin(self.request_resources(
                    operation_id,
                    op_type,
                    priority,
                    estimated_memory_mb,
                ))
                .await
            }
            ResourceAction::Reject { reason } => {
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    reason.clone(),
                )
                .await;

                Err(anyhow::anyhow!("Resource request rejected: {reason}"))
            }
            ResourceAction::EmergencyStop => {
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    "Emergency resource stop triggered".to_string(),
                )
                .await;

                // Trigger emergency cleanup
                self.emergency_cleanup().await?;

                Err(anyhow::anyhow!(
                    "Operation rejected due to emergency resource conditions"
                ))
            }
        }
    }
}
