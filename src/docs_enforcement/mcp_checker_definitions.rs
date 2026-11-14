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

    let infos = live_tool_infos();
    anyhow::ensure!(
        infos.len() == LIVE_MCP_TOOLS.len(),
        "live_tool_infos() yields {} handlers but LIVE_MCP_TOOLS lists {}",
        infos.len(),
        LIVE_MCP_TOOLS.len()
    );

    let mut tools = Vec::with_capacity(infos.len());
    for (info, &(name, catalog_description)) in infos.into_iter().zip(LIVE_MCP_TOOLS) {
        let info = info.ok_or_else(|| anyhow::anyhow!("live tool `{name}` declares no metadata"))?;
        tools.push(McpToolDefinition {
            name: name.to_string(),
            description: info
                .description
                .unwrap_or_else(|| catalog_description.to_string()),
            input_schema: info.input_schema,
        });
    }

    Ok(tools)
}
