//! MACS F6 (Component 32): canonical MCP tool manifest.
//!
//! Sub-spec: `docs/specifications/components/modern-agentic-coding-support.md`
//! Contract: `contracts/macs-artifacts-v1.yaml#manifest_faithful`
//!
//! `LIVE_MCP_TOOLS` is the single source of truth for which tools the live
//! `SimpleUnifiedServer` advertises. `render_manifest` regenerates the root
//! `mcp.json` from it deterministically (pure: names in registration order,
//! canonical JSON), and CB-1656 compares the committed manifest against a
//! fresh render — hand-edits are drift. The `manifest_matches_server`
//! drift-guard test pins this list to the server's actual `.tool(...)`
//! registrations so the two can never silently diverge.

/// The 16 tools registered by `SimpleUnifiedServer::run()`, in registration
/// order, with their catalog descriptions. Source of truth for `mcp.json`.
pub const LIVE_MCP_TOOLS: &[(&str, &str)] = &[
    (
        "analyze_complexity",
        "Analyze cyclomatic and cognitive complexity of code",
    ),
    (
        "analyze_satd",
        "Detect self-admitted technical debt (TODO/FIXME/HACK)",
    ),
    (
        "analyze_dead_code",
        "Detect unreachable functions and modules",
    ),
    (
        "analyze_dag",
        "Build the dependency/call graph of a project",
    ),
    (
        "analyze_deep_context",
        "Generate comprehensive annotated project context",
    ),
    (
        "analyze_big_o",
        "Estimate algorithmic complexity of functions",
    ),
    (
        "quality_gate",
        "Run the quality gate (complexity, satd, lint, tests)",
    ),
    ("quality_proxy", "Proxy a quality-scored analysis request"),
    (
        "pdmt_deterministic_todos",
        "Generate deterministic PDMT todo lists",
    ),
    ("git_operation", "Perform a git operation for the workflow"),
    (
        "generate_context",
        "Generate LLM-optimized codebase context",
    ),
    ("scaffold_project", "Scaffold a project or agent skeleton"),
    ("pmat_query_code", "Semantic code search ranked by quality"),
    (
        "pmat_get_function",
        "Return a full function with quality metrics",
    ),
    (
        "pmat_find_similar",
        "Find functions similar to a target (refactoring)",
    ),
    (
        "pmat_index_stats",
        "Report agent-context index health and statistics",
    ),
];

/// A minimal object inputSchema for a tool (the manifest advertises the
/// surface; full per-tool schemas live in the pmcp handler metadata and are
/// covered by the deterministic sweep, MACS-011).
fn tool_schema(name: &str) -> serde_json::Value {
    // No-arg tools carry an empty, closed object; the rest accept a paths
    // array — the shared shape across the analyze/generate family.
    let no_arg = matches!(
        name,
        "refactor.nextIteration" | "refactor.getState" | "refactor.stop" | "pmat_index_stats"
    );
    if no_arg {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    } else {
        serde_json::json!({
            "type": "object",
            "properties": {
                "paths": {"type": "array", "items": {"type": "string"}}
            }
        })
    }
}

/// Render `mcp.json` deterministically from `LIVE_MCP_TOOLS`. Pure: output
/// depends only on the tool list (canonical, sorted keys via serde with the
/// tools map ordered by registration index encoded in a `tools` array-of-
/// objects to preserve order stably).
pub fn render_manifest(version: &str) -> String {
    let tools: Vec<serde_json::Value> = LIVE_MCP_TOOLS
        .iter()
        .map(|(name, desc)| {
            serde_json::json!({
                "name": name,
                "description": desc,
                "inputSchema": tool_schema(name),
            })
        })
        .collect();
    let manifest = serde_json::json!({
        "name": "pmat",
        "version": version,
        "description": "Project Analysis and Intelligence Modeling Toolkit",
        // NOT "target/release/pmat". This manifest ships inside the published
        // crate, and a build-artifact path only resolves on a machine that has
        // built from source in this exact layout — for anyone who ran
        // `cargo install pmat` it points at nothing. The installed binary is on
        // PATH under its own name, which is what a client can actually launch.
        "main": "pmat",
        "bin": {"pmat": "pmat"},
        "mcp": {
            "runtime": "binary",
            "launch": {"env": {"MCP_VERSION": "1"}},
            "tool_count": LIVE_MCP_TOOLS.len(),
            "tools": tools,
        }
    });
    // Pretty + trailing newline: byte-stable, diff-friendly.
    let mut out =
        serde_json::to_string_pretty(&manifest).expect("manifest serialization is infallible");
    out.push('\n');
    out
}

/// Tool names declared by a parsed `mcp.json` (new array shape or the legacy
/// object-map shape), sorted for set comparison.
pub fn manifest_tool_names(manifest: &serde_json::Value) -> Vec<String> {
    let tools = manifest.get("mcp").and_then(|m| m.get("tools"));
    let mut names: Vec<String> = match tools {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(str::to_string))
            .collect(),
        Some(serde_json::Value::Object(map)) => map.keys().cloned().collect(),
        _ => Vec::new(),
    };
    names.sort_unstable();
    names
}

/// The canonical tool-name set (sorted), for CB-1656/README reconciliation.
pub fn canonical_tool_names() -> Vec<String> {
    let mut names: Vec<String> = LIVE_MCP_TOOLS.iter().map(|(n, _)| n.to_string()).collect();
    names.sort_unstable();
    names
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_matches_server() {
        // Drift guard: LIVE_MCP_TOOLS must equal the server's actual
        // `.tool(...)` registrations in run(). If a tool is added/removed,
        // this fails until the const is updated (mirrors the 16-tool pin).
        let src = include_str!("simple_unified_server.rs");
        let run_start = src.find("pub async fn run").expect("run() present");
        // Sentinel must not embed the tool COUNT: it was `fn test_all_20`, so
        // renaming that test to `test_all_16` silently widened this scan to the
        // whole file — including test fixtures that also contain `.tool(` —
        // and the guard started comparing against the wrong set. A guard keyed
        // on a number that changes when the thing it guards changes is not a
        // guard.
        let run_end = src[run_start..]
            .find("fn test_all_")
            .map(|i| run_start + i)
            .expect("the registry drift sentinel `fn test_all_` must exist");
        let registered: Vec<String> = src[run_start..run_end]
            .split(".tool(")
            .skip(1)
            .filter_map(|seg| {
                let q1 = seg.find('"')?;
                let rest = &seg[q1 + 1..];
                let q2 = rest.find('"')?;
                Some(rest[..q2].to_string())
            })
            .collect();
        let declared: Vec<String> = LIVE_MCP_TOOLS.iter().map(|(n, _)| n.to_string()).collect();
        assert_eq!(
            registered, declared,
            "LIVE_MCP_TOOLS drifted from server .tool() registrations"
        );
        assert_eq!(
            declared.len(),
            16,
            "the live server advertises 16 tools; the 4 refactor.* tools were unregistered in EV-0 (#999) because their engine synthesizes violations from a path substring"
        );
    }

    #[test]
    fn generated_equals_tool_defs() {
        let manifest: serde_json::Value =
            serde_json::from_str(&render_manifest("9.9.9")).expect("render is valid JSON");
        assert_eq!(manifest_tool_names(&manifest), canonical_tool_names());
        assert_eq!(
            manifest["mcp"]["tool_count"].as_u64(),
            Some(LIVE_MCP_TOOLS.len() as u64)
        );
    }

    /// The manifest must not advertise a build-artifact path.
    ///
    /// `mcp.json` ships inside the published crate (verified: it is present in
    /// pmat-3.30.1.crate). It carried `target/release/pmat`, which resolves
    /// only on a machine that has built from source in that layout — for a
    /// `cargo install pmat` user it names a file that does not exist.
    ///
    /// This pins the property rather than the version: CB-1656 deliberately
    /// ignores version churn so a release bump does not turn it red
    /// (`check_macs_artifacts.rs:5-6`), and a version-equality assertion here
    /// would fight that decision and redden every release.
    #[test]
    fn manifest_advertises_no_build_artifact_path() {
        let rendered = render_manifest("9.9.9");
        for bad in ["target/release", "target/debug", "../target"] {
            assert!(
                !rendered.contains(bad),
                "manifest advertises the build-artifact path '{bad}', which does not exist for an installed binary:\n{rendered}"
            );
        }
        let committed = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("mcp.json"),
        )
        .expect("committed mcp.json");
        for bad in ["target/release", "target/debug"] {
            assert!(
                !committed.contains(bad),
                "committed mcp.json still advertises '{bad}' — regenerate it: cargo test --lib regenerate_mcp_json -- --ignored"
            );
        }
    }

    #[test]
    fn two_runs_identical() {
        assert_eq!(render_manifest("1.2.3"), render_manifest("1.2.3"));
    }

    /// Regenerates the committed root mcp.json. Run manually after changing
    /// LIVE_MCP_TOOLS:  cargo test --lib regenerate_mcp_json -- --ignored
    #[test]
    #[ignore = "regenerator, not a test — writes root mcp.json"]
    fn regenerate_mcp_json() {
        let root = env!("CARGO_MANIFEST_DIR");
        let version = env!("CARGO_PKG_VERSION");
        std::fs::write(
            std::path::Path::new(root).join("mcp.json"),
            render_manifest(version),
        )
        .expect("write mcp.json");
    }

    #[test]
    fn manifest_names_reads_legacy_object_shape() {
        // The old mcp.json used an object map of tool_name -> def.
        let legacy = serde_json::json!({
            "mcp": {"tools": {"generate_template": {}, "generate_unified_context": {}}}
        });
        assert_eq!(
            manifest_tool_names(&legacy),
            vec!["generate_template", "generate_unified_context"]
        );
    }
}
