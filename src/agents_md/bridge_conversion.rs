// Bridge conversion, translation, and quality methods
// Included from bridge.rs — shares parent module scope

impl McpAgentsMdBridge {
    /// Create new bridge
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            config: BridgeConfig::default(),
            tool_registry: Vec::new(),
        }
    }

    /// Create with config
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_config(config: BridgeConfig) -> Self {
        Self {
            config,
            tool_registry: Vec::new(),
        }
    }

    /// Convert AGENTS.md document to MCP tools
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn agents_to_mcp(&self, doc: &AgentsMdDocument) -> Vec<McpTool> {
        let mut tools = Vec::new();

        // Convert commands to tools
        for cmd in &doc.commands {
            tools.push(self.command_to_tool(cmd));
        }

        // Add quality gates if configured
        if self.config.quality_level != QualityLevel::None {
            tools.push(self.create_quality_tool());
        }

        tools
    }

    /// Convert MCP capabilities to AGENTS.md
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn mcp_to_agents(&self, tools: &[McpTool]) -> String {
        debug_assert!(!tools.is_empty(), "tools must not be empty");
        let mut output = String::new();
        output.push_str("# AGENTS.md\n\n");
        output.push_str("## Available Tools\n\n");

        for tool in tools {
            output.push_str(&format!("### {}\n", tool.name));
            output.push_str(&format!("{}\n\n", tool.description));

            if let ToolHandler::Command(ref cmd) = tool.handler {
                output.push_str("```bash\n");
                output.push_str(&format!("{}\n", cmd.command));
                output.push_str("```\n\n");
            }
        }

        output
    }

    /// Translate request between protocols
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn translate_request(&self, req: Request) -> TranslatedRequest {
        let metadata = TranslationMetadata {
            timestamp: std::time::SystemTime::now(),
            quality_checks: vec![],
            warnings: vec![],
        };

        let translated = match req {
            Request::AgentsMd(ref agents_req) => {
                Request::Mcp(self.agents_request_to_mcp(agents_req))
            }
            Request::Mcp(ref mcp_req) => Request::AgentsMd(self.mcp_request_to_agents(mcp_req)),
        };

        TranslatedRequest {
            original: req,
            translated,
            metadata,
        }
    }

    /// Unify response handling
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn unify_response(&self, resp: Response) -> UnifiedResponse {
        let unified = match resp {
            Response::AgentsMd(ref agents_resp) => self.agents_response_to_unified(agents_resp),
            Response::Mcp(ref mcp_resp) => self.mcp_response_to_unified(mcp_resp),
        };

        let quality_report = if self.config.quality_level == QualityLevel::None {
            None
        } else {
            Some(self.check_response_quality(&unified))
        };

        UnifiedResponse {
            original: resp,
            unified,
            quality_report,
        }
    }

    /// Convert command to MCP tool
    fn command_to_tool(&self, cmd: &Command) -> McpTool {
        debug_assert!(true, "contract: command_to_tool");
        McpTool {
            name: cmd.name.clone(),
            description: format!("Execute: {}", cmd.command),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "args": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                }
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "stdout": {"type": "string"},
                    "stderr": {"type": "string"},
                    "exit_code": {"type": "integer"}
                }
            }),
            handler: ToolHandler::Command(cmd.clone()),
        }
    }

    /// Create quality gate tool
    fn create_quality_tool(&self) -> McpTool {
        debug_assert!(true, "contract: create_quality_tool");
        McpTool {
            name: "quality_gate".to_string(),
            description: "Run PMAT quality gates".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string"},
                    "level": {"type": "string"}
                }
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "passed": {"type": "boolean"},
                    "score": {"type": "number"},
                    "issues": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                }
            }),
            handler: ToolHandler::Function("quality_gate".to_string()),
        }
    }

    /// Convert AGENTS.md request to MCP
    fn agents_request_to_mcp(&self, req: &AgentsMdRequest) -> McpRequest {
        debug_assert!(true, "contract: agents_request_to_mcp");
        McpRequest {
            method: req.request_type.clone(),
            params: req.params.clone(),
        }
    }

    /// Convert MCP request to AGENTS.md
    fn mcp_request_to_agents(&self, req: &McpRequest) -> AgentsMdRequest {
        debug_assert!(true, "contract: mcp_request_to_agents");
        AgentsMdRequest {
            request_type: req.method.clone(),
            params: req.params.clone(),
        }
    }

    /// Convert AGENTS.md response to unified format
    fn agents_response_to_unified(&self, resp: &AgentsMdResponse) -> JsonValue {
        debug_assert!(true, "contract: agents_response_to_unified");
        json!({
            "success": resp.success,
            "result": resp.result,
            "error": resp.error,
        })
    }

    /// Convert MCP response to unified format
    fn mcp_response_to_unified(&self, resp: &McpResponse) -> JsonValue {
        debug_assert!(true, "contract: mcp_response_to_unified");
        json!({
            "success": resp.error.is_none(),
            "result": resp.result,
            "error": resp.error,
        })
    }

    /// Check response quality
    fn check_response_quality(&self, response: &JsonValue) -> QualityReport {
        debug_assert!(true, "contract: check_response_quality");
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut score: f64 = 100.0;

        // Check for errors
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                issues.push("Response contains error".to_string());
                score -= 20.0;
            }
        }

        // Check for empty results
        if let Some(result) = response.get("result") {
            if result.is_null() || (result.is_string() && result.as_str() == Some("")) {
                issues.push("Empty result".to_string());
                suggestions.push("Provide meaningful output".to_string());
                score -= 10.0;
            }
        }

        QualityReport {
            score: score.max(0.0),
            issues,
            suggestions,
        }
    }

    /// Register MCP tool
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn register_tool(&mut self, tool: McpTool) {
        self.tool_registry.push(tool);
    }

    /// Get registered tools
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_tools(&self) -> &[McpTool] {
        &self.tool_registry
    }
}
