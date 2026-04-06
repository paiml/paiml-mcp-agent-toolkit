async fn handle_session(
    session: McpSession,
    config: ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let transport = session.transport.clone();

    loop {
        tokio::select! {
            message = transport.receive() => {
                let message = message?;

                match message {
                    McpMessage::JsonRpc(JsonRpcMessage::Request(request)) => {
                        let response = match tokio::time::timeout(
                            config.request_timeout,
                            session.handle_request(request)
                        ).await {
                            Ok(response) => response,
                            Err(_) => {
                                return Err(Box::new(McpError {
                                    code: -32002,
                                    message: "Request timed out".to_string(),
                                    data: None,
                                }));
                            }
                        };

                        let message = McpMessage::JsonRpc(JsonRpcMessage::Response(response));
                        transport.send(message).await?;
                    }
                    McpMessage::JsonRpc(JsonRpcMessage::Notification(notification)) => {
                        // Handle notifications
                        match notification.method.as_str() {
                            "notifications/cancelled" => {
                                // Handle cancellation
                            }
                            "notifications/progress" => {
                                // Handle progress updates
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// TCP transport implementation
struct TcpTransport {
    stream: Arc<tokio::sync::Mutex<tokio::net::TcpStream>>,
}

impl TcpTransport {
    fn new(stream: tokio::net::TcpStream) -> Self {
        Self {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
        }
    }
}

#[async_trait]
impl McpTransport for TcpTransport {
    async fn send(&self, message: McpMessage) -> Result<(), McpError> {
        let json = serde_json::to_string(&message).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to serialize message: {}", e),
            data: None,
        })?;

        let mut stream = self.stream.lock().await;
        stream
            .write_all(json.as_bytes())
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send message: {}", e),
                data: None,
            })?;
        stream.write_all(b"\n").await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to send newline: {}", e),
            data: None,
        })?;
        stream.flush().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to flush stream: {}", e),
            data: None,
        })?;

        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        let mut stream = self.stream.lock().await;
        let mut reader = BufReader::new(&mut *stream);
        let mut line = String::new();

        reader.read_line(&mut line).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to read message: {}", e),
            data: None,
        })?;

        if line.is_empty() {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Connection closed".to_string(),
                data: None,
            });
        }

        serde_json::from_str(&line).map_err(|e| McpError {
            code: error_codes::PARSE_ERROR,
            message: format!("Failed to parse message: {}", e),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to close connection: {}", e),
            data: None,
        })
    }
}

// Unix socket transport
struct UnixTransport {
    stream: Arc<tokio::sync::Mutex<tokio::net::UnixStream>>,
}

impl UnixTransport {
    fn new(stream: tokio::net::UnixStream) -> Self {
        Self {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
        }
    }
}

#[async_trait]
impl McpTransport for UnixTransport {
    async fn send(&self, message: McpMessage) -> Result<(), McpError> {
        let json = serde_json::to_string(&message).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to serialize message: {}", e),
            data: None,
        })?;

        let mut stream = self.stream.lock().await;
        stream
            .write_all(json.as_bytes())
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send message: {}", e),
                data: None,
            })?;
        stream.write_all(b"\n").await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to send newline: {}", e),
            data: None,
        })?;
        stream.flush().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to flush stream: {}", e),
            data: None,
        })?;

        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        let mut stream = self.stream.lock().await;
        let mut reader = BufReader::new(&mut *stream);
        let mut line = String::new();

        reader.read_line(&mut line).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to read message: {}", e),
            data: None,
        })?;

        if line.is_empty() {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Connection closed".to_string(),
                data: None,
            });
        }

        serde_json::from_str(&line).map_err(|e| McpError {
            code: error_codes::PARSE_ERROR,
            message: format!("Failed to parse message: {}", e),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to close connection: {}", e),
            data: None,
        })
    }
}

// Stdio transport for CLI usage
struct StdioTransport;

impl StdioTransport {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send(&self, message: McpMessage) -> Result<(), McpError> {
        let json = serde_json::to_string(&message).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to serialize message: {}", e),
            data: None,
        })?;

        println!("{}", json);
        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        use tokio::io::{self, AsyncBufReadExt};

        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin);
        let mut line = String::new();

        reader.read_line(&mut line).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to read message: {}", e),
            data: None,
        })?;

        if line.is_empty() {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "EOF reached".to_string(),
                data: None,
            });
        }

        serde_json::from_str(&line).map_err(|e| McpError {
            code: error_codes::PARSE_ERROR,
            message: format!("Failed to parse message: {}", e),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        Ok(())
    }
}
