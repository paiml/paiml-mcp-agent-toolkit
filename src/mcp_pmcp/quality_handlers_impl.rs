// ToolHandler implementations for quality gate tools
// This file is included by quality_handlers.rs via include!()
// NO `use` imports or `#!` inner attributes allowed

#[async_trait]
impl ToolHandler for QualityGateTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate with args: {}", args);

        let params: QualityGateArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

        // If a specific file is requested, check only that file
        if let Some(file_path) = params.file {
            let file_path = PathBuf::from(file_path);
            // A named file that is missing, or that does not parse, is a bad
            // argument. `resolve_existing_paths` one line up already answers
            // that class with -32602, and one tool must not report the same
            // refusal as -32603 "Internal error", which tells an agent the
            // server is at fault and the call is worth retrying.
            if !file_path.exists() {
                return Err(Error::validation(format!(
                    "file not found: {}. Gating a nonexistent file would report zero \
                     violations, which is indistinguishable from a clean result.",
                    file_path.display()
                )));
            }
            crate::tdg::ensure_parseable(&file_path)
                .map_err(|e| Error::validation(e.to_string()))?;

            let result = tool_functions::check_quality_gate_file(file_path.as_ref(), params.strict)
                .await
                .map_err(|e| Error::internal(format!("Quality gate check failed: {e}")))?;

            return Ok(result);
        }

        // A single FILE named in `paths` takes the same refusal path as `file`
        // above — `check_quality_gates` calls `analyze_file` directly and
        // propagates the "did not parse" bail — but it reached the wire as
        // `-32603 Internal error` instead of the `-32602` the `file` argument
        // one line up already answers with, for the identical mistake. One tool
        // must not report the same refusal under two JSON-RPC codes depending
        // on which of its two arguments carried the path. (A DIRECTORY is
        // untouched: an unparseable file inside one is disclosed as a
        // `not_graded` violation, and that structured answer is the better one.)
        if let Some(first) = paths.first().filter(|p| p.is_file()) {
            crate::tdg::ensure_parseable(first).map_err(|e| Error::validation(e.to_string()))?;
        }

        // Otherwise check all paths
        let results = tool_functions::check_quality_gates(&paths, params.strict)
            .await
            .map_err(|e| Error::internal(format!("Quality gate check failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "strict": { "type": "boolean", "description": "Fail the gate on any violation (no tolerance)" },
            "file":   { "type": "string",  "description": "Check only this single file instead of the paths list" }
        });
        Some(build_tool_info(
            "quality_gate",
            "Run comprehensive quality-gate checks (complexity, SATD, dead code, lint, docs, etc.) against the given paths.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

#[async_trait]
impl ToolHandler for QualityGateSummaryTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.summary with args: {}", args);

        let params: QualityGateSummaryArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

        let summary = tool_functions::quality_gate_summary(&paths)
            .await
            .map_err(|e| Error::internal(format!("Quality gate summary failed: {e}")))?;

        // Return the summary in requested format
        match params.format.as_deref() {
            Some("markdown") => Ok(json!({
                "summary": summary,
                "markdown": "Quality gate summary in markdown format (not implemented)"
            })),
            Some("json") | None => Ok(summary),
            Some(format) => Err(Error::validation(format!("Unsupported format: {format}"))),
        }
    }
}

#[async_trait]
impl ToolHandler for QualityGateBaselineTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.baseline with args: {}", args);

        let params: QualityGateBaselineArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;
        let output_path = params.output.map(PathBuf::from);

        let baseline = tool_functions::quality_gate_baseline(&paths, output_path.as_deref())
            .await
            .map_err(|e| Error::internal(format!("Failed to create baseline: {e}")))?;

        Ok(baseline)
    }
}

#[async_trait]
impl ToolHandler for QualityGateCompareTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.compare with args: {}", args);

        let params: QualityGateCompareArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let baseline_path = PathBuf::from(params.baseline);
        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

        let comparison = tool_functions::quality_gate_compare(baseline_path.as_ref(), &paths)
            .await
            .map_err(|e| Error::internal(format!("Failed to compare with baseline: {e}")))?;

        Ok(comparison)
    }
}
