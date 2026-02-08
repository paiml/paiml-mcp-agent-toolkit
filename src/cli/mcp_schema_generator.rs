#![cfg_attr(coverage_nightly, coverage(off))]
//! MCP Schema Generator - Generate MCP tool definitions from CommandRegistry
//!
//! This module generates JSON Schema for MCP tools from the single source of truth,
//! ensuring MCP tool definitions never drift from CLI implementations.
//!
//! # Architecture (Toyota Way - Poka-yoke)
//!
//! ```text
//! CommandRegistry → McpSchemaGenerator → tools/list response
//!                                            └─ JSON Schema
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - MCP Protocol: https://spec.modelcontextprotocol.io/

use crate::cli::registry::{CommandMetadata, CommandRegistry, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Generates MCP tool definitions from CommandRegistry.
pub struct McpSchemaGenerator {
    registry: CommandRegistry,
}

/// MCP tool definition as per protocol spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<McpToolAnnotations>,
}

/// MCP tool annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// Schema consistency error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    MissingSchemaProperty {
        tool: String,
        property: String,
    },
    TypeMismatch {
        tool: String,
        property: String,
        expected: String,
        actual: String,
    },
    DuplicateToolName {
        tool_name: String,
        command1: String,
        command2: String,
    },
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSchemaProperty { tool, property } => {
                write!(
                    f,
                    "Tool '{}' missing required property '{}'",
                    tool, property
                )
            }
            Self::TypeMismatch {
                tool,
                property,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Tool '{}' property '{}' type mismatch: expected {}, got {}",
                    tool, property, expected, actual
                )
            }
            Self::DuplicateToolName {
                tool_name,
                command1,
                command2,
            } => {
                write!(
                    f,
                    "Duplicate MCP tool name '{}' in commands '{}' and '{}'",
                    tool_name, command1, command2
                )
            }
        }
    }
}

impl std::error::Error for SchemaError {}

impl McpSchemaGenerator {
    /// Create a new McpSchemaGenerator
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    /// Generate tools/list response for MCP protocol
    pub fn generate_tools_list(&self) -> Vec<McpToolDefinition> {
        self.registry
            .commands
            .values()
            .filter_map(|cmd| self.generate_tool_definition(cmd))
            .collect()
    }

    /// Generate a single tool definition from command metadata
    fn generate_tool_definition(&self, cmd: &CommandMetadata) -> Option<McpToolDefinition> {
        let mcp = cmd.mcp.as_ref()?;

        Some(McpToolDefinition {
            name: mcp.tool_name.clone(),
            description: if cmd.long_description.is_empty() {
                cmd.short_description.clone()
            } else {
                cmd.long_description.clone()
            },
            input_schema: if mcp.input_schema.is_null() {
                self.generate_schema_from_args(cmd)
            } else {
                mcp.input_schema.clone()
            },
            annotations: Some(McpToolAnnotations {
                title: Some(cmd.name.clone()),
                read_only_hint: Some(!mcp.is_mutation),
                destructive_hint: Some(false),
                idempotent_hint: Some(!mcp.is_mutation),
                open_world_hint: Some(true),
            }),
        })
    }

    /// Generate JSON Schema from command arguments
    fn generate_schema_from_args(&self, cmd: &CommandMetadata) -> Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for arg in &cmd.arguments {
            let prop_schema = self.arg_to_json_schema(arg);
            properties.insert(self.arg_to_property_name(&arg.name), prop_schema);

            if arg.required {
                required.push(self.arg_to_property_name(&arg.name));
            }
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }

    /// Convert argument to JSON Schema property
    fn arg_to_json_schema(&self, arg: &crate::cli::registry::ArgumentMetadata) -> Value {
        let mut schema = serde_json::Map::new();

        // Type mapping
        let json_type = match arg.value_type {
            ValueType::String => "string",
            ValueType::Integer => "integer",
            ValueType::Float => "number",
            ValueType::Boolean => "boolean",
            ValueType::Path => "string",
            ValueType::Enum => "string",
            ValueType::List => "array",
        };
        schema.insert("type".to_string(), json!(json_type));

        // Description
        if !arg.description.is_empty() {
            schema.insert("description".to_string(), json!(arg.description));
        }

        // Default value
        if let Some(default) = &arg.default {
            schema.insert("default".to_string(), json!(default));
        }

        // Enum values
        if !arg.possible_values.is_empty() {
            schema.insert("enum".to_string(), json!(arg.possible_values));
        }

        // Path format hint
        if matches!(arg.value_type, ValueType::Path) {
            schema.insert("format".to_string(), json!("path"));
        }

        // Array items type
        if matches!(arg.value_type, ValueType::List) {
            schema.insert("items".to_string(), json!({"type": "string"}));
        }

        Value::Object(schema)
    }

    /// Convert CLI argument name to MCP property name (kebab-case to snake_case)
    fn arg_to_property_name(&self, name: &str) -> String {
        name.replace('-', "_")
    }

    /// Validate consistency between registry and MCP schemas
    pub fn validate_consistency(&self) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // Check for duplicate tool names
        let mut seen_tools: std::collections::HashMap<&str, &str> =
            std::collections::HashMap::new();
        for (name, cmd) in &self.registry.commands {
            if let Some(mcp) = &cmd.mcp {
                if let Some(existing) = seen_tools.get(mcp.tool_name.as_str()) {
                    errors.push(SchemaError::DuplicateToolName {
                        tool_name: mcp.tool_name.clone(),
                        command1: (*existing).to_string(),
                        command2: name.clone(),
                    });
                } else {
                    seen_tools.insert(mcp.tool_name.as_str(), name.as_str());
                }

                // Validate that all arguments are in the schema
                if !mcp.input_schema.is_null() {
                    for arg in &cmd.arguments {
                        let prop_name = self.arg_to_property_name(&arg.name);
                        if !self.schema_has_property(&mcp.input_schema, &prop_name) {
                            errors.push(SchemaError::MissingSchemaProperty {
                                tool: mcp.tool_name.clone(),
                                property: prop_name,
                            });
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if JSON schema has a property
    fn schema_has_property(&self, schema: &Value, property: &str) -> bool {
        schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|props| props.contains_key(property))
            .unwrap_or(false)
    }

    /// Generate OpenAPI-compatible schema for all tools
    pub fn generate_openapi_schema(&self) -> Value {
        let tools = self.generate_tools_list();

        let mut paths = serde_json::Map::new();
        for tool in &tools {
            paths.insert(
                format!("/tools/{}", tool.name),
                json!({
                    "post": {
                        "summary": tool.description,
                        "operationId": tool.name,
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": tool.input_schema
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "Successful response"
                            }
                        }
                    }
                }),
            );
        }

        json!({
            "openapi": "3.0.0",
            "info": {
                "title": "PMAT MCP Tools",
                "version": self.registry.version
            },
            "paths": paths
        })
    }

    /// Generate markdown documentation for all MCP tools
    pub fn generate_docs(&self) -> String {
        let mut doc = String::new();
        doc.push_str("# PMAT MCP Tools\n\n");
        doc.push_str(&format!("Version: {}\n\n", self.registry.version));

        let tools = self.generate_tools_list();
        for tool in &tools {
            doc.push_str(&format!("## {}\n\n", tool.name));
            doc.push_str(&format!("{}\n\n", tool.description));

            doc.push_str("### Input Schema\n\n");
            doc.push_str("```json\n");
            doc.push_str(&serde_json::to_string_pretty(&tool.input_schema).unwrap_or_default());
            doc.push_str("\n```\n\n");

            if let Some(annotations) = &tool.annotations {
                doc.push_str("### Annotations\n\n");
                if let Some(title) = &annotations.title {
                    doc.push_str(&format!("- **Title**: {}\n", title));
                }
                if let Some(ro) = annotations.read_only_hint {
                    doc.push_str(&format!("- **Read-only**: {}\n", ro));
                }
                if let Some(idempotent) = annotations.idempotent_hint {
                    doc.push_str(&format!("- **Idempotent**: {}\n", idempotent));
                }
                doc.push('\n');
            }
        }

        doc
    }
}

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
