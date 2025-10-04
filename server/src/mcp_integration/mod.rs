// MCP (Model Context Protocol) integration for agent system
pub mod prompts;
pub mod resources;
pub mod server;
pub mod service_registry;
pub mod tools;
#[cfg(feature = "deep-wasm")]
pub mod deep_wasm_tools;
pub mod mutation_tools;
pub mod transport;

#[cfg(test)]
mod tools_integration_tests;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";

// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub experimental: Option<HashMap<String, Value>>,
    pub logging: Option<LoggingCapabilities>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapabilities {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
pub enum McpMessage {
    #[serde(rename = "2.0")]
    JsonRpc(JsonRpcMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: RequestId,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MCP Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for McpError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    pub method: String,
    pub params: Option<Value>,
}

// MCP error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

// MCP context for agent integration
pub struct McpContext {
    pub server_info: ServerInfo,
    pub capabilities: ServerCapabilities,
    pub tools: Arc<RwLock<ToolRegistry>>,
    pub resources: Arc<RwLock<ResourceRegistry>>,
    pub prompts: Arc<RwLock<PromptRegistry>>,
    pub agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub protocol_version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "PMAT Agent Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: MCP_VERSION.to_string(),
        }
    }
}

// Tool registry for MCP
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn McpTool>>,
    metadata: HashMap<String, ToolMetadata>,
}

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
        self.tools.get(name).cloned()
    }
}

// Resource registry for MCP
pub struct ResourceRegistry {
    resources: HashMap<String, Arc<dyn McpResource>>,
    templates: HashMap<String, ResourceTemplate>,
}

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

// Prompt registry for MCP
pub struct PromptRegistry {
    prompts: HashMap<String, Arc<dyn McpPrompt>>,
    metadata: HashMap<String, PromptMetadata>,
}

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
        self.prompts.get(name).cloned()
    }
}

// MCP session management
pub struct McpSession {
    pub id: Uuid,
    pub context: Arc<McpContext>,
    pub transport: Arc<dyn McpTransport>,
    pub active_subscriptions: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn send(&self, message: McpMessage) -> Result<(), McpError>;
    async fn receive(&self) -> Result<McpMessage, McpError>;
    async fn close(&self) -> Result<(), McpError>;
}

impl McpSession {
    pub fn new(context: Arc<McpContext>, transport: Arc<dyn McpTransport>) -> Self {
        Self {
            id: Uuid::new_v4(),
            context,
            transport,
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tool_call(request.params).await,
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resource_read(request.params).await,
            "resources/subscribe" => self.handle_resource_subscribe(request.params).await,
            "prompts/list" => self.handle_prompts_list().await,
            "prompts/get" => self.handle_prompt_get(request.params).await,
            "completion/complete" => self.handle_completion(request.params).await,
            _ => Err(McpError {
                code: error_codes::METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => McpResponse {
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(err) => McpResponse {
                id: request.id,
                result: None,
                error: Some(err),
            },
        }
    }

    async fn handle_initialize(&self, _params: Option<Value>) -> Result<Value, McpError> {
        Ok(serde_json::json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": self.context.capabilities,
            "serverInfo": self.context.server_info,
        }))
    }

    async fn handle_tools_list(&self) -> Result<Value, McpError> {
        let tools = self.context.tools.read().list();
        Ok(serde_json::json!({ "tools": tools }))
    }

    async fn handle_tool_call(&self, params: Option<Value>) -> Result<Value, McpError> {
        let params = params.ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let name = params["name"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

        let tool_params = params["arguments"].clone();

        let tool = self
            .context
            .tools
            .read()
            .get(name)
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Tool not found: {}", name),
                data: None,
            })?;

        let result = tool.execute(tool_params).await?;
        Ok(serde_json::json!({ "content": [result] }))
    }

    async fn handle_resources_list(&self) -> Result<Value, McpError> {
        let resources = self.context.resources.read().list();
        Ok(serde_json::json!({ "resources": resources }))
    }

    async fn handle_resource_read(&self, params: Option<Value>) -> Result<Value, McpError> {
        let params = params.ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let uri = params["uri"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing URI".to_string(),
            data: None,
        })?;

        let resource = self
            .context
            .resources
            .read()
            .find_matching(uri)
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Resource not found for URI: {}", uri),
                data: None,
            })?;

        let content = resource.read(uri).await?;
        Ok(serde_json::json!({ "contents": [content] }))
    }

    async fn handle_resource_subscribe(&self, params: Option<Value>) -> Result<Value, McpError> {
        let params = params.ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let uri = params["uri"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing URI".to_string(),
            data: None,
        })?;

        let resource = self
            .context
            .resources
            .read()
            .find_matching(uri)
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Resource not found for URI: {}", uri),
                data: None,
            })?;

        if let Some(mut receiver) = resource.subscribe(uri) {
            let uri_clone = uri.to_string();
            let transport = self.transport.clone();

            let handle = tokio::spawn(async move {
                while receiver.changed().await.is_ok() {
                    let content = receiver.borrow().clone();
                    let notification = McpNotification {
                        method: "notifications/resources/updated".to_string(),
                        params: Some(serde_json::json!({
                            "uri": uri_clone,
                            "contents": [content],
                        })),
                    };

                    let message = McpMessage::JsonRpc(JsonRpcMessage::Notification(notification));
                    let _ = transport.send(message).await;
                }
            });

            self.active_subscriptions
                .write()
                .insert(uri.to_string(), handle);
        }

        Ok(serde_json::json!({}))
    }

    async fn handle_prompts_list(&self) -> Result<Value, McpError> {
        let prompts = self.context.prompts.read().list();
        Ok(serde_json::json!({ "prompts": prompts }))
    }

    async fn handle_prompt_get(&self, params: Option<Value>) -> Result<Value, McpError> {
        let params = params.ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let name = params["name"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing prompt name".to_string(),
            data: None,
        })?;

        let arguments = params["arguments"].as_object().map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });

        let prompt = self
            .context
            .prompts
            .read()
            .get(name)
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Prompt not found: {}", name),
                data: None,
            })?;

        let messages = prompt.get(arguments).await?;
        Ok(serde_json::json!({ "messages": messages }))
    }

    async fn handle_completion(&self, params: Option<Value>) -> Result<Value, McpError> {
        // Integrate with agent system for completions
        let _params = params.ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        // This would integrate with the agent system to provide completions
        Ok(serde_json::json!({
            "completion": {
                "values": [],
                "total": 0,
                "hasMore": false,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let info = ServerInfo::default();
        assert_eq!(info.protocol_version, MCP_VERSION);
        assert_eq!(info.name, "PMAT Agent Server");
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_resource_registry() {
        let registry = ResourceRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_prompt_registry() {
        let registry = PromptRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }
}
