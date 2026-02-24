impl ResourceAllocation {
    fn new(
        operation_id: String,
        permit: tokio::sync::OwnedSemaphorePermit,
        active_operations: Arc<RwLock<HashMap<String, OperationContext>>>,
    ) -> Self {
        Self {
            operation_id,
            permit,
            active_operations,
        }
    }
}

impl Drop for ResourceAllocation {
    fn drop(&mut self) {
        // Remove operation from active list when dropped
        let operation_id = self.operation_id.clone();
        let active_ops = self.active_operations.clone();

        tokio::spawn(async move {
            let mut ops = active_ops.write().await;
            ops.remove(&operation_id);
        });
    }
}

impl ResourceEnforcementStats {
    /// Format stats for diagnostic display
    #[must_use]
    pub fn format_diagnostic(&self) -> String {
        let success_rate = if self.total_requests > 0 {
            (self.allowed_requests as f64 / self.total_requests as f64) * 100.0
        } else {
            100.0
        };

        format!(
            "Resource Control Stats (5min window):\n\
             - Total requests: {}\n\
             - Success rate: {:.1}%\n\
             - Allowed: {}, Throttled: {}, Queued: {}, Rejected: {}\n\
             - Active operations: {}",
            self.total_requests,
            success_rate,
            self.allowed_requests,
            self.throttled_requests,
            self.queued_requests,
            self.rejected_requests,
            self.current_active_operations
        )
    }
}

impl ResourceControllerFactory {
    /// Create controller with default limits
    #[must_use]
    pub fn create_default() -> PlatformResourceController {
        PlatformResourceController::new(ResourceLimits::default())
    }

    /// Create controller optimized for development
    #[must_use]
    pub fn create_dev_optimized() -> PlatformResourceController {
        let limits = ResourceLimits {
            max_memory_mb: 512.0,   // Lower memory for dev
            max_concurrent_ops: 5,  // Fewer concurrent ops
            check_interval_secs: 2, // More frequent checks
            ..Default::default()
        };
        PlatformResourceController::new(limits)
    }

    /// Create controller optimized for production
    #[must_use]
    pub fn create_prod_optimized() -> PlatformResourceController {
        let limits = ResourceLimits {
            max_memory_mb: 2048.0,         // Higher memory for prod
            max_concurrent_ops: 50,        // More concurrent ops
            check_interval_secs: 10,       // Less frequent checks
            cpu_warning_threshold: 0.5,    // Conservative CPU warning
            memory_warning_threshold: 0.6, // Conservative memory warning
            ..Default::default()
        };
        PlatformResourceController::new(limits)
    }

    /// Create controller for CI/CD environments
    #[must_use]
    pub fn create_ci_optimized() -> PlatformResourceController {
        let limits = ResourceLimits {
            max_memory_mb: 1024.0,
            max_cpu_utilization: 0.9, // Can use more CPU in CI
            max_concurrent_ops: 10,
            check_interval_secs: 5,
            cpu_warning_threshold: 0.8,
            memory_warning_threshold: 0.8,
        };
        PlatformResourceController::new(limits)
    }
}
