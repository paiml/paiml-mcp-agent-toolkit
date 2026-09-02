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
//!
//! There are two places a client can read a tool's description, and they used
//! to disagree: `tools/list` serves each handler's `metadata()`, while the
//! packaged `mcp.json` serves this file's. Only names and counts were ever
//! compared, so the texts drifted for releases —
//! `manifest_descriptions_match_handler_metadata` now compares the texts too.
//! `docs/mcp/TOOLS.md` is a third answer, in prose, and
//! `tools_doc_states_the_live_tool_count` holds its numbers to this list in any
//! tree that carries that file — which is every checkout, but not the published
//! crate, where `/docs/` is excluded (`Cargo.toml:26`).

/// The 19 tools registered by `SimpleUnifiedServer::run()`, in registration
/// order, with their catalog descriptions. Source of truth for `mcp.json`.
///
/// Which `analyze_*` tools belong here is not a judgement made in this file:
/// [`crate::cli::analyze_mcp_exposure`] decides it with a total match over
/// `AnalyzeCommands`, and `every_advertised_tool_is_actually_served` fails when
/// this list does not carry what that registry promises (#1029).
///
/// The descriptions are not an editorial second opinion either. Each is the
/// byte-identical text the tool's own `metadata()` puts on the wire for
/// `tools/list`, pinned by `manifest_descriptions_match_handler_metadata`.
/// They diverged once and it was not cosmetic: `quality_proxy` shipped here as
/// "Proxy a quality-scored analysis request" while the handler advertised
/// "Proxy a file operation (write/edit/append) through the quality gate", so
/// an agent that read the packaged `mcp.json` rather than calling `tools/list`
/// never learned that the tool writes files. `git_operation` and
/// `scaffold_project` were worse than vague — they named mutations ("Perform a
/// git operation", "Scaffold a project or agent skeleton") for handlers that
/// only read (`GitStatusTool`) and only summarise (`ContextSummaryTool`).
pub const LIVE_MCP_TOOLS: &[(&str, &str)] = &[
    (
        "analyze_complexity",
        "Analyze cyclomatic and cognitive complexity for source files.",
    ),
    (
        "analyze_satd",
        "Detect self-admitted technical debt (TODO, FIXME, HACK markers) in source code.",
    ),
    (
        "analyze_dead_code",
        "Find unreachable or unused code (functions, types, or modules).",
    ),
    (
        "analyze_dag",
        "Generate a project dependency graph (call graph, import graph, inheritance, or full dependency DAG).",
    ),
    (
        "analyze_deep_context",
        "Run the full deep-context analysis pipeline (AST, complexity, churn, dead code) over the given paths.",
    ),
    (
        "analyze_big_o",
        "Classify the Big-O time complexity of functions in the given paths.",
    ),
    (
        "analyze_reachability",
        "Report tracked .rs files that no compilation unit reaches — orphaned modules \
         that compile to nothing and whose tests never run.",
    ),
    (
        "analyze_hardcoded_paths",
        "Find machine-specific absolute paths baked into source (a user's home, a nix \
         store hash, a build root) — correct where they were written, inert everywhere else.",
    ),
    (
        "analyze_vacuous_tests",
        "Find #[test] functions that cannot fail — no assertion, an assertion over \
         constants, or a body that silently returns when a fixture is missing.",
    ),
    (
        "quality_gate",
        "Run the `pmat quality-gate --checks all` suite (complexity, dead code, SATD, entropy, \
         security, duplicates, coverage, documentation sections, provability) plus a TDG score \
         against the given paths. Any check a path could not answer is named in `not_measured` \
         and, with its reason, in `checks.not_run`.",
    ),
    (
        "quality_proxy",
        "Proxy a file operation (write/edit/append) through the quality gate, optionally auto-fixing violations.",
    ),
    (
        "pdmt_deterministic_todos",
        "Generate deterministic, quality-enforced todo lists from a list of requirements.",
    ),
    (
        "git_operation",
        "Query git working-tree status for the given repository path.",
    ),
    (
        "generate_context",
        "Generate project context (file tree + optional dependency graph) for LLM/agent consumption.",
    ),
    (
        "scaffold_project",
        "Produce a high-level project summary scaffold for the given paths.",
    ),
    (
        "pmat_query_code",
        "Search code functions by natural language query with TDG quality filtering. Returns semantically ranked results with complexity, fault patterns, and call graph context.",
    ),
    (
        "pmat_get_function",
        "Get detailed information about a specific function by its ID. Returns full function metadata including source code, quality metrics, and SATD markers.",
    ),
    (
        "pmat_find_similar",
        "Find functions similar to a reference function. Useful for finding related code, potential duplicates, or implementations of similar patterns.",
    ),
    (
        "pmat_index_stats",
        "Get statistics about the code index including function counts, quality distribution, and index health.",
    ),
];

/// The 19 live tools' `metadata()`, in registration order — the SAME objects
/// `tools/list` serves, so the manifest cannot describe a tool differently
/// from the server (CRUX-09, #1150).
///
/// The previous renderer chose one of three canned inputSchemas by tool NAME:
/// 15 of 19 tools were advertised as taking `paths: string[]` whatever they
/// actually took, six could not be called at all as the shipped file described
/// them, and `pmat_index_stats` was declared `additionalProperties: false` with
/// no properties while the live tool accepts `rebuild` — so a validating
/// client rejected its own valid call. Descriptions were pinned to the handlers
/// in 3.33.0; schemas were left in exactly that state.
///
/// Cheap to construct: `IndexManager::new` only stores the path and opens
/// nothing. The list mirrors the `.tool(...)` registrations in
/// `SimpleUnifiedServer::run()`; `manifest_descriptions_match_handler_metadata`
/// asserts the order against `LIVE_MCP_TOOLS` positionally.
#[must_use]
pub fn live_tool_infos() -> Vec<Option<pmcp::types::ToolInfo>> {
    use crate::mcp::tools::agent_context_tools::IndexManager;
    use crate::mcp_pmcp::agent_context_handlers::{
        PmatFindSimilarHandler, PmatGetFunctionHandler, PmatIndexStatsHandler, PmatQueryCodeHandler,
    };
    use crate::mcp_pmcp::analyze_handlers::{
        AnalyzeBigOTool, AnalyzeComplexityTool, AnalyzeDagTool, AnalyzeDeadCodeTool,
        AnalyzeDeepContextTool, AnalyzeSatdTool, HardcodedPathsTool, ReachabilityTool,
        VacuousTestsTool,
    };
    use crate::mcp_pmcp::context_handlers::{GenerateContextTool, GitTool, ScaffoldProjectTool};
    use crate::mcp_pmcp::pdmt_handler::PdmtTool;
    use crate::mcp_pmcp::quality_handlers::QualityGateTool;
    use crate::mcp_pmcp::quality_proxy_handler::QualityProxyTool;
    use pmcp::ToolHandler;
    use std::path::PathBuf;
    use std::sync::Arc;

    let index_manager = Arc::new(IndexManager::new(PathBuf::from(".")));
    vec![
        AnalyzeComplexityTool.metadata(),
        AnalyzeSatdTool.metadata(),
        AnalyzeDeadCodeTool.metadata(),
        AnalyzeDagTool.metadata(),
        AnalyzeDeepContextTool.metadata(),
        AnalyzeBigOTool.metadata(),
        ReachabilityTool.metadata(),
        HardcodedPathsTool.metadata(),
        VacuousTestsTool.metadata(),
        QualityGateTool.metadata(),
        QualityProxyTool.metadata(),
        PdmtTool::new().metadata(),
        GitTool.metadata(),
        GenerateContextTool.metadata(),
        ScaffoldProjectTool.metadata(),
        PmatQueryCodeHandler::new(index_manager.clone()).metadata(),
        PmatGetFunctionHandler::new(index_manager.clone()).metadata(),
        PmatFindSimilarHandler::new(index_manager.clone()).metadata(),
        PmatIndexStatsHandler::new(index_manager).metadata(),
    ]
}

/// The inputSchema a tool serves over `tools/list`, or `None` when its handler
/// declares no metadata at all — which `render_manifest` refuses to paper over.
fn served_schema(name: &str) -> Option<serde_json::Value> {
    live_tool_infos()
        .into_iter()
        .flatten()
        .find(|info| info.name == name)
        .map(|info| info.input_schema)
}

/// Render `mcp.json` deterministically from `LIVE_MCP_TOOLS` and the handlers'
/// own `metadata()`. Pure: output depends only on the tool list (canonical, sorted keys via serde with the
/// tools map ordered by registration index encoded in a `tools` array-of-
/// objects to preserve order stably).
pub fn render_manifest(version: &str) -> String {
    let tools: Vec<serde_json::Value> = LIVE_MCP_TOOLS
        .iter()
        .map(|(name, desc)| {
            // The served schema, never a canned one. A tool whose handler
            // declares no metadata gets an honest open object rather than a
            // shape invented from its name; `every_live_tool_declares_metadata`
            // keeps that branch unreachable.
            let schema =
                served_schema(name).unwrap_or_else(|| serde_json::json!({"type": "object"}));
            serde_json::json!({
                "name": name,
                "description": desc,
                "inputSchema": schema,
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
            19,
            "the live server advertises 19 tools: 16 after the 4 refactor.* tools were unregistered in EV-0 (#999) for synthesizing violations from a path substring, plus the 3 forensic analyzers #1029 found CLI-only by omission"
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
        // Names and count were the whole of this guard, and a name is not what
        // a client reads to decide whether to call a tool. `quality_proxy`
        // rendered into mcp.json as "Proxy a quality-scored analysis request"
        // for releases while the server advertised a file-WRITING proxy, and
        // this test stayed green throughout because both texts sit under the
        // same name. Descriptions are payload, so compare the payload.
        let rendered = manifest["mcp"]["tools"]
            .as_array()
            .expect("the rendered manifest carries a `tools` array");
        assert_eq!(
            rendered.len(),
            LIVE_MCP_TOOLS.len(),
            "render_manifest emitted a different number of tools than it was given"
        );
        for (tool, &(name, description)) in rendered.iter().zip(LIVE_MCP_TOOLS) {
            assert_eq!(
                tool["name"].as_str(),
                Some(name),
                "render_manifest reordered or renamed a tool on its way into mcp.json"
            );
            assert_eq!(
                tool["description"].as_str(),
                Some(description),
                "{name}: render_manifest dropped or rewrote the description on its way into \
                 mcp.json"
            );
        }
    }

    /// `LIVE_MCP_TOOLS` and each handler's `metadata()` are two answers to
    /// "what is this tool for", and a client sees only one of them at a time:
    /// `tools/list` serves the handler's text, the packaged `mcp.json` serves
    /// this const's. Nothing compared the two, so they diverged in the way that
    /// matters most — the manifest described `git_operation` as "Perform a git
    /// operation for the workflow" and `scaffold_project` as "Scaffold a
    /// project or agent skeleton", naming a mutation for a handler that only
    /// reads (`GitStatusTool`) and one that only summarises
    /// (`ContextSummaryTool`), while `quality_proxy`'s manifest text did not
    /// mention that the tool writes files at all.
    ///
    /// The list below mirrors the `.tool(...)` registrations in
    /// `SimpleUnifiedServer::run()` in order, and is zipped positionally, so a
    /// registration added without a matching const entry fails on the length
    /// assertion before it can reach a release carrying two descriptions.
    #[test]
    fn manifest_descriptions_match_handler_metadata() {
        let advertised = live_tool_infos();
        assert_eq!(
            advertised.len(),
            LIVE_MCP_TOOLS.len(),
            "this list must mirror the `.tool(...)` registrations in run() one for one"
        );

        for (info, &(name, description)) in advertised.into_iter().zip(LIVE_MCP_TOOLS) {
            let (advertised_name, advertised_description) = match info {
                Some(tool_info) => (tool_info.name, tool_info.description),
                // `metadata()` defaults to None, which pmcp turns into an empty
                // description on the wire — the divergence in its worst form.
                None => (String::new(), None),
            };
            assert_eq!(
                advertised_name, name,
                "registration order drifted: LIVE_MCP_TOOLS names `{name}` where the handler in \
                 the same position advertises `{advertised_name}`"
            );
            assert_eq!(
                advertised_description.as_deref(),
                Some(description),
                "{name}: tools/list and mcp.json describe this tool differently. The handler's \
                 metadata() is what a live client sees; LIVE_MCP_TOOLS is what the packaged \
                 mcp.json ships. They must be one text — fix the const, then regenerate the \
                 manifest: cargo test --lib regenerate_mcp_json -- --ignored"
            );
        }
    }

    /// Read `docs/mcp/TOOLS.md`, or `None` when the packaging rules removed it.
    ///
    /// Split out of the test body so the absent-file branch is itself testable
    /// — see `an_absent_tools_doc_skips_instead_of_failing`. Only `NotFound`
    /// yields `None`; any other IO error is a file that IS there and could not
    /// be read, which is a real failure and stays one.
    fn read_tools_doc(path: &std::path::Path) -> Option<String> {
        let read = std::fs::read_to_string(path);
        if matches!(&read, Err(e) if e.kind() == std::io::ErrorKind::NotFound) {
            eprintln!(
                "SKIPPING tools_doc_states_the_live_tool_count: {} does not exist. \
                 `/docs/` is in Cargo.toml's `exclude` list (Cargo.toml:26) and this source \
                 file is not, so a tree with one and not the other is a packaged or vendored \
                 copy rather than a checkout. In a checkout the file is tracked and this test \
                 asserts.",
                path.display()
            );
            return None;
        }
        Some(read.expect("docs/mcp/TOOLS.md is present but could not be read"))
    }

    /// `docs/mcp/TOOLS.md` is the human-readable catalog of this surface, and
    /// it has now drifted twice, both times over-promising. The revision this
    /// test was written against claimed "the surface is 16 tools, not 20" in
    /// one paragraph and "The response lists all 20 tools" in another, while
    /// the server advertised 19; its table of contents reached 20 by counting
    /// the four `refactor.*` tools unregistered in EV-0 (#999) and omitting the
    /// three forensic analyzers added by #1029.
    ///
    /// Prose beside a machine-readable const drifts unless something compares
    /// them, so this compares them — the header count, every "N tools"
    /// sentence, and the per-section counts in BOTH the table of contents and
    /// the section headings. Checking only the header would have passed on the
    /// exact document that shipped two contradictory numbers.
    ///
    /// The path is resolved from `CARGO_MANIFEST_DIR`, not the working
    /// directory: `cargo test` sets CWD to the package root today, but nothing
    /// guarantees it, and a doc test that silently reads nothing is worse than
    /// no test.
    ///
    /// An ABSENT `TOOLS.md` skips, loudly, on stderr — and nothing else does.
    /// That is not the "a skipped leg is a gate that cannot fail" hole: this
    /// source file ships inside the published crate and `docs/mcp/TOOLS.md`
    /// does not, because `/docs/` is in the `exclude` list (`Cargo.toml:26`).
    /// Its absence is therefore a state the packaging rules PRODUCE — unpack
    /// the `.crate` and run `cargo test --lib`, or build from a vendored copy
    /// of the source — not a path that rotted. In a git checkout, the only tree
    /// where TOOLS.md can be edited and so the only tree where there is drift
    /// to catch, the file is tracked and every assertion below runs. A file
    /// that is PRESENT but unreadable still fails hard.
    /// `src/maintenance/ticket_tests.rs:283` skips a `docs/` fixture the same
    /// way for the same packaging reason.
    #[test]
    fn tools_doc_states_the_live_tool_count() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/mcp/TOOLS.md");
        let Some(doc) = read_tools_doc(&path) else {
            return;
        };
        let live = LIVE_MCP_TOOLS.len();

        let header: usize = regex::Regex::new(r"(?m)^\*\*Total Tools\*\*:\s*(\d+)")
            .expect("static regex must compile")
            .captures(&doc)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .expect("docs/mcp/TOOLS.md must carry a `**Total Tools**: N` header line");
        assert_eq!(
            header, live,
            "docs/mcp/TOOLS.md says {header} tools; the live server advertises {live}"
        );

        for capture in regex::Regex::new(r"\b(\d+) tools\b")
            .expect("static regex must compile")
            .captures_iter(&doc)
        {
            let claimed: usize = capture[1]
                .parse()
                .expect("the regex captured decimal digits");
            assert_eq!(
                claimed, live,
                "docs/mcp/TOOLS.md claims `{claimed} tools` somewhere in its prose; \
                 the live server advertises {live}"
            );
        }

        // Both inventories of the same sections. They disagreed once, which is
        // how a table of contents summing to 20 sat above headings that no
        // longer added up to it.
        let toc: Vec<usize> = regex::Regex::new(r"(?m)^\d+\.\s*\[[^\]]*\((\d+)\)\]\(#")
            .expect("static regex must compile")
            .captures_iter(&doc)
            .filter_map(|c| c[1].parse::<usize>().ok())
            .collect();
        let headings: Vec<usize> = regex::Regex::new(r"(?m)^##\s+.*\((\d+)\)\s*$")
            .expect("static regex must compile")
            .captures_iter(&doc)
            .filter_map(|c| c[1].parse::<usize>().ok())
            .collect();
        assert_eq!(
            toc, headings,
            "docs/mcp/TOOLS.md's table of contents and its section headings declare different \
             per-category counts"
        );
        assert!(
            !toc.is_empty(),
            "docs/mcp/TOOLS.md declares no per-category counts — the table-of-contents check \
             would be vacuous"
        );
        assert_eq!(
            toc.iter().sum::<usize>(),
            live,
            "docs/mcp/TOOLS.md's per-category counts {toc:?} sum to {} but the live server \
             advertises {live} tools",
            toc.iter().sum::<usize>()
        );
    }

    /// The skip above has to be reachable without unpacking a `.crate`.
    ///
    /// The previous body read the doc with `.expect("docs/mcp/TOOLS.md is a
    /// tracked file and must be readable")`, which panics on `NotFound` — the
    /// exact call `cargo test --lib` makes inside an unpacked package, where
    /// `/docs/` cannot be present because `Cargo.toml:26` excludes it. This
    /// asserts the branch instead of trusting the comment describing it.
    #[test]
    fn an_absent_tools_doc_skips_instead_of_failing() {
        let absent = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/mcp/TOOLS.md.absent-on-purpose");
        assert!(
            !absent.exists(),
            "{} must not exist, or the assertion below proves nothing",
            absent.display()
        );
        assert_eq!(
            read_tools_doc(&absent),
            None,
            "an absent TOOLS.md must skip, not fail: /docs/ is excluded from the published \
             crate (Cargo.toml:26), so this is what `cargo test --lib` sees in an unpacked \
             .crate or a vendored copy of the source"
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

    /// CRUX-09 (#1150): the shipped inputSchema must BE the served one, for
    /// every tool, not a shape chosen by name.
    #[test]
    fn manifest_schemas_match_handler_metadata() {
        let rendered: serde_json::Value =
            serde_json::from_str(&render_manifest("9.9.9")).expect("manifest is JSON");
        let shipped: std::collections::BTreeMap<String, serde_json::Value> = rendered["mcp"]
            ["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .map(|t| {
                (
                    t["name"].as_str().expect("name").to_string(),
                    t["inputSchema"].clone(),
                )
            })
            .collect();
        let served: Vec<pmcp::types::ToolInfo> = live_tool_infos().into_iter().flatten().collect();
        assert_eq!(
            served.len(),
            LIVE_MCP_TOOLS.len(),
            "every live tool declares metadata"
        );
        for info in served {
            assert_eq!(
                shipped.get(&info.name),
                Some(&info.input_schema),
                "{}: the packaged mcp.json and tools/list advertise different inputSchemas",
                info.name
            );
        }
    }

    /// The `unwrap_or_else` open-object fallback in `render_manifest` must stay
    /// unreachable: a handler with no metadata is a defect, not a tool with an
    /// open schema.
    #[test]
    fn every_live_tool_declares_metadata() {
        for (i, info) in live_tool_infos().iter().enumerate() {
            assert!(
                info.is_some(),
                "handler at registration index {i} ({}) declares no metadata",
                LIVE_MCP_TOOLS[i].0
            );
        }
    }

    /// Fixing the renderer without regenerating the file leaves the defect
    /// wholly intact in the tarball, so the committed file is pinned to the
    /// renderer byte-for-byte. Version-only churn is the one tolerated diff:
    /// `check_macs_artifacts.rs` ignores it for the same reason.
    #[test]
    fn committed_mcp_json_is_pinned_to_the_renderer() {
        let committed = include_str!("../../mcp.json");
        let version = serde_json::from_str::<serde_json::Value>(committed)
            .ok()
            .and_then(|v| v["version"].as_str().map(str::to_string))
            .expect("committed mcp.json carries a version");
        assert_eq!(
            committed,
            render_manifest(&version),
            "mcp.json is not what the renderer produces — regenerate it: \
             cargo test --lib regenerate_mcp_json -- --ignored"
        );
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
