// ToolHandler implementations for quality gate tools
// This file is included by quality_handlers.rs via include!()
// NO `use` imports or `#!` inner attributes allowed

#[async_trait]
impl ToolHandler for QualityGateTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate with args: {}", args);

        let params: QualityGateArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        // If a specific file is requested, check only that file
        if let Some(file_path) = params.file {
            let file_path = PathBuf::from(file_path);
            let result = tool_functions::check_quality_gate_file(file_path.as_ref(), params.strict)
                .await
                .map_err(|e| Error::internal(format!("Quality gate check failed: {e}")))?;

            return Ok(result);
        }

        // Otherwise check all paths
        let results = tool_functions::check_quality_gates(&paths, params.strict)
            .await
            .map_err(|e| Error::internal(format!("Quality gate check failed: {e}")))?;

        Ok(results)
    }
}

#[async_trait]
impl ToolHandler for QualityGateSummaryTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.summary with args: {}", args);

        let params: QualityGateSummaryArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

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

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();
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
        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let comparison = tool_functions::quality_gate_compare(baseline_path.as_ref(), &paths)
            .await
            .map_err(|e| Error::internal(format!("Failed to compare with baseline: {e}")))?;

        Ok(comparison)
    }
}
