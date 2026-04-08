// MCP ProtocolAdapter implementation (decode/encode) and McpReader methods
// Included from mcp.rs - NO use imports or #! inner attributes allowed

#[async_trait]
impl ProtocolAdapter for McpAdapter {
    type Input = McpInput;
    type Output = String;

    fn protocol(&self) -> Protocol {
        Protocol::Mcp
    }

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        debug!("Decoding MCP input: {:?}", input);

        let json_rpc: JsonRpcRequest = match input {
            McpInput::Line(line) => serde_json::from_str(&line)
                .map_err(|e| ProtocolError::DecodeError(format!("Invalid JSON-RPC: {e}")))?,
            McpInput::Request(req) => req,
        };

        // Validate JSON-RPC structure
        if json_rpc.jsonrpc != "2.0" {
            return Err(ProtocolError::InvalidFormat(
                "Invalid JSON-RPC version, expected '2.0'".to_string(),
            ));
        }

        // Convert to unified request
        let path = format!("/mcp/{}", json_rpc.method);
        let body = serde_json::to_vec(&json_rpc.params.unwrap_or(Value::Null))?;

        let unified_request = UnifiedRequest::new(Method::POST, path)
            .with_body(Body::from(body))
            .with_header("content-type", "application/json")
            .with_extension("protocol", Protocol::Mcp)
            .with_extension(
                "mcp_context",
                McpContext {
                    id: json_rpc.id.clone(),
                    method: json_rpc.method.clone(),
                },
            );

        debug!(
            method = %json_rpc.method,
            id = ?json_rpc.id,
            "Decoded MCP request"
        );

        Ok(unified_request)
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        debug!(status = %response.status, "Encoding MCP response");

        // Extract MCP context to get the request ID
        let body_bytes = http_body_util::BodyExt::collect(response.body)
            .await
            .map_err(|e| ProtocolError::EncodeError(format!("Failed to read response body: {e}")))?
            .to_bytes();

        let response_data: Value = serde_json::from_slice(&body_bytes)?;

        // Check if this is already a JSON-RPC response
        if response_data.get("jsonrpc").is_some() {
            return Ok(serde_json::to_string(&response_data)?);
        }

        // Build JSON-RPC response
        let json_rpc_response = if response.status.is_success() {
            JsonRpcResponse::success(response_data, None) // ID would come from context
        } else {
            // Try to extract error information
            let error_code = match response.status.as_u16() {
                400 => -32602, // Invalid params
                404 => -32601, // Method not found
                500 => -32603, // Internal error
                _ => -32000,   // Server error
            };

            let error_message = response_data
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            JsonRpcResponse::error(
                JsonRpcError {
                    code: error_code,
                    message: error_message,
                    data: response_data.get("data").cloned(),
                },
                None,
            )
        };

        let result = serde_json::to_string(&json_rpc_response)?;
        debug!(response = %result, "Encoded MCP response");

        Ok(result)
    }
}

impl McpReader {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(stdin: Stdin) -> Self {
        Self {
            reader: AsyncBufReader::new(stdin),
        }
    }

    /// Read a single JSON-RPC message from stdin
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn read_message(&mut self) -> Result<JsonRpcRequest, ProtocolError> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err(ProtocolError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF on stdin",
            )));
        }

        let line = line.trim();
        if line.is_empty() {
            return Err(ProtocolError::InvalidFormat("Empty line".to_string()));
        }

        serde_json::from_str(line)
            .map_err(|e| ProtocolError::DecodeError(format!("Invalid JSON: {e}")))
    }
}
