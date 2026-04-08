#![cfg_attr(coverage_nightly, coverage(off))]
//! Request Router for Agent Protocols
//!
//! Routes requests between different agent protocols.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent request router
pub struct AgentRouter {
    /// Registered handlers
    handlers: Arc<RwLock<Vec<Box<dyn Handler>>>>,

    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
}

/// Request handler trait
#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    /// Handle request
    async fn handle(&self, request: &AgentRequest) -> Result<AgentResponse>;

    /// Get supported protocol
    fn protocol(&self) -> Protocol;
}

/// Agent request
#[derive(Debug, Clone)]
pub struct AgentRequest {
    /// Request ID
    pub id: String,

    /// Protocol
    pub protocol: Protocol,

    /// Payload
    pub payload: serde_json::Value,
}

/// Agent response
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// Request ID
    pub request_id: String,

    /// Success status
    pub success: bool,

    /// Result
    pub result: serde_json::Value,
}

/// Route decision
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// Request
    pub request: AgentRequest,

    /// Selected handler index
    pub handler_index: usize,
}

/// Circuit breaker for failing agents
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Failure threshold
    pub failure_threshold: u32,

    /// Reset timeout in seconds
    pub reset_timeout: u64,

    /// Current state
    state: Arc<RwLock<CircuitState>>,
}

/// Circuit breaker state
#[derive(Debug, Clone)]
enum CircuitState {
    Closed,

    Open,

    HalfOpen,
}

/// Supported protocols
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    AgentsMd,
    Mcp,
    Http,
    WebSocket,
}

// Routing implementation (AgentRouter + CircuitBreaker impl blocks)
include!("router_routing.rs");

// Tests
include!("router_tests.rs");
