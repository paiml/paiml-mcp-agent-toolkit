//! MCP-AGENTS.md Bridge
//!
//! Bidirectional bridge between AGENTS.md and MCP protocols.

use super::{Command, AgentsMdDocument};
use serde_json::{json, Value as JsonValue};

/// MCP-AGENTS.md protocol bridge
pub struct McpAgentsMdBridge {
    /// Protocol configuration
    config: BridgeConfig,

    /// Tool registry
    tool_registry: Vec<McpTool>,
}

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Enable bidirectional sync
    pub bidirectional: bool,

    /// Auto-discover AGENTS.md files
    pub auto_discover: bool,

    /// Quality enforcement level
    pub quality_level: QualityLevel,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            bidirectional: true,
            auto_discover: true,
            quality_level: QualityLevel::Standard,
        }
    }
}

/// Quality enforcement levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualityLevel {
    None,
    Basic,
    Standard,
    Strict,
    Extreme,
}

/// MCP tool representation
#[derive(Debug, Clone)]
pub struct McpTool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema
    pub input_schema: JsonValue,

    /// Output schema
    pub output_schema: JsonValue,

    /// Handler function
    pub handler: ToolHandler,
}

/// Tool handler type
#[derive(Debug, Clone)]
pub enum ToolHandler {
    Command(Command),
    Function(String),
    External(String),
}

/// Request types
#[derive(Debug, Clone)]
pub enum Request {
    AgentsMd(AgentsMdRequest),
    Mcp(McpRequest),
}

/// AGENTS.md request
#[derive(Debug, Clone)]
pub struct AgentsMdRequest {
    /// Request type
    pub request_type: String,

    /// Parameters
    pub params: JsonValue,
}

/// MCP request
#[derive(Debug, Clone)]
pub struct McpRequest {
    /// Method name
    pub method: String,

    /// Parameters
    pub params: JsonValue,
}

/// Translated request
#[derive(Debug, Clone)]
pub struct TranslatedRequest {
    /// Original request
    pub original: Request,

    /// Translated format
    pub translated: Request,

    /// Translation metadata
    pub metadata: TranslationMetadata,
}

/// Translation metadata
#[derive(Debug, Clone)]
pub struct TranslationMetadata {
    /// Translation timestamp
    pub timestamp: std::time::SystemTime,

    /// Quality checks applied
    pub quality_checks: Vec<String>,

    /// Warnings
    pub warnings: Vec<String>,
}

/// Response types
#[derive(Debug, Clone)]
pub enum Response {
    AgentsMd(AgentsMdResponse),
    Mcp(McpResponse),
}

/// AGENTS.md response
#[derive(Debug, Clone)]
pub struct AgentsMdResponse {
    /// Success status
    pub success: bool,

    /// Result data
    pub result: JsonValue,

    /// Error if any
    pub error: Option<String>,
}

/// MCP response
#[derive(Debug, Clone)]
pub struct McpResponse {
    /// Result data
    pub result: JsonValue,

    /// Error if any
    pub error: Option<JsonValue>,
}

/// Unified response
#[derive(Debug, Clone)]
pub struct UnifiedResponse {
    /// Original response
    pub original: Response,

    /// Unified format
    pub unified: JsonValue,

    /// Quality report
    pub quality_report: Option<QualityReport>,
}

/// Quality report for responses
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Quality score
    pub score: f64,

    /// Issues found
    pub issues: Vec<String>,

    /// Suggestions
    pub suggestions: Vec<String>,
}

impl Default for McpAgentsMdBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl McpAgentsMdBridge {
    /// Create new bridge
    #[must_use] 
    pub fn new() -> Self {
        Self {
            config: BridgeConfig::default(),
            tool_registry: Vec::new(),
        }
    }

    /// Create with config
    #[must_use] 
    pub fn with_config(config: BridgeConfig) -> Self {
        Self {
            config,
            tool_registry: Vec::new(),
        }
    }

    /// Convert AGENTS.md document to MCP tools
    #[must_use] 
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
    pub fn mcp_to_agents(&self, tools: &[McpTool]) -> String {
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
        McpRequest {
            method: req.request_type.clone(),
            params: req.params.clone(),
        }
    }

    /// Convert MCP request to AGENTS.md
    fn mcp_request_to_agents(&self, req: &McpRequest) -> AgentsMdRequest {
        AgentsMdRequest {
            request_type: req.method.clone(),
            params: req.params.clone(),
        }
    }

    /// Convert AGENTS.md response to unified format
    fn agents_response_to_unified(&self, resp: &AgentsMdResponse) -> JsonValue {
        json!({
            "success": resp.success,
            "result": resp.result,
            "error": resp.error,
        })
    }

    /// Convert MCP response to unified format
    fn mcp_response_to_unified(&self, resp: &McpResponse) -> JsonValue {
        json!({
            "success": resp.error.is_none(),
            "result": resp.result,
            "error": resp.error,
        })
    }

    /// Check response quality
    fn check_response_quality(&self, response: &JsonValue) -> QualityReport {
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
    pub fn register_tool(&mut self, tool: McpTool) {
        self.tool_registry.push(tool);
    }

    /// Get registered tools
    #[must_use] 
    pub fn get_tools(&self) -> &[McpTool] {
        &self.tool_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents_md::DocumentMetadata;
    use std::path::PathBuf;

    #[test]
    fn test_bridge_creation() {
        let bridge = McpAgentsMdBridge::new();
        assert!(bridge.config.bidirectional);
        assert!(bridge.config.auto_discover);
        assert_eq!(bridge.config.quality_level, QualityLevel::Standard);
    }

    #[test]
    fn test_agents_to_mcp_conversion() {
        let bridge = McpAgentsMdBridge::new();

        let doc = AgentsMdDocument {
            metadata: DocumentMetadata {
                path: PathBuf::from("AGENTS.md"),
                modified: std::time::SystemTime::now(),
                version: None,
                project: None,
            },
            sections: vec![],
            commands: vec![Command {
                name: "Build".to_string(),
                command: "cargo build".to_string(),
                working_dir: None,
                env: vec![],
                timeout: Some(60),
                safe: true,
            }],
            guidelines: vec![],
            quality_rules: None,
        };

        let tools = bridge.agents_to_mcp(&doc);
        assert_eq!(tools.len(), 2); // Command + quality tool
        assert_eq!(tools[0].name, "Build");
        assert_eq!(tools[1].name, "quality_gate");
    }

    #[test]
    fn test_mcp_to_agents_conversion() {
        let bridge = McpAgentsMdBridge::new();

        let tools = vec![McpTool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            input_schema: json!({}),
            output_schema: json!({}),
            handler: ToolHandler::Function("test".to_string()),
        }];

        let agents_md = bridge.mcp_to_agents(&tools);
        assert!(agents_md.contains("# AGENTS.md"));
        assert!(agents_md.contains("test_tool"));
        assert!(agents_md.contains("Test tool"));
    }

    #[test]
    fn test_request_translation() {
        let bridge = McpAgentsMdBridge::new();

        let agents_req = AgentsMdRequest {
            request_type: "execute".to_string(),
            params: json!({"command": "test"}),
        };

        let translated = bridge.translate_request(Request::AgentsMd(agents_req));

        if let Request::Mcp(mcp_req) = translated.translated {
            assert_eq!(mcp_req.method, "execute");
            assert_eq!(mcp_req.params, json!({"command": "test"}));
        } else {
            panic!("Expected MCP request");
        }
    }

    #[test]
    fn test_response_unification() {
        let bridge = McpAgentsMdBridge::new();

        let agents_resp = AgentsMdResponse {
            success: true,
            result: json!({"output": "test"}),
            error: None,
        };

        let unified = bridge.unify_response(Response::AgentsMd(agents_resp));
        assert!(unified.quality_report.is_some());

        let report = unified.quality_report.unwrap();
        assert_eq!(report.score, 100.0);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn test_quality_checking() {
        let bridge = McpAgentsMdBridge::new();

        let response = json!({
            "success": false,
            "result": "",
            "error": "Test error"
        });

        let report = bridge.check_response_quality(&response);
        assert!(report.score < 100.0);
        assert!(!report.issues.is_empty());
        assert!(report
            .issues
            .contains(&"Response contains error".to_string()));
    }

    #[test]
    fn test_tool_registration() {
        let mut bridge = McpAgentsMdBridge::new();

        let tool = McpTool {
            name: "custom_tool".to_string(),
            description: "Custom tool".to_string(),
            input_schema: json!({}),
            output_schema: json!({}),
            handler: ToolHandler::External("external".to_string()),
        };

        bridge.register_tool(tool);
        assert_eq!(bridge.get_tools().len(), 1);
        assert_eq!(bridge.get_tools()[0].name, "custom_tool");
    }
}
