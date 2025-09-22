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

// Pattern matching for request types
pub trait Request: Serialize + for<'de> Deserialize<'de> + Send {
    type Response: Serialize + for<'de> Deserialize<'de> + Send;

    fn priority(&self) -> Priority {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_request_response() {
        let router = Arc::new(MessageRouter::new());
        let broker = RequestResponseBroker::new(router.clone());

        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        // Simulate response handler
        let broker_clone = Arc::new(broker);
        let broker_handle = broker_clone.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;

            let response = AgentMessage::new(to, from, "response")
                .unwrap()
                .with_correlation(Uuid::new_v4()); // Would need actual correlation ID

            broker_handle.handle_response(response);
        });

        // Test timeout
        let result = broker_clone
            .request(from, to, "test", Duration::from_millis(5))
            .await;

        assert!(matches!(result, Err(RequestError::Timeout)));
    }
}
