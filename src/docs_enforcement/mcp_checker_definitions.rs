// MCP checker tool definitions - included by mcp_checker.rs

/// Load MCP tool definitions for the tools the live server advertises.
///
/// Derived from the handlers' own `metadata()` via
/// `crate::mcp_pmcp::tool_manifest::live_tool_infos` — the same objects
/// `tools/list` serves — so this inventory cannot drift from the server
/// (GH-1230). It used to be a hand-written list of four tools, one of which
/// (`scaffold_agent`) never existed on the MCP surface.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn load_mcp_tool_definitions() -> Result<Vec<McpToolDefinition>> {
    use crate::mcp_pmcp::tool_manifest::{live_tool_infos, LIVE_MCP_TOOLS};

    let tools = live_tool_infos()
        .into_iter()
        .zip(LIVE_MCP_TOOLS)
        .map(|(info, &(name, catalog_description))| {
            let (description, input_schema) = match info {
                Some(info) => (
                    info.description
                        .unwrap_or_else(|| catalog_description.to_string()),
                    info.input_schema,
                ),
                None => (
                    catalog_description.to_string(),
                    serde_json::json!({"type": "object", "properties": {}}),
                ),
            };
            McpToolDefinition {
                name: name.to_string(),
                description,
                input_schema,
            }
        })
        .collect();

    Ok(tools)
}
