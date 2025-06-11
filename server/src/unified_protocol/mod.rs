use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

pub mod adapters;
pub mod error;
pub mod service;
// pub mod test_harness; // TRACKED: Fix Future type issues

/// Core unified request abstraction that can represent any protocol
#[derive(Debug)]
pub struct UnifiedRequest {
    pub method: Method,
    pub path: String,
    pub headers: HeaderMap,
    pub body: Body,
    pub extensions: HashMap<String, Value>,
    pub trace_id: Uuid,
}

impl UnifiedRequest {
    pub fn new(method: Method, path: String) -> Self {
        Self {
            method,
            path,
            headers: HeaderMap::new(),
            body: Body::empty(),
            extensions: HashMap::with_capacity(4),
            trace_id: Uuid::new_v4(),
        }
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            key.parse::<http::HeaderName>(),
            value.parse::<http::HeaderValue>(),
        ) {
            self.headers.insert(name, val);
        }
        self
    }

    pub fn with_extension<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.extensions.insert(key.to_string(), json_value);
        }
        self
    }

    pub fn get_extension<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.extensions
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Core unified response abstraction
#[derive(Debug)]
pub struct UnifiedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Body,
    pub trace_id: Uuid,
}

impl UnifiedResponse {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Body::empty(),
            trace_id: Uuid::new_v4(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn with_json<T: Serialize>(self, data: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_vec(data)?;
        Ok(self
            .with_body(Body::from(json))
            .with_header("content-type", "application/json"))
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            key.parse::<http::HeaderName>(),
            value.parse::<http::HeaderValue>(),
        ) {
            self.headers.insert(name, val);
        }
        self
    }
}

impl IntoResponse for UnifiedResponse {
    fn into_response(self) -> Response {
        let mut response = Response::builder().status(self.status);

        for (key, value) in self.headers.iter() {
            response = response.header(key, value);
        }

        response.body(self.body).unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to build response"))
                .unwrap()
        })
    }
}

/// Protocol enumeration for type-safe protocol handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Mcp,
    Http,
    Cli,
    WebSocket,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Mcp => write!(f, "mcp"),
            Protocol::Http => write!(f, "http"),
            Protocol::Cli => write!(f, "cli"),
            Protocol::WebSocket => write!(f, "websocket"),
        }
    }
}

/// Core trait for protocol adapters
#[async_trait]
pub trait ProtocolAdapter: Send + Sync + 'static {
    type Input: Send + 'static;
    type Output: Send + 'static;

    /// Convert protocol-specific input to unified request
    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError>;

    /// Convert unified response to protocol-specific output
    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError>;

    /// Get the protocol type this adapter handles
    fn protocol(&self) -> Protocol;
}

/// Registry for managing protocol adapters
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: HashMap<Protocol, Arc<dyn ProtocolAdapter<Input = Value, Output = Value>>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A>(&mut self, adapter: A) -> &mut Self
    where
        A: ProtocolAdapter + 'static,
        A::Input: Into<Value> + for<'de> Deserialize<'de>,
        A::Output: From<Value> + Serialize,
    {
        let protocol = adapter.protocol();
        let wrapped = Arc::new(AdapterWrapper::new(adapter));
        self.adapters.insert(protocol, wrapped);
        self
    }

    pub fn get(
        &self,
        protocol: Protocol,
    ) -> Option<&Arc<dyn ProtocolAdapter<Input = Value, Output = Value>>> {
        self.adapters.get(&protocol)
    }
}

/// Wrapper to handle type erasure for different adapter types
struct AdapterWrapper<A> {
    inner: A,
}

impl<A> AdapterWrapper<A> {
    fn new(adapter: A) -> Self {
        Self { inner: adapter }
    }
}

#[async_trait]
impl<A> ProtocolAdapter for AdapterWrapper<A>
where
    A: ProtocolAdapter + Send + Sync + 'static,
    A::Input: for<'de> Deserialize<'de> + Send + 'static,
    A::Output: Serialize + Send + 'static,
{
    type Input = Value;
    type Output = Value;

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        let typed_input: A::Input =
            serde_json::from_value(input).map_err(|e| ProtocolError::DecodeError(e.to_string()))?;
        self.inner.decode(typed_input).await
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        let output = self.inner.encode(response).await?;
        serde_json::to_value(output).map_err(|e| ProtocolError::EncodeError(e.to_string()))
    }

    fn protocol(&self) -> Protocol {
        self.inner.protocol()
    }
}

/// Protocol-specific errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Failed to decode request: {0}")]
    DecodeError(String),

    #[error("Failed to encode response: {0}")]
    EncodeError(String),

    #[error("Protocol not supported: {0}")]
    UnsupportedProtocol(String),

    #[error("Invalid request format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(String),
}

/// Context information for MCP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub id: Option<Value>,
    pub method: String,
}

/// Context information for CLI requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliContext {
    pub command: String,
    pub args: Vec<String>,
}

/// Context information for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpContext {
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_request_creation() {
        let req = UnifiedRequest::new(Method::GET, "/test".to_string())
            .with_header("content-type", "application/json")
            .with_extension("test_key", "test_value");

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.path, "/test");
        assert!(req.headers.contains_key("content-type"));
        assert_eq!(
            req.get_extension::<String>("test_key"),
            Some("test_value".to_string())
        );
    }

    #[test]
    fn test_unified_response_creation() {
        let test_data = serde_json::json!({"message": "test"});
        let response = UnifiedResponse::ok().with_json(&test_data).unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.headers.contains_key("content-type"));
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Mcp.to_string(), "mcp");
        assert_eq!(Protocol::Http.to_string(), "http");
        assert_eq!(Protocol::Cli.to_string(), "cli");
        assert_eq!(Protocol::WebSocket.to_string(), "websocket");
    }
}
