//! MCP Documentation Checker
//!
//! TICKET: PMAT-7001 Phase 2 (GREEN)
//!
//! This module validates that MCP tools have complete, accurate documentation
//! including tool descriptions, parameter schemas, and non-generic text.

use crate::docs_enforcement::generic_detector::is_generic_description;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP tool definition (simplified for validation)
#[derive(Debug, Clone)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP documentation validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpDocumentationReport {
    pub tool_name: String,
    pub has_description: bool,
    pub description_length: usize,
    pub description_is_generic: bool,
    pub has_input_schema: bool,
    pub parameters: Vec<ParameterReport>,
    pub issues: Vec<String>,
}

/// Parameter documentation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterReport {
    pub name: String,
    pub has_description: bool,
    pub description: String,
    pub description_is_generic: bool,
    pub has_type: bool,
    pub param_type: String,
    pub is_required: bool,
    pub has_default: bool,
    pub issues: Vec<String>,
}

impl McpDocumentationReport {
    pub fn is_valid(&self) -> bool {
        self.has_description
            && self.description_length > 20
            && !self.description_is_generic
            && self.has_input_schema
            && self.parameters.iter().all(|p| p.is_valid())
            && self.issues.is_empty()
    }
}

impl ParameterReport {
    pub fn is_valid(&self) -> bool {
        self.has_description
            && !self.description_is_generic
            && self.has_type
            && self.issues.is_empty()
    }
}

/// Validate MCP tool documentation
///
/// Checks that the tool has:
/// - Non-empty, non-generic description (>20 chars)
/// - Valid input schema
/// - All parameters documented
/// - Parameter types specified
/// - Required parameters marked
pub fn validate_mcp_documentation(tool: &McpToolDefinition) -> Result<McpDocumentationReport> {
    let mut report = McpDocumentationReport {
        tool_name: tool.name.clone(),
        has_description: !tool.description.is_empty(),
        description_length: tool.description.len(),
        description_is_generic: is_generic_description(&tool.description),
        has_input_schema: !tool.input_schema.is_null(),
        parameters: Vec::new(),
        issues: Vec::new(),
    };

    // Check description
    if !report.has_description {
        report.issues.push("Tool has no description".to_string());
    } else if report.description_length < 20 {
        report.issues.push(format!(
            "Tool description too short ({} chars, min 20)",
            report.description_length
        ));
    }

    if report.description_is_generic {
        report.issues.push(format!(
            "Tool description is generic: '{}'",
            tool.description
        ));
    }

    // Check input schema
    if !report.has_input_schema {
        report.issues.push("Tool has no input_schema".to_string());
        return Ok(report);
    }

    // Extract parameters from schema
    if let Some(properties) = tool.input_schema.get("properties") {
        if let Some(props_obj) = properties.as_object() {
            let required_params = tool.input_schema.get("required")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            for (param_name, param_schema) in props_obj {
                let param_report = validate_parameter(
                    param_name,
                    param_schema,
                    required_params.contains(&param_name.as_str()),
                );
                report.parameters.push(param_report);
            }
        }
    } else {
        report.issues.push("Input schema has no 'properties' field".to_string());
    }

    Ok(report)
}

/// Validate a single parameter's documentation
fn validate_parameter(
    name: &str,
    schema: &Value,
    is_required: bool,
) -> ParameterReport {
    let description = schema.get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let param_type = schema.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    let has_default = schema.get("default").is_some();

    let mut issues = Vec::new();

    // Check description
    let has_description = !description.is_empty();
    let description_is_generic = if has_description {
        is_generic_description(description)
    } else {
        true // Empty is generic
    };

    if !has_description {
        issues.push(format!("Parameter '{}' has no description", name));
    } else if description.len() < 15 {
        issues.push(format!(
            "Parameter '{}' description too short ({} chars)",
            name,
            description.len()
        ));
    }

    if description_is_generic {
        issues.push(format!(
            "Parameter '{}' has generic description: '{}'",
            name,
            description
        ));
    }

    // Check type
    let has_type = param_type != "unknown";
    if !has_type {
        issues.push(format!("Parameter '{}' has no type specified", name));
    }

    // Check for defaults on optional params
    if !is_required && !has_default && !description.contains("default") {
        issues.push(format!(
            "Optional parameter '{}' should document default value",
            name
        ));
    }

    ParameterReport {
        name: name.to_string(),
        has_description,
        description: description.to_string(),
        description_is_generic,
        has_type,
        param_type: param_type.to_string(),
        is_required,
        has_default,
        issues,
    }
}

/// Load MCP tool definitions from MCP server
///
/// This would connect to the actual MCP server to get tool definitions.
/// For testing, we'll parse from the mcp_impl.rs handlers.
pub fn load_mcp_tool_definitions() -> Result<Vec<McpToolDefinition>> {
    // TODO: Implement actual loading from MCP server
    // For now, return hardcoded tool definitions based on PMAT-6017, PMAT-6019, etc.

    let tools = vec![
        McpToolDefinition {
            name: "scaffold_agent".to_string(),
            description: "Scaffold a deterministic MCP agent with complete project structure, tests, and documentation".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Agent project name (lowercase, alphanumeric, hyphens only)"
                    },
                    "template": {
                        "type": "string",
                        "description": "Template type: mcp-server (basic), state-machine (stateful), hybrid (deterministic + LLM), calculator (math example)",
                        "default": "mcp-server"
                    },
                    "output_dir": {
                        "type": "string",
                        "description": "Output directory where the agent project will be created (default: current directory)"
                    },
                    "quality_level": {
                        "type": "string",
                        "description": "Quality level: standard (fast basic scaffolding), high (thorough with tests), extreme (comprehensive with ML and mutation testing)",
                        "default": "standard"
                    },
                    "features": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Features to include: testing, docs, ci, mutation, property-testing, tui, http-server (comma-separated, default: empty array)"
                    }
                },
                "required": ["name"]
            }),
        },
        McpToolDefinition {
            name: "validate_roadmap".to_string(),
            description: "Validate ROADMAP.md structure and check that all tickets referenced in roadmap exist as files".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "roadmap_path": {
                        "type": "string",
                        "description": "Path to ROADMAP.md file for validation (default: ./ROADMAP.md in project root)"
                    },
                    "tickets_dir": {
                        "type": "string",
                        "description": "Directory containing ticket files (default: ./docs/tickets)"
                    }
                },
                "required": []
            }),
        },
        McpToolDefinition {
            name: "health_check".to_string(),
            description: "Run comprehensive project health checks including build, tests, coverage, complexity, and technical debt analysis".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_dir": {
                        "type": "string",
                        "description": "Path to project directory to analyze (default: current directory)"
                    },
                    "quick": {
                        "type": "boolean",
                        "description": "Quick mode: run only fast checks (build only, skip tests/coverage/analysis)",
                        "default": false
                    },
                    "check_build": {
                        "type": "boolean",
                        "description": "Check if project builds successfully (cargo check/build)",
                        "default": true
                    },
                    "check_tests": {
                        "type": "boolean",
                        "description": "Run test suite and verify all tests pass",
                        "default": true
                    },
                    "check_coverage": {
                        "type": "boolean",
                        "description": "Measure code coverage and verify meets threshold (default: 70%)",
                        "default": true
                    },
                    "check_complexity": {
                        "type": "boolean",
                        "description": "Analyze code complexity and flag violations (cyclomatic >8, cognitive >15)",
                        "default": true
                    },
                    "check_satd": {
                        "type": "boolean",
                        "description": "Scan for Self-Admitted Technical Debt (TODO, FIXME, HACK annotations)",
                        "default": true
                    }
                },
                "required": []
            }),
        },
        McpToolDefinition {
            name: "generate_tickets".to_string(),
            description: "Generate ticket files from ROADMAP.md entries that don't have corresponding ticket files yet".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "roadmap_path": {
                        "type": "string",
                        "description": "Path to ROADMAP.md file containing ticket list (default: ./ROADMAP.md)"
                    },
                    "tickets_dir": {
                        "type": "string",
                        "description": "Directory where ticket files should be created (default: ./docs/tickets)"
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "Dry-run mode: show what would be generated without creating files (preview only)",
                        "default": false
                    }
                },
                "required": []
            }),
        },
    ];

    Ok(tools)
}

/// Validation summary for JSON output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_tools: usize,
    pub valid_tools: usize,
    pub invalid_tools: usize,
    pub total_issues: usize,
    pub tools: Vec<McpDocumentationReport>,
}

/// Generate comprehensive validation report as JSON
pub fn generate_validation_report_json() -> Result<String> {
    let tools = load_mcp_tool_definitions()?;
    let mut reports = Vec::new();
    let mut valid_count = 0;
    let mut total_issues = 0;

    for tool in tools {
        let report = validate_mcp_documentation(&tool)?;
        if report.is_valid() {
            valid_count += 1;
        }
        total_issues += report.issues.len() + report.parameters.iter().map(|p| p.issues.len()).sum::<usize>();
        reports.push(report);
    }

    let summary = ValidationSummary {
        total_tools: reports.len(),
        valid_tools: valid_count,
        invalid_tools: reports.len() - valid_count,
        total_issues,
        tools: reports,
    };

    Ok(serde_json::to_string_pretty(&summary)?)
}

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
}
