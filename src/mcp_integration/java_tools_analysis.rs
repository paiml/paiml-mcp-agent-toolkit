#[async_trait]
impl McpTool for JavaAnalysisTool {
    fn metadata(&self) -> ToolMetadata {
        debug_assert!(true, "contract: metadata");
        ToolMetadata {
            name: "analyze_java".to_string(),
            description:
                "Analyzes Java source code for complexity, structure, and quality metrics."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to Java file or directory to analyze"
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
        debug_assert!(true, "contract: execute");
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
        info!("Analyzing Java at path: {}", path.display());
        let result = if path.is_dir() {
            analyze_java_directory(&path, max_depth, include_metrics, include_ast).await
        } else if path.extension().is_some_and(|ext| ext == "java") {
            analyze_java_file(&path, include_metrics, include_ast).await
        } else {
            return Err(McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: format!("Not a Java file: {}", path.display()),
                data: Some(json!({
                    "path": path.display().to_string(),
                    "suggestion": "File should have a .java extension"
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

/// Analyzes a single Java file
async fn analyze_java_file(
    path: &std::path::Path,
    include_metrics: bool,
    include_ast: bool,
) -> Result<Value> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Read the file content
    let content = fs::read_to_string(path).await?;

    // Create visitor and analyze
    let visitor = JavaAstVisitor::new(path);
    match visitor.analyze_java_source(&content) {
        Ok(items) => {
            // Calculate metrics
            let class_count = items
                .iter()
                .filter(|item| extract_kind(item) == "class" || extract_kind(item) == "struct")
                .count();

            let interface_count = items
                .iter()
                .filter(|item| extract_kind(item) == "interface" || extract_kind(item) == "trait")
                .count();

            let method_count = items
                .iter()
                .filter(|item| extract_kind(item) == "method" || extract_kind(item) == "function")
                .count();

            let package_name = items
                .iter()
                .find(|item| extract_kind(item) == "package" || extract_kind(item) == "module")
                .map(extract_name)
                .unwrap_or_else(|| "default".to_string());

            // Build response
            let mut result = json!({
                "status": "completed",
                "path": path.display().to_string(),
                "language": "java",
                "summary": {
                    "class_count": class_count,
                    "interface_count": interface_count,
                    "method_count": method_count,
                    "package": package_name,
                    "total_items": items.len()
                }
            });

            // Add metrics if requested
            if include_metrics {
                // Calculate complexity metrics
                let total_complexity: u32 = items.iter().map(extract_complexity).sum();

                let max_complexity = items.iter().map(extract_complexity).max().unwrap_or(0);

                let avg_complexity = if method_count > 0 {
                    (total_complexity as f64) / (method_count as f64)
                } else {
                    0.0
                };

                // Add metrics to result
                result["metrics"] = json!({
                    "total_complexity": total_complexity,
                    "max_complexity": max_complexity,
                    "avg_complexity": avg_complexity,
                    "loc": content.lines().count()
                });
            }

            // Add AST items if requested
            if include_ast {
                result["items"] = serde_json::to_value(&items)?;
            }

            Ok(result)
        }
        Err(e) => {
            warn!("Failed to parse Java file {}: {}", path.display(), e);
            Ok(json!({
                "status": "error",
                "path": path.display().to_string(),
                "language": "java",
                "error": e
            }))
        }
    }
}

/// Analyzes a directory of Java files recursively
async fn analyze_java_directory(
    path: &std::path::Path,
    max_depth: u64,
    include_metrics: bool,
    include_ast: bool,
) -> Result<Value> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Use walkdir to find all Java files
    let java_files = find_java_files(path, max_depth as usize)?;

    if java_files.is_empty() {
        return Ok(json!({
            "status": "completed",
            "path": path.display().to_string(),
            "language": "java",
            "summary": {
                "file_count": 0,
                "message": "No Java files found"
            }
        }));
    }

    // Analyze each file
    let mut file_results = Vec::new();
    let mut total_classes = 0;
    let mut total_interfaces = 0;
    let mut total_methods = 0;
    let mut total_complexity = 0;
    let mut max_complexity = 0;
    let mut total_loc = 0;

    for file_path in &java_files {
        match analyze_java_file(file_path, include_metrics, false).await {
            Ok(result) => {
                accumulate_summary_counts(&result, &mut total_classes, &mut total_interfaces, &mut total_methods);
                if include_metrics {
                    accumulate_metrics(&result, &mut total_complexity, &mut max_complexity, &mut total_loc);
                }
                file_results.push(result);
            }
            Err(e) => {
                warn!("Error analyzing Java file {}: {}", file_path.display(), e);
            }
        }
    }

    // Calculate average complexity
    let avg_complexity = if total_methods > 0 {
        (total_complexity as f64) / (total_methods as f64)
    } else {
        0.0
    };

    // Build aggregate response
    let mut result = json!({
        "status": "completed",
        "path": path.display().to_string(),
        "language": "java",
        "summary": {
            "file_count": java_files.len(),
            "class_count": total_classes,
            "interface_count": total_interfaces,
            "method_count": total_methods,
        }
    });

    // Add metrics if requested
    if include_metrics {
        result["metrics"] = json!({
            "total_complexity": total_complexity,
            "max_complexity": max_complexity,
            "avg_complexity": avg_complexity,
            "total_loc": total_loc
        });
    }

    // Add file-level results if include_ast was requested
    if include_ast {
        result["files"] = serde_json::to_value(&file_results)?;
    }

    Ok(result)
}

/// Helper function to find all Java files in a directory
fn find_java_files(path: &std::path::Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut java_files = Vec::new();

    let walker = walkdir::WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(Result::ok);

    for entry in walker {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "java") {
            java_files.push(path.to_path_buf());
        }
    }

    Ok(java_files)
}

fn accumulate_summary_counts(
    result: &Value,
    total_classes: &mut u64,
    total_interfaces: &mut u64,
    total_methods: &mut u64,
) {
    debug_assert!(true, "contract: accumulate_summary_counts");
    if let Some(summary) = result["summary"].as_object() {
        if let Some(c) = summary["class_count"].as_u64() {
            *total_classes += c;
        }
        if let Some(i) = summary["interface_count"].as_u64() {
            *total_interfaces += i;
        }
        if let Some(m) = summary["method_count"].as_u64() {
            *total_methods += m;
        }
    }
}

fn accumulate_metrics(
    result: &Value,
    total_complexity: &mut u64,
    max_complexity: &mut u64,
    total_loc: &mut u64,
) {
    debug_assert!(true, "contract: accumulate_metrics");
    if let Some(metrics) = result["metrics"].as_object() {
        if let Some(c) = metrics["total_complexity"].as_u64() {
            *total_complexity += c;
        }
        if let Some(c) = metrics["max_complexity"].as_u64() {
            *max_complexity = std::cmp::max(*max_complexity, c);
        }
        if let Some(l) = metrics["loc"].as_u64() {
            *total_loc += l;
        }
    }
}
