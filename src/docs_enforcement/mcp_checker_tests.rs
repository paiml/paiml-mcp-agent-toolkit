// MCP checker tests - included by mcp_checker.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_good_tool() {
        let tool = McpToolDefinition {
            name: "scaffold_agent".to_string(),
            description: "Scaffold a deterministic MCP agent with complete structure".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Agent project name (lowercase, alphanumeric, hyphens only)"
                    }
                },
                "required": ["name"]
            }),
        };

        let report = validate_mcp_documentation(&tool).unwrap();
        assert!(report.has_description);
        assert!(!report.description_is_generic);
        assert_eq!(report.parameters.len(), 1);
    }

    #[test]
    fn test_validate_tool_with_generic_description() {
        let tool = McpToolDefinition {
            name: "test_tool".to_string(),
            description: "Tool name".to_string(),
            input_schema: json!({}),
        };

        let report = validate_mcp_documentation(&tool).unwrap();
        assert!(report.description_is_generic);
        assert!(!report.is_valid());
    }

    #[test]
    fn test_validate_parameter_with_generic_description() {
        let schema = json!({
            "type": "string",
            "description": "Name value"
        });

        let report = validate_parameter("name", &schema, true);
        assert!(report.description_is_generic);
        assert!(!report.is_valid());
    }

    /// GH-1230: the checker's inventory is derived from the live server's, so
    /// adding a tool to `LIVE_MCP_TOOLS` changes this count with it — and a
    /// name that is not live (the old `scaffold_agent`) cannot appear.
    #[test]
    fn test_load_mcp_tool_definitions_match_live_inventory() {
        let tools = load_mcp_tool_definitions().unwrap();
        let loaded: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        let live: Vec<&str> = crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS
            .iter()
            .map(|&(n, _)| n)
            .collect();
        assert_eq!(loaded, live);
        assert!(!loaded.contains(&"scaffold_agent"));
        assert!(loaded.contains(&"scaffold_project"));
    }

    #[test]
    fn test_load_mcp_tool_definitions_descriptions_are_meaningful() {
        let tools = load_mcp_tool_definitions().unwrap();
        for tool in &tools {
            assert!(
                tool.description.len() > 20,
                "Tool '{}' description too short: '{}'",
                tool.name,
                tool.description
            );
        }
    }

    #[test]
    fn test_load_mcp_tool_definitions_schemas_are_valid() {
        let tools = load_mcp_tool_definitions().unwrap();
        for tool in &tools {
            let schema = &tool.input_schema;
            assert_eq!(
                schema["type"].as_str().unwrap(),
                "object",
                "Tool '{}' schema type should be 'object'",
                tool.name
            );
            assert!(
                schema.get("properties").is_some(),
                "Tool '{}' missing 'properties'",
                tool.name
            );
        }
    }

    #[test]
    fn test_loaded_tools_pass_validation() {
        let tools = load_mcp_tool_definitions().unwrap();
        for tool in &tools {
            let report = validate_mcp_documentation(tool).unwrap();
            assert!(
                report.has_description,
                "Tool '{}' failed: missing description",
                tool.name
            );
            assert!(
                !report.description_is_generic,
                "Tool '{}' failed: generic description",
                tool.name
            );
        }
    }
}
