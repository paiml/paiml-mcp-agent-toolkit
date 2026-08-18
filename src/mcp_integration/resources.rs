use super::*;
use crate::agents::registry::AgentRegistry;
use std::sync::Arc;

// Agent state resource
/// Agent state resource.
pub struct AgentStateResource {
    _registry: Arc<AgentRegistry>,
}

impl AgentStateResource {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpResource for AgentStateResource {
    fn template(&self) -> ResourceTemplate {
        ResourceTemplate {
            uri_template: "agent://state/{agent_id}".to_string(),
            name: "Agent State".to_string(),
            description: Some("Current state of an agent".to_string()),
            mime_type: Some("application/json".to_string()),
        }
    }

    async fn read(&self, uri: &str) -> Result<ResourceContent, McpError> {
        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            content: ResourceContentType::Text {
                text: "{}".to_string(),
            },
        })
    }

    fn subscribe(&self, _uri: &str) -> Option<tokio::sync::watch::Receiver<ResourceContent>> {
        None
    }
}

// Metrics resource
/// Metrics resource.
pub struct MetricsResource {
    _registry: Arc<AgentRegistry>,
}

impl MetricsResource {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpResource for MetricsResource {
    fn template(&self) -> ResourceTemplate {
        ResourceTemplate {
            uri_template: "metrics://{type}".to_string(),
            name: "System Metrics".to_string(),
            description: Some("System and agent metrics".to_string()),
            mime_type: Some("application/json".to_string()),
        }
    }

    async fn read(&self, uri: &str) -> Result<ResourceContent, McpError> {
        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            content: ResourceContentType::Text {
                text: "{}".to_string(),
            },
        })
    }

    fn subscribe(&self, _uri: &str) -> Option<tokio::sync::watch::Receiver<ResourceContent>> {
        None
    }
}

// Quality report resource
/// Quality report resource.
pub struct QualityReportResource;

impl Default for QualityReportResource {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityReportResource {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpResource for QualityReportResource {
    fn template(&self) -> ResourceTemplate {
        ResourceTemplate {
            uri_template: "quality://report/{id}".to_string(),
            name: "Quality Report".to_string(),
            description: Some("Code quality analysis report".to_string()),
            mime_type: Some("application/json".to_string()),
        }
    }

    async fn read(&self, uri: &str) -> Result<ResourceContent, McpError> {
        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            content: ResourceContentType::Text {
                text: "{}".to_string(),
            },
        })
    }

    fn subscribe(&self, _uri: &str) -> Option<tokio::sync::watch::Receiver<ResourceContent>> {
        None
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::agents::registry::AgentRegistry;

    // ============================================================
    // AgentStateResource Tests
    // ============================================================

    #[test]
    fn test_agent_state_resource_new() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = AgentStateResource::new(registry);
        // Should create without panic
        let _ = resource;
    }

    #[test]
    fn test_agent_state_resource_template() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = AgentStateResource::new(registry);
        let template = resource.template();

        assert_eq!(template.uri_template, "agent://state/{agent_id}");
        assert_eq!(template.name, "Agent State");
        assert!(template.description.is_some());
        assert!(template.description.as_ref().unwrap().contains("state"));
        assert_eq!(template.mime_type, Some("application/json".to_string()));
    }

    #[tokio::test]
    async fn test_agent_state_resource_read() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = AgentStateResource::new(registry);

        let content = resource.read("agent://state/test-agent-123").await.unwrap();

        assert_eq!(content.uri, "agent://state/test-agent-123");
        assert_eq!(content.mime_type, Some("application/json".to_string()));

        match content.content {
            ResourceContentType::Text { text } => {
                assert_eq!(text, "{}");
            }
            _ => panic!("Expected Text content type"),
        }
    }

    #[tokio::test]
    async fn test_agent_state_resource_read_various_uris() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = AgentStateResource::new(registry);

        for uri in [
            "agent://state/abc",
            "agent://state/123",
            "agent://state/test-agent",
            "agent://state/uuid-1234-5678",
        ] {
            let content = resource.read(uri).await.unwrap();
            assert_eq!(content.uri, uri);
        }
    }

    #[test]
    fn test_agent_state_resource_subscribe_returns_none() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = AgentStateResource::new(registry);

        let subscription = resource.subscribe("agent://state/test");
        assert!(subscription.is_none());
    }

    // ============================================================
    // MetricsResource Tests
    // ============================================================

    #[test]
    fn test_metrics_resource_new() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = MetricsResource::new(registry);
        let _ = resource;
    }

    #[test]
    fn test_metrics_resource_template() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = MetricsResource::new(registry);
        let template = resource.template();

        assert_eq!(template.uri_template, "metrics://{type}");
        assert_eq!(template.name, "System Metrics");
        assert!(template.description.is_some());
        assert!(template.description.as_ref().unwrap().contains("metrics"));
        assert_eq!(template.mime_type, Some("application/json".to_string()));
    }

    #[tokio::test]
    async fn test_metrics_resource_read() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = MetricsResource::new(registry);

        let content = resource.read("metrics://cpu").await.unwrap();

        assert_eq!(content.uri, "metrics://cpu");
        assert_eq!(content.mime_type, Some("application/json".to_string()));

        match content.content {
            ResourceContentType::Text { text } => {
                assert_eq!(text, "{}");
            }
            _ => panic!("Expected Text content type"),
        }
    }

    #[tokio::test]
    async fn test_metrics_resource_read_various_types() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = MetricsResource::new(registry);

        for uri in [
            "metrics://cpu",
            "metrics://memory",
            "metrics://disk",
            "metrics://network",
            "metrics://agent-performance",
        ] {
            let content = resource.read(uri).await.unwrap();
            assert_eq!(content.uri, uri);
        }
    }

    #[test]
    fn test_metrics_resource_subscribe_returns_none() {
        let registry = Arc::new(AgentRegistry::new());
        let resource = MetricsResource::new(registry);

        let subscription = resource.subscribe("metrics://cpu");
        assert!(subscription.is_none());
    }

    // ============================================================
    // QualityReportResource Tests
    // ============================================================

    #[test]
    fn test_quality_report_resource_new() {
        let resource = QualityReportResource::new();
        let _ = resource;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_quality_report_resource_default() {
        // See the note on the prompt `*_default` tests: the `Default` impl is
        // the subject, so the lint's rewrite would delete the test. Compare an
        // observable instead of discarding the value.
        let resource = QualityReportResource::default();
        assert_eq!(
            resource.template().uri_template,
            QualityReportResource::new().template().uri_template
        );
    }

    #[test]
    fn test_quality_report_resource_template() {
        let resource = QualityReportResource::new();
        let template = resource.template();

        assert_eq!(template.uri_template, "quality://report/{id}");
        assert_eq!(template.name, "Quality Report");
        assert!(template.description.is_some());
        assert!(template.description.as_ref().unwrap().contains("quality"));
        assert_eq!(template.mime_type, Some("application/json".to_string()));
    }

    #[tokio::test]
    async fn test_quality_report_resource_read() {
        let resource = QualityReportResource::new();

        let content = resource.read("quality://report/abc-123").await.unwrap();

        assert_eq!(content.uri, "quality://report/abc-123");
        assert_eq!(content.mime_type, Some("application/json".to_string()));

        match content.content {
            ResourceContentType::Text { text } => {
                assert_eq!(text, "{}");
            }
            _ => panic!("Expected Text content type"),
        }
    }

    #[tokio::test]
    async fn test_quality_report_resource_read_various_ids() {
        let resource = QualityReportResource::new();

        for uri in [
            "quality://report/1",
            "quality://report/test-report",
            "quality://report/uuid-abc-def",
            "quality://report/latest",
        ] {
            let content = resource.read(uri).await.unwrap();
            assert_eq!(content.uri, uri);
        }
    }

    #[test]
    fn test_quality_report_resource_subscribe_returns_none() {
        let resource = QualityReportResource::new();

        let subscription = resource.subscribe("quality://report/test");
        assert!(subscription.is_none());
    }

    // ============================================================
    // Cross-Resource Tests
    // ============================================================

    #[test]
    fn test_all_resources_have_unique_uri_templates() {
        let registry = Arc::new(AgentRegistry::new());

        let templates = vec![
            AgentStateResource::new(Arc::clone(&registry))
                .template()
                .uri_template,
            MetricsResource::new(Arc::clone(&registry))
                .template()
                .uri_template,
            QualityReportResource::new().template().uri_template,
        ];

        let mut unique_templates = templates.clone();
        unique_templates.sort();
        unique_templates.dedup();

        assert_eq!(
            templates.len(),
            unique_templates.len(),
            "All resource URI templates should be unique"
        );
    }

    #[test]
    fn test_all_resources_have_unique_names() {
        let registry = Arc::new(AgentRegistry::new());

        let names = vec![
            AgentStateResource::new(Arc::clone(&registry))
                .template()
                .name,
            MetricsResource::new(Arc::clone(&registry)).template().name,
            QualityReportResource::new().template().name,
        ];

        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();

        assert_eq!(
            names.len(),
            unique_names.len(),
            "All resource names should be unique"
        );
    }

    #[test]
    fn test_all_resources_return_json_mime_type() {
        let registry = Arc::new(AgentRegistry::new());

        let mime_types = vec![
            AgentStateResource::new(Arc::clone(&registry))
                .template()
                .mime_type,
            MetricsResource::new(Arc::clone(&registry))
                .template()
                .mime_type,
            QualityReportResource::new().template().mime_type,
        ];

        for mime_type in mime_types {
            assert_eq!(mime_type, Some("application/json".to_string()));
        }
    }

    #[tokio::test]
    async fn test_all_resources_return_valid_content() {
        let registry = Arc::new(AgentRegistry::new());

        // AgentStateResource
        let agent_resource = AgentStateResource::new(Arc::clone(&registry));
        let content = agent_resource.read("agent://state/test").await.unwrap();
        assert!(!content.uri.is_empty());

        // MetricsResource
        let metrics_resource = MetricsResource::new(Arc::clone(&registry));
        let content = metrics_resource.read("metrics://cpu").await.unwrap();
        assert!(!content.uri.is_empty());

        // QualityReportResource
        let quality_resource = QualityReportResource::new();
        let content = quality_resource.read("quality://report/1").await.unwrap();
        assert!(!content.uri.is_empty());
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[tokio::test]
    async fn test_resources_handle_empty_uri() {
        let registry = Arc::new(AgentRegistry::new());

        let agent_resource = AgentStateResource::new(Arc::clone(&registry));
        let content = agent_resource.read("").await.unwrap();
        assert_eq!(content.uri, "");

        let metrics_resource = MetricsResource::new(Arc::clone(&registry));
        let content = metrics_resource.read("").await.unwrap();
        assert_eq!(content.uri, "");

        let quality_resource = QualityReportResource::new();
        let content = quality_resource.read("").await.unwrap();
        assert_eq!(content.uri, "");
    }

    #[tokio::test]
    async fn test_resources_handle_special_characters_in_uri() {
        let registry = Arc::new(AgentRegistry::new());

        let agent_resource = AgentStateResource::new(registry);
        let uri = "agent://state/test-agent_with.special+chars";
        let content = agent_resource.read(uri).await.unwrap();
        assert_eq!(content.uri, uri);
    }

    #[tokio::test]
    async fn test_resources_handle_unicode_in_uri() {
        let registry = Arc::new(AgentRegistry::new());

        let agent_resource = AgentStateResource::new(registry);
        let uri = "agent://state/日本語テスト";
        let content = agent_resource.read(uri).await.unwrap();
        assert_eq!(content.uri, uri);
    }

    #[tokio::test]
    async fn test_resources_handle_very_long_uri() {
        let registry = Arc::new(AgentRegistry::new());

        let long_id = "x".repeat(1000);
        let agent_resource = AgentStateResource::new(registry);
        let uri = format!("agent://state/{}", long_id);
        let content = agent_resource.read(&uri).await.unwrap();
        assert_eq!(content.uri, uri);
    }

    #[test]
    fn test_resource_template_clone() {
        let resource = QualityReportResource::new();
        let template = resource.template();
        let cloned = template.clone();

        assert_eq!(template.uri_template, cloned.uri_template);
        assert_eq!(template.name, cloned.name);
        assert_eq!(template.description, cloned.description);
        assert_eq!(template.mime_type, cloned.mime_type);
    }

    #[tokio::test]
    async fn test_resource_content_clone() {
        let resource = QualityReportResource::new();
        let content = resource.read("quality://report/test").await.unwrap();
        let cloned = content.clone();

        assert_eq!(content.uri, cloned.uri);
        assert_eq!(content.mime_type, cloned.mime_type);
    }
}
