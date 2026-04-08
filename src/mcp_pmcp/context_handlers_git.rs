// Git tool handlers: GitCloneTool and GitStatusTool implementations.
// Included from context_handlers.rs -- do NOT add `use` imports or `#!` inner attributes.

// Git Clone Tool

/// MCP args for git.clone: repository URL with optional target_dir, branch, and shallow depth.
#[derive(Debug, Deserialize)]
struct GitCloneArgs {
    url: String,
    #[serde(default)]
    target_dir: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    depth: Option<u32>,
}

impl GitCloneTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitCloneTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for GitCloneTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling git.clone with args: {}", args);

        let params: GitCloneArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let target_dir = params.target_dir.map(PathBuf::from);

        let result = tool_functions::git_clone(
            &params.url,
            target_dir.as_deref(),
            params.branch.as_deref(),
            params.depth,
        )
        .await
        .map_err(|e| Error::internal(format!("Git clone failed: {e}")))?;

        Ok(json!({
            "status": "success",
            "path": result.display().to_string(),
            "message": format!("Successfully cloned repository to {}", result.display())
        }))
    }
}

// Git Status Tool

/// MCP args for git.status: path to the repository to query for working-tree status.
#[derive(Debug, Deserialize)]
struct GitStatusArgs {
    path: String,
}

impl GitStatusTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitStatusTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for GitStatusTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling git.status with args: {}", args);

        let params: GitStatusArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let path = PathBuf::from(params.path);

        let status = tool_functions::git_status(path.as_ref())
            .await
            .map_err(|e| Error::internal(format!("Failed to get git status: {e}")))?;

        Ok(status)
    }
}
