#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use actix::prelude::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Request response broker.
pub struct RequestResponseBroker {
    pending_requests: Arc<RwLock<HashMap<Uuid, oneshot::Sender<AgentMessage>>>>,
    router: Arc<MessageRouter>,
}

#[derive(Debug, thiserror::Error)]
/// Error variants for request operations.
pub enum RequestError {
    #[error("Request timeout")]
    Timeout,
    #[error("Request cancelled")]
    Cancelled,
    #[error("Serialization error: {0}")]
    Serialization(#[from] rmp_serde::decode::Error),
    #[error("Encode error: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("Router error: {0}")]
    Router(#[from] RouterError),
}

// Actor wrapper for request-response
/// Request response actor.
pub struct RequestResponseActor {
    broker: Arc<RequestResponseBroker>,
}

// Pattern matching for request types
/// Trait defining Request behavior.
pub trait Request: Serialize + for<'de> Deserialize<'de> + Send {
    type Response: Serialize + for<'de> Deserialize<'de> + Send;

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

// Example request types
#[derive(Debug, Serialize, Deserialize)]
/// Request for analyze operation.
pub struct AnalyzeRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Response from analyze operation.
pub struct AnalyzeResponse {
    pub complexity: u32,
    pub lines: usize,
}

impl Request for AnalyzeRequest {
    type Response = AnalyzeResponse;
}

#[derive(Debug, Serialize, Deserialize)]
/// Request for transform operation.
pub struct TransformRequest {
    pub code: String,
    pub transform_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Response from transform operation.
pub struct TransformResponse {
    pub transformed_code: String,
}

impl Request for TransformRequest {
    type Response = TransformResponse;
}

#[derive(Debug, Serialize, Deserialize)]
/// Request for validate operation.
pub struct ValidateRequest {
    pub code: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Response from validate operation.
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl Request for ValidateRequest {
    type Response = ValidateResponse;
}

// --- Impl blocks and methods ---
include!("request_response_broker.rs");

// --- Tests ---
include!("request_response_tests.rs");
