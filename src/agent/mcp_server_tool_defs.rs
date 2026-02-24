// MCP Server Tool Definitions - included by mcp_server.rs
// Each tool schema is a separate function to keep cognitive complexity low.

fn tool_start_quality_monitoring() -> Value {
    json!({
        "description": "Start continuous code quality monitoring for a project",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project_path": { "type": "string", "description": "Path to project root" },
                "watch_patterns": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "File patterns to monitor (optional)"
                },
                "complexity_threshold": {
                    "type": "number",
                    "description": "Complexity threshold for alerts (optional)"
                }
            },
            "required": ["project_path"]
        }
    })
}

fn tool_stop_quality_monitoring() -> Value {
    json!({
        "description": "Stop quality monitoring for a project",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project_id": { "type": "string", "description": "Project identifier" }
            },
            "required": ["project_id"]
        }
    })
}

fn tool_get_quality_status() -> Value {
    json!({
        "description": "Get current quality status for a monitored project",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project_id": { "type": "string", "description": "Project identifier" }
            },
            "required": ["project_id"]
        }
    })
}

fn tool_run_quality_gates() -> Value {
    json!({
        "description": "Execute Toyota Way quality gates with detailed reporting",
        "inputSchema": {
            "type": "object",
            "properties": {
                "target_path": { "type": "string", "description": "File or directory to analyze" },
                "output_format": {
                    "type": "string",
                    "enum": ["json", "markdown", "claude-friendly"],
                    "description": "Output format for results"
                }
            },
            "required": ["target_path"]
        }
    })
}

fn tool_analyze_complexity() -> Value {
    json!({
        "description": "Perform complexity analysis on files or directories",
        "inputSchema": {
            "type": "object",
            "properties": {
                "target_path": { "type": "string", "description": "Path to analyze" },
                "top_files": { "type": "number", "description": "Number of top complex files to return" }
            },
            "required": ["target_path"]
        }
    })
}

fn tool_health_check() -> Value {
    json!({
        "description": "Comprehensive codebase health assessment",
        "inputSchema": {
            "type": "object",
            "properties": {
                "target_path": { "type": "string", "description": "Path to analyze" },
                "include_satd": { "type": "boolean", "description": "Include SATD analysis" },
                "include_dead_code": { "type": "boolean", "description": "Include dead code analysis" },
                "generate_recommendations": { "type": "boolean", "description": "Generate improvement recommendations" }
            },
            "required": ["target_path"]
        }
    })
}

impl ClaudeCodeAgentMcpServer {
    /// Get tool capabilities for MCP
    fn get_tool_capabilities(&self) -> Value {
        json!({
            "start_quality_monitoring": tool_start_quality_monitoring(),
            "stop_quality_monitoring": tool_stop_quality_monitoring(),
            "get_quality_status": tool_get_quality_status(),
            "run_quality_gates": tool_run_quality_gates(),
            "analyze_complexity": tool_analyze_complexity(),
            "health_check": tool_health_check()
        })
    }
}
