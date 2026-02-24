/// Analyzes Scala source code for complexity and structure
pub struct ScalaAnalysisTool {
    #[allow(dead_code)]
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl ScalaAnalysisTool {
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl McpTool for ScalaAnalysisTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "analyze_scala".to_string(),
            description:
                "Analyzes Scala source code for complexity, structure, and quality metrics."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to Scala file or directory to analyze"
                    },
                    "max_depth": {
                        "type": "number",
                        "description": "Maximum depth for recursive directory analysis",
                        "default": 3
                    },
                    "include_metrics": {
                        "type": "boolean",
                        "description": "Include detailed complexity metrics",
                        "default": true
                    },
                    "include_ast": {
                        "type": "boolean",
                        "description": "Include AST items in result",
                        "default": false
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        // Extract parameters
        let path_str = params["path"].as_str().ok_or_else(|| McpError {
            code: crate::mcp_integration::error_codes::INVALID_PARAMS,
            message: "Missing path parameter".to_string(),
            data: None,
        })?;

        let path = PathBuf::from(path_str);
        let max_depth = params["max_depth"].as_u64().unwrap_or(3);
        let include_metrics = params["include_metrics"].as_bool().unwrap_or(true);
        let include_ast = params["include_ast"].as_bool().unwrap_or(false);

        // Validate path
        if PathValidator::ensure_exists(&path).is_err() {
            return Err(McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: format!("Path does not exist: {}", path.display()),
                data: Some(json!({
                    "path": path.display().to_string(),
                    "suggestion": "Please provide a valid file or directory path"
                })),
            });
        }

        // Analyze the file or directory
        info!("Analyzing Scala at path: {}", path.display());
        let result = if path.is_dir() {
            analyze_scala_directory(&path, max_depth, include_metrics, include_ast).await
        } else if path
            .extension()
            .is_some_and(|ext| ext == "scala" || ext == "sc")
        {
            analyze_scala_file(&path, include_metrics, include_ast).await
        } else {
            return Err(McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: format!("Not a Scala file: {}", path.display()),
                data: Some(json!({
                    "path": path.display().to_string(),
                    "suggestion": "File should have a .scala or .sc extension"
                })),
            });
        };

        match result {
            Ok(analysis) => Ok(analysis),
            Err(e) => Err(McpError {
                code: crate::mcp_integration::error_codes::INTERNAL_ERROR,
                message: format!("Analysis failed: {}", e),
                data: None,
            }),
        }
    }
}
