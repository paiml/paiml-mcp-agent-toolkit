#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use actix::prelude::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct RequestResponseBroker {
    pending_requests: Arc<RwLock<HashMap<Uuid, oneshot::Sender<AgentMessage>>>>,
    router: Arc<MessageRouter>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Request timeout")]
    Timeout,
    #[error("Request cancelled")]
    Cancelled,
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Router error: {0}")]
    Router(#[from] RouterError),
}

// Actor wrapper for request-response
pub struct RequestResponseActor {
    broker: Arc<RequestResponseBroker>,
}

// Pattern matching for request types
pub trait Request: Serialize + for<'de> Deserialize<'de> + Send {
    type Response: Serialize + for<'de> Deserialize<'de> + Send;

    fn priority(&self) -> Priority {
        debug_assert!(true, "contract: priority");
        Priority::Normal
    }
}

// Example request types
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub complexity: u32,
    pub lines: usize,
}

impl Request for AnalyzeRequest {
    type Response = AnalyzeResponse;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransformRequest {
    pub code: String,
    pub transform_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransformResponse {
    pub transformed_code: String,
}

impl Request for TransformRequest {
    type Response = TransformResponse;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub code: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
