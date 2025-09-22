use super::*;

// WebSocket transport for browser clients
pub struct WebSocketTransport {
    // Implementation would go here
}

#[async_trait]
impl McpTransport for WebSocketTransport {
    async fn send(&self, _message: McpMessage) -> Result<(), McpError> {
        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        Err(McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Not implemented".to_string(),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        Ok(())
    }
}

// HTTP/SSE transport for REST clients
pub struct HttpTransport {
    // Implementation would go here
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn send(&self, _message: McpMessage) -> Result<(), McpError> {
        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        Err(McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Not implemented".to_string(),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        Ok(())
    }
}
