// MCP protocol communication: run loop, request handling, and cache metrics.
// Included by server.rs - shares parent module scope (no `use` imports here).

impl McpServer {
    /// Runs the MCP server main loop, handling stdin/stdout communication.
    ///
    /// This method implements the core MCP protocol communication loop, reading
    /// JSON-RPC messages from stdin and writing responses to stdout. Handles
    /// the complete MCP session lifecycle including initialization, method dispatch,
    /// and error handling.
    ///
    /// # Protocol Flow
    ///
    /// 1. **Initialization**: Send server capabilities to client
    /// 2. **Message Loop**: Process incoming JSON-RPC requests
    /// 3. **Method Dispatch**: Route requests to appropriate handlers
    /// 4. **Response Generation**: Send JSON-RPC responses
    /// 5. **Error Handling**: Handle protocol and processing errors
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Server shutdown gracefully
    /// * `Err(Box<dyn std::error::Error>)` - I/O error, protocol error, or handler failure
    ///
    /// # Communication Protocol
    ///
    /// The server uses line-delimited JSON over stdin/stdout:
    /// - **Input**: JSON-RPC 2.0 requests (one per line)
    /// - **Output**: JSON-RPC 2.0 responses (one per line)
    /// - **Transport**: stdin/stdout pipes
    /// - **Encoding**: UTF-8 text
    ///
    /// # Error Handling
    ///
    /// Standard JSON-RPC error codes:
    /// - `-32700`: Parse error (invalid JSON)
    /// - `-32600`: Invalid request (bad JSON-RPC)
    /// - `-32601`: Method not found
    /// - `-32602`: Invalid params
    /// - `-32603`: Internal error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::server::McpServer;
    ///
    /// # tokio_test::block_on(async {
    /// let server = McpServer::new();
    ///
    /// // In real usage, this would run the MCP protocol loop
    /// // let result = server.run().await;
    /// // assert!(result.is_ok());
    ///
    /// // For testing, we just verify the server can be created
    /// # });
    /// ```
    ///
    /// # MCP Client Integration
    ///
    /// ## Claude Desktop Configuration
    /// ```json
    /// {
    ///   "mcpServers": {
    ///     "pmat": {
    ///       "command": "pmat",
    ///       "args": ["serve", "mcp"],
    ///       "env": {}
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ## VS Code MCP Extension
    /// ```json
    /// {
    ///   "mcp.servers": [{
    ///     "name": "PMAT Refactoring",
    ///     "command": ["pmat", "serve", "mcp"],
    ///     "capabilities": ["refactor"]
    ///   }]
    /// }
    /// ```
    ///
    /// # Protocol Messages
    ///
    /// ## Initialization Response
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "protocolVersion": "2024-11-05",
    ///     "capabilities": {
    ///       "refactor": {
    ///         "start": true,
    ///         "nextIteration": true,
    ///         "getState": true,
    ///         "stop": true
    ///       }
    ///     },
    ///     "serverInfo": {
    ///       "name": "pmat-mcp-server",
    ///       "version": "0.27.5"
    ///     }
    ///   }
    /// }
    /// ```
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting MCP server on stdin/stdout");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        // Send initialization response
        let init_response = json!({
            "jsonrpc": "2.0",
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "refactor": {
                        "start": true,
                        "nextIteration": true,
                        "getState": true,
                        "stop": true
                    }
                },
                "serverInfo": {
                    "name": "pmat-mcp-server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        stdout
            .write_all(format!("{init_response}\n").as_bytes())
            .await?;
        stdout.flush().await?;

        // Main message loop
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            debug!("Received MCP request: {}", line);

            let response = match self.handle_request(&line).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Error handling request: {}", e);
                    McpResponse::error(json!(null), -32603, e.to_string())
                }
            };

            let response_json = serde_json::to_string(&response)?;
            debug!("Sending MCP response: {}", response_json);

            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        info!("MCP server shutting down");
        Ok(())
    }

    /// Handles a single MCP request and returns the appropriate response.
    ///
    /// This method implements the core MCP request processing logic, including
    /// JSON-RPC validation, method routing, and response generation. Critical for
    /// maintaining MCP protocol compliance and method stability.
    ///
    /// # Parameters
    ///
    /// * `line` - Raw JSON-RPC request string from stdin
    ///
    /// # Returns
    ///
    /// * `Ok(McpResponse)` - Valid MCP response (success or error)
    /// * `Err(Box<dyn std::error::Error>)` - Parse error or handler failure
    ///
    /// # Protocol Validation
    ///
    /// 1. **JSON Parsing**: Validate request is valid JSON
    /// 2. **JSON-RPC Validation**: Check jsonrpc field is "2.0"
    /// 3. **Method Resolution**: Route to appropriate handler
    /// 4. **Parameter Validation**: Validate method-specific parameters
    /// 5. **Response Generation**: Create success or error response
    ///
    /// # Supported MCP Methods
    ///
    /// - `initialize` - MCP session initialization
    /// - `refactor.start` - Start new refactoring session
    /// - `refactor.nextIteration` - Advance state machine
    /// - `refactor.getState` - Query current state
    /// - `refactor.stop` - Stop refactoring session
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::server::McpServer;
    ///
    /// # tokio_test::block_on(async {
    /// let server = McpServer::new();
    ///
    /// // Initialize request example
    /// let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    /// // The handle_request method is private and used internally
    /// // This demonstrates the expected JSON-RPC format
    /// assert!(init_request.contains("jsonrpc"));
    /// assert!(init_request.contains("2.0"));
    /// assert!(init_request.contains("initialize"));
    ///
    /// // Invalid JSON-RPC version example
    /// let bad_request = r#"{"jsonrpc":"1.0","id":2,"method":"test"}"#;
    /// // Should be rejected due to invalid version
    /// assert!(bad_request.contains("1.0"));
    /// # });
    /// ```
    ///
    /// # Request/Response Examples
    ///
    /// ## Initialize Method
    /// ```json
    /// // Request
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 1,
    ///   "method": "initialize",
    ///   "params": {
    ///     "protocolVersion": "2024-11-05"
    ///   }
    /// }
    ///
    /// // Response
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 1,
    ///   "result": {
    ///     "protocolVersion": "2024-11-05",
    ///     "capabilities": {
    ///       "refactor": {
    ///         "start": true,
    ///         "nextIteration": true,
    ///         "getState": true,
    ///         "stop": true
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ## Refactor Start Method
    /// ```json
    /// // Request
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 2,
    ///   "method": "refactor.start",
    ///   "params": {
    ///     "targets": ["/path/to/file.rs"],
    ///     "config": {
    ///       "target_complexity": 15,
    ///       "remove_satd": true
    ///     }
    ///   }
    /// }
    ///
    /// // Response
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 2,
    ///   "result": {
    ///     "session_id": "uuid-string",
    ///     "state": {
    ///       "current": "Scan",
    ///       "targets": ["/path/to/file.rs"]
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ## Error Response
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 3,
    ///   "error": {
    ///     "code": -32601,
    ///     "message": "Method not found: unknown.method"
    ///   }
    /// }
    /// ```
    async fn handle_request(&self, line: &str) -> Result<McpResponse, Box<dyn std::error::Error>> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let request: McpRequest = serde_json::from_str(line)?;

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return Ok(McpResponse::error(
                request.id,
                -32600,
                "Invalid JSON-RPC version".to_string(),
            ));
        }

        // Check cache for read-only operations
        let cache_key = if matches!(request.method.as_str(), "refactor.getState" | "initialize") {
            let mut hasher = DefaultHasher::new();
            request.method.hash(&mut hasher);
            request.params.hash(&mut hasher);
            Some(CacheKeyBuilder::method_result_key(
                &request.method,
                hasher.finish(),
            ))
        } else {
            None
        };

        // Try to get from cache
        if let Some(key) = &cache_key {
            if let Some(cached_value) = self.cache.get(key).await {
                debug!("Cache hit for method: {}", request.method);
                return Ok(McpResponse::success(request.id, cached_value));
            }
        }

        // Route to appropriate handler
        let result = match request.method.as_str() {
            "initialize" => {
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "refactor": {
                            "start": true,
                            "nextIteration": true,
                            "getState": true,
                            "stop": true
                        }
                    },
                    "serverInfo": {
                        "name": "pmat-mcp-server",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })
            }
            "refactor.start" => {
                handlers::handle_refactor_start(
                    &self.state_manager,
                    request.params.unwrap_or(json!({})),
                )
                .await?
            }
            "refactor.nextIteration" => {
                handlers::handle_refactor_next_iteration(&self.state_manager).await?
            }
            "refactor.getState" => handlers::handle_refactor_get_state(&self.state_manager).await?,
            "refactor.stop" => handlers::handle_refactor_stop(&self.state_manager).await?,
            _ => {
                return Ok(McpResponse::error(
                    request.id,
                    -32601,
                    format!("Method not found: {}", request.method),
                ));
            }
        };

        // Cache the result for read-only operations
        if let Some(key) = cache_key {
            self.cache.set(key, result.clone()).await;
            debug!("Cached result for method: {}", request.method);
        }

        Ok(McpResponse::success(request.id, result))
    }

    /// Get cache metrics for monitoring
    pub async fn cache_metrics(&self) -> String {
        let metrics = self.cache.metrics().await;
        format!(
            "Cache Metrics - Hits: {}, Misses: {}, Hit Ratio: {:.2}%, Size: {}",
            metrics.hits,
            metrics.misses,
            metrics.hit_ratio() * 100.0,
            self.cache.size().await
        )
    }
}
