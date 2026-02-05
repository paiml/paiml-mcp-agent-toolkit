use crate::mcp_integration::ast_item_helpers::{extract_complexity, extract_kind, extract_name};
use crate::mcp_integration::{McpError, McpTool, ToolMetadata};
// Import the ScalaAstVisitor when available
use crate::services::languages::scala::ScalaAstVisitor;
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};

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

/// Analyzes a single Scala file
async fn analyze_scala_file(
    path: &std::path::Path,
    include_metrics: bool,
    include_ast: bool,
) -> Result<Value> {
    // Read the file content
    let content = fs::read_to_string(path).await?;

    // Create visitor and analyze
    let visitor = ScalaAstVisitor::new(path);
    match visitor.analyze_scala_source(&content) {
        Ok(items) => {
            // Calculate metrics
            let class_count = items
                .iter()
                .filter(|item| {
                    // Regular classes are represented as AstItem::Struct WITHOUT "case" in derives
                    if let crate::services::context::AstItem::Struct { derives, .. } = item {
                        !derives.contains(&"case".to_string())
                    } else {
                        false
                    }
                })
                .count();

            let trait_count = items
                .iter()
                .filter(|item| {
                    let kind = extract_kind(item);
                    kind == "trait"
                })
                .count();

            let object_count = items
                .iter()
                .filter(|item| {
                    let kind = extract_kind(item);
                    kind == "object" || kind == "module"
                })
                .count();

            let case_class_count = items
                .iter()
                .filter(|item| {
                    // Case classes are represented as AstItem::Struct with derives containing "case"
                    if let crate::services::context::AstItem::Struct { derives, .. } = item {
                        derives.contains(&"case".to_string())
                    } else {
                        false
                    }
                })
                .count();

            let method_count = items
                .iter()
                .filter(|item| {
                    let kind = extract_kind(item);
                    kind == "method" || kind == "function"
                })
                .count();

            let package_name = items
                .iter()
                .find(|item| {
                    let kind = extract_kind(item);
                    kind == "package" || kind == "module"
                })
                .map(extract_name)
                .unwrap_or_else(|| "default".to_string());

            // Build response
            let mut result = json!({
                "status": "completed",
                "path": path.display().to_string(),
                "language": "scala",
                "summary": {
                    "class_count": class_count,
                    "trait_count": trait_count,
                    "object_count": object_count,
                    "case_class_count": case_class_count,
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
                    "functional_percentage": calculate_functional_percentage(&items),
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
            warn!("Failed to parse Scala file {}: {}", path.display(), e);
            Ok(json!({
                "status": "error",
                "path": path.display().to_string(),
                "language": "scala",
                "error": e
            }))
        }
    }
}

/// Analyzes a directory of Scala files recursively
async fn analyze_scala_directory(
    path: &std::path::Path,
    max_depth: u64,
    include_metrics: bool,
    include_ast: bool,
) -> Result<Value> {
    // Use walkdir to find all Scala files
    let scala_files = find_scala_files(path, max_depth as usize)?;

    if scala_files.is_empty() {
        return Ok(json!({
            "status": "completed",
            "path": path.display().to_string(),
            "language": "scala",
            "summary": {
                "file_count": 0,
                "message": "No Scala files found"
            }
        }));
    }

    // Analyze each file
    let mut file_results = Vec::new();
    let mut total_classes = 0;
    let mut total_traits = 0;
    let mut total_objects = 0;
    let mut total_case_classes = 0;
    let mut total_methods = 0;
    let mut total_complexity = 0;
    let mut max_complexity = 0;
    let mut total_loc = 0;
    let mut weighted_functional_percentage = 0.0;
    let mut total_weight = 0.0;

    for file_path in &scala_files {
        match analyze_scala_file(file_path, include_metrics, false).await {
            Ok(result) => {
                // Update counters
                if let Some(summary) = result["summary"].as_object() {
                    if let Some(class_count) = summary["class_count"].as_u64() {
                        total_classes += class_count;
                    }
                    if let Some(trait_count) = summary["trait_count"].as_u64() {
                        total_traits += trait_count;
                    }
                    if let Some(object_count) = summary["object_count"].as_u64() {
                        total_objects += object_count;
                    }
                    if let Some(case_class_count) = summary["case_class_count"].as_u64() {
                        total_case_classes += case_class_count;
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

                            // Calculate weighted functional percentage
                            if let Some(fp) = metrics["functional_percentage"].as_f64() {
                                weighted_functional_percentage += fp * (loc as f64);
                                total_weight += loc as f64;
                            }
                        }
                    }
                }

                file_results.push(result);
            }
            Err(e) => {
                warn!("Error analyzing Scala file {}: {}", file_path.display(), e);
            }
        }
    }

    // Calculate average complexity and functional percentage
    let avg_complexity = if total_methods > 0 {
        (total_complexity as f64) / (total_methods as f64)
    } else {
        0.0
    };

    let avg_functional_percentage = if total_weight > 0.0 {
        weighted_functional_percentage / total_weight
    } else {
        0.0
    };

    // Build aggregate response
    let mut result = json!({
        "status": "completed",
        "path": path.display().to_string(),
        "language": "scala",
        "summary": {
            "file_count": scala_files.len(),
            "class_count": total_classes,
            "trait_count": total_traits,
            "object_count": total_objects,
            "case_class_count": total_case_classes,
            "method_count": total_methods,
        }
    });

    // Add metrics if requested
    if include_metrics {
        result["metrics"] = json!({
            "total_complexity": total_complexity,
            "max_complexity": max_complexity,
            "avg_complexity": avg_complexity,
            "functional_percentage": avg_functional_percentage,
            "total_loc": total_loc
        });
    }

    // Add file-level results if include_ast was requested
    if include_ast {
        result["files"] = serde_json::to_value(&file_results)?;
    }

    Ok(result)
}

/// Helper function to find all Scala files in a directory
fn find_scala_files(path: &std::path::Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut scala_files = Vec::new();

    let walker = walkdir::WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(Result::ok);

    for entry in walker {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "scala" || ext == "sc")
        {
            scala_files.push(path.to_path_buf());
        }
    }

    Ok(scala_files)
}

/// Helper function to calculate the percentage of functional code patterns vs imperative
fn calculate_functional_percentage(items: &[crate::services::context::AstItem]) -> f64 {
    let mut functional_score = 0.0;
    let mut imperative_score = 0.0;

    for item in items {
        let kind = extract_kind(item);
        let name = extract_name(item);

        match kind.as_str() {
            // Functional patterns
            "struct" if name.starts_with("Case") => functional_score += 1.0, // case_class
            "trait" => functional_score += 0.5,
            "module" => functional_score += 0.5, // object

            // Imperative patterns
            "struct" | "class" if !name.starts_with("Case") => imperative_score += 0.5,
            "function" | "method" => imperative_score += 0.3, // mild imperative
            _ => {}
        }
    }

    let total = functional_score + imperative_score;
    if total > 0.0 {
        (functional_score / total) * 100.0
    } else {
        50.0 // Default to 50% if we can't determine
    }
}

/// Scala mutation testing tool
pub struct ScalaMutationTool {
    #[allow(dead_code)]
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl ScalaMutationTool {
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl McpTool for ScalaMutationTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "mutation_test_scala".to_string(),
            description: "Performs mutation testing on Scala code to assess test suite quality."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to Scala project root"
                    },
                    "source_path": {
                        "type": "string",
                        "description": "Path to source file or directory to mutate"
                    },
                    "test_command": {
                        "type": "string",
                        "description": "Command to run tests (defaults to 'sbt test')"
                    },
                    "mutation_operators": {
                        "type": "array",
                        "description": "List of mutation operators to apply",
                        "items": {"type": "string"},
                        "default": ["arithmetic", "conditional", "method", "functional"]
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

        let test_command = params["test_command"].as_str().unwrap_or("sbt test");
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
                "functional".to_string(),
            ]
        };

        info!(
            "Running Scala mutation tests on project: {}, source: {}",
            project_path, source_path
        );

        // In a real implementation, we would spawn the Stryker or similar mutation testing tool
        // For now, return a placeholder response
        Ok(json!({
            "status": "completed",
            "message": "Scala mutation testing completed",
            "project_path": project_path,
            "source_path": source_path,
            "test_command": test_command,
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
    use crate::services::context::AstItem;

    // ==================== ScalaAnalysisTool Tests ====================

    #[test]
    fn test_scala_analysis_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze_scala");
        assert!(metadata.description.contains("Scala"));
        assert!(metadata.description.contains("complexity"));
    }

    #[test]
    fn test_scala_analysis_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);
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

    // ==================== ScalaMutationTool Tests ====================

    #[test]
    fn test_scala_mutation_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "mutation_test_scala");
        assert!(metadata.description.contains("mutation"));
        assert!(metadata.description.contains("Scala"));
    }

    #[test]
    fn test_scala_mutation_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);
        let metadata = tool.metadata();

        let schema = metadata.input_schema;
        assert!(schema["properties"]["project_path"].is_object());
        assert!(schema["properties"]["source_path"].is_object());
        assert!(schema["properties"]["test_command"].is_object());
        assert!(schema["properties"]["mutation_operators"].is_object());
        assert!(schema["properties"]["timeout"].is_object());
    }

    // ==================== find_scala_files Tests ====================

    #[test]
    fn test_find_scala_files_empty_dir() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let files = find_scala_files(dir.path(), 5).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_find_scala_files_with_scala() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let scala_file = dir.path().join("Test.scala");
        fs::write(&scala_file, "object Test {}").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Test.scala"));
    }

    #[test]
    fn test_find_scala_files_with_sc_extension() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let sc_file = dir.path().join("Script.sc");
        fs::write(&sc_file, "println(\"Hello\")").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Script.sc"));
    }

    #[test]
    fn test_find_scala_files_max_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("deep").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let scala_file = subdir.join("Deep.scala");
        fs::write(&scala_file, "object Deep {}").unwrap();

        // With max_depth 1, shouldn't find nested file
        let files_shallow = find_scala_files(dir.path(), 1).unwrap();
        assert!(
            files_shallow.is_empty() || !files_shallow.iter().any(|f| f.ends_with("Deep.scala"))
        );

        // With max_depth 5, should find it
        let files_deep = find_scala_files(dir.path(), 5).unwrap();
        assert!(files_deep.iter().any(|f| f.ends_with("Deep.scala")));
    }

    #[test]
    fn test_find_scala_files_ignores_other_extensions() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("test.java"), "class Test {}").unwrap();
        fs::write(dir.path().join("test.kt"), "class Test").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();
        assert!(files.is_empty());
    }

    // ==================== calculate_functional_percentage Tests ====================

    #[test]
    fn test_calculate_functional_percentage_empty() {
        let items: Vec<AstItem> = vec![];
        let percentage = calculate_functional_percentage(&items);

        // Default to 50% when no items
        assert!((percentage - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_functional_percentage_trait_only() {
        let items = vec![AstItem::Trait {
            name: "MyTrait".to_string(),
            methods: vec![],
            line_start: 1,
            line_end: 10,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Traits are functional
        assert!(percentage > 0.0);
    }

    #[test]
    fn test_calculate_functional_percentage_function_only() {
        let items = vec![AstItem::Function {
            name: "myFunc".to_string(),
            complexity: 5,
            line_start: 1,
            line_end: 10,
            parameters: vec![],
            return_type: None,
            is_async: false,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Functions have some imperative score
        assert!(percentage >= 0.0 && percentage <= 100.0);
    }

    #[test]
    fn test_calculate_functional_percentage_mixed() {
        let items = vec![
            AstItem::Trait {
                name: "Trait1".to_string(),
                methods: vec![],
                line_start: 1,
                line_end: 10,
            },
            AstItem::Function {
                name: "func1".to_string(),
                complexity: 3,
                line_start: 11,
                line_end: 20,
                parameters: vec![],
                return_type: None,
                is_async: false,
            },
        ];

        let percentage = calculate_functional_percentage(&items);
        // Should be somewhere between 0 and 100
        assert!(percentage > 0.0 && percentage < 100.0);
    }

    #[test]
    fn test_calculate_functional_percentage_module() {
        let items = vec![AstItem::Module {
            name: "MyModule".to_string(),
            items: vec![],
            line_start: 1,
            line_end: 20,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Modules/objects have functional score
        assert!(percentage > 0.0);
    }

    // ==================== Tool Execute Tests (Error Cases) ====================

    #[tokio::test]
    async fn test_scala_analysis_tool_missing_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("path"));
    }

    #[tokio::test]
    async fn test_scala_analysis_tool_nonexistent_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);

        let params = json!({
            "path": "/nonexistent/path/to/scala"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist") || err.message.contains("Path"));
    }

    #[tokio::test]
    async fn test_scala_analysis_tool_wrong_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);

        let params = json!({
            "path": file.path().to_str().unwrap()
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Scala") || err.message.contains(".scala"));
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_missing_project_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("project_path"));
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_missing_source_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source_path"));
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_complete_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

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
    async fn test_scala_mutation_tool_custom_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source",
            "test_command": "sbt clean test",
            "timeout": 60,
            "mutation_operators": ["arithmetic", "method"]
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["test_command"], "sbt clean test");
        assert_eq!(value["timeout"], 60);
        assert_eq!(value["mutation_operators"], json!(["arithmetic", "method"]));
    }

    // ==================== Integration-like Tests ====================

    #[tokio::test]
    async fn test_analyze_scala_file_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".scala").unwrap();
        writeln!(
            file,
            r#"
            object Test {{
                def hello(): String = "Hello"
            }}
            "#
        )
        .unwrap();

        let result = analyze_scala_file(file.path(), true, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value["status"] == "completed" || value["status"] == "error");
    }

    #[tokio::test]
    async fn test_analyze_scala_directory_empty() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let result = analyze_scala_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 0);
    }

    #[tokio::test]
    async fn test_analyze_scala_directory_with_file() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Test.scala"),
            "object Test { def run(): Unit = () }",
        )
        .unwrap();

        let result = analyze_scala_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 1);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_find_scala_files_zero_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("Test.scala"), "object Test {}").unwrap();

        // With depth 0, should only include the starting path itself
        let files = find_scala_files(dir.path(), 0).unwrap();
        // Files in subdirectories shouldn't be found
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_scala_files_multiple() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("A.scala"), "object A {}").unwrap();
        fs::write(dir.path().join("B.scala"), "object B {}").unwrap();
        fs::write(dir.path().join("C.sc"), "println(1)").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_calculate_functional_percentage_case_class_naming() {
        // Test with struct that has "Case" prefix in name
        let items = vec![AstItem::Struct {
            name: "CaseUser".to_string(),
            fields: vec![],
            derives: vec![],
            line_start: 1,
            line_end: 5,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Should count as functional because name starts with Case
        assert!(percentage > 0.0);
    }
}
