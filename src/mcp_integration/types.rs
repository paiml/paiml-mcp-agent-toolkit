#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";

// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Server capabilities.
pub struct ServerCapabilities {
    pub experimental: Option<HashMap<String, Value>>,
    pub logging: Option<LoggingCapabilities>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Logging capabilities.
pub struct LoggingCapabilities {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Prompts capability.
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Resources capability.
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Tools capability.
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
/// Message variants for mcp.
pub enum McpMessage {
    #[serde(rename = "2.0")]
    JsonRpc(JsonRpcMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Message variants for json rpc.
pub enum JsonRpcMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Request for mcp operation.
pub struct McpRequest {
    pub id: RequestId,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Request id.
pub enum RequestId {
    String(String),
    Number(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Response from mcp operation.
pub struct McpResponse {
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Error type for mcp operations.
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
/// Mcp notification.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Information about server.
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
