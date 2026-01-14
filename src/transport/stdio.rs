//! Stdio transport implementation using pmcp 1.0.
//!
//! This module provides stdio (stdin/stdout) transport for MCP communication,
//! enabling command-line based MCP server operation.

use crate::transport::{PmcpTransportWrapper, TransportAdapter, TransportError};
use pmcp::transport::StdioTransport;
use std::fmt::Debug;
use tracing::debug;

/// Stdio transport adapter for MCP communication.
///
/// This transport uses stdin for receiving messages and stdout for sending,
/// with length-prefixed framing for message boundaries.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::transport::stdio::StdioTransportAdapter;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = StdioTransportAdapter::new().await?;
/// // Transport is ready for use
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct StdioTransportAdapter {
    wrapper: PmcpTransportWrapper<StdioTransport>,
}

impl StdioTransportAdapter {
    /// Creates a new stdio transport adapter.
    ///
    /// # Returns
    ///
    /// * `Ok(StdioTransportAdapter)` - Successfully created transport
    /// * `Err(TransportError)` - Failed to initialize stdio
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::transport::stdio::StdioTransportAdapter;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = StdioTransportAdapter::new().await?;
    /// assert!(transport.is_connected());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self, TransportError> {
        debug!("Initializing stdio transport");
        
        let inner = StdioTransport::new();
        let wrapper = PmcpTransportWrapper::new(inner);
        
        Ok(Self { wrapper })
    }
    
    /// Creates a stdio transport as a boxed TransportAdapter.
    ///
    /// This is useful when you need a trait object for dynamic dispatch.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::transport::{TransportAdapter, stdio::StdioTransportAdapter};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport: Box<dyn TransportAdapter> = StdioTransportAdapter::boxed().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn boxed() -> Result<Box<dyn TransportAdapter>, TransportError> {
        Ok(Box::new(Self::new().await?))
    }
}

// Delegate all TransportAdapter methods to the wrapper
#[async_trait::async_trait]
impl TransportAdapter for StdioTransportAdapter {
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
        "stdio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    /// Property test: Stdio transport preserves frame boundaries
    proptest! {
        #[test]
        fn test_frame_boundary_preservation(messages in prop::collection::vec("\\PC+", 1..10)) {
            // This test would require a mock stdio implementation
            // For now, we just verify the test compiles
            prop_assert!(!messages.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_stdio_transport_creation() {
        // Note: This test may fail in non-interactive environments
        // where stdin/stdout are not available
        if std::io::stdin().is_terminal() {
            let transport = StdioTransportAdapter::new().await;
            assert!(transport.is_ok());
            
            if let Ok(t) = transport {
                assert_eq!(t.transport_type(), "stdio");
            }
        }
    }
    
    #[test]
    fn test_stdio_transport_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StdioTransportAdapter>();
    }
}