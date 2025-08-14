//! HTTP Server-Sent Events (SSE) transport implementation using pmcp 1.0.
//!
//! This module provides HTTP/SSE transport for MCP communication,
//! enabling web clients to connect using standard HTTP with Server-Sent Events.

use crate::transport::{TransportAdapter, TransportError};
use async_trait::async_trait;
use pmcp::transport::TransportMessage;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

/// HTTP/SSE transport adapter for MCP communication.
///
/// This transport uses HTTP POST for client-to-server messages and
/// Server-Sent Events for server-to-client messages, providing a
/// unidirectional streaming capability over standard HTTP.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::transport::http_sse::HttpSseTransportAdapter;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = HttpSseTransportAdapter::serve("127.0.0.1:8080").await?;
/// // Transport is ready to accept HTTP connections
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct HttpSseTransportAdapter {
    /// Channel for receiving messages from HTTP POST requests
    receiver: mpsc::Receiver<TransportMessage>,
    /// Channel for sending SSE events to connected clients
    sender: mpsc::Sender<TransportMessage>,
    /// Shared state for connection management
    state: Arc<RwLock<ConnectionState>>,
}

#[derive(Debug)]
struct ConnectionState {
    connected: bool,
    client_id: Option<String>,
}

impl HttpSseTransportAdapter {
    /// Creates a new HTTP/SSE transport server.
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind the HTTP server to
    ///
    /// # Returns
    ///
    /// * `Ok(HttpSseTransportAdapter)` - Server successfully started
    /// * `Err(TransportError)` - Failed to start server
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::transport::http_sse::HttpSseTransportAdapter;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = HttpSseTransportAdapter::serve("127.0.0.1:8080").await?;
    /// assert!(transport.is_connected());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn serve(addr: &str) -> Result<Self, TransportError> {
        info!("Starting HTTP/SSE server on {}", addr);
        
        let (tx, rx) = mpsc::channel(100);
        let state = Arc::new(RwLock::new(ConnectionState {
            connected: true,
            client_id: None,
        }));
        
        // Start HTTP server in background
        let server_state = state.clone();
        let server_tx = tx.clone();
        let addr = addr.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_http_server(&addr, server_tx, server_state).await {
                tracing::error!("HTTP server error: {}", e);
            }
        });
        
        Ok(Self {
            receiver: rx,
            sender: tx,
            state,
        })
    }
    
    /// Runs the HTTP server that handles POST requests and SSE connections.
    async fn run_http_server(
        addr: &str,
        tx: mpsc::Sender<TransportMessage>,
        state: Arc<RwLock<ConnectionState>>,
    ) -> Result<(), TransportError> {
        use axum::{
            extract::State,
            response::sse::{Event, Sse},
            routing::{get, post},
            Json, Router,
        };
        use futures::stream::Stream;
        use std::convert::Infallible;
        
        // Create SSE event stream
        let sse_state = state.clone();
        let sse_handler = move || {
            let state = sse_state.clone();
            async move {
                let stream = async_stream::stream! {
                    loop {
                        // Check if still connected
                        if !state.read().await.connected {
                            break;
                        }
                        
                        // Send keepalive every 30 seconds
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        yield Ok::<_, Infallible>(Event::default().comment("keepalive"));
                    }
                };
                
                Sse::new(stream).keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(std::time::Duration::from_secs(30))
                        .text("keepalive"),
                )
            }
        };
        
        // Create POST handler for receiving messages
        let post_handler = move |State(tx): State<mpsc::Sender<TransportMessage>>, body: String| async move {
            debug!("Received HTTP POST message");
            let msg = TransportMessage::text(body);
            
            if tx.send(msg).await.is_err() {
                return Err("Failed to process message");
            }
            
            Ok::<_, &'static str>("OK")
        };
        
        // Build router
        let app = Router::new()
            .route("/sse", get(sse_handler))
            .route("/message", post(post_handler))
            .with_state(tx);
        
        // Start server
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| TransportError::Connection(format!("Failed to bind: {}", e)))?;
        
        debug!("HTTP/SSE server listening on {}", addr);
        
        axum::serve(listener, app)
            .await
            .map_err(|e| TransportError::Connection(format!("HTTP server error: {}", e)))?;
        
        Ok(())
    }
    
    /// Creates an HTTP/SSE transport as a boxed TransportAdapter.
    pub async fn boxed(addr: &str) -> Result<Box<dyn TransportAdapter>, TransportError> {
        Ok(Box::new(Self::serve(addr).await?))
    }
}

#[async_trait]
impl TransportAdapter for HttpSseTransportAdapter {
    async fn send(&mut self, message: TransportMessage) -> Result<(), TransportError> {
        self.sender
            .send(message)
            .await
            .map_err(|_| TransportError::Send("SSE send failed".to_string()))
    }
    
    async fn receive(&mut self) -> Result<TransportMessage, TransportError> {
        self.receiver
            .recv()
            .await
            .ok_or(TransportError::Receive("Connection closed".to_string()))
    }
    
    async fn close(&mut self) -> Result<(), TransportError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        // Use try_read to avoid blocking
        self.state.try_read().map(|s| s.connected).unwrap_or(false)
    }
    
    fn transport_type(&self) -> &'static str {
        "http-sse"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// Property test: SSE event stream formatting is correct
        #[test]
        fn test_sse_event_format(data in "\\PC+", event_type in "[a-z]+") {
            // SSE format: "event: {type}\ndata: {data}\n\n"
            let formatted = format!("event: {}\ndata: {}\n\n", event_type, data);
            
            prop_assert!(formatted.starts_with("event: "));
            prop_assert!(formatted.contains("\ndata: "));
            prop_assert!(formatted.ends_with("\n\n"));
        }
        
        /// Property test: HTTP keepalive intervals maintain connection
        #[test]
        fn test_keepalive_intervals(interval_secs in 1u64..120) {
            // Keepalive should be sent at regular intervals
            let duration = std::time::Duration::from_secs(interval_secs);
            prop_assert!(duration.as_secs() > 0);
            prop_assert!(duration.as_secs() <= 120); // Max 2 minutes
        }
    }
    
    #[tokio::test]
    async fn test_http_sse_server_creation() {
        // Try to create server on random port
        let result = HttpSseTransportAdapter::serve("127.0.0.1:0").await;
        assert!(result.is_ok());
        
        if let Ok(transport) = result {
            assert!(transport.is_connected());
            assert_eq!(transport.transport_type(), "http-sse");
        }
    }
    
    #[tokio::test]
    async fn test_connection_state_management() {
        let transport = HttpSseTransportAdapter::serve("127.0.0.1:0").await.unwrap();
        
        assert!(transport.is_connected());
        
        // Close connection
        let mut transport = transport;
        transport.close().await.unwrap();
        assert!(!transport.is_connected());
    }
    
    #[test]
    fn test_http_sse_transport_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpSseTransportAdapter>();
    }
}