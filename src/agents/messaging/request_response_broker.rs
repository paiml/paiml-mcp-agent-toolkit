impl RequestResponseBroker {
    pub fn new(router: Arc<MessageRouter>) -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            router,
        }
    }

    pub async fn request(
        &self,
        from: Uuid,
        to: Uuid,
        request: impl Serialize,
        timeout: Duration,
    ) -> Result<AgentMessage, RequestError> {
        let correlation_id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();

        // Register response handler
        self.pending_requests.write().insert(correlation_id, tx);

        // Create and send request
        let message = AgentMessage::new(from, to, request)?
            .with_correlation(correlation_id)
            .with_ttl(timeout);

        self.router.route(message)?;

        // Wait for response with timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(RequestError::Cancelled),
            Err(_) => {
                // Clean up on timeout
                self.pending_requests.write().remove(&correlation_id);
                Err(RequestError::Timeout)
            }
        }
    }

    pub fn handle_response(&self, message: AgentMessage) {
        if let Some(correlation_id) = message.header.correlation_id {
            if let Some(tx) = self.pending_requests.write().remove(&correlation_id) {
                let _ = tx.send(message);
            }
        }
    }
}

impl Actor for RequestResponseActor {
    type Context = Context<Self>;
}

impl Handler<AgentMessage> for RequestResponseActor {
    type Result = Result<crate::agents::AgentResponse, crate::agents::AgentError>;

    fn handle(&mut self, msg: AgentMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.broker.handle_response(msg);
        Ok(crate::agents::AgentResponse::Success(serde_json::json!({})))
    }
}

// Typed request handler
pub async fn typed_request<R: Request>(
    broker: &RequestResponseBroker,
    from: Uuid,
    to: Uuid,
    request: R,
    timeout: Duration,
) -> Result<R::Response, RequestError> {
    let response = broker.request(from, to, request, timeout).await?;
    Ok(response.deserialize_payload()?)
}
