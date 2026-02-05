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

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    // === TransportError tests ===

    #[test]
    fn test_transport_error_connection() {
        let err = TransportError::Connection("connection lost".to_string());
        assert!(err.to_string().contains("connection lost"));
        assert!(err.to_string().contains("Connection"));
    }

    #[test]
    fn test_transport_error_send() {
        let err = TransportError::Send("send failed".to_string());
        assert!(err.to_string().contains("send failed"));
        assert!(err.to_string().contains("Send"));
    }

    #[test]
    fn test_transport_error_receive() {
        let err = TransportError::Receive("receive failed".to_string());
        assert!(err.to_string().contains("receive failed"));
        assert!(err.to_string().contains("Receive"));
    }

    #[test]
    fn test_transport_error_serialization() {
        let err = TransportError::Serialization("serialization failed".to_string());
        assert!(err.to_string().contains("serialization failed"));
        assert!(err.to_string().contains("Serialization"));
    }

    #[test]
    fn test_transport_error_protocol() {
        let err = TransportError::Protocol("protocol error".to_string());
        assert!(err.to_string().contains("protocol error"));
        assert!(err.to_string().contains("Protocol"));
    }

    #[test]
    fn test_transport_error_timeout() {
        let err = TransportError::Timeout;
        assert!(err.to_string().contains("Timeout"));
    }

    #[test]
    fn test_transport_error_closed() {
        let err = TransportError::Closed;
        assert!(err.to_string().contains("closed"));
    }

    #[test]
    fn test_transport_error_debug() {
        let err = TransportError::Connection("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Connection"));
        assert!(debug_str.contains("test"));
    }

    // === From<pmcp::Error> tests ===

    #[test]
    fn test_from_pmcp_error() {
        let pmcp_err = pmcp::Error::internal("internal error");
        let transport_err: TransportError = pmcp_err.into();

        match transport_err {
            TransportError::Protocol(msg) => {
                assert!(msg.contains("internal"));
            }
            _ => panic!("Expected Protocol error"),
        }
    }

    // === PmcpTransportWrapper tests ===

    #[test]
    fn test_pmcp_wrapper_debug() {
        use crate::transport::mock::MockTransport;

        let mock = MockTransport::new();
        let wrapper = PmcpTransportWrapper::new(mock);
        let debug_str = format!("{:?}", wrapper);
        assert!(debug_str.contains("PmcpTransportWrapper"));
    }

    #[tokio::test]
    async fn test_wrapper_close() {
        use crate::transport::mock::MockTransport;

        let mock = MockTransport::new();
        let mut wrapper = PmcpTransportWrapper::new(mock);

        let result = wrapper.close().await;
        assert!(result.is_ok());
    }
}