// The three analyzers #1029 named: reachability, hardcoded paths, vacuous
// tests. All three shipped in 3.32.0 and were CLI-only by omission — the MCP
// tool list was maintained beside `AnalyzeCommands` rather than derived from
// it, so they appeared in `pmat analyze --help` the moment the clap variant
// existed and on MCP never.
//
// They are exposed rather than declared CLI-only because all three are
// read-only, bounded by `git ls-files` / `cargo metadata`, and answer the
// question an agent asks most often about an unfamiliar tree: what in here is
// not real? Their MCP status now lives in `cli::analyze_mcp_exposure`, whose
// total match makes the next such omission a compile error.
//
// Each payload mirrors the CLI's `--format json` field-for-field. Two surfaces
// that answer the same question with differently-named fields are a
// CLI-vs-MCP contradiction of the kind the round-5 dogfood catalogued 24 of;
// the point of registering these tools is that the two agree.
//
// Every handler refuses an empty scan rather than reporting a clean one, the
// same way the CLI routes do: an unmeasured run must not be indistinguishable
// from a measured, passing one.

/// MCP args for `analyze_reachability`: one project root.
#[derive(Debug, Deserialize)]
struct ReachabilityArgs {
    project_path: String,
}

/// Tool handler reporting tracked `.rs` files no compilation unit reaches.
///
/// rustc emits no diagnostic for a `.rs` file that no `mod`, `#[path]` or
/// `include!` reaches, so an orphaned module compiles to nothing and its tests
/// report `0 passed; ok`.
pub struct ReachabilityTool;

impl ReachabilityTool {
    /// Creates a new reachability analysis tool handler.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReachabilityTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a reachability report as the CLI's `--format json` payload.
///
/// Extracted so `handle` stays straight-line: the shape is the contract with
/// the CLI, and it is easier to compare two field lists than to read one out of
/// a control-flow graph.
fn reachability_payload(report: &crate::services::reachability::Report) -> Value {
    json!({
        "reachable": report.reachable,
        "roots": report.roots,
        "orphan_count": report.orphans.len(),
        "orphan_lines": report.orphan_lines(),
        "orphan_tests": report.orphan_tests(),
        "unresolved_mods": report.unresolved.len(),
        "summary": report.summary(),
        "orphans": report.orphans.iter().map(|o| json!({
            "file": o.path, "lines": o.lines, "tests": o.tests
        })).collect::<Vec<_>>(),
    })
}

#[async_trait]
impl ToolHandler for ReachabilityTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.reachability with args: {}", args);

        let params: ReachabilityArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;
        let root = crate::mcp_pmcp::tool_schemas::resolve_existing_root(&params.project_path)?;

        let (roots, tracked) = crate::services::reachability::discover(&root)
            .map_err(|e| Error::internal(format!("Reachability discovery failed: {e}")))?;
        if roots.is_empty() {
            // Refuse rather than answer "0 orphans" over a tree we never
            // walked, exactly as `pmat analyze reachability` does. An
            // unmeasured run must not look like a clean one.
            return Err(Error::validation(format!(
                "no cargo targets found under {} — `cargo metadata --no-deps` returned none, \
                 so reachability could not be measured (this is not a clean result)",
                root.display()
            )));
        }

        let report = crate::services::reachability::analyze(&root, &roots, &tracked);
        Ok(reachability_payload(&report))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(build_tool_info(
            "analyze_reachability",
            "Report tracked .rs files that no compilation unit reaches — orphaned modules \
             that compile to nothing and whose tests never run.",
            crate::mcp_pmcp::tool_schemas::project_root_schema(json!({})),
        ))
    }
}

/// MCP args for `analyze_hardcoded_paths`: one project root.
#[derive(Debug, Deserialize)]
struct HardcodedPathsArgs {
    project_path: String,
}

/// Tool handler finding machine-specific absolute paths baked into source.
///
/// Flags a path only when it names a specific user, nix store hash or build
/// root; `/usr/bin/env` and `/home/$USER` are portable and are not findings.
pub struct HardcodedPathsTool;

impl HardcodedPathsTool {
    /// Creates a new hardcoded-path analysis tool handler.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for HardcodedPathsTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a hardcoded-paths report as the CLI's `--format json` payload.
fn hardcoded_paths_payload(report: &crate::services::hardcoded_paths::Report) -> Value {
    json!({
        "summary": report.summary(),
        "files_scanned": report.files_scanned,
        "literals_scanned": report.literals_scanned,
        "finding_count": report.findings.len(),
        "shipped_count": report.shipped(),
        "by_kind": report.by_kind(),
        "skipped": report.skipped,
        "findings": report.findings,
    })
}

#[async_trait]
impl ToolHandler for HardcodedPathsTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.hardcoded-paths with args: {}", args);

        let params: HardcodedPathsArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;
        let root = crate::mcp_pmcp::tool_schemas::resolve_existing_root(&params.project_path)?;

        let files = crate::services::hardcoded_paths::tracked_files(&root)
            .map_err(|e| Error::internal(format!("Hardcoded-path scan failed: {e}")))?;
        if files.is_empty() {
            // Refuse rather than print "0 findings" over a tree we never
            // opened (#1015) — that confusion is the exact defect this
            // analyzer hunts.
            return Err(Error::validation(format!(
                "no scannable tracked files under {} — `git ls-files` returned none, so no \
                 path scan was performed (this is not a clean result)",
                root.display()
            )));
        }

        let report = crate::services::hardcoded_paths::analyze(&root, &files);
        Ok(hardcoded_paths_payload(&report))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(build_tool_info(
            "analyze_hardcoded_paths",
            "Find machine-specific absolute paths baked into source (a user's home, a nix \
             store hash, a build root) — correct where they were written, inert everywhere else.",
            crate::mcp_pmcp::tool_schemas::project_root_schema(json!({})),
        ))
    }
}

/// MCP args for `analyze_vacuous_tests`: one project root.
#[derive(Debug, Deserialize)]
struct VacuousTestsArgs {
    project_path: String,
}

/// Tool handler finding `#[test]` functions that cannot fail.
pub struct VacuousTestsTool;

impl VacuousTestsTool {
    /// Creates a new vacuous-test analysis tool handler.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for VacuousTestsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for VacuousTestsTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.vacuous-tests with args: {}", args);

        let params: VacuousTestsArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;
        let root = crate::mcp_pmcp::tool_schemas::resolve_existing_root(&params.project_path)?;

        let files = crate::services::vacuous_tests::tracked_rust_files(&root)
            .map_err(|e| Error::internal(format!("Vacuous-test scan failed: {e}")))?;
        if files.is_empty() {
            return Err(Error::validation(format!(
                "no tracked .rs files under {} — `git ls-files` returned none, so no test \
                 was examined (this is not a clean result)",
                root.display()
            )));
        }

        let report = crate::services::vacuous_tests::analyze(&root, &files);
        if report.tests_examined == 0 {
            // Zero vacuous tests out of zero tests is not a pass. Say so —
            // the CLI route does.
            return Err(Error::validation(format!(
                "no #[test] functions found in {} parsed file(s) under {} — nothing was \
                 judged, so this is not a clean result",
                report.files_parsed,
                root.display()
            )));
        }

        serde_json::to_value(&report)
            .map_err(|e| Error::internal(format!("Vacuous-test report is not serializable: {e}")))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(build_tool_info(
            "analyze_vacuous_tests",
            "Find #[test] functions that cannot fail — no assertion, an assertion over \
             constants, or a body that silently returns when a fixture is missing.",
            crate::mcp_pmcp::tool_schemas::project_root_schema(json!({})),
        ))
    }
}
