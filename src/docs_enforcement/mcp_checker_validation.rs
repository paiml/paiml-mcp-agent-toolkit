// MCP checker validation logic - included by mcp_checker.rs

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
            let required_params = tool
                .input_schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
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
        report
            .issues
            .push("Input schema has no 'properties' field".to_string());
    }

    Ok(report)
}

/// Validate a single parameter's documentation
fn validate_parameter(name: &str, schema: &Value, is_required: bool) -> ParameterReport {
    let description = schema
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let param_type = schema
        .get("type")
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
            name, description
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
        total_issues += report.issues.len()
            + report
                .parameters
                .iter()
                .map(|p| p.issues.len())
                .sum::<usize>();
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
