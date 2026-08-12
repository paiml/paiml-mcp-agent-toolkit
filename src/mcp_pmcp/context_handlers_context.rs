// Context tool handlers: ContextGenerateTool, ContextAnalyzeTool, ContextSummaryTool implementations.
// Included from context_handlers.rs -- do NOT add `use` imports or `#!` inner attributes.

// Context Generate Tool

/// MCP args for context.generate: paths to analyze, optional output format, max_depth, and dependency inclusion.
#[derive(Debug, Deserialize)]
struct ContextGenerateArgs {
    paths: Vec<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    max_depth: Option<usize>,
    #[serde(default)]
    include_dependencies: bool,
}

/// Accept only the formats `generate_context` can actually produce.
///
/// `format: "markdown"` and `format: "xml"` used to come back as the JSON
/// context plus the literal string "Context in markdown format (not
/// implemented)", with status completed and isError=false — a client could not
/// tell the stub from a rendered document, and the inputSchema advertised both.
/// This tool emits JSON; anything else is now a validation error.
fn validate_context_format(format: Option<&str>) -> Result<()> {
    match format {
        None | Some("json") => Ok(()),
        Some(other) => Err(Error::validation(format!(
            "Unsupported format: {other} (generate_context produces \"json\" only)"
        ))),
    }
}

impl ContextGenerateTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextGenerateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ContextGenerateTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling context.generate with args: {}", args);

        let params: ContextGenerateArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        // Checked before any analysis runs: an unsupported format is an
        // argument error, not something to discover after the work is done.
        validate_context_format(params.format.as_deref())?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

        let context =
            tool_functions::generate_context(&paths, params.max_depth, params.include_dependencies)
                .await
                .map_err(|e| Error::internal(format!("Context generation failed: {e}")))?;

        Ok(context)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "format":               { "type": "string", "enum": ["json"], "description": "Output format" },
            "max_depth":            { "type": "integer", "description": "Max directory-tree depth to include" },
            "include_dependencies": { "type": "boolean", "description": "Include dependency graph" }
        });
        // Registered as `generate_context` in server.rs.
        Some(build_tool_info(
            "generate_context",
            "Generate project context (file tree + optional dependency graph) for LLM/agent consumption.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

// Context Analyze Tool

/// MCP args for context.analyze: paths to inspect and list of analysis_types to run.
#[derive(Debug, Deserialize)]
struct ContextAnalyzeArgs {
    paths: Vec<String>,
    #[serde(default)]
    analysis_types: Vec<String>,
}

impl ContextAnalyzeTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextAnalyzeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ContextAnalyzeTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling context.analyze with args: {}", args);

        let params: ContextAnalyzeArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

        let analyses = tool_functions::analyze_context(&paths, &params.analysis_types)
            .await
            .map_err(|e| Error::internal(format!("Context analysis failed: {e}")))?;

        Ok(analyses)
    }
}

// Context Summary Tool

/// MCP args for context.summary: paths to summarize with optional detail level.
#[derive(Debug, Deserialize)]
struct ContextSummaryArgs {
    paths: Vec<String>,
    #[serde(default)]
    level: Option<String>,
}

impl ContextSummaryTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextSummaryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ContextSummaryTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling context.summary with args: {}", args);

        let params: ContextSummaryArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

        // A value outside the `level` enum this tool's own schema advertises is
        // a bad ARGUMENT. Dispatching first and wrapping the refusal in
        // `Error::internal` reported it as `-32603 Internal error`, so
        // `level:"deep"` was indistinguishable from a server crash. Same rule,
        // same source of truth as `context_summary` itself — only the JSON-RPC
        // code changes.
        tool_functions::resolve_summary_level(params.level.as_deref())
            .map_err(|e| Error::validation(e.to_string()))?;

        let summary = tool_functions::context_summary(&paths, params.level.as_deref())
            .await
            .map_err(|e| Error::internal(format!("Context summary failed: {e}")))?;

        Ok(summary)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "level": { "type": "string", "enum": ["brief", "normal", "detailed"], "description": "Summary detail level" }
        });
        // Registered as `scaffold_project` in server.rs (historical alias).
        Some(build_tool_info(
            "scaffold_project",
            "Produce a high-level project summary scaffold for the given paths.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

#[cfg(test)]
mod context_format_tests {
    //! `generate_context` must not advertise, nor silently stub, a format it
    //! cannot render.
    use super::*;

    #[test]
    fn only_json_is_accepted() {
        assert!(validate_context_format(None).is_ok());
        assert!(validate_context_format(Some("json")).is_ok());
        for stubbed in ["markdown", "xml"] {
            let err = validate_context_format(Some(stubbed))
                .expect_err("a format the tool cannot render must be an error, not a stub string");
            assert!(
                err.to_string().contains("Unsupported format"),
                "got: {err}"
            );
        }
    }

    #[test]
    fn input_schema_advertises_only_renderable_formats() {
        use pmcp::ToolHandler;

        let info = ContextGenerateTool::new()
            .metadata()
            .expect("generate_context advertises metadata");
        let formats = info.input_schema["properties"]["format"]["enum"].clone();
        assert_eq!(
            formats,
            json!(["json"]),
            "the schema must not offer formats the handler rejects"
        );
    }
}
