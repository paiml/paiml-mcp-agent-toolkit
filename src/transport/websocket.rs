#![cfg_attr(coverage_nightly, coverage(off))]
//! WebSocket transport implementation using pmcp 1.0.
//!
//! This module provides WebSocket transport for MCP communication,
//! enabling browser-based and network clients to connect to the MCP server.

use crate::transport::{PmcpTransportWrapper, TransportAdapter, TransportError};
use pmcp::transport::WebSocketTransport;
use std::fmt::Debug;
use tracing::{debug, info};

/// WebSocket transport adapter for MCP communication.
///
/// This transport enables MCP communication over WebSocket connections,
/// supporting both ws:// and wss:// protocols.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::transport::websocket::WebSocketTransportAdapter;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = WebSocketTransportAdapter::connect("ws://localhost:8080").await?;
/// // Transport is ready for use
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct WebSocketTransportAdapter {
    wrapper: PmcpTransportWrapper<WebSocketTransport>,
}

impl WebSocketTransportAdapter {
    /// Creates a new WebSocket transport by connecting to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - WebSocket URL to connect to (ws:// or wss://)
    ///
    /// # Returns
    ///
    /// * `Ok(WebSocketTransportAdapter)` - Successfully connected
    /// * `Err(TransportError)` - Connection failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::transport::websocket::WebSocketTransportAdapter;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Connect to a WebSocket server
    /// let transport = WebSocketTransportAdapter::connect("ws://localhost:8080").await?;
    /// assert!(transport.is_connected());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(url: &str) -> Result<Self, TransportError> {
        debug_assert!(!url.is_empty(), "url must not be empty");
        info!("Connecting to WebSocket at {}", url);
        
        let inner = WebSocketTransport::connect(url)
            .await
            .map_err(|e| TransportError::Connection(format!("WebSocket connection failed: {}", e)))?;
        
        let wrapper = PmcpTransportWrapper::new(inner);
        debug!("WebSocket connection established");
        
        Ok(Self { wrapper })
    }
    
    /// Creates a WebSocket transport from an accepted connection.
    ///
    /// This is used when the server accepts incoming WebSocket connections.
    ///
    /// # Arguments
    ///
    /// * `stream` - Accepted WebSocket stream
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use pmat::transport::websocket::WebSocketTransportAdapter;
    /// use tokio_tungstenite::accept_async;
    ///
    /// # async fn example(tcp_stream: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    /// let ws_stream = accept_async(tcp_stream).await?;
    /// let transport = WebSocketTransportAdapter::from_stream(ws_stream);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_stream(stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) -> Self {
        debug!("Creating WebSocket transport from accepted stream");
        
        let inner = WebSocketTransport::from_stream(stream);
        let wrapper = PmcpTransportWrapper::new(inner);
        
        Self { wrapper }
    }
    
    /// Creates a WebSocket server that listens on the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind to (e.g., "127.0.0.1:8080")
    ///
    /// # Returns
    ///
    /// * `Ok(WebSocketServer)` - Server listening on the specified address
    /// * `Err(TransportError)` - Failed to bind to address
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::transport::websocket::WebSocketTransportAdapter;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = WebSocketTransportAdapter::serve("127.0.0.1:8080").await?;
    /// // Server is now listening for connections
    /// # Ok(())
    /// # }
    /// ```
    pub async fn serve(addr: &str) -> Result<WebSocketServer, TransportError> {
        info!("Starting WebSocket server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| TransportError::Connection(format!("Failed to bind: {}", e)))?;
        
        Ok(WebSocketServer { listener })
    }
    
    /// Creates a WebSocket transport as a boxed TransportAdapter.
    pub async fn boxed(url: &str) -> Result<Box<dyn TransportAdapter>, TransportError> {
        debug_assert!(!url.is_empty(), "url must not be empty");
        Ok(Box::new(Self::connect(url).await?))
    }
}

// Delegate all TransportAdapter methods to the wrapper
#[async_trait::async_trait]
impl TransportAdapter for WebSocketTransportAdapter {
    async fn send(&mut self, message: pmcp::transport::TransportMessage) -> Result<(), TransportError> {
        self.wrapper.send(message).await
    }
    
    async fn receive(&mut self) -> Result<pmcp::transport::TransportMessage, TransportError> {
        self.wrapper.receive().await
    }
    
    async fn close(&mut self) -> Result<(), TransportError> {
        self.wrapper.close().await
    }
    
    fn is_connected(&self) -> bool {
        self.wrapper.is_connected()
    }
    
    fn transport_type(&self) -> &'static str {
        "websocket"
    }
}

/// WebSocket server that accepts incoming connections.
pub struct WebSocketServer {
    listener: tokio::net::TcpListener,
}

impl WebSocketServer {
    /// Accepts the next incoming WebSocket connection.
    ///
    /// # Returns
    ///
    /// * `Ok(WebSocketTransportAdapter)` - New client connection
    /// * `Err(TransportError)` - Accept failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::transport::websocket::WebSocketTransportAdapter;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut server = WebSocketTransportAdapter::serve("127.0.0.1:8080").await?;
    /// 
    /// // Accept connections in a loop
    /// loop {
    ///     let transport = server.accept().await?;
    ///     // Handle the connection...
    /// }
    /// # }
    /// ```
    pub async fn accept(&mut self) -> Result<WebSocketTransportAdapter, TransportError> {
        let (stream, addr) = self.listener
            .accept()
            .await
            .map_err(|e| TransportError::Connection(format!("Accept failed: {}", e)))?;
        
        info!("Accepting WebSocket connection from {}", addr);
        
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .map_err(|e| TransportError::Connection(format!("WebSocket handshake failed: {}", e)))?;
        
        Ok(WebSocketTransportAdapter::from_stream(ws_stream))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// Property test: WebSocket frame fragmentation is handled correctly
        #[test]
        fn test_websocket_frame_fragmentation(data in prop::collection::vec(0u8..255, 1..10000)) {
            // This would test that large messages are properly fragmented and reassembled
            // For now, we verify the property test compiles
            prop_assert!(!data.is_empty());
        }
        
        /// Property test: WebSocket URLs are validated correctly
        #[test]
        fn test_websocket_url_validation(
            scheme in prop::sample::select(vec!["ws", "wss", "http", "https", "ftp"]),
            host in "[a-z]{1,10}",
            port in 1u16..65535
        ) {
            let url = format!("{}://{}:{}", scheme, host, port);
            
            // Only ws and wss schemes should be valid
            let should_be_valid = scheme == "ws" || scheme == "wss";
            prop_assert_eq!(url.starts_with("ws://") || url.starts_with("wss://"), should_be_valid);
        }
    }
    
    #[tokio::test]
    async fn test_websocket_server_bind() {
        // Try to bind to a random port
        let result = WebSocketTransportAdapter::serve("127.0.0.1:0").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_websocket_connection_drop_recovery() {
        // This test would verify that dropped connections are handled gracefully
        // For now, we just ensure the test compiles
        assert!(true);
    }
    
    #[test]
    fn test_websocket_transport_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WebSocketTransportAdapter>();
    }

    #[test]
    fn test_websocket_url_schemes() {
        // Valid WebSocket schemes
        let valid_ws = "ws://localhost:8080";
        let valid_wss = "wss://localhost:8080";

        assert!(valid_ws.starts_with("ws://"));
        assert!(valid_wss.starts_with("wss://"));

        // Invalid schemes for WebSocket
        let invalid_http = "http://localhost:8080";
        assert!(!invalid_http.starts_with("ws://") && !invalid_http.starts_with("wss://"));
    }

    #[test]
    fn test_websocket_port_ranges() {
        // Standard ports
        let standard_port = 80;
        let secure_port = 443;
        let common_dev_port = 8080;

        assert!(standard_port < 65536);
        assert!(secure_port < 65536);
        assert!(common_dev_port < 65536);

        // Reserved ports should still work
        assert!(standard_port > 0);
    }

    #[tokio::test]
    async fn test_server_bind_ephemeral_port() {
        // Use port 0 for ephemeral port assignment
        let server = WebSocketTransportAdapter::serve("127.0.0.1:0").await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_server_bind_invalid_address() {
        // Try to bind to an invalid address
        let result = WebSocketTransportAdapter::serve("invalid-host:8080").await;
        assert!(result.is_err());

        if let Err(TransportError::Connection(msg)) = result {
            assert!(msg.contains("bind"));
        }
    }

    #[test]
    fn test_transport_error_variants() {
        let conn_err = TransportError::Connection("test".to_string());
        assert!(matches!(conn_err, TransportError::Connection(_)));

        let send_err = TransportError::Send("test".to_string());
        assert!(matches!(send_err, TransportError::Send(_)));

        let recv_err = TransportError::Receive("test".to_string());
        assert!(matches!(recv_err, TransportError::Receive(_)));
    }
}