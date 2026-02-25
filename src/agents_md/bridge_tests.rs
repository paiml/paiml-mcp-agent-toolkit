// Bridge unit tests
// Included from bridge.rs — shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
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
