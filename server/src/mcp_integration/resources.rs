use super::*;
use crate::agents::registry::AgentRegistry;
use std::sync::Arc;

// Agent state resource
pub struct AgentStateResource {
    _registry: Arc<AgentRegistry>,
}

impl AgentStateResource {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { _registry: registry }
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
pub struct MetricsResource {
    _registry: Arc<AgentRegistry>,
}

impl MetricsResource {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { _registry: registry }
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
pub struct QualityReportResource;

impl Default for QualityReportResource {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityReportResource {
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
