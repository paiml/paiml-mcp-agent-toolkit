/// Returns all analysis tool definitions (standard + vectorized + reporting).
fn analysis_tool_definitions() -> Vec<serde_json::Value> {
    let mut tools = standard_analysis_tool_definitions();
    tools.extend(vectorized_tool_definitions());
    tools.extend(reporting_tool_definitions());
    tools
}
