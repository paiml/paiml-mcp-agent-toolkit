// MCP Server Resource & Prompt Capabilities - included by mcp_server.rs

impl ClaudeCodeAgentMcpServer {
    /// Get resource capabilities for MCP
    fn get_resource_capabilities(&self) -> Value {
        json!({
            "quality-metrics": { "description": "Real-time quality metrics and trends", "mimeType": "application/json" },
            "complexity-heatmap": { "description": "Visual complexity distribution across codebase", "mimeType": "application/json" },
            "refactor-suggestions": { "description": "AI-generated refactoring opportunities", "mimeType": "application/json" },
            "quality-reports": { "description": "Historical quality gate results and trends", "mimeType": "application/json" }
        })
    }

    /// Get prompt template capabilities for MCP
    fn get_prompt_capabilities(&self) -> Value {
        json!({
            "quality-summary": {
                "description": "Generate quality summary for a project",
                "arguments": { "project_id": { "type": "string", "description": "Project identifier" } }
            },
            "refactoring-guide": {
                "description": "Generate Toyota Way refactoring guidance",
                "arguments": {
                    "file_path": { "type": "string", "description": "File to refactor" },
                    "complexity_target": { "type": "number", "description": "Target complexity" }
                }
            }
        })
    }
}
