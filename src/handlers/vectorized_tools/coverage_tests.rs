use super::*;

#[test]
fn test_get_vectorized_tools_info_returns_seven_tools() {
    let tools = get_vectorized_tools_info();
    assert_eq!(tools.len(), 7);
}

#[test]
fn test_vectorized_tools_have_required_fields() {
    let tools = get_vectorized_tools_info();
    for tool in &tools {
        assert!(
            tool.get("name").is_some(),
            "Tool missing 'name' field: {:?}",
            tool
        );
        assert!(
            tool.get("description").is_some(),
            "Tool missing 'description' field"
        );
        assert!(
            tool.get("inputSchema").is_some(),
            "Tool missing 'inputSchema' field"
        );
    }
}

#[test]
fn test_vectorized_tools_names_are_valid() {
    let tools = get_vectorized_tools_info();
    let expected_names = [
        "analyze_duplicates_vectorized",
        "analyze_graph_metrics_vectorized",
        "analyze_name_similarity_vectorized",
        "analyze_symbol_table_vectorized",
        "analyze_incremental_coverage_vectorized",
        "analyze_big_o_vectorized",
        "generate_enhanced_report",
    ];
    for (i, tool) in tools.iter().enumerate() {
        let name = tool["name"].as_str().unwrap();
        assert_eq!(name, expected_names[i]);
    }
}

#[test]
fn test_vectorized_tools_schemas_are_objects() {
    let tools = get_vectorized_tools_info();
    for tool in &tools {
        let schema = &tool["inputSchema"];
        assert_eq!(schema["type"].as_str().unwrap(), "object");
        assert!(schema.get("properties").is_some());
    }
}

#[test]
fn test_vectorized_tools_descriptions_are_meaningful() {
    let tools = get_vectorized_tools_info();
    for tool in &tools {
        let desc = tool["description"].as_str().unwrap();
        assert!(desc.len() > 20, "Description too short: '{}'", desc);
    }
}

#[test]
fn test_vectorized_tools_have_project_path() {
    let tools = get_vectorized_tools_info();
    // Most analysis tools should have project_path parameter
    for tool in tools.iter().take(6) {
        let props = &tool["inputSchema"]["properties"];
        assert!(
            props.get("project_path").is_some(),
            "Tool {} missing project_path",
            tool["name"]
        );
    }
}
