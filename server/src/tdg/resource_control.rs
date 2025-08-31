use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{interval, sleep};


/// Operation types for resource planning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Analysis,
    Commit,
    Background,
    Storage,
    Cleanup,
}

/// Resource limits for platform control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    /// Maximum CPU utilization (0.0 - 1.0)
    pub max_cpu_utilization: f64,
    /// Maximum concurrent operations
    pub max_concurrent_ops: usize,
    /// Memory warning threshold (0.0 - 1.0)
    pub memory_warning_threshold: f64,
    /// CPU warning threshold (0.0 - 1.0)  
    pub cpu_warning_threshold: f64,
    /// Resource check interval in seconds
    pub check_interval_secs: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024.0,      // 1GB limit
            max_cpu_utilization: 0.8,   // 80% CPU max
            max_concurrent_ops: 20,     // 20 concurrent operations
            memory_warning_threshold: 0.7, // 70% warning
            cpu_warning_threshold: 0.6,    // 60% warning
            check_interval_secs: 5,     // Check every 5 seconds
        }
    }
}

/// Current resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub memory_mb: f64,
    pub cpu_utilization: f64,
    pub active_operations: usize,
    pub memory_pressure: ResourcePressure,
    pub cpu_pressure: ResourcePressure,
}

/// Resource pressure levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourcePressure {
    Low,     // Below warning threshold
    Medium,  // Above warning, below limit
    High,    // At or above limit
    Critical, // Emergency conditions
}

/// Resource enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAction {
    /// Allow operation to proceed
    Allow,
    /// Throttle operation (add delay)
    Throttle { delay_ms: u64 },
    /// Queue operation for later
    Queue { estimated_wait_ms: u64 },
    /// Reject operation due to resource constraints
    Reject { reason: String },
    /// Emergency shutdown of non-critical operations
    EmergencyStop,
}

/// Platform resource controller
pub struct PlatformResourceController {
    limits: ResourceLimits,
    current_usage: Arc<RwLock<ResourceUsage>>,
    operation_semaphore: Arc<Semaphore>,
    active_operations: Arc<RwLock<HashMap<String, OperationContext>>>,
    enforcement_history: Arc<RwLock<Vec<EnforcementEvent>>>,
    monitoring_active: Arc<RwLock<bool>>,
}

/// Context for active operations
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub id: String,
    pub operation_type: OperationType,
    pub started_at: Instant,
    pub estimated_memory_mb: f64,
    pub priority: OperationPriority,
}


/// Operation priority for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationPriority {
    Critical,  // User commits, must succeed
    High,      // Interactive analysis
    Medium,    // Background analysis
    Low,       // Cleanup, maintenance
}

/// Resource enforcement event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementEvent {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub operation_id: String,
    pub action: ResourceAction,
    pub resource_usage: ResourceUsage,
    pub reason: String,
}

impl PlatformResourceController {
    /// Create new resource controller
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
        let action = self.evaluate_resource_request(
            &current_usage,
            &op_type,
            &priority,
            estimated_memory_mb,
        ).await?;

        match action.clone() {
            ResourceAction::Allow => {
                // Acquire semaphore permit
                let permit = Arc::clone(&self.operation_semaphore).acquire_owned().await?;
                
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
                ).await;

                Ok(ResourceAllocation::new(
                    operation_id,
                    permit,
                    self.active_operations.clone(),
                ))
            },
            ResourceAction::Throttle { delay_ms } => {
                // Add delay before allowing
                sleep(Duration::from_millis(delay_ms)).await;
                
                // Retry allocation after throttling
                Box::pin(self.request_resources(operation_id, op_type, priority, estimated_memory_mb)).await
            },
            ResourceAction::Queue { estimated_wait_ms } => {
                // Log queuing event
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    format!("Operation queued, estimated wait: {}ms", estimated_wait_ms),
                ).await;
                
                // Wait and retry
                sleep(Duration::from_millis(estimated_wait_ms)).await;
                Box::pin(self.request_resources(operation_id, op_type, priority, estimated_memory_mb)).await
            },
            ResourceAction::Reject { reason } => {
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    reason.clone(),
                ).await;
                
                Err(anyhow::anyhow!("Resource request rejected: {}", reason))
            },
            ResourceAction::EmergencyStop => {
                self.log_enforcement_event(
                    operation_id.clone(),
                    action,
                    current_usage,
                    "Emergency resource stop triggered".to_string(),
                ).await;
                
                // Trigger emergency cleanup
                self.emergency_cleanup().await?;
                
                Err(anyhow::anyhow!("Operation rejected due to emergency resource conditions"))
            }
        }
    }

    /// Evaluate whether to allow resource request
    async fn evaluate_resource_request(
        &self,
        current_usage: &ResourceUsage,
        op_type: &OperationType,
        priority: &OperationPriority,
        estimated_memory_mb: f64,
    ) -> Result<ResourceAction> {
        // Critical operations always get priority
        if *priority == OperationPriority::Critical {
            if current_usage.memory_pressure == ResourcePressure::Critical {
                return Ok(ResourceAction::EmergencyStop);
            }
            return Ok(ResourceAction::Allow);
        }

        // Check memory constraints
        let projected_memory = current_usage.memory_mb + estimated_memory_mb;
        if projected_memory > self.limits.max_memory_mb {
            if *priority <= OperationPriority::Medium {
                return Ok(ResourceAction::Reject {
                    reason: format!(
                        "Memory limit exceeded: {:.1}MB + {:.1}MB > {:.1}MB",
                        current_usage.memory_mb,
                        estimated_memory_mb,
                        self.limits.max_memory_mb
                    ),
                });
            } else {
                // High priority operations get queued
                let wait_time = self.estimate_resource_wait_time().await;
                return Ok(ResourceAction::Queue {
                    estimated_wait_ms: wait_time,
                });
            }
        }

        // Check CPU constraints
        if current_usage.cpu_utilization > self.limits.max_cpu_utilization {
            match priority {
                OperationPriority::Critical | OperationPriority::High => {
                    // Throttle high priority operations
                    let delay = ((current_usage.cpu_utilization - self.limits.max_cpu_utilization) * 1000.0) as u64;
                    return Ok(ResourceAction::Throttle {
                        delay_ms: delay.min(5000), // Max 5s delay
                    });
                },
                _ => {
                    return Ok(ResourceAction::Reject {
                        reason: format!(
                            "CPU utilization too high: {:.1}% > {:.1}%",
                            current_usage.cpu_utilization * 100.0,
                            self.limits.max_cpu_utilization * 100.0
                        ),
                    });
                }
            }
        }

        // Check operation limits
        if current_usage.active_operations >= self.limits.max_concurrent_ops {
            if *priority >= OperationPriority::High {
                let wait_time = self.estimate_operation_wait_time().await;
                return Ok(ResourceAction::Queue {
                    estimated_wait_ms: wait_time,
                });
            } else {
                return Ok(ResourceAction::Reject {
                    reason: format!(
                        "Too many concurrent operations: {} >= {}",
                        current_usage.active_operations,
                        self.limits.max_concurrent_ops
                    ),
                });
            }
        }

        Ok(ResourceAction::Allow)
    }

    /// Measure current system resource usage
    async fn measure_current_usage(
        limits: &ResourceLimits,
        active_ops: &Arc<RwLock<HashMap<String, OperationContext>>>,
    ) -> ResourceUsage {
        let ops = active_ops.read().await;
        let active_count = ops.len();
        
        // Estimate memory usage from active operations
        let estimated_memory: f64 = ops.values()
            .map(|ctx| ctx.estimated_memory_mb)
            .sum::<f64>() + 100.0; // Base memory usage
        
        // Estimate CPU usage based on operation types and age
        let estimated_cpu = ops.values()
            .map(|ctx| {
                let age_factor = (ctx.started_at.elapsed().as_secs() as f64 / 60.0).min(1.0);
                match ctx.operation_type {
                    OperationType::Analysis => 0.3 * age_factor,
                    OperationType::Commit => 0.1 * age_factor,
                    OperationType::Background => 0.2 * age_factor,
                    OperationType::Storage => 0.15 * age_factor,
                    OperationType::Cleanup => 0.05 * age_factor,
                }
            })
            .sum::<f64>()
            .min(1.0);

        // Calculate pressure levels
        let memory_pressure = if estimated_memory > limits.max_memory_mb {
            ResourcePressure::Critical
        } else if estimated_memory > limits.max_memory_mb * 0.9 {
            ResourcePressure::High
        } else if estimated_memory > limits.max_memory_mb * limits.memory_warning_threshold {
            ResourcePressure::Medium
        } else {
            ResourcePressure::Low
        };

        let cpu_pressure = if estimated_cpu > limits.max_cpu_utilization {
            ResourcePressure::Critical
        } else if estimated_cpu > limits.max_cpu_utilization * 0.9 {
            ResourcePressure::High
        } else if estimated_cpu > limits.max_cpu_utilization * limits.cpu_warning_threshold {
            ResourcePressure::Medium
        } else {
            ResourcePressure::Low
        };

        ResourceUsage {
            timestamp: Instant::now(),
            memory_mb: estimated_memory,
            cpu_utilization: estimated_cpu,
            active_operations: active_count,
            memory_pressure,
            cpu_pressure,
        }
    }

    /// Estimate time to wait for resources to become available
    async fn estimate_resource_wait_time(&self) -> u64 {
        let ops = self.active_operations.read().await;
        if ops.is_empty() {
            return 100; // Minimum wait
        }

        // Find oldest non-critical operation
        let oldest_age = ops.values()
            .filter(|ctx| ctx.priority != OperationPriority::Critical)
            .map(|ctx| ctx.started_at.elapsed().as_millis() as u64)
            .max()
            .unwrap_or(1000);

        // Estimate based on typical operation duration
        (oldest_age / 2).max(500).min(30000) // 0.5s to 30s range
    }

    /// Estimate time for operation slot to become available
    async fn estimate_operation_wait_time(&self) -> u64 {
        let available_permits = self.operation_semaphore.available_permits();
        if available_permits > 0 {
            return 100;
        }

        // Estimate based on average operation completion time
        let ops = self.active_operations.read().await;
        let avg_age = if ops.is_empty() {
            5000 // Default 5s estimate
        } else {
            let total_age: u64 = ops.values()
                .map(|ctx| ctx.started_at.elapsed().as_millis() as u64)
                .sum();
            total_age / ops.len() as u64
        };

        (avg_age / ops.len() as u64).max(1000).min(15000) // 1s to 15s range
    }

    /// Trigger emergency cleanup of non-critical operations
    async fn emergency_cleanup(&self) -> Result<()> {
        let ops = self.active_operations.read().await;
        let low_priority_count = ops.values()
            .filter(|ctx| ctx.priority == OperationPriority::Low)
            .count();

        // In a full implementation, this would send cancellation signals
        // For now, we just log the emergency action
        println!(
            "EMERGENCY: Would cancel {} low-priority operations due to resource pressure",
            low_priority_count
        );

        Ok(())
    }

    /// Log resource enforcement event
    async fn log_enforcement_event(
        &self,
        operation_id: String,
        action: ResourceAction,
        usage: ResourceUsage,
        reason: String,
    ) {
        let event = EnforcementEvent {
            timestamp: Instant::now(),
            operation_id,
            action,
            resource_usage: usage,
            reason,
        };

        let mut history = self.enforcement_history.write().await;
        history.push(event);

        // Keep only recent events
        if history.len() > 1000 {
            history.drain(..500); // Keep last 500 events
        }
    }

    /// Get current resource usage
    pub async fn get_current_usage(&self) -> ResourceUsage {
        self.current_usage.read().await.clone()
    }

    /// Get resource enforcement statistics
    pub async fn get_enforcement_stats(&self) -> ResourceEnforcementStats {
        let history = self.enforcement_history.read().await;
        let recent_events: Vec<_> = history.iter()
            .filter(|e| e.timestamp.elapsed() < Duration::from_secs(300)) // Last 5 minutes
            .collect();

        let total_requests = recent_events.len();
        let allowed = recent_events.iter()
            .filter(|e| matches!(e.action, ResourceAction::Allow))
            .count();
        let throttled = recent_events.iter()
            .filter(|e| matches!(e.action, ResourceAction::Throttle { .. }))
            .count();
        let queued = recent_events.iter()
            .filter(|e| matches!(e.action, ResourceAction::Queue { .. }))
            .count();
        let rejected = recent_events.iter()
            .filter(|e| matches!(e.action, ResourceAction::Reject { .. }))
            .count();

        ResourceEnforcementStats {
            total_requests,
            allowed_requests: allowed,
            throttled_requests: throttled,
            queued_requests: queued,
            rejected_requests: rejected,
            current_active_operations: {
                let ops = self.active_operations.read().await;
                ops.len()
            },
        }
    }

    /// Update resource limits at runtime
    pub async fn update_limits(&mut self, new_limits: ResourceLimits) {
        self.limits = new_limits.clone();
        
        // Update semaphore if concurrent ops limit changed
        if new_limits.max_concurrent_ops != self.limits.max_concurrent_ops {
            self.operation_semaphore = Arc::new(Semaphore::new(new_limits.max_concurrent_ops));
        }
    }
}

/// Resource allocation guard - automatically releases resources when dropped
pub struct ResourceAllocation {
    operation_id: String,
    #[allow(dead_code)]
    permit: tokio::sync::OwnedSemaphorePermit,
    active_operations: Arc<RwLock<HashMap<String, OperationContext>>>,
}

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

/// Resource enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEnforcementStats {
    pub total_requests: usize,
    pub allowed_requests: usize,
    pub throttled_requests: usize,
    pub queued_requests: usize,
    pub rejected_requests: usize,
    pub current_active_operations: usize,
}

impl ResourceEnforcementStats {
    /// Format stats for diagnostic display
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

/// Factory for creating resource controllers
pub struct ResourceControllerFactory;

impl ResourceControllerFactory {
    /// Create controller with default limits
    pub fn create_default() -> PlatformResourceController {
        PlatformResourceController::new(ResourceLimits::default())
    }

    /// Create controller optimized for development
    pub fn create_dev_optimized() -> PlatformResourceController {
        let limits = ResourceLimits {
            max_memory_mb: 512.0,       // Lower memory for dev
            max_concurrent_ops: 5,      // Fewer concurrent ops
            check_interval_secs: 2,     // More frequent checks
            ..Default::default()
        };
        PlatformResourceController::new(limits)
    }

    /// Create controller optimized for production
    pub fn create_prod_optimized() -> PlatformResourceController {
        let limits = ResourceLimits {
            max_memory_mb: 2048.0,      // Higher memory for prod
            max_concurrent_ops: 50,     // More concurrent ops
            check_interval_secs: 10,    // Less frequent checks
            cpu_warning_threshold: 0.5, // Conservative CPU warning
            memory_warning_threshold: 0.6, // Conservative memory warning
            ..Default::default()
        };
        PlatformResourceController::new(limits)
    }

    /// Create controller for CI/CD environments
    pub fn create_ci_optimized() -> PlatformResourceController {
        let limits = ResourceLimits {
            max_memory_mb: 1024.0,
            max_cpu_utilization: 0.9,   // Can use more CPU in CI
            max_concurrent_ops: 10,
            check_interval_secs: 5,
            cpu_warning_threshold: 0.8,
            memory_warning_threshold: 0.8,
        };
        PlatformResourceController::new(limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_resource_controller_creation() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        let usage = controller.get_current_usage().await;
        
        assert_eq!(usage.active_operations, 0);
        assert_eq!(usage.memory_pressure, ResourcePressure::Low);
    }

    #[tokio::test]
    async fn test_resource_allocation_success() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        controller.start_monitoring().await.unwrap();

        let allocation = controller.request_resources(
            "test-op-1".to_string(),
            OperationType::Analysis,
            OperationPriority::High,
            100.0,
        ).await.unwrap();

        let usage = controller.get_current_usage().await;
        assert_eq!(usage.active_operations, 1);
        
        drop(allocation);
        sleep(Duration::from_millis(100)).await; // Allow cleanup to complete
        
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        let limits = ResourceLimits {
            max_memory_mb: 200.0, // Small limit for testing
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        // Request more memory than limit
        let result = controller.request_resources(
            "test-op-memory".to_string(),
            OperationType::Analysis,
            OperationPriority::Low,
            300.0, // Exceeds 200MB limit
        ).await;

        assert!(result.is_err());
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_critical_priority_bypass() {
        let limits = ResourceLimits {
            max_memory_mb: 100.0, // Very small limit
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        // Critical operations should bypass normal limits
        let allocation = controller.request_resources(
            "critical-op".to_string(),
            OperationType::Commit,
            OperationPriority::Critical,
            150.0, // Exceeds limit but should be allowed
        ).await;

        assert!(allocation.is_ok());
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_operation_counting() {
        let limits = ResourceLimits {
            max_concurrent_ops: 2, // Only 2 operations allowed
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        let _alloc1 = controller.request_resources(
            "op-1".to_string(),
            OperationType::Analysis,
            OperationPriority::High,
            50.0,
        ).await.unwrap();

        let _alloc2 = controller.request_resources(
            "op-2".to_string(),
            OperationType::Analysis,
            OperationPriority::High,
            50.0,
        ).await.unwrap();

        // Third operation should be rejected or queued
        let result = controller.request_resources(
            "op-3".to_string(),
            OperationType::Background,
            OperationPriority::Low,
            50.0,
        ).await;

        assert!(result.is_err());
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_enforcement_stats() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        controller.start_monitoring().await.unwrap();

        // Make some resource requests
        let _alloc = controller.request_resources(
            "stats-test".to_string(),
            OperationType::Analysis,
            OperationPriority::Medium,
            100.0,
        ).await.unwrap();

        let stats = controller.get_enforcement_stats().await;
        assert!(stats.total_requests > 0);
        assert!(stats.allowed_requests > 0);

        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_factory_patterns() {
        let default_ctrl = ResourceControllerFactory::create_default();
        let dev_ctrl = ResourceControllerFactory::create_dev_optimized();
        let prod_ctrl = ResourceControllerFactory::create_prod_optimized();
        let ci_ctrl = ResourceControllerFactory::create_ci_optimized();

        // Test that all controllers can start monitoring
        default_ctrl.start_monitoring().await.unwrap();
        dev_ctrl.start_monitoring().await.unwrap();
        prod_ctrl.start_monitoring().await.unwrap();
        ci_ctrl.start_monitoring().await.unwrap();

        // Verify they can handle resource requests
        let _alloc1 = default_ctrl.request_resources(
            "factory-test-1".to_string(),
            OperationType::Analysis,
            OperationPriority::Medium,
            50.0,
        ).await.unwrap();

        let _alloc2 = dev_ctrl.request_resources(
            "factory-test-2".to_string(),
            OperationType::Analysis,
            OperationPriority::Medium,
            50.0,
        ).await.unwrap();

        // Cleanup
        default_ctrl.stop_monitoring().await;
        dev_ctrl.stop_monitoring().await;
        prod_ctrl.stop_monitoring().await;
        ci_ctrl.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_resource_monitoring_lifecycle() {
        let controller = PlatformResourceController::new(ResourceLimits::default());
        
        // Should start successfully
        controller.start_monitoring().await.unwrap();
        
        // Starting again should be idempotent
        controller.start_monitoring().await.unwrap();
        
        // Should stop cleanly
        controller.stop_monitoring().await;
        
        // Should be able to restart
        controller.start_monitoring().await.unwrap();
        controller.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_resource_pressure_levels() {
        let limits = ResourceLimits {
            max_memory_mb: 1000.0,
            memory_warning_threshold: 0.7, // 700MB warning
            ..Default::default()
        };
        let controller = PlatformResourceController::new(limits);
        controller.start_monitoring().await.unwrap();

        // Low pressure - under warning threshold
        let _alloc1 = controller.request_resources(
            "pressure-low".to_string(),
            OperationType::Analysis,
            OperationPriority::Medium,
            500.0, // 50% of limit
        ).await.unwrap();

        let usage1 = controller.get_current_usage().await;
        assert_eq!(usage1.memory_pressure, ResourcePressure::Low);

        // Medium pressure - over warning threshold
        let _alloc2 = controller.request_resources(
            "pressure-medium".to_string(),
            OperationType::Analysis,
            OperationPriority::Medium,
            250.0, // Total ~75% of limit
        ).await.unwrap();

        let usage2 = controller.get_current_usage().await;
        assert_eq!(usage2.memory_pressure, ResourcePressure::Medium);

        controller.stop_monitoring().await;
    }
}