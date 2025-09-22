use super::{AgentClass, AgentId, AgentSpec};
use dashmap::DashMap;
use std::sync::Arc;

pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentEntry>>,
}

struct AgentEntry {
    spec: AgentSpec,
    // Will add actor address later
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
        }
    }

    pub async fn spawn_agent(&self, spec: AgentSpec) -> Result<AgentId, super::AgentError> {
        let id = spec.id;
        let entry = AgentEntry { spec };

        self.agents.insert(id, entry);
        Ok(id)
    }

    pub async fn get_agent(&self, name: &str) -> Option<AgentId> {
        // TODO: Implement proper agent lookup by name
        // For now, return a dummy agent ID
        None
    }
}
