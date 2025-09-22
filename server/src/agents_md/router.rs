//! Request Router for Agent Protocols
//!
//! Routes requests between different agent protocols.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent request router
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    #[allow(dead_code)]
    Open,
    #[allow(dead_code)]
    HalfOpen,
}

impl Default for AgentRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRouter {
    /// Create new router
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            circuit_breaker: CircuitBreaker::new(),
        }
    }

    /// Route request to appropriate handler
    pub async fn route(&self, request: AgentRequest) -> Result<AgentResponse> {
        let handlers = self.handlers.read().await;

        for handler in handlers.iter() {
            if handler.protocol() == request.protocol {
                return handler.handle(&request).await;
            }
        }

        Err(anyhow::anyhow!(
            "No handler for protocol: {:?}",
            request.protocol
        ))
    }

    /// Register protocol handler
    pub async fn register_handler(&self, handler: Box<dyn Handler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// Load balance requests
    #[must_use]
    pub fn balance_load(&self, requests: Vec<AgentRequest>) -> Vec<RouteDecision> {
        requests
            .into_iter()
            .enumerate()
            .map(|(i, request)| {
                RouteDecision {
                    request,
                    handler_index: i % 3, // Simple round-robin
                }
            })
            .collect()
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    /// Create new circuit breaker
    #[must_use]
    pub fn new() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: 60,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
        }
    }
}

/// Supported protocols
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    AgentsMd,
    Mcp,
    Http,
    WebSocket,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler;

    #[async_trait::async_trait]
    impl Handler for TestHandler {
        async fn handle(&self, _request: &AgentRequest) -> Result<AgentResponse> {
            Ok(AgentResponse {
                request_id: "test".to_string(),
                success: true,
                result: serde_json::json!({}),
            })
        }

        fn protocol(&self) -> Protocol {
            Protocol::AgentsMd
        }
    }

    #[tokio::test]
    async fn test_router_creation() {
        let router = AgentRouter::new();
        let handlers = router.handlers.read().await;
        assert_eq!(handlers.len(), 0);
    }

    #[tokio::test]
    async fn test_handler_registration() {
        let router = AgentRouter::new();
        let handler = Box::new(TestHandler);
        router.register_handler(handler).await;

        let handlers = router.handlers.read().await;
        assert_eq!(handlers.len(), 1);
    }

    #[tokio::test]
    async fn test_request_routing() {
        let router = AgentRouter::new();
        router.register_handler(Box::new(TestHandler)).await;

        let request = AgentRequest {
            id: "test".to_string(),
            protocol: Protocol::AgentsMd,
            payload: serde_json::json!({}),
        };

        let response = router.route(request).await.unwrap();
        assert!(response.success);
    }

    #[test]
    fn test_load_balancing() {
        let router = AgentRouter::new();

        let requests = vec![
            AgentRequest {
                id: "1".to_string(),
                protocol: Protocol::AgentsMd,
                payload: serde_json::json!({}),
            },
            AgentRequest {
                id: "2".to_string(),
                protocol: Protocol::Mcp,
                payload: serde_json::json!({}),
            },
        ];

        let decisions = router.balance_load(requests);
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].handler_index, 0);
        assert_eq!(decisions[1].handler_index, 1);
    }
}
