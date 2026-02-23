#![cfg_attr(coverage_nightly, coverage(off))]

use async_trait::async_trait;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::registry::{PromptRegistry, ResourceRegistry, ToolRegistry};
use super::types::{
    error_codes, JsonRpcMessage, McpError, McpMessage, McpNotification, McpRequest, McpResponse,
    MCP_VERSION,
};
use super::types::{ServerCapabilities, ServerInfo};

// MCP context for agent integration
pub struct McpContext {
    pub server_info: ServerInfo,
    pub capabilities: ServerCapabilities,
    pub tools: Arc<RwLock<ToolRegistry>>,
    pub resources: Arc<RwLock<ResourceRegistry>>,
    pub prompts: Arc<RwLock<PromptRegistry>>,
    pub agent_registry: Arc<crate::agents::registry::AgentRegistry>,
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
