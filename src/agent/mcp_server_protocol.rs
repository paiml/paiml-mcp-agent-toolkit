// MCP Server Protocol Handling - included by mcp_server.rs
// Contains: constructor, stdio transport, MCP protocol, request dispatch,
// tool handlers, and formatting utilities.

/// Sentinel prefix for errors that should map to JSON-RPC -32602 (Invalid params).
///
/// R21-5 / D99: Prevents cwd-exfiltration by failing loudly on missing/empty
/// path arguments instead of silently defaulting to the server's cwd.
const INVALID_PARAMS_PREFIX: &str = "INVALID_PARAMS: ";

/// Helper to validate paths against directory traversal
fn validate_path(path_str: &str) -> Result<()> {
    let path = std::path::Path::new(path_str);
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            anyhow::bail!("{INVALID_PARAMS_PREFIX}directory traversal (..) is not allowed");
        }
    }
    Ok(())
}

/// Extract a required string path argument from a JSON value, rejecting
/// missing, null, non-string, or empty/whitespace-only values.
///
/// Returns an anyhow error tagged with `INVALID_PARAMS_PREFIX` so the
/// protocol layer can map it to JSON-RPC code `-32602`.
///
/// R21-5 / D99: Empty arguments (`{}` or `{"field": null}`) previously
/// silently scanned the server's cwd, enabling data exfiltration. This
/// helper enforces explicit, non-empty path inputs for all `analyze_*`
/// MCP handlers, and also prevents directory traversal.
fn require_path_arg(arguments: &Value, field: &str) -> Result<String> {
    match arguments.get(field) {
        None => Err(anyhow::anyhow!(
            "{INVALID_PARAMS_PREFIX}missing required parameter '{field}'; \
             refusing to default to server cwd"
        )),
        Some(Value::Null) => Err(anyhow::anyhow!(
            "{INVALID_PARAMS_PREFIX}parameter '{field}' is null; \
             refusing to default to server cwd"
        )),
        Some(Value::String(s)) => {
            if s.trim().is_empty() {
                Err(anyhow::anyhow!(
                    "{INVALID_PARAMS_PREFIX}parameter '{field}' is empty; \
                     refusing to default to server cwd"
                ))
            } else {
                validate_path(s)?;
                Ok(s.clone())
            }
        }
        Some(other) => Err(anyhow::anyhow!(
            "{INVALID_PARAMS_PREFIX}parameter '{field}' must be a string, got {}",
            other
        )),
    }
}

impl ClaudeCodeAgentMcpServer {
    /// Create new MCP server instance
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            monitored_projects: HashMap::new(),
            quality_monitor: None,
            quality_gate_service: Arc::new(QualityGateService::new()),
            analysis_service: Arc::new(AnalysisService::new()),
        }
    }

    /// Start the MCP server with stdio transport
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn start_stdio(&mut self) -> Result<()> {
        // Don't log during MCP protocol to avoid interfering with stdio
        // All communication should happen via JSON-RPC over stdio

        // If we need to start a background monitor, do it here
        let (tx, rx) = mpsc::channel(100);
        self.quality_monitor = Some(tx);

        // Start the monitoring task in background
        let monitor_self = self.clone();
        tokio::spawn(async move {
            if let Err(e) = monitor_self.run_quality_monitor(rx).await {
                debug!("Monitor task error: {}", e);
            }
        });

        self.run_mcp_protocol().await
    }

    /// Run the MCP protocol handler
    async fn run_mcp_protocol(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        // MCP server waits for client to initiate with initialize request
        // No need to send server capabilities proactively

        loop {
            line.clear();

            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF reached
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Process MCP request without logging to avoid stdio interference

                    // Parse and handle MCP request
                    match self.handle_mcp_request(trimmed).await {
                        Ok(Some(response)) => {
                            // Send response
                            let response_json = serde_json::to_string(&response)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Ok(None) => {
                            // No response needed (notification)
                        }
                        Err(e) => {
                            // Send error response (don't log to avoid stdio interference).
                            // R21-5 / D99: Errors tagged INVALID_PARAMS_PREFIX map to
                            // JSON-RPC -32602 (Invalid params); everything else is -32603.
                            let err_msg = e.to_string();
                            let (code, message) =
                                if let Some(detail) = err_msg.strip_prefix(INVALID_PARAMS_PREFIX) {
                                    (-32602, format!("Invalid params: {detail}"))
                                } else {
                                    (-32603, format!("Internal error: {err_msg}"))
                                };
                            let error_response = json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": code,
                                    "message": message
                                }
                            });
                            let error_json = serde_json::to_string(&error_response)?;
                            stdout.write_all(error_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(_e) => {
                    // Error reading from stdin, exit gracefully
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming MCP request
    async fn handle_mcp_request(&mut self, request_json: &str) -> Result<Option<Value>> {
        // Parse JSON-RPC request
        let request: Value = serde_json::from_str(request_json)?;

        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid method"))?;

        let id = request.get("id").cloned();
        let params = request.get("params").cloned().unwrap_or(Value::Null);

        // Handle different MCP methods
        let result = match method {
            "initialize" => self.handle_initialize(params).await?,
            "tools/list" => self.handle_tools_list().await?,
            "tools/call" => self.handle_tool_call(params).await?,
            "health_check" => self.handle_health_check().await?,
            _ => {
                return Ok(Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                })));
            }
        };

        // Build response if this was a request (not notification)
        if let Some(id) = id {
            Ok(Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            })))
        } else {
            // Notification - no response needed
            Ok(None)
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, _params: Value) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": self.get_tool_capabilities(),
                "resources": self.get_resource_capabilities(),
                "prompts": self.get_prompt_capabilities()
            },
            "serverInfo": {
                "name": self.config.name,
                "version": self.config.version
            }
        }))
    }

    /// Handle tools list request
    async fn handle_tools_list(&self) -> Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "start_quality_monitoring",
                    "description": "Start monitoring code quality for a project",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project directory"
                            },
                            "project_id": {
                                "type": "string",
                                "description": "Unique identifier for the project"
                            }
                        },
                        "required": ["project_path", "project_id"]
                    }
                },
                {
                    "name": "run_quality_gates",
                    "description": "Execute quality gates on a project",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project directory"
                            }
                        },
                        "required": ["project_path"]
                    }
                },
                {
                    "name": "analyze_complexity",
                    "description": "Analyze code complexity for a project or file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Path to the file or project to analyze"
                            }
                        },
                        "required": ["file_path"]
                    }
                },
                {
                    "name": "health_check",
                    "description": "Check the health status of the agent",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false
                    }
                }
            ]
        }))
    }

    /// Handle tool call request
    async fn handle_tool_call(&mut self, params: Value) -> Result<Value> {
        let tool_name = params
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

        let arguments = params.get("arguments").unwrap_or(&Value::Null);

        match tool_name {
            "start_quality_monitoring" => self.handle_start_monitoring(arguments).await,
            "stop_quality_monitoring" => self.handle_stop_monitoring(arguments).await,
            "get_monitoring_status" => self.handle_get_status(arguments).await,
            "run_quality_gates" => self.handle_run_quality_gates(arguments).await,
            "analyze_complexity" => self.handle_analyze_complexity(arguments).await,
            "health_check" => self.handle_health_check().await,
            _ => Err(anyhow::anyhow!("Unknown tool: {tool_name}")),
        }
    }

    async fn handle_run_quality_gates(&self, arguments: &Value) -> Result<Value> {
        // R21-5 / D99: Require explicit target_path. Silent cwd fallback
        // previously leaked info about the server's launch directory.
        let target_path = require_path_arg(arguments, "target_path")?;
        let target_path = target_path.as_str();

        let path = PathBuf::from(target_path);
        let input = QualityGateInput {
            path: path.clone(),
            checks: vec![
                QualityCheck::Complexity { max: 20 },
                QualityCheck::Satd { tolerance: 0 },
                QualityCheck::DeadCode {
                    max_percentage: 10.0,
                },
                QualityCheck::Lint,
            ],
            strict: true,
        };

        let quality_result = self.quality_gate_service.process(input).await?;
        let result_text = self.format_quality_gate_results(target_path, &quality_result);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": result_text
            }]
        }))
    }

    fn format_quality_gate_results(
        &self,
        target_path: &str,
        quality_result: &QualityGateOutput,
    ) -> String {
        let mut result_text = format!("🏁 Quality Gate Results for {target_path}\n\n");

        let all_passed = quality_result.results.iter().all(|r| r.passed);
        result_text.push_str(&format!(
            "Status: {}\n",
            if all_passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        ));

        self.format_failed_checks(&mut result_text, quality_result);
        self.format_quality_summary(&mut result_text, quality_result);

        result_text
    }

    fn format_failed_checks(&self, result_text: &mut String, quality_result: &QualityGateOutput) {
        let failed_checks = quality_result.results.iter().filter(|r| !r.passed).count();

        if failed_checks > 0 {
            result_text.push_str(&format!(
                "\n⚠️  Failed Checks: {}/{}\n",
                failed_checks,
                quality_result.results.len()
            ));

            for result in &quality_result.results {
                if !result.passed {
                    result_text.push_str(&format!("  ❌ {}: {}\n", result.check, result.message));
                }
            }
        }
    }

    fn format_quality_summary(&self, result_text: &mut String, quality_result: &QualityGateOutput) {
        result_text.push_str("\n📋 Summary:\n");
        result_text.push_str(&format!(
            "• Total Checks: {}\n",
            quality_result.summary.total_checks
        ));
        result_text.push_str(&format!(
            "• Passed: {}\n",
            quality_result.summary.passed_checks
        ));
        result_text.push_str(&format!(
            "• Failed: {}\n",
            quality_result.summary.failed_checks
        ));
    }

    async fn handle_analyze_complexity(&self, arguments: &Value) -> Result<Value> {
        // R21-5 / D99: Require explicit file_path. Silent cwd fallback
        // previously allowed a malicious or buggy client to trigger
        // complexity scans of the server's launch directory.
        let file_path = require_path_arg(arguments, "file_path")?;

        let result_text = self.format_complexity_analysis_results(&file_path);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": result_text
            }]
        }))
    }

    /// Report the complexity of `file_path` — which this server does not measure.
    ///
    /// #1090 / T7: every line of this function used to be a `push_str` of a
    /// constant. "Files analyzed: 1 / Average complexity: 8.5 / Max complexity:
    /// 15 / ✅ All functions are within Toyota Way standards" came back for a
    /// path that does not exist, for a directory of ten thousand files, and for
    /// an empty string — the function never opened anything, and `file_path` was
    /// interpolated into the header only. A verdict computed from no input is
    /// not a weak measurement, it is a fabrication, and an MCP client cannot
    /// tell the two apart from the payload.
    ///
    /// `AnalysisService` cannot fill the hole: its `analyze_complexity` was the
    /// same three constants and now refuses rather than answering. So the text
    /// states what was measured — nothing — in the `not_measured` vocabulary
    /// `pmat quality-gate` and the MCP `quality_gate` tool use for an unanswered
    /// check, and names the command that does measure it. The old claim is
    /// retracted explicitly, because clients and transcripts have already seen
    /// it asserted.
    fn format_complexity_analysis_results(&self, file_path: &str) -> String {
        format!(
            "🧮 Complexity Analysis for {file_path}\n\n\
             📊 Summary:\n\
             • Files analyzed: 0\n\
             • Average complexity: not_measured\n\
             • Max complexity: not_measured\n\
             \n\
             ⚠️  not_measured: this agent server has no complexity analyzer wired in, so \
             nothing above is a statement that {file_path} is within Toyota Way standards. \
             Run `pmat analyze complexity --path {file_path}` for a real measurement."
        )
    }

    /// Handle health check request
    async fn handle_health_check(&self) -> Result<Value> {
        Ok(json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": self.config.version,
            "uptime_seconds": 0 // Uptime tracking managed by daemon lifecycle
        }))
    }

    /// Send server information and capabilities
    async fn send_server_info(&self, stdout: &mut Stdout) -> Result<()> {
        let server_info = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": self.get_tool_capabilities(),
                    "resources": self.get_resource_capabilities(),
                    "prompts": self.get_prompt_capabilities()
                },
                "serverInfo": {
                    "name": self.config.name,
                    "version": self.config.version
                }
            }
        });

        let info_json = serde_json::to_string(&server_info)?;
        stdout.write_all(info_json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;

        Ok(())
    }
}

/// #1090 / T7: the `analyze_complexity` tool must not answer with numbers
/// nobody measured.
///
/// These live here rather than in `mcp_server_tests.rs` because they belong
/// beside the function whose defect they pin.
#[cfg(test)]
mod complexity_disclosure_tests {
    use super::*;

    /// Fails against the pre-change function, whose entire body was
    /// `push_str("• Average complexity: 8.5")` and
    /// `push_str("\n✅ All functions are within Toyota Way standards (≤20 complexity)")`
    /// — asserted here for a path that does not exist.
    #[test]
    fn the_complexity_report_states_that_nothing_was_measured() {
        let server = ClaudeCodeAgentMcpServer::new(AgentConfig::default());

        let text = server.format_complexity_analysis_results("/does/not/exist.rs");

        assert!(
            text.contains("not_measured"),
            "a check that ran no analyzer has to name itself: {text}"
        );
        assert!(
            !text.contains("Average complexity: 8.5"),
            "8.5 was a constant, not this path's average complexity: {text}"
        );
        assert!(
            !text.contains("Max complexity: 15"),
            "15 was the max complexity of every path in the world: {text}"
        );
        assert!(
            !text.contains("All functions are within Toyota Way standards"),
            "a verdict derived from no input must not be asserted: {text}"
        );
    }

    /// The same, through the tool call an MCP client actually makes, so the
    /// disclosure cannot be lost between the formatter and the payload.
    #[tokio::test]
    async fn the_analyze_complexity_tool_discloses_that_it_measured_nothing() {
        let server = ClaudeCodeAgentMcpServer::new(AgentConfig::default());
        let arguments = json!({ "file_path": "/does/not/exist.rs" });

        let result = server
            .handle_analyze_complexity(&arguments)
            .await
            .expect("the tool call itself still succeeds");

        let text = result["content"][0]["text"]
            .as_str()
            .expect("the tool returns text content");
        assert!(
            text.contains("not_measured"),
            "the payload, not just the formatter, has to carry the disclosure: {text}"
        );
        assert!(
            !text.contains("All functions are within Toyota Way standards"),
            "the fabricated verdict must not reach a client: {text}"
        );
    }
}
