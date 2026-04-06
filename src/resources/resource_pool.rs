// Resource pool for sharing resources across agents
pub struct ResourcePool {
    _total_limits: ResourceLimits,
    allocated: Arc<RwLock<Vec<(uuid::Uuid, ResourceLimits)>>>,
    available: Arc<RwLock<ResourceLimits>>,
}

impl ResourcePool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(total_limits: ResourceLimits) -> Self {
        Self {
            available: Arc::new(RwLock::new(total_limits.clone())),
            _total_limits: total_limits,
            allocated: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn request(
        &self,
        agent_id: uuid::Uuid,
        requested: ResourceLimits,
    ) -> Result<ResourceLimits, ResourceError> {
        let mut available = self.available.write();

        // Check if resources are available
        if requested.cpu.cores > available.cpu.cores {
            return Err(ResourceError::NotAvailable(
                "Insufficient CPU cores".to_string(),
            ));
        }
        if requested.memory.max_bytes > available.memory.max_bytes {
            return Err(ResourceError::NotAvailable(
                "Insufficient memory".to_string(),
            ));
        }

        // Allocate resources
        available.cpu.cores -= requested.cpu.cores;
        available.memory.max_bytes -= requested.memory.max_bytes;
        available.network.ingress_bytes_per_sec -= requested
            .network
            .ingress_bytes_per_sec
            .min(available.network.ingress_bytes_per_sec);
        available.network.egress_bytes_per_sec -= requested
            .network
            .egress_bytes_per_sec
            .min(available.network.egress_bytes_per_sec);

        self.allocated.write().push((agent_id, requested.clone()));

        Ok(requested)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn release(&self, agent_id: uuid::Uuid) -> Result<(), ResourceError> {
        let mut allocated = self.allocated.write();
        let mut available = self.available.write();

        if let Some(pos) = allocated.iter().position(|(id, _)| *id == agent_id) {
            let (_, limits) = allocated.remove(pos);

            // Return resources to pool
            available.cpu.cores += limits.cpu.cores;
            available.memory.max_bytes += limits.memory.max_bytes;
            available.network.ingress_bytes_per_sec += limits.network.ingress_bytes_per_sec;
            available.network.egress_bytes_per_sec += limits.network.egress_bytes_per_sec;
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_available(&self) -> ResourceLimits {
        self.available.read().clone()
    }

    pub fn get_allocated(&self) -> Vec<(uuid::Uuid, ResourceLimits)> {
        self.allocated.read().clone()
    }
}
