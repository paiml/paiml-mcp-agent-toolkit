//! Unit test for documentation enforcement quality check
//!
//! PMAT-7001 Phase 3

use pmat::docs_enforcement::mcp_checker::{
    generate_validation_report_json, load_mcp_tool_definitions, validate_mcp_documentation,
};

#[test]
fn test_load_and_validate_mcp_tools() {
    let tools_result = load_mcp_tool_definitions();
    assert!(tools_result.is_ok(), "Should load MCP tools: {:?}", tools_result.err());

    let tools = tools_result.unwrap();
    assert!(!tools.is_empty(), "Should have at least one tool");

    println!("Loaded {} MCP tools", tools.len());

    for tool in tools {
        println!("Validating tool: {}", tool.name);
        let report = validate_mcp_documentation(&tool);
        assert!(report.is_ok(), "Validation should succeed for {}: {:?}", tool.name, report.err());

        let report = report.unwrap();
        println!("  Tool {} valid: {}, issues: {}", tool.name, report.is_valid(), report.issues.len());
        println!("    has_description: {}, description_length: {}, description_is_generic: {}",
            report.has_description, report.description_length, report.description_is_generic);
        println!("    has_input_schema: {}, parameters: {}", report.has_input_schema, report.parameters.len());

        for param in &report.parameters {
            if !param.is_valid() {
                println!("    Invalid param: {}", param.name);
                println!("      has_description: {}, description_is_generic: {}, has_type: {}, issues: {}",
                    param.has_description, param.description_is_generic, param.has_type, param.issues.len());
                for issue in &param.issues {
                    println!("        - {}", issue);
                }
            }
        }

        if !report.is_valid() {
            for issue in &report.issues {
                println!("    Issue: {}", issue);
            }
        }
    }
}

#[test]
fn test_json_validation_report() {
    let json_result = generate_validation_report_json();
    assert!(json_result.is_ok(), "Should generate JSON report: {:?}", json_result.err());

    let json = json_result.unwrap();
    println!("JSON Validation Report:");
    println!("{}", json);

    // Parse to verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");

    assert!(parsed["total_tools"].is_number(), "Should have total_tools count");
    assert!(parsed["valid_tools"].is_number(), "Should have valid_tools count");
    assert!(parsed["invalid_tools"].is_number(), "Should have invalid_tools count");
    assert!(parsed["tools"].is_array(), "Should have tools array");

    let total = parsed["total_tools"].as_u64().unwrap();
    assert!(total > 0, "Should have at least one tool");

    let valid = parsed["valid_tools"].as_u64().unwrap();
    assert_eq!(valid, total, "All tools should be valid");
}
