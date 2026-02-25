use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::demo::protocol_harness::{DemoProtocol, ProtocolMetadata};

/// HTTP/REST protocol adapter for demo harness
pub struct HttpDemoAdapter;

/// HTTP-specific request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub remote_addr: Option<String>,
}

/// HTTP-specific response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: HttpResponseBody,
    pub request_id: String,
}

/// HTTP response body variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HttpResponseBody {
    /// Immediate response with analysis results
    Analysis {
        protocol: String,
        base_command: String,
        request: HttpRequestInfo,
        response_time_ms: u64,
        cache_hit: bool,
        result: Value,
    },
    /// Asynchronous response with request tracking
    Async {
        request_id: String,
        status: String,
        message: String,
        poll_url: String,
    },
    /// Status check response
    Status {
        request_id: String,
        status: String,
        progress: Option<f32>,
        result: Option<Value>,
        error: Option<String>,
    },
    /// API introspection response
    Introspection {
        protocol: String,
        version: String,
        endpoints: Vec<HttpEndpoint>,
        schemas: HashMap<String, Value>,
        examples: HashMap<String, Value>,
    },
}

/// HTTP request information for introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestInfo {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

/// HTTP endpoint description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub parameters: Vec<HttpParameter>,
    pub responses: HashMap<String, String>,
}

/// HTTP parameter description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpParameter {
    pub name: String,
    pub location: String, // "query", "path", "header", "body"
    pub required: bool,
    pub param_type: String,
    pub description: String,
}

/// HTTP-specific errors
#[derive(Debug, Error)]
pub enum HttpDemoError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid path parameter: {0}")]
    InvalidPath(String),

    #[error("Analysis execution failed: {0}")]
    AnalysisFailed(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Default for HttpDemoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Value> for HttpRequest {
    fn from(value: Value) -> Self {
        let method = value
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string();

        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/demo/analyze")
            .to_string();

        let query_params = value
            .get("query")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let headers = value
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_else(|| {
                [("Accept".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect()
            });

        HttpRequest {
            method,
            path,
            query_params,
            headers,
            body: value.get("body").cloned(),
            remote_addr: value
                .get("remote_addr")
                .and_then(|v| v.as_str())
                .map(std::string::ToString::to_string),
        }
    }
}

impl From<HttpResponse> for Value {
    fn from(val: HttpResponse) -> Self {
        serde_json::to_value(val).unwrap_or(Value::Null)
    }
}

include!("http_handlers.rs");
include!("http_protocol.rs");
include!("http_tests.rs");
