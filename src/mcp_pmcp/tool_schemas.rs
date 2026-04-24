//! Shared schema helpers for the pmcp-backed MCP tool handlers.
//!
//! KAIZEN-0189 / R17 #3: pmcp 1.10's `ToolHandler::metadata()` defaults to
//! `None`, which causes `ServerCoreBuilder::tool()` to fall back to an empty
//! `ToolInfo { description: None, input_schema: json!({}) }`. MCP clients
//! then see every tool as "no description, no schema" in `tools/list`.
//!
//! The fix is for each `ToolHandler` impl to override `metadata()` and return
//! a `ToolInfo` whose `input_schema` reflects the actual `#[derive(Deserialize)]`
//! args struct. Since the 24 core tools here do **not** share a `schema()`
//! method on their inner types, we declare each tool's schema inline via the
//! `build_tool_info` helper below and keep every schema adjacent to the handler
//! file that uses it.
//!
//! Companion PR #351 applied the same pattern to the 4 `pmat_*` handlers in
//! `agent_context_handlers.rs`, where the inner `McpTool::schema()` already
//! existed. Together these three PRs (KAIZEN-0174 / KAIZEN-0189) ensure every
//! MCP tool advertises a non-empty `inputSchema`.

use pmcp::types::ToolInfo;
use serde_json::{json, Value};

/// Build a `ToolInfo` with a non-empty description and JSON-Schema.
///
/// Every `ToolHandler::metadata()` override in this crate should funnel
/// through this helper so the behavior stays consistent and auditable.
#[inline]
pub fn build_tool_info(name: &str, description: &str, input_schema: Value) -> ToolInfo {
    ToolInfo::new(name, Some(description.to_string()), input_schema)
}

/// Convenience for "paths: string[]"-shaped tools that take only a paths
/// array plus optional fields. Extra properties can be added before calling.
#[inline]
pub fn paths_object_schema(extra_properties: Value, required: Vec<&str>) -> Value {
    let mut base_props = json!({
        "paths": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Filesystem paths (files or directories) to analyze"
        }
    });
    if let (Some(base), Some(extra)) = (base_props.as_object_mut(), extra_properties.as_object()) {
        for (k, v) in extra {
            base.insert(k.clone(), v.clone());
        }
    }
    let req: Vec<Value> = required
        .into_iter()
        .map(|s| Value::String(s.into()))
        .collect();
    json!({
        "type": "object",
        "properties": base_props,
        "required": req
    })
}
