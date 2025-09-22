use super::{AgentId, AgentSpec};
use dashmap::DashMap;
use std::sync::Arc;

pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentEntry>>,
    agents_by_name: Arc<DashMap<String, Arc<dyn std::any::Any + Send + Sync>>>,
}

struct AgentEntry {
    _spec: AgentSpec,
    // Will add actor address later
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
        }
    }

    pub async fn spawn_agent(&self, spec: AgentSpec) -> Result<AgentId, super::AgentError> {
        let id = spec.id;
        let entry = AgentEntry { _spec: spec };

        self.agents.insert(id, entry);
        Ok(id)
    }

    pub async fn get_agent(&self, _name: &str) -> Option<AgentId> {
        // TODO: Implement proper agent lookup by name
        // For now, return a dummy agent ID
        None
    }

    pub async fn register(&self, name: &str, agent: Arc<dyn std::any::Any + Send + Sync>) {
        self.agents_by_name.insert(name.to_string(), agent);
    }

    pub async fn list_agents(&self) -> Vec<String> {
        self.agents_by_name.iter().map(|entry| entry.key().clone()).collect()
    }
}
