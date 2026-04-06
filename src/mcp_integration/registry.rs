#![cfg_attr(coverage_nightly, coverage(off))]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::McpError;

// ============================================================
// Tool Registry
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[async_trait]
pub trait McpTool: Send + Sync {
    fn metadata(&self) -> ToolMetadata;
    async fn execute(&self, params: Value) -> Result<Value, McpError>;
}

pub struct ToolRegistry {
    pub(super) tools: HashMap<String, Arc<dyn McpTool>>,
    pub(super) metadata: HashMap<String, ToolMetadata>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn McpTool>) {
        let metadata = tool.metadata();
        self.tools.insert(metadata.name.clone(), tool);
        self.metadata.insert(metadata.name.clone(), metadata);
    }

    pub fn list(&self) -> Vec<ToolMetadata> {
        self.metadata.values().cloned().collect()
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.tools.get(name).cloned()
    }
}

// ============================================================
// Resource Registry
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    pub uri_template: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    #[serde(flatten)]
    pub content: ResourceContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceContentType {
    Text { text: String },
    Blob { blob: String }, // Base64 encoded
}

#[async_trait]
pub trait McpResource: Send + Sync {
    fn template(&self) -> ResourceTemplate;
    async fn read(&self, uri: &str) -> Result<ResourceContent, McpError>;
    fn subscribe(&self, uri: &str) -> Option<tokio::sync::watch::Receiver<ResourceContent>>;
}

pub struct ResourceRegistry {
    pub(super) resources: HashMap<String, Arc<dyn McpResource>>,
    pub(super) templates: HashMap<String, ResourceTemplate>,
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, resource: Arc<dyn McpResource>) {
        let template = resource.template();
        self.resources
            .insert(template.uri_template.clone(), resource);
        self.templates
            .insert(template.uri_template.clone(), template);
    }

    pub fn list(&self) -> Vec<ResourceTemplate> {
        self.templates.values().cloned().collect()
    }

    pub fn get(&self, uri_template: &str) -> Option<Arc<dyn McpResource>> {
        self.resources.get(uri_template).cloned()
    }

    pub fn find_matching(&self, uri: &str) -> Option<Arc<dyn McpResource>> {
        // Simple pattern matching - could be enhanced
        for (template, resource) in &self.resources {
            if uri.starts_with(&template.replace("{}", "")) {
                return Some(resource.clone());
            }
        }
        None
    }
}

// ============================================================
// Prompt Registry
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String },
}

#[async_trait]
pub trait McpPrompt: Send + Sync {
    fn metadata(&self) -> PromptMetadata;
    async fn get(
        &self,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<Vec<PromptMessage>, McpError>;
}

pub struct PromptRegistry {
    pub(super) prompts: HashMap<String, Arc<dyn McpPrompt>>,
    pub(super) metadata: HashMap<String, PromptMetadata>,
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptRegistry {
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn register(&mut self, prompt: Arc<dyn McpPrompt>) {
        let metadata = prompt.metadata();
        self.prompts.insert(metadata.name.clone(), prompt);
        self.metadata.insert(metadata.name.clone(), metadata);
    }

    pub fn list(&self) -> Vec<PromptMetadata> {
        self.metadata.values().cloned().collect()
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn McpPrompt>> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.prompts.get(name).cloned()
    }
}
