//! EXTREME TDD: Agent Registry Tests
//!
//! RED PHASE: All tests written BEFORE implementation

#[cfg(test)]
mod agent_registry_red_tests {
    use crate::agents::registry::AgentRegistry;
    use crate::agents::{AgentSpec, AgentClass};
    use uuid::Uuid;

    // ===== Agent Registration Tests =====

    #[tokio::test]
    async fn red_must_register_agent_by_name() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry.register_agent_with_name("analyzer", agent_id).await;

        let found = registry.get_agent("analyzer").await;
        assert_eq!(found, Some(agent_id));
    }

    #[tokio::test]
    async fn red_must_return_none_for_unknown_agent() {
        let registry = AgentRegistry::new();

        let found = registry.get_agent("nonexistent").await;
        assert_eq!(found, None);
    }

    #[tokio::test]
    async fn red_must_overwrite_agent_on_duplicate_registration() {
        let registry = AgentRegistry::new();
        let agent_id1 = Uuid::new_v4();
        let agent_id2 = Uuid::new_v4();

        registry.register_agent_with_name("analyzer", agent_id1).await;
        registry.register_agent_with_name("analyzer", agent_id2).await;

        let found = registry.get_agent("analyzer").await;
        assert_eq!(found, Some(agent_id2), "Must use latest registration");
    }

    // ===== Agent Listing Tests =====

    #[tokio::test]
    async fn red_must_list_all_registered_agents() {
        let registry = AgentRegistry::new();

        registry.register_agent_with_name("analyzer", Uuid::new_v4()).await;
        registry.register_agent_with_name("transformer", Uuid::new_v4()).await;
        registry.register_agent_with_name("validator", Uuid::new_v4()).await;

        let agents = registry.list_agents().await;
        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&"analyzer".to_string()));
        assert!(agents.contains(&"transformer".to_string()));
        assert!(agents.contains(&"validator".to_string()));
    }

    #[tokio::test]
    async fn red_must_return_empty_list_when_no_agents() {
        let registry = AgentRegistry::new();

        let agents = registry.list_agents().await;
        assert_eq!(agents.len(), 0);
    }

    // ===== Agent Spec Management =====

    #[tokio::test]
    async fn red_must_store_agent_spec_on_spawn() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        let spec = AgentSpec {
            id: agent_id,
            class: AgentClass::Analyzer,
            config: serde_json::json!({"key": "value"}),
        };

        let spawned_id = registry.spawn_agent(spec.clone()).await.unwrap();
        assert_eq!(spawned_id, agent_id);

        let retrieved_spec = registry.get_agent_spec(agent_id).await;
        assert!(retrieved_spec.is_some(), "Must retrieve stored spec");
    }

    #[tokio::test]
    async fn red_must_return_none_for_unknown_agent_spec() {
        let registry = AgentRegistry::new();
        let unknown_id = Uuid::new_v4();

        let spec = registry.get_agent_spec(unknown_id).await;
        assert_eq!(spec, None);
    }

    // ===== Routing by Capability =====

    #[tokio::test]
    async fn red_must_route_to_agent_by_capability() {
        let registry = AgentRegistry::new();
        let analyzer_id = Uuid::new_v4();

        registry.register_agent_with_capability("analyze", analyzer_id).await;

        let found = registry.find_agent_for_capability("analyze").await;
        assert_eq!(found, Some(analyzer_id));
    }

    #[tokio::test]
    async fn red_must_return_none_for_unknown_capability() {
        let registry = AgentRegistry::new();

        let found = registry.find_agent_for_capability("unknown_capability").await;
        assert_eq!(found, None);
    }

    // ===== Agent Removal Tests =====

    #[tokio::test]
    async fn red_must_remove_agent_from_registry() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry.register_agent_with_name("analyzer", agent_id).await;
        assert!(registry.get_agent("analyzer").await.is_some());

        registry.remove_agent("analyzer").await;
        assert_eq!(registry.get_agent("analyzer").await, None);
    }

    // ===== Agent Status Tests =====

    #[tokio::test]
    async fn red_must_track_agent_health_status() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry.register_agent_with_name("analyzer", agent_id).await;
        registry.mark_agent_healthy("analyzer").await;

        let is_healthy = registry.is_agent_healthy("analyzer").await;
        assert!(is_healthy, "Must track agent as healthy");
    }

    #[tokio::test]
    async fn red_must_track_agent_unhealthy_status() {
        let registry = AgentRegistry::new();
        let agent_id = Uuid::new_v4();

        registry.register_agent_with_name("analyzer", agent_id).await;
        registry.mark_agent_unhealthy("analyzer", "test error").await;

        let is_healthy = registry.is_agent_healthy("analyzer").await;
        assert!(!is_healthy, "Must track agent as unhealthy");
    }
}
