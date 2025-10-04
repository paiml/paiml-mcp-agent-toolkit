//! Mock transport implementation for testing.
//!
//! This module provides a deterministic mock transport for testing MCP
//! communication without requiring actual network or stdio connections.

use crate::transport::{TransportAdapter, TransportError};
use async_trait::async_trait;
use pmcp::transport::{Transport as PmcpTransport, TransportMessage};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

/// Mock transport for testing MCP communication.
///
/// This transport provides deterministic behavior for testing, with
/// configurable failure injection and message recording.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::transport::mock::MockTransport;
/// use pmcp::transport::TransportMessage;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut transport = MockTransport::new();
/// 
/// // Queue a response
/// transport.queue_response(TransportMessage::text(r#"{"result": "ok"}"#));
/// 
/// // Send a message
/// transport.send(TransportMessage::text(r#"{"method": "test"}"#)).await?;
/// 
/// // Receive the queued response
/// let response = transport.receive().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MockTransport {
    state: Arc<Mutex<MockState>>,
}

#[derive(Debug)]
struct MockState {
    /// Queue of messages to return on receive()
    receive_queue: VecDeque<TransportMessage>,
    /// History of all sent messages
    sent_messages: Vec<TransportMessage>,
    /// Whether the transport is connected
    connected: bool,
    /// Optional error to return on next operation
    next_error: Option<String>,
    /// Whether to simulate network delay
    simulate_delay: bool,
    /// Delay duration in milliseconds
    delay_ms: u64,
}

impl MockTransport {
    /// Creates a new mock transport.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                receive_queue: VecDeque::new(),
                sent_messages: Vec::new(),
                connected: true,
                next_error: None,
                simulate_delay: false,
                delay_ms: 10,
            })),
        }
    }
    
    /// Queues a message to be returned by the next receive() call.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::transport::mock::MockTransport;
    /// use pmcp::transport::TransportMessage;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut transport = MockTransport::new();
    /// transport.queue_response(TransportMessage::text("response")).await;
    /// 
    /// let msg = transport.receive().await?;
    /// assert_eq!(msg, TransportMessage::text("response"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn queue_response(&mut self, message: TransportMessage) {
        let mut state = self.state.lock().await;
        state.receive_queue.push_back(message);
    }
    
    /// Gets all messages that have been sent through this transport.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::transport::mock::MockTransport;
    /// use pmcp::transport::TransportMessage;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut transport = MockTransport::new();
    /// transport.send(TransportMessage::text("test")).await?;
    /// 
    /// let sent = transport.get_sent_messages().await;
    /// assert_eq!(sent.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_sent_messages(&self) -> Vec<TransportMessage> {
        let state = self.state.lock().await;
        state.sent_messages.clone()
    }
    
    /// Injects an error to be returned on the next operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::transport::mock::MockTransport;
    /// use pmcp::transport::TransportMessage;
    ///
    /// # async fn example() {
    /// let mut transport = MockTransport::new();
    /// transport.inject_error("Network error").await;
    /// 
    /// let result = transport.send(TransportMessage::text("test")).await;
    /// assert!(result.is_err());
    /// # }
    /// ```
    pub async fn inject_error(&mut self, error: impl Into<String>) {
        let mut state = self.state.lock().await;
        state.next_error = Some(error.into());
    }
    
    /// Simulates network delay for testing timeout handling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::transport::mock::MockTransport;
    ///
    /// # async fn example() {
    /// let mut transport = MockTransport::new();
    /// transport.set_delay(100).await; // 100ms delay
    /// # }
    /// ```
    pub async fn set_delay(&mut self, delay_ms: u64) {
        let mut state = self.state.lock().await;
        state.simulate_delay = true;
        state.delay_ms = delay_ms;
    }
    
    /// Simulates a connection drop.
    pub async fn disconnect(&mut self) {
        let mut state = self.state.lock().await;
        state.connected = false;
    }
    
    /// Clears all queued messages and sent history.
    pub async fn reset(&mut self) {
        let mut state = self.state.lock().await;
        state.receive_queue.clear();
        state.sent_messages.clear();
        state.next_error = None;
        state.connected = true;
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

// Implement pmcp Transport trait for direct use
#[async_trait]
impl PmcpTransport for MockTransport {
    async fn send(&mut self, message: TransportMessage) -> pmcp::Result<()> {
        let mut state = self.state.lock().await;
        
        // Check for injected error
        if let Some(error) = state.next_error.take() {
            return Err(pmcp::Error::transport(error));
        }
        
        // Check connection
        if !state.connected {
            return Err(pmcp::Error::transport("Not connected"));
        }
        
        // Simulate delay if configured
        if state.simulate_delay {
            tokio::time::sleep(tokio::time::Duration::from_millis(state.delay_ms)).await;
        }
        
        debug!("Mock transport sending message");
        state.sent_messages.push(message);
        Ok(())
    }
    
    async fn receive(&mut self) -> pmcp::Result<TransportMessage> {
        let mut state = self.state.lock().await;
        
        // Check for injected error
        if let Some(error) = state.next_error.take() {
            return Err(pmcp::Error::transport(error));
        }
        
        // Check connection
        if !state.connected {
            return Err(pmcp::Error::transport("Not connected"));
        }
        
        // Simulate delay if configured
        if state.simulate_delay {
            tokio::time::sleep(tokio::time::Duration::from_millis(state.delay_ms)).await;
        }
        
        state.receive_queue
            .pop_front()
            .ok_or_else(|| pmcp::Error::transport("No messages in queue"))
    }
    
    async fn close(&mut self) -> pmcp::Result<()> {
        let mut state = self.state.lock().await;
        state.connected = false;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.state.try_lock().map(|s| s.connected).unwrap_or(false)
    }
    
    fn transport_type(&self) -> &'static str {
        "mock"
    }
}

// Also implement our TransportAdapter trait
#[async_trait]
impl TransportAdapter for MockTransport {
    async fn send(&mut self, message: TransportMessage) -> Result<(), TransportError> {
        PmcpTransport::send(self, message)
            .await
            .map_err(|e| TransportError::Send(e.to_string()))
    }
    
    async fn receive(&mut self) -> Result<TransportMessage, TransportError> {
        PmcpTransport::receive(self)
            .await
            .map_err(|e| TransportError::Receive(e.to_string()))
    }
    
    async fn close(&mut self) -> Result<(), TransportError> {
        PmcpTransport::close(self)
            .await
            .map_err(|e| TransportError::Connection(e.to_string()))
    }
    
    fn is_connected(&self) -> bool {
        PmcpTransport::is_connected(self)
    }
    
    fn transport_type(&self) -> &'static str {
        PmcpTransport::transport_type(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// Property test: Mock transport preserves message ordering
        #[test]
        fn test_message_ordering(messages in prop::collection::vec("\\PC+", 1..20)) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut transport = MockTransport::new();
                
                // Queue all messages
                for msg in &messages {
                    transport.queue_response(TransportMessage::text(msg)).await;
                }
                
                // Receive should return in same order
                for expected in &messages {
                    let received = transport.receive().await.unwrap();
                    assert_eq!(received, TransportMessage::text(expected));
                }
            });
        }
        
        /// Property test: Sent messages are recorded correctly
        #[test]
        fn test_sent_message_recording(messages in prop::collection::vec("\\PC+", 1..10)) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut transport = MockTransport::new();
                
                // Send all messages
                for msg in &messages {
                    transport.send(TransportMessage::text(msg)).await.unwrap();
                }
                
                // Check they were recorded
                let sent = transport.get_sent_messages().await;
                assert_eq!(sent.len(), messages.len());
                
                for (sent_msg, expected) in sent.iter().zip(messages.iter()) {
                    assert_eq!(*sent_msg, TransportMessage::text(expected));
                }
            });
        }
    }
    
    #[tokio::test]
    async fn test_mock_transport_error_injection() {
        let mut transport = MockTransport::new();
        
        // Inject error
        transport.inject_error("Test error").await;
        
        // Next operation should fail
        let result = transport.send(TransportMessage::text("test")).await;
        assert!(result.is_err());
        
        // Subsequent operations should work
        let result = transport.send(TransportMessage::text("test")).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_mock_transport_connection_state() {
        let mut transport = MockTransport::new();
        
        assert!(transport.is_connected());
        
        // Disconnect
        transport.disconnect().await;
        assert!(!transport.is_connected());
        
        // Operations should fail
        let result = transport.send(TransportMessage::text("test")).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_mock_transport_delay_simulation() {
        let mut transport = MockTransport::new();
        transport.set_delay(50).await;
        
        let start = tokio::time::Instant::now();
        transport.send(TransportMessage::text("test")).await.unwrap();
        let elapsed = start.elapsed();
        
        // Should have taken at least 50ms
        assert!(elapsed.as_millis() >= 50);
    }
}