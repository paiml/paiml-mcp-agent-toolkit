use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
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

impl McpResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message,
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTemplatesArgs {
    pub toolchain: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaffoldProjectArgs {
    pub toolchain: String,
    pub templates: Vec<String>,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTemplatesArgs {
    pub query: String,
    pub toolchain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptGetParams {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
