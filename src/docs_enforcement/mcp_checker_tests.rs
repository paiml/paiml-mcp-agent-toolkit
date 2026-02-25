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

    #[test]
    fn test_load_mcp_tool_definitions_returns_four_tools() {
        let tools = load_mcp_tool_definitions().unwrap();
        assert_eq!(tools.len(), 4);
    }

    #[test]
    fn test_load_mcp_tool_definitions_tool_names() {
        let tools = load_mcp_tool_definitions().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"scaffold_agent"));
        assert!(names.contains(&"validate_roadmap"));
        assert!(names.contains(&"health_check"));
        assert!(names.contains(&"generate_tickets"));
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
    fn test_scaffold_agent_requires_name() {
        let tools = load_mcp_tool_definitions().unwrap();
        let scaffold = tools.iter().find(|t| t.name == "scaffold_agent").unwrap();
        let required = scaffold.input_schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("name")));
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
