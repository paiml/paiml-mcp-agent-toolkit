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

/// Schema for a tool that analyses ONE project root, plus optional fields.
///
/// The `paths`-array shape next door is wrong for the repo-wide analyzers
/// (`analyze_reachability`, `analyze_hardcoded_paths`, `analyze_vacuous_tests`):
/// each is rooted at a manifest or a git worktree and enumerates its own file
/// set from there, so a list of paths would have to be silently reduced to its
/// first element — the exact defect R13 pinned for `quality_gate`
/// (`{"paths":["a","b"]}` graded `a`, dropped `b`, and reported
/// `not_measured: []`). Taking a single root advertises what is actually
/// honoured, and matches the CLI, whose `--path` is likewise one path.
#[inline]
pub fn project_root_schema(extra_properties: Value) -> Value {
    let mut props = json!({
        "project_path": {
            "type": "string",
            "description": "Project root to analyze (the directory holding Cargo.toml / the git worktree)"
        }
    });
    if let (Some(base), Some(extra)) = (props.as_object_mut(), extra_properties.as_object()) {
        for (k, v) in extra {
            base.insert(k.clone(), v.clone());
        }
    }
    json!({
        "type": "object",
        "properties": props,
        "required": ["project_path"]
    })
}

/// Convert a single MCP `project_path` argument to a `PathBuf`, rejecting one
/// that does not exist or is not a directory.
///
/// The single-root counterpart of [`resolve_existing_paths`], and it exists for
/// the same reason (GH #639): a repo-wide analyzer pointed at a nonexistent
/// root enumerates zero files and reports zero findings, which an MCP client —
/// with no exit code to fall back on — cannot tell apart from a clean tree.
///
/// A FILE is refused as well as a missing path. These analyzers run
/// `cargo metadata` or `git ls-files` from the root they are given; handed a
/// file, both return nothing and the report would again read as clean.
///
/// # Errors
///
/// Returns a validation error naming the path when it is missing, empty, or not
/// a directory.
pub fn resolve_existing_root(project_path: &str) -> pmcp::Result<std::path::PathBuf> {
    if project_path.trim().is_empty() {
        return Err(pmcp::Error::validation(
            "`project_path` must name a directory; an empty string selects nothing, \
             and a report over nothing is indistinguishable from a clean result.",
        ));
    }
    let root = std::path::PathBuf::from(project_path);
    if !root.exists() {
        return Err(pmcp::Error::validation(format!(
            "project_path not found: {}. Analysing a nonexistent root would report zero \
             findings, which is indistinguishable from a clean result.",
            root.display()
        )));
    }
    if !root.is_dir() {
        return Err(pmcp::Error::validation(format!(
            "project_path must be a directory, but {} is a file. These analyzers enumerate \
             their file set with `cargo metadata` / `git ls-files` from the root given, so a \
             file yields an empty scan — indistinguishable from a clean result.",
            root.display()
        )));
    }
    Ok(root)
}

/// Convert MCP `paths` arguments to `PathBuf`s, rejecting ones that do not exist.
///
/// # Why this exists (GH #639)
///
/// Every path-taking tool did `params.paths.into_iter().map(PathBuf::from)` and
/// analysed whatever came back. A path that does not exist walks to zero files,
/// so `tools/call analyze_complexity {"paths": ["/typo"]}` returned
/// `isError: false` with `{"status":"completed","total_files":0,"violations":[]}`
/// — indistinguishable from a clean repository. Unlike the CLI, an MCP client
/// has no exit code to fall back on, so a mistyped path read as a passing
/// quality check.
///
/// The CLI was given the same guard in v3.28.1
/// (`cli::ensure_analysis_path_exists`); this is its MCP counterpart, so the two
/// surfaces now agree. It also makes the surface self-consistent: `quality_gate`
/// already errored on a nonexistent `file`.
///
/// # Errors
///
/// Returns a validation error naming every missing path, and one for an empty
/// `paths` list.
///
/// The empty list used to be waved through on the theory that "tools interpret
/// that as use-the-default-scope". No tool does: every `tool_functions` entry
/// point opens with `if paths.is_empty() { bail!("At least one path must be
/// provided") }`, and each handler wraps that in `Error::internal`, so
/// `{"paths": []}` — a schema violation the caller can fix — came back as
/// `-32603 Internal error`, which tells a client the SERVER is broken and the
/// call is worth retrying. Rejecting it here fixes the whole class at once
/// rather than in each of the 22 call sites.
pub fn resolve_existing_paths(paths: Vec<String>) -> pmcp::Result<Vec<std::path::PathBuf>> {
    if paths.is_empty() {
        return Err(pmcp::Error::validation(
            "`paths` must name at least one file or directory; an empty list \
             selects nothing, and a report over nothing is indistinguishable \
             from a clean result.",
        ));
    }

    let resolved: Vec<std::path::PathBuf> =
        paths.into_iter().map(std::path::PathBuf::from).collect();

    let missing: Vec<String> = resolved
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.display().to_string())
        .collect();

    if !missing.is_empty() {
        return Err(pmcp::Error::validation(format!(
            "path(s) not found: {}. Analysing a nonexistent path would report zero \
             findings, which is indistinguishable from a clean result.",
            missing.join(", ")
        )));
    }

    Ok(resolved)
}
