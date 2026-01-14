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

    struct McpHandler;

    #[async_trait::async_trait]
    impl Handler for McpHandler {
        async fn handle(&self, request: &AgentRequest) -> Result<AgentResponse> {
            Ok(AgentResponse {
                request_id: request.id.clone(),
                success: true,
                result: serde_json::json!({"protocol": "mcp"}),
            })
        }

        fn protocol(&self) -> Protocol {
            Protocol::Mcp
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

    // === AgentRouter additional tests ===

    #[tokio::test]
    async fn test_router_default() {
        let router = AgentRouter::default();
        // Should be equivalent to AgentRouter::new()
        let handlers = router.handlers.read().await;
        assert!(handlers.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_handler_registration() {
        let router = AgentRouter::new();
        router.register_handler(Box::new(TestHandler)).await;
        router.register_handler(Box::new(McpHandler)).await;

        let handlers = router.handlers.read().await;
        assert_eq!(handlers.len(), 2);
    }

    #[tokio::test]
    async fn test_routing_no_handler() {
        let router = AgentRouter::new();
        router.register_handler(Box::new(TestHandler)).await;

        let request = AgentRequest {
            id: "test".to_string(),
            protocol: Protocol::Http, // No handler for HTTP
            payload: serde_json::json!({}),
        };

        let result = router.route(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No handler"));
    }

    #[tokio::test]
    async fn test_routing_selects_correct_handler() {
        let router = AgentRouter::new();
        router.register_handler(Box::new(TestHandler)).await;
        router.register_handler(Box::new(McpHandler)).await;

        let request = AgentRequest {
            id: "mcp-test".to_string(),
            protocol: Protocol::Mcp,
            payload: serde_json::json!({}),
        };

        let response = router.route(request).await.unwrap();
        assert!(response.success);
        assert_eq!(response.result["protocol"], "mcp");
    }

    // === AgentRequest tests ===

    #[test]
    fn test_agent_request_clone() {
        let request = AgentRequest {
            id: "test-id".to_string(),
            protocol: Protocol::AgentsMd,
            payload: serde_json::json!({"key": "value"}),
        };

        let cloned = request.clone();
        assert_eq!(cloned.id, request.id);
        assert_eq!(cloned.protocol, request.protocol);
        assert_eq!(cloned.payload, request.payload);
    }

    #[test]
    fn test_agent_request_debug() {
        let request = AgentRequest {
            id: "debug-test".to_string(),
            protocol: Protocol::Mcp,
            payload: serde_json::json!({}),
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("AgentRequest"));
        assert!(debug_str.contains("debug-test"));
    }

    // === AgentResponse tests ===

    #[test]
    fn test_agent_response_clone() {
        let response = AgentResponse {
            request_id: "req-1".to_string(),
            success: true,
            result: serde_json::json!({"data": 123}),
        };

        let cloned = response.clone();
        assert_eq!(cloned.request_id, response.request_id);
        assert_eq!(cloned.success, response.success);
        assert_eq!(cloned.result, response.result);
    }

    #[test]
    fn test_agent_response_debug() {
        let response = AgentResponse {
            request_id: "req-debug".to_string(),
            success: false,
            result: serde_json::json!(null),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("AgentResponse"));
        assert!(debug_str.contains("req-debug"));
    }

    // === RouteDecision tests ===

    #[test]
    fn test_route_decision_clone() {
        let decision = RouteDecision {
            request: AgentRequest {
                id: "dec-1".to_string(),
                protocol: Protocol::Http,
                payload: serde_json::json!({}),
            },
            handler_index: 5,
        };

        let cloned = decision.clone();
        assert_eq!(cloned.request.id, decision.request.id);
        assert_eq!(cloned.handler_index, decision.handler_index);
    }

    #[test]
    fn test_route_decision_debug() {
        let decision = RouteDecision {
            request: AgentRequest {
                id: "dec-debug".to_string(),
                protocol: Protocol::WebSocket,
                payload: serde_json::json!({}),
            },
            handler_index: 3,
        };

        let debug_str = format!("{:?}", decision);
        assert!(debug_str.contains("RouteDecision"));
        assert!(debug_str.contains("handler_index"));
    }

    // === CircuitBreaker tests ===

    #[test]
    fn test_circuit_breaker_new() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.failure_threshold, 5);
        assert_eq!(cb.reset_timeout, 60);
    }

    #[test]
    fn test_circuit_breaker_default() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.failure_threshold, 5);
        assert_eq!(cb.reset_timeout, 60);
    }

    #[test]
    fn test_circuit_breaker_clone() {
        let cb = CircuitBreaker::new();
        let cloned = cb.clone();
        assert_eq!(cloned.failure_threshold, cb.failure_threshold);
        assert_eq!(cloned.reset_timeout, cb.reset_timeout);
    }

    #[test]
    fn test_circuit_breaker_debug() {
        let cb = CircuitBreaker::new();
        let debug_str = format!("{:?}", cb);
        assert!(debug_str.contains("CircuitBreaker"));
        assert!(debug_str.contains("failure_threshold"));
    }

    // === Protocol tests ===

    #[test]
    fn test_protocol_eq() {
        assert_eq!(Protocol::AgentsMd, Protocol::AgentsMd);
        assert_eq!(Protocol::Mcp, Protocol::Mcp);
        assert_eq!(Protocol::Http, Protocol::Http);
        assert_eq!(Protocol::WebSocket, Protocol::WebSocket);
    }

    #[test]
    fn test_protocol_ne() {
        assert_ne!(Protocol::AgentsMd, Protocol::Mcp);
        assert_ne!(Protocol::Http, Protocol::WebSocket);
    }

    #[test]
    fn test_protocol_clone() {
        let p = Protocol::AgentsMd;
        let cloned = p.clone();
        assert_eq!(p, cloned);
    }

    #[test]
    fn test_protocol_debug() {
        assert!(format!("{:?}", Protocol::AgentsMd).contains("AgentsMd"));
        assert!(format!("{:?}", Protocol::Mcp).contains("Mcp"));
        assert!(format!("{:?}", Protocol::Http).contains("Http"));
        assert!(format!("{:?}", Protocol::WebSocket).contains("WebSocket"));
    }

    // === Load balancing tests ===

    #[test]
    fn test_load_balancing_empty() {
        let router = AgentRouter::new();
        let decisions = router.balance_load(vec![]);
        assert!(decisions.is_empty());
    }

    #[test]
    fn test_load_balancing_round_robin() {
        let router = AgentRouter::new();

        let requests: Vec<AgentRequest> = (0..6)
            .map(|i| AgentRequest {
                id: i.to_string(),
                protocol: Protocol::AgentsMd,
                payload: serde_json::json!({}),
            })
            .collect();

        let decisions = router.balance_load(requests);

        // Round robin mod 3
        assert_eq!(decisions[0].handler_index, 0);
        assert_eq!(decisions[1].handler_index, 1);
        assert_eq!(decisions[2].handler_index, 2);
        assert_eq!(decisions[3].handler_index, 0);
        assert_eq!(decisions[4].handler_index, 1);
        assert_eq!(decisions[5].handler_index, 2);
    }

    #[test]
    fn test_load_balancing_preserves_requests() {
        let router = AgentRouter::new();

        let requests = vec![
            AgentRequest {
                id: "first".to_string(),
                protocol: Protocol::AgentsMd,
                payload: serde_json::json!({"data": 1}),
            },
            AgentRequest {
                id: "second".to_string(),
                protocol: Protocol::Http,
                payload: serde_json::json!({"data": 2}),
            },
        ];

        let decisions = router.balance_load(requests);

        assert_eq!(decisions[0].request.id, "first");
        assert_eq!(decisions[1].request.id, "second");
    }
}
