use super::{AgentId, AgentSpec};
use dashmap::DashMap;
use std::sync::Arc;

pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentEntry>>,
    agents_by_name: Arc<DashMap<String, AgentId>>,
    agents_by_capability: Arc<DashMap<String, AgentId>>,
    agent_health: Arc<DashMap<String, AgentHealth>>,
}

struct AgentEntry {
    spec: AgentSpec,
    // Will add actor address later
}

struct AgentHealth {
    healthy: bool,
    #[allow(dead_code)]
    last_error: Option<String>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            agents_by_name: Arc::new(DashMap::new()),
            agents_by_capability: Arc::new(DashMap::new()),
            agent_health: Arc::new(DashMap::new()),
        }
    }

    pub async fn spawn_agent(&self, spec: AgentSpec) -> Result<AgentId, super::AgentError> {
        let id = spec.id;
        let entry = AgentEntry { spec };

        self.agents.insert(id, entry);
        Ok(id)
    }

    pub async fn get_agent(&self, name: &str) -> Option<AgentId> {
        self.agents_by_name.get(name).map(|entry| *entry.value())
    }

    pub async fn register_agent_with_name(&self, name: &str, agent_id: AgentId) {
        self.agents_by_name.insert(name.to_string(), agent_id);
    }

    pub async fn register_agent_with_capability(&self, capability: &str, agent_id: AgentId) {
        self.agents_by_capability.insert(capability.to_string(), agent_id);
    }

    pub async fn find_agent_for_capability(&self, capability: &str) -> Option<AgentId> {
        self.agents_by_capability.get(capability).map(|entry| *entry.value())
    }

    pub async fn get_agent_spec(&self, agent_id: AgentId) -> Option<AgentSpec> {
        self.agents.get(&agent_id).map(|entry| entry.spec.clone())
    }

    pub async fn remove_agent(&self, name: &str) {
        self.agents_by_name.remove(name);
    }

    pub async fn mark_agent_healthy(&self, name: &str) {
        self.agent_health.insert(name.to_string(), AgentHealth {
            healthy: true,
            last_error: None,
        });
    }

    pub async fn mark_agent_unhealthy(&self, name: &str, error: &str) {
        self.agent_health.insert(name.to_string(), AgentHealth {
            healthy: false,
            last_error: Some(error.to_string()),
        });
    }

    pub async fn is_agent_healthy(&self, name: &str) -> bool {
        self.agent_health.get(name).map(|entry| entry.healthy).unwrap_or(false)
    }

    pub async fn register(&self, _name: &str, agent: Arc<dyn std::any::Any + Send + Sync>) {
        // Legacy method - kept for compatibility
        let _ = agent; // Suppress unused warning
        // Extract AgentId if needed
    }

    pub async fn list_agents(&self) -> Vec<String> {
        self.agents_by_name
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}
