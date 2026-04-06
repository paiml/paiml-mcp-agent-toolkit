#![cfg_attr(coverage_nightly, coverage(off))]
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader, Stdin};
use tracing::debug;

use crate::unified_protocol::{
    McpContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

/// MCP (Model Context Protocol) adapter for JSON-RPC over STDIO
pub struct McpAdapter {
    #[allow(dead_code)]
    stdin: Option<AsyncBufReader<Stdin>>,
}

impl McpAdapter {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self { stdin: None }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_stdin(stdin: Stdin) -> Self {
        Self {
            stdin: Some(AsyncBufReader::new(stdin)),
        }
    }
}

impl Default for McpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Input types for MCP adapter
#[derive(Debug)]
pub enum McpInput {
    Line(String),
    Request(JsonRpcRequest),
}

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(method: String, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self::new(method, params, None)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn request(method: String, params: Option<Value>, id: Value) -> Self {
        Self::new(method, params, Some(id))
    }
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcResponse {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn error(error: JsonRpcError, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    // Standard JSON-RPC error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse_error() -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn invalid_request() -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn method_not_found(method: &str) -> Self {
        debug_assert!(!method.is_empty(), "method must not be empty");
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: format!("Invalid params: {message}"),
            data: None,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn internal_error(message: &str) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: format!("Internal error: {message}"),
            data: None,
        }
    }
}

/// Helper for reading MCP messages from STDIO
pub struct McpReader {
    reader: AsyncBufReader<Stdin>,
}

// --- Include files for implementation and tests ---
include!("mcp_adapter_impl.rs");
include!("mcp_tests.rs");
include!("mcp_extended_tests.rs");
