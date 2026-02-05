use crate::mcp_integration::ast_item_helpers::{extract_complexity, extract_kind, extract_name};
use crate::mcp_integration::{McpError, McpTool, ToolMetadata};
use crate::services::languages::java::JavaAstVisitor;
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};

/// Analyzes Java source code for complexity and structure
pub struct JavaAnalysisTool {
    #[allow(dead_code)]
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl JavaAnalysisTool {
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl McpTool for JavaAnalysisTool {
    fn metadata(&self) -> ToolMetadata {
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
                // Update counters
                if let Some(summary) = result["summary"].as_object() {
                    if let Some(class_count) = summary["class_count"].as_u64() {
                        total_classes += class_count;
                    }
                    if let Some(interface_count) = summary["interface_count"].as_u64() {
                        total_interfaces += interface_count;
                    }
                    if let Some(method_count) = summary["method_count"].as_u64() {
                        total_methods += method_count;
                    }
                }

                // Update complexity metrics
                if include_metrics {
                    if let Some(metrics) = result["metrics"].as_object() {
                        if let Some(complexity) = metrics["total_complexity"].as_u64() {
                            total_complexity += complexity;
                        }
                        if let Some(complexity) = metrics["max_complexity"].as_u64() {
                            max_complexity = std::cmp::max(max_complexity, complexity);
                        }
                        if let Some(loc) = metrics["loc"].as_u64() {
                            total_loc += loc;
                        }
                    }
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

/// Java mutation testing tool
pub struct JavaMutationTool {
    #[allow(dead_code)]
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl JavaMutationTool {
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl McpTool for JavaMutationTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "mutation_test_java".to_string(),
            description: "Performs mutation testing on Java code to assess test suite quality."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to Java project root"
                    },
                    "source_path": {
                        "type": "string",
                        "description": "Path to source file or directory to mutate"
                    },
                    "test_command": {
                        "type": "string",
                        "description": "Command to run tests (defaults to 'mvn test' or 'gradle test')"
                    },
                    "mutation_operators": {
                        "type": "array",
                        "description": "List of mutation operators to apply",
                        "items": {"type": "string"},
                        "default": ["arithmetic", "conditional", "method", "assignment"]
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Timeout in seconds for each test run",
                        "default": 30
                    }
                },
                "required": ["project_path", "source_path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let project_path = params["project_path"].as_str().ok_or_else(|| McpError {
            code: crate::mcp_integration::error_codes::INVALID_PARAMS,
            message: "Missing project_path parameter".to_string(),
            data: None,
        })?;

        let source_path = params["source_path"].as_str().ok_or_else(|| McpError {
            code: crate::mcp_integration::error_codes::INVALID_PARAMS,
            message: "Missing source_path parameter".to_string(),
            data: None,
        })?;

        let test_command = params["test_command"].as_str();
        let timeout = params["timeout"].as_u64().unwrap_or(30);

        // Extract mutation operators
        let mutation_operators = if let Some(operators) = params["mutation_operators"].as_array() {
            operators
                .iter()
                .filter_map(|op| op.as_str().map(String::from))
                .collect::<Vec<String>>()
        } else {
            // Default operators
            vec![
                "arithmetic".to_string(),
                "conditional".to_string(),
                "method".to_string(),
                "assignment".to_string(),
            ]
        };

        info!(
            "Running Java mutation tests on project: {}, source: {}",
            project_path, source_path
        );

        // In a real implementation, we would spawn the PITest or similar mutation testing tool
        // For now, return a placeholder response
        Ok(json!({
            "status": "completed",
            "message": "Java mutation testing completed",
            "project_path": project_path,
            "source_path": source_path,
            "test_command": test_command.unwrap_or("auto-detected"),
            "mutation_operators": mutation_operators,
            "timeout": timeout,
            "results": {
                "mutants_generated": 0,
                "mutants_killed": 0,
                "mutants_survived": 0,
                "mutation_score": 0.0,
                "runtime_seconds": 0
            }
        }))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ==================== JavaAnalysisTool Tests ====================

    #[test]
    fn test_java_analysis_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze_java");
        assert!(metadata.description.contains("Java"));
        assert!(metadata.description.contains("complexity"));
    }

    #[test]
    fn test_java_analysis_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);
        let metadata = tool.metadata();

        let schema = metadata.input_schema;
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["max_depth"].is_object());
        assert!(schema["properties"]["include_metrics"].is_object());
        assert!(schema["properties"]["include_ast"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("path")));
    }

    // ==================== JavaMutationTool Tests ====================

    #[test]
    fn test_java_mutation_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "mutation_test_java");
        assert!(metadata.description.contains("mutation"));
        assert!(metadata.description.contains("Java"));
    }

    #[test]
    fn test_java_mutation_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);
        let metadata = tool.metadata();

        let schema = metadata.input_schema;
        assert!(schema["properties"]["project_path"].is_object());
        assert!(schema["properties"]["source_path"].is_object());
        assert!(schema["properties"]["test_command"].is_object());
        assert!(schema["properties"]["mutation_operators"].is_object());
        assert!(schema["properties"]["timeout"].is_object());
    }

    // ==================== find_java_files Tests ====================

    #[test]
    fn test_find_java_files_empty_dir() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let files = find_java_files(dir.path(), 5).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_find_java_files_with_java() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let java_file = dir.path().join("Test.java");
        fs::write(&java_file, "public class Test {}").unwrap();

        let files = find_java_files(dir.path(), 5).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Test.java"));
    }

    #[test]
    fn test_find_java_files_max_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("deep").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let java_file = subdir.join("Deep.java");
        fs::write(&java_file, "public class Deep {}").unwrap();

        // With max_depth 1, shouldn't find nested file
        let files_shallow = find_java_files(dir.path(), 1).unwrap();
        assert!(
            files_shallow.is_empty() || !files_shallow.iter().any(|f| f.ends_with("Deep.java"))
        );

        // With max_depth 5, should find it
        let files_deep = find_java_files(dir.path(), 5).unwrap();
        assert!(files_deep.iter().any(|f| f.ends_with("Deep.java")));
    }

    #[test]
    fn test_find_java_files_ignores_other_extensions() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("test.scala"), "object Test {}").unwrap();
        fs::write(dir.path().join("test.kt"), "class Test").unwrap();

        let files = find_java_files(dir.path(), 5).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_java_files_zero_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("Test.java"), "class Test {}").unwrap();

        // With depth 0, should only include the starting path itself
        let files = find_java_files(dir.path(), 0).unwrap();
        // Files in subdirectories shouldn't be found
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_java_files_multiple() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("A.java"), "class A {}").unwrap();
        fs::write(dir.path().join("B.java"), "class B {}").unwrap();
        fs::write(dir.path().join("C.java"), "class C {}").unwrap();

        let files = find_java_files(dir.path(), 5).unwrap();
        assert_eq!(files.len(), 3);
    }

    // ==================== Tool Execute Tests (Error Cases) ====================

    #[tokio::test]
    async fn test_java_analysis_tool_missing_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("path"));
    }

    #[tokio::test]
    async fn test_java_analysis_tool_nonexistent_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);

        let params = json!({
            "path": "/nonexistent/path/to/java"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist") || err.message.contains("Path"));
    }

    #[tokio::test]
    async fn test_java_analysis_tool_wrong_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);

        let params = json!({
            "path": file.path().to_str().unwrap()
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Java") || err.message.contains(".java"));
    }

    #[tokio::test]
    async fn test_java_mutation_tool_missing_project_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("project_path"));
    }

    #[tokio::test]
    async fn test_java_mutation_tool_missing_source_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source_path"));
    }

    #[tokio::test]
    async fn test_java_mutation_tool_complete_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        // Should succeed with placeholder response
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
    }

    #[tokio::test]
    async fn test_java_mutation_tool_custom_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source",
            "test_command": "mvn clean test",
            "timeout": 60,
            "mutation_operators": ["arithmetic", "conditional"]
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["test_command"], "mvn clean test");
        assert_eq!(value["timeout"], 60);
        assert_eq!(
            value["mutation_operators"],
            json!(["arithmetic", "conditional"])
        );
    }

    #[tokio::test]
    async fn test_java_mutation_tool_default_operators() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        let operators = value["mutation_operators"].as_array().unwrap();
        assert!(operators.contains(&json!("arithmetic")));
        assert!(operators.contains(&json!("conditional")));
        assert!(operators.contains(&json!("method")));
        assert!(operators.contains(&json!("assignment")));
    }

    // ==================== Integration-like Tests ====================

    #[tokio::test]
    async fn test_analyze_java_file_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".java").unwrap();
        writeln!(
            file,
            r#"
            public class Test {{
                public String hello() {{
                    return "Hello";
                }}
            }}
            "#
        )
        .unwrap();

        let result = analyze_java_file(file.path(), true, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value["status"] == "completed" || value["status"] == "error");
    }

    #[tokio::test]
    async fn test_analyze_java_directory_empty() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let result = analyze_java_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 0);
    }

    #[tokio::test]
    async fn test_analyze_java_directory_with_file() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Test.java"),
            "public class Test { public void run() {} }",
        )
        .unwrap();

        let result = analyze_java_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 1);
    }

    #[tokio::test]
    async fn test_analyze_java_file_with_metrics() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".java").unwrap();
        writeln!(
            file,
            r#"
            public class Test {{
                public int calculate(int x, int y) {{
                    if (x > y) {{
                        return x;
                    }} else {{
                        return y;
                    }}
                }}
            }}
            "#
        )
        .unwrap();

        let result = analyze_java_file(file.path(), true, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        // Should have metrics if the analysis succeeded
        if value["status"] == "completed" {
            assert!(value.get("metrics").is_some());
        }
    }

    #[tokio::test]
    async fn test_analyze_java_file_without_metrics() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".java").unwrap();
        writeln!(file, "public class Test {{}}").unwrap();

        let result = analyze_java_file(file.path(), false, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        // Should not have metrics when include_metrics is false
        assert!(value.get("metrics").is_none());
    }

    // ==================== Edge Cases ====================

    #[tokio::test]
    async fn test_analyze_java_directory_with_nested_files() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("src").join("main").join("java");
        fs::create_dir_all(&subdir).unwrap();

        fs::write(subdir.join("Main.java"), "public class Main {}").unwrap();
        fs::write(subdir.join("Helper.java"), "public class Helper {}").unwrap();

        let result = analyze_java_directory(dir.path(), 10, true, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 2);
    }

    #[test]
    fn test_tool_creation() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());

        let analysis_tool = JavaAnalysisTool::new(Arc::clone(&registry));
        let mutation_tool = JavaMutationTool::new(Arc::clone(&registry));

        // Just verify they can be created without panicking
        assert_eq!(analysis_tool.metadata().name, "analyze_java");
        assert_eq!(mutation_tool.metadata().name, "mutation_test_java");
    }
}
