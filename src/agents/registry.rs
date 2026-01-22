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
        self.agents_by_capability
            .insert(capability.to_string(), agent_id);
    }

    pub async fn find_agent_for_capability(&self, capability: &str) -> Option<AgentId> {
        self.agents_by_capability
            .get(capability)
            .map(|entry| *entry.value())
    }

    pub async fn get_agent_spec(&self, agent_id: AgentId) -> Option<AgentSpec> {
        self.agents.get(&agent_id).map(|entry| entry.spec.clone())
    }

    pub async fn remove_agent(&self, name: &str) {
        self.agents_by_name.remove(name);
    }

    pub async fn mark_agent_healthy(&self, name: &str) {
        self.agent_health.insert(
            name.to_string(),
            AgentHealth {
                healthy: true,
                last_error: None,
            },
        );
    }

    pub async fn mark_agent_unhealthy(&self, name: &str, error: &str) {
        self.agent_health.insert(
            name.to_string(),
            AgentHealth {
                healthy: false,
                last_error: Some(error.to_string()),
            },
        );
    }

    pub async fn is_agent_healthy(&self, name: &str) -> bool {
        self.agent_health
            .get(name)
            .map(|entry| entry.healthy)
            .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::super::AgentClass;
    use super::*;
    use uuid::Uuid;

    fn create_test_spec() -> (AgentId, AgentSpec) {
        let id = Uuid::new_v4();
        let spec = AgentSpec {
            id,
            class: AgentClass::Analyzer,
            config: serde_json::json!({}),
        };
        (id, spec)
    }

    #[test]
    fn test_registry_new() {
        let registry = AgentRegistry::new();
        let _ = registry;
    }

    #[test]
    fn test_registry_default() {
        let registry = AgentRegistry::default();
        let _ = registry;
    }

    #[tokio::test]
    async fn test_spawn_agent() {
        let registry = AgentRegistry::new();
        let (expected_id, spec) = create_test_spec();

        let result = registry.spawn_agent(spec).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio::test]
    async fn test_register_agent_with_name() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry
            .register_agent_with_name("test_agent", agent_id)
            .await;

        let found = registry.get_agent("test_agent").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap(), agent_id);
    }

    #[tokio::test]
    async fn test_get_agent_not_found() {
        let registry = AgentRegistry::new();

        let found = registry.get_agent("nonexistent").await;
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_register_agent_with_capability() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry
            .register_agent_with_capability("analyze", agent_id)
            .await;

        let found = registry.find_agent_for_capability("analyze").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap(), agent_id);
    }

    #[tokio::test]
    async fn test_find_capability_not_found() {
        let registry = AgentRegistry::new();

        let found = registry.find_agent_for_capability("nonexistent").await;
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_get_agent_spec() {
        let registry = AgentRegistry::new();
        let (expected_id, spec) = create_test_spec();

        registry.spawn_agent(spec.clone()).await.unwrap();

        let found = registry.get_agent_spec(expected_id).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, expected_id);
    }

    #[tokio::test]
    async fn test_get_agent_spec_not_found() {
        let registry = AgentRegistry::new();

        let found = registry.get_agent_spec(Uuid::new_v4()).await;
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_remove_agent() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry
            .register_agent_with_name("to_remove", agent_id)
            .await;
        assert!(registry.get_agent("to_remove").await.is_some());

        registry.remove_agent("to_remove").await;
        assert!(registry.get_agent("to_remove").await.is_none());
    }

    #[tokio::test]
    async fn test_mark_agent_healthy() {
        let registry = AgentRegistry::new();

        registry.mark_agent_healthy("healthy_agent").await;

        assert!(registry.is_agent_healthy("healthy_agent").await);
    }

    #[tokio::test]
    async fn test_mark_agent_unhealthy() {
        let registry = AgentRegistry::new();

        registry
            .mark_agent_unhealthy("unhealthy_agent", "connection failed")
            .await;

        assert!(!registry.is_agent_healthy("unhealthy_agent").await);
    }

    #[tokio::test]
    async fn test_is_agent_healthy_unknown() {
        let registry = AgentRegistry::new();

        // Unknown agents are considered unhealthy
        assert!(!registry.is_agent_healthy("unknown").await);
    }

    #[tokio::test]
    async fn test_list_agents_empty() {
        let registry = AgentRegistry::new();

        let agents = registry.list_agents().await;
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_list_agents_with_entries() {
        let registry = AgentRegistry::new();

        registry
            .register_agent_with_name("agent1", Uuid::new_v4())
            .await;
        registry
            .register_agent_with_name("agent2", Uuid::new_v4())
            .await;

        let agents = registry.list_agents().await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_register_legacy_method() {
        let registry = AgentRegistry::new();
        let dummy_agent: Arc<dyn std::any::Any + Send + Sync> = Arc::new(());

        // Should not panic
        registry.register("legacy", dummy_agent).await;
    }
}
