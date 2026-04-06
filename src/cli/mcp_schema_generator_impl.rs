// MCP Schema Generator - Implementation methods
// Included from mcp_schema_generator.rs - shares parent module scope

impl McpSchemaGenerator {
    /// Create a new McpSchemaGenerator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    /// Generate tools/list response for MCP protocol
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_tools_list(&self) -> Vec<McpToolDefinition> {
        self.registry
            .commands
            .values()
            .filter_map(|cmd| self.generate_tool_definition(cmd))
            .collect()
    }

    /// Generate a single tool definition from command metadata
    fn generate_tool_definition(&self, cmd: &CommandMetadata) -> Option<McpToolDefinition> {
        debug_assert!(true, "contract: generate_tool_definition");
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
        debug_assert!(true, "contract: generate_schema_from_args");
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
        debug_assert!(true, "contract: arg_to_json_schema");
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
        debug_assert!(!name.is_empty(), "name must not be empty");
        name.replace('-', "_")
    }

    /// Validate consistency between registry and MCP schemas
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(!property.is_empty(), "property must not be empty");
        schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|props| props.contains_key(property))
            .unwrap_or(false)
    }

    /// Generate OpenAPI-compatible schema for all tools
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
