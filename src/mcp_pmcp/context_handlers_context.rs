// Context tool handlers: ContextGenerateTool, ContextAnalyzeTool, ContextSummaryTool implementations.
// Included from context_handlers.rs -- do NOT add `use` imports or `#!` inner attributes.

// Context Generate Tool

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

impl ContextGenerateTool {
    #[must_use]
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

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let context =
            tool_functions::generate_context(&paths, params.max_depth, params.include_dependencies)
                .await
                .map_err(|e| Error::internal(format!("Context generation failed: {e}")))?;

        // Format the output based on requested format
        match params.format.as_deref() {
            Some("markdown") => Ok(json!({
                "context": context,
                "markdown": "Context in markdown format (not implemented)"
            })),
            Some("xml") => Ok(json!({
                "context": context,
                "xml": "Context in XML format (not implemented)"
            })),
            Some("json") | None => Ok(context),
            Some(format) => Err(Error::validation(format!("Unsupported format: {format}"))),
        }
    }
}

// Context Analyze Tool

#[derive(Debug, Deserialize)]
struct ContextAnalyzeArgs {
    paths: Vec<String>,
    #[serde(default)]
    analysis_types: Vec<String>,
}

impl ContextAnalyzeTool {
    #[must_use]
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

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let analyses = tool_functions::analyze_context(&paths, &params.analysis_types)
            .await
            .map_err(|e| Error::internal(format!("Context analysis failed: {e}")))?;

        Ok(analyses)
    }
}

// Context Summary Tool

#[derive(Debug, Deserialize)]
struct ContextSummaryArgs {
    paths: Vec<String>,
    #[serde(default)]
    level: Option<String>,
}

impl ContextSummaryTool {
    #[must_use]
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

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let summary = tool_functions::context_summary(&paths, params.level.as_deref())
            .await
            .map_err(|e| Error::internal(format!("Context summary failed: {e}")))?;

        Ok(summary)
    }
}
