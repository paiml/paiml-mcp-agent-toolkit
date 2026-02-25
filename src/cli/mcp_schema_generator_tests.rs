// MCP Schema Generator - Tests
// Included from mcp_schema_generator.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::registry::{ArgumentMetadata, ExecutionTime, McpAnnotations, McpToolMetadata};

    fn sample_registry_with_mcp() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code metrics")
                .long_description("Run various code analysis tools on your project")
                .argument(ArgumentMetadata {
                    name: "project-path".to_string(),
                    short: Some('p'),
                    long: Some("project-path".to_string()),
                    description: "Path to project".to_string(),
                    required: false,
                    default: Some(".".to_string()),
                    value_type: ValueType::Path,
                    ..Default::default()
                })
                .argument(ArgumentMetadata {
                    name: "format".to_string(),
                    short: Some('f'),
                    long: Some("format".to_string()),
                    description: "Output format".to_string(),
                    required: false,
                    value_type: ValueType::Enum,
                    possible_values: vec![
                        "json".to_string(),
                        "text".to_string(),
                        "table".to_string(),
                    ],
                    ..Default::default()
                })
                .mcp(McpToolMetadata {
                    tool_name: "pmat_analyze".to_string(),
                    input_schema: serde_json::Value::Null, // Auto-generate
                    is_mutation: false,
                    execution_time: ExecutionTime::Medium,
                    annotations: McpAnnotations::default(),
                })
                .category("analysis")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("scaffold")
                .short_description("Scaffold new project")
                .mcp(McpToolMetadata {
                    tool_name: "pmat_scaffold".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "template": {"type": "string"},
                            "output": {"type": "string"}
                        },
                        "required": ["template"]
                    }),
                    is_mutation: true,
                    execution_time: ExecutionTime::Fast,
                    annotations: McpAnnotations::default(),
                })
                .category("generation")
                .build(),
        );

        // Command without MCP (should be filtered out)
        registry.register(
            CommandMetadata::builder("internal")
                .short_description("Internal command")
                .category("internal")
                .build(),
        );

        registry
    }

    #[test]
    fn test_mcp_generator_creation() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let tools = gen.generate_tools_list();

        // Should only include commands with MCP metadata
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_generate_tool_definition() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let tools = gen.generate_tools_list();

        let analyze_tool = tools.iter().find(|t| t.name == "pmat_analyze").unwrap();
        assert_eq!(
            analyze_tool.description,
            "Run various code analysis tools on your project"
        );
        assert!(analyze_tool.input_schema.is_object());
    }

    #[test]
    fn test_auto_generate_schema_from_args() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let tools = gen.generate_tools_list();

        let analyze_tool = tools.iter().find(|t| t.name == "pmat_analyze").unwrap();
        let schema = &analyze_tool.input_schema;

        // Check properties were generated
        let props = schema.get("properties").unwrap().as_object().unwrap();
        assert!(props.contains_key("project_path"));
        assert!(props.contains_key("format"));

        // Check format has enum values
        let format_schema = props.get("format").unwrap();
        assert!(format_schema.get("enum").is_some());
    }

    #[test]
    fn test_explicit_schema_preserved() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let tools = gen.generate_tools_list();

        let scaffold_tool = tools.iter().find(|t| t.name == "pmat_scaffold").unwrap();
        let schema = &scaffold_tool.input_schema;

        // Check explicit schema was preserved
        let props = schema.get("properties").unwrap().as_object().unwrap();
        assert!(props.contains_key("template"));
        assert!(props.contains_key("output"));
    }

    #[test]
    fn test_annotations() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let tools = gen.generate_tools_list();

        let analyze_tool = tools.iter().find(|t| t.name == "pmat_analyze").unwrap();
        let annotations = analyze_tool.annotations.as_ref().unwrap();

        assert_eq!(annotations.read_only_hint, Some(true)); // Not a mutation
        assert_eq!(annotations.idempotent_hint, Some(true));
    }

    #[test]
    fn test_validate_consistency_success() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);

        // Note: scaffold has explicit schema but no arguments, so validation passes
        // analyze has auto-generated schema, so validation passes
        let result = gen.validate_consistency();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consistency_duplicate_tool() {
        let mut registry = CommandRegistry::new("1.0.0");

        registry.register(
            CommandMetadata::builder("cmd1")
                .mcp(McpToolMetadata {
                    tool_name: "duplicate_tool".to_string(),
                    ..Default::default()
                })
                .build(),
        );
        registry.register(
            CommandMetadata::builder("cmd2")
                .mcp(McpToolMetadata {
                    tool_name: "duplicate_tool".to_string(),
                    ..Default::default()
                })
                .build(),
        );

        let gen = McpSchemaGenerator::new(registry);
        let result = gen.validate_consistency();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, SchemaError::DuplicateToolName { .. })));
    }

    #[test]
    fn test_arg_to_property_name() {
        let registry = CommandRegistry::new("1.0.0");
        let gen = McpSchemaGenerator::new(registry);

        assert_eq!(gen.arg_to_property_name("project-path"), "project_path");
        assert_eq!(gen.arg_to_property_name("output"), "output");
        assert_eq!(gen.arg_to_property_name("include-tests"), "include_tests");
    }

    #[test]
    fn test_generate_openapi_schema() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let openapi = gen.generate_openapi_schema();

        assert_eq!(openapi.get("openapi").unwrap(), "3.0.0");
        assert!(openapi
            .get("paths")
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("/tools/pmat_analyze"));
    }

    #[test]
    fn test_generate_docs() {
        let registry = sample_registry_with_mcp();
        let gen = McpSchemaGenerator::new(registry);
        let docs = gen.generate_docs();

        assert!(docs.contains("# PMAT MCP Tools"));
        assert!(docs.contains("## pmat_analyze"));
        assert!(docs.contains("## pmat_scaffold"));
        assert!(docs.contains("### Input Schema"));
    }

    #[test]
    fn test_schema_type_mapping() {
        let registry = CommandRegistry::new("1.0.0");
        let gen = McpSchemaGenerator::new(registry);

        let string_arg = ArgumentMetadata {
            name: "test".to_string(),
            value_type: ValueType::String,
            ..Default::default()
        };
        let schema = gen.arg_to_json_schema(&string_arg);
        assert_eq!(schema.get("type").unwrap(), "string");

        let int_arg = ArgumentMetadata {
            name: "test".to_string(),
            value_type: ValueType::Integer,
            ..Default::default()
        };
        let schema = gen.arg_to_json_schema(&int_arg);
        assert_eq!(schema.get("type").unwrap(), "integer");

        let bool_arg = ArgumentMetadata {
            name: "test".to_string(),
            value_type: ValueType::Boolean,
            ..Default::default()
        };
        let schema = gen.arg_to_json_schema(&bool_arg);
        assert_eq!(schema.get("type").unwrap(), "boolean");
    }
}
