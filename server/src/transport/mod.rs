//! Transport layer abstractions for MCP communication.
//!
//! This module provides a unified transport abstraction that wraps pmcp 1.0's
//! transport implementations, enabling stdio, WebSocket, and HTTP/SSE communication
//! for the MCP server.

use async_trait::async_trait;
use pmcp::transport::{Transport as PmcpTransport, TransportMessage};
use std::fmt::Debug;
use thiserror::Error;

pub mod http_sse;
pub mod stdio;
pub mod websocket;

#[cfg(test)]
pub mod mock;

/// Transport error types
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Send error: {0}")]
    Send(String),
    
    #[error("Receive error: {0}")]
    Receive(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Transport closed")]
    Closed,
}

impl From<pmcp::Error> for TransportError {
    fn from(err: pmcp::Error) -> Self {
        TransportError::Protocol(err.to_string())
    }
}

/// Unified transport adapter trait that wraps pmcp transports.
///
/// This trait provides a common interface for all transport types,
/// ensuring consistent behavior across stdio, WebSocket, and HTTP/SSE.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::transport::{TransportAdapter, stdio::StdioTransportAdapter};
/// use pmcp::transport::TransportMessage;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut transport = StdioTransportAdapter::new().await?;
/// 
/// // Send a message
/// let msg = TransportMessage::text(r#"{"jsonrpc":"2.0","method":"test"}"#);
/// transport.send(msg).await?;
/// 
/// // Receive a message
/// let response = transport.receive().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait TransportAdapter: Send + Sync + Debug {
    /// Sends a message through the transport.
    async fn send(&mut self, message: TransportMessage) -> Result<(), TransportError>;
    
    /// Receives a message from the transport.
    async fn receive(&mut self) -> Result<TransportMessage, TransportError>;
    
    /// Closes the transport connection.
    async fn close(&mut self) -> Result<(), TransportError>;
    
    /// Checks if the transport is connected.
    fn is_connected(&self) -> bool;
    
    /// Returns the transport type name.
    fn transport_type(&self) -> &'static str;
}

/// Wrapper that adapts pmcp transports to our TransportAdapter trait.
#[derive(Debug)]
pub struct PmcpTransportWrapper<T: PmcpTransport> {
    inner: T,
}

impl<T: PmcpTransport> PmcpTransportWrapper<T> {
    /// Creates a new transport wrapper.
    pub fn new(transport: T) -> Self {
        Self { inner: transport }
    }
}

#[async_trait]
impl<T: PmcpTransport + 'static> TransportAdapter for PmcpTransportWrapper<T> {
    async fn send(&mut self, message: TransportMessage) -> Result<(), TransportError> {
        self.inner
            .send(message)
            .await
            .map_err(|e| TransportError::Send(e.to_string()))
    }
    
    async fn receive(&mut self) -> Result<TransportMessage, TransportError> {
        self.inner
            .receive()
            .await
            .map_err(|e| TransportError::Receive(e.to_string()))
    }
    
    async fn close(&mut self) -> Result<(), TransportError> {
        self.inner
            .close()
            .await
            .map_err(|e| TransportError::Connection(e.to_string()))
    }
    
    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    
    fn transport_type(&self) -> &'static str {
        self.inner.transport_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// Property test: Transport error conversions preserve error information
        #[test]
        fn test_error_conversion_preserves_info(msg in "\\PC+") {
            let pmcp_err = pmcp::Error::internal(msg.clone());
            let transport_err: TransportError = pmcp_err.into();
            
            match transport_err {
                TransportError::Protocol(err_msg) => {
                    prop_assert!(err_msg.contains(&msg));
                }
                _ => prop_assert!(false, "Expected Protocol error"),
            }
        }
    }
    
    #[tokio::test]
    async fn test_transport_wrapper_delegates_correctly() {
        use crate::transport::mock::MockTransport;
        
        let mock = MockTransport::new();
        let mut wrapper = PmcpTransportWrapper::new(mock);
        
        assert!(wrapper.is_connected());
        assert_eq!(wrapper.transport_type(), "mock");
        
        // Test send/receive
        let msg = TransportMessage::text("test");
        wrapper.send(msg.clone()).await.unwrap();
        
        let received = wrapper.receive().await.unwrap();
        assert_eq!(received, msg);
    }
}