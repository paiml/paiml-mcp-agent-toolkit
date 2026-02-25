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
