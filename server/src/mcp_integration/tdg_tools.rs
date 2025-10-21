//! MCP tools for TDG (Technical Debt Gradient) analysis
//!
//! Exposes PMAT's TDG quality analysis system via MCP to enable
//! AI agents to assess code quality and get actionable recommendations.

use super::*;
use crate::agents::registry::AgentRegistry;
use crate::tdg::analyzer_simple::TdgAnalyzer;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Analyze technical debt tool - comprehensive quality analysis
pub struct AnalyzeTechnicalDebtTool {
    _registry: Arc<AgentRegistry>,
}

impl AnalyzeTechnicalDebtTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for AnalyzeTechnicalDebtTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "analyze_technical_debt".to_string(),
            description: "Analyze technical debt gradient (TDG) for a file or project, returning quality scores and metrics".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to file or directory to analyze"
                    },
                    "analysis_type": {
                        "type": "string",
                        "enum": ["file", "project", "auto"],
                        "description": "Type of analysis (auto-detects if not specified)",
                        "default": "auto"
                    },
                    "include_penalties": {
                        "type": "boolean",
                        "description": "Include detailed penalty breakdown in response",
                        "default": true
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let path_str = params["path"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing path parameter".to_string(),
            data: None,
        })?;

        let path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Path does not exist: {}", path_str),
                data: Some(json!({
                    "path": path_str,
                    "suggestion": "Please provide a valid file or directory path"
                })),
            });
        }

        let analysis_type = params["analysis_type"]
            .as_str()
            .unwrap_or("auto");

        let include_penalties = params["include_penalties"]
            .as_bool()
            .unwrap_or(true);

        // Create TDG analyzer
        let analyzer = TdgAnalyzer::new().map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to create TDG analyzer: {}", e),
            data: None,
        })?;

        // Determine analysis type
        let is_dir = path.is_dir();
        let should_analyze_project = match analysis_type {
            "project" => true,
            "file" => false,
            "auto" => is_dir,
            _ => is_dir, // Default to auto behavior
        };

        if should_analyze_project {
            // Project analysis
            let project_score = analyzer.analyze_project(&path).map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Project analysis failed: {}", e),
                data: Some(json!({
                    "path": path_str,
                    "suggestion": "Ensure the directory contains analyzable source files"
                })),
            })?;

            Ok(json!({
                "status": "completed",
                "analysis_type": "project",
                "path": path_str,
                "total_files": project_score.total_files,
                "average_score": project_score.average_score,
                "average_grade": format!("{:?}", project_score.average_grade),
                "file_scores": project_score.files.iter().map(|score| json!({
                    "file": score.file_path.as_ref().map(|p: &std::path::PathBuf| p.to_string_lossy().to_string()),
                    "total": score.total,
                    "grade": format!("{:?}", score.grade),
                })).collect::<Vec<_>>(),
            }))
        } else {
            // File analysis
            let score = analyzer.analyze_file(&path).map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("File analysis failed: {}", e),
                data: Some(json!({
                    "path": path_str,
                    "suggestion": "Ensure the file is a valid source code file"
                })),
            })?;

            Ok(json!({
                "status": "completed",
                "analysis_type": "file",
                "path": path_str,
                "score": {
                    "total": score.total,
                    "grade": format!("{:?}", score.grade),
                    "confidence": score.confidence,
                    "language": format!("{:?}", score.language),
                    "structural_complexity": score.structural_complexity,
                    "semantic_complexity": score.semantic_complexity,
                    "duplication_ratio": score.duplication_ratio,
                    "coupling_score": score.coupling_score,
                    "doc_coverage": score.doc_coverage,
                    "consistency_score": score.consistency_score,
                    "entropy_score": score.entropy_score,
                },
                "penalties": if include_penalties {
                    Some(score.penalties_applied.iter().map(|p| json!({
                        "source_metric": format!("{:?}", p.source_metric),
                        "amount": p.amount,
                        "issue": p.issue,
                    })).collect::<Vec<_>>())
                } else {
                    None
                },
            }))
        }
    }
}

/// Get quality recommendations tool - actionable improvement suggestions
pub struct GetQualityRecommendationsTool {
    _registry: Arc<AgentRegistry>,
}

impl GetQualityRecommendationsTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for GetQualityRecommendationsTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "get_quality_recommendations".to_string(),
            description: "Get actionable quality improvement recommendations based on TDG analysis".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to file or directory to analyze"
                    },
                    "max_recommendations": {
                        "type": "number",
                        "description": "Maximum number of recommendations to return",
                        "default": 5
                    },
                    "min_severity": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Minimum severity level for recommendations",
                        "default": "medium"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let path_str = params["path"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing path parameter".to_string(),
            data: None,
        })?;

        let path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Path does not exist: {}", path_str),
                data: Some(json!({
                    "path": path_str,
                    "suggestion": "Please provide a valid file or directory path"
                })),
            });
        }

        let max_recommendations = params["max_recommendations"]
            .as_u64()
            .unwrap_or(5) as usize;

        let min_severity = params["min_severity"]
            .as_str()
            .unwrap_or("medium");

        // Create TDG analyzer
        let analyzer = TdgAnalyzer::new().map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to create TDG analyzer: {}", e),
            data: None,
        })?;

        // Analyze file or project
        let score = if path.is_dir() {
            analyzer.analyze_project(&path).map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Analysis failed: {}", e),
                data: None,
            })?.average()
        } else {
            analyzer.analyze_file(&path).map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Analysis failed: {}", e),
                data: None,
            })?
        };

        // Generate recommendations based on penalties
        let mut recommendations = Vec::new();

        for penalty in &score.penalties_applied {
            let severity = if penalty.amount > 10.0 {
                "critical"
            } else if penalty.amount > 5.0 {
                "high"
            } else if penalty.amount > 2.0 {
                "medium"
            } else {
                "low"
            };

            // Filter by minimum severity
            if !should_include_severity(severity, min_severity) {
                continue;
            }

            let recommendation = json!({
                "severity": severity,
                "category": format!("{:?}", penalty.source_metric),
                "issue": penalty.issue.clone(),
                "suggestion": generate_suggestion(&penalty.issue, &format!("{:?}", penalty.source_metric)),
                "impact": penalty.amount,
            });

            recommendations.push(recommendation);

            if recommendations.len() >= max_recommendations {
                break;
            }
        }

        // Add general recommendations if score is low
        if score.total < 70.0 && recommendations.len() < max_recommendations {
            recommendations.push(json!({
                "severity": "high",
                "category": "General",
                "issue": format!("Overall quality score is low: {:.1}/100", score.total),
                "suggestion": "Consider refactoring to improve overall code quality. Focus on reducing complexity and improving documentation.",
                "impact": 100.0 - score.total,
            }));
        }

        Ok(json!({
            "status": "completed",
            "path": path_str,
            "score": {
                "total": score.total,
                "grade": format!("{:?}", score.grade),
            },
            "recommendations": recommendations,
            "total_recommendations": recommendations.len(),
            "min_severity_applied": min_severity,
        }))
    }
}

// Helper function to determine if severity should be included
fn should_include_severity(severity: &str, min_severity: &str) -> bool {
    let severity_levels = ["low", "medium", "high", "critical"];
    let min_index = severity_levels.iter().position(|&s| s == min_severity).unwrap_or(1);
    let severity_index = severity_levels.iter().position(|&s| s == severity).unwrap_or(0);
    severity_index >= min_index
}

// Helper function to generate actionable suggestions
fn generate_suggestion(reason: &str, category: &str) -> String {
    let reason_lower = reason.to_lowercase();

    if reason_lower.contains("cyclomatic complexity") || reason_lower.contains("complexity") {
        "Consider breaking down complex functions into smaller, single-responsibility functions. Extract nested logic into helper methods.".to_string()
    } else if reason_lower.contains("nesting") || reason_lower.contains("deep") {
        "Reduce nesting depth by using early returns, guard clauses, or extracting nested logic into separate functions.".to_string()
    } else if reason_lower.contains("duplication") {
        "Extract duplicated code into reusable functions or modules. Consider using design patterns like Template Method or Strategy.".to_string()
    } else if reason_lower.contains("documentation") || reason_lower.contains("doc") {
        "Add comprehensive documentation including function descriptions, parameter explanations, and usage examples.".to_string()
    } else if reason_lower.contains("coupling") {
        "Reduce coupling by using dependency injection, interfaces, or event-driven architecture. Apply SOLID principles.".to_string()
    } else if reason_lower.contains("consistency") {
        "Improve code consistency by following established style guides and naming conventions. Use automated formatters.".to_string()
    } else {
        format!("Review {} and apply refactoring techniques to improve code quality.", category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// RED TEST: Analyze technical debt tool should have correct metadata
    #[test]
    fn red_analyze_technical_debt_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze_technical_debt");
        assert!(metadata.description.contains("TDG"));
        assert!(metadata.description.contains("quality"));

        let schema = metadata.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["analysis_type"].is_object());
        assert_eq!(schema["required"], json!(["path"]));
    }

    /// RED TEST: Get quality recommendations tool should have correct metadata
    #[test]
    fn red_get_quality_recommendations_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = GetQualityRecommendationsTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "get_quality_recommendations");
        assert!(metadata.description.contains("actionable"));
        assert!(metadata.description.contains("recommendations"));

        let schema = metadata.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["max_recommendations"].is_object());
        assert_eq!(schema["required"], json!(["path"]));
    }

    /// RED TEST: Analyze technical debt should return error for missing path
    #[tokio::test]
    async fn red_analyze_technical_debt_missing_path() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("path"));
    }

    /// RED TEST: Analyze technical debt should return error for nonexistent path
    #[tokio::test]
    async fn red_analyze_technical_debt_nonexistent_path() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);

        let result = tool.execute(json!({
            "path": "/nonexistent/path/to/file.rs"
        })).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("does not exist"));
    }

    /// RED TEST: Analyze technical debt should analyze valid file
    #[tokio::test]
    async fn red_analyze_technical_debt_valid_file() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);

        // Create temporary Rust file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn simple_function() {{").unwrap();
        writeln!(temp_file, "    let x = 1;").unwrap();
        writeln!(temp_file, "    println!(\"{{}}\" x);").unwrap();
        writeln!(temp_file, "}}").unwrap();
        temp_file.flush().unwrap();

        let result = tool.execute(json!({
            "path": temp_file.path().to_str().unwrap(),
            "analysis_type": "file"
        })).await;

        assert!(result.is_ok(), "Should analyze valid file: {:?}", result);
        let response = result.unwrap();

        assert_eq!(response["status"], "completed");
        assert_eq!(response["analysis_type"], "file");
        assert!(response["score"]["total"].is_number());
        assert!(response["score"]["grade"].is_string());
    }

    /// RED TEST: Get quality recommendations should return error for missing path
    #[tokio::test]
    async fn red_get_quality_recommendations_missing_path() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = GetQualityRecommendationsTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }

    /// RED TEST: Get quality recommendations should generate suggestions
    #[tokio::test]
    async fn red_get_quality_recommendations_valid_file() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = GetQualityRecommendationsTool::new(registry);

        // Create temporary file with high complexity
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn complex_function(a: i32, b: i32, c: i32) {{").unwrap();
        for i in 0..10 {
            writeln!(temp_file, "    if a > {} {{", i).unwrap();
            writeln!(temp_file, "        if b > {} {{", i).unwrap();
            writeln!(temp_file, "            println!(\"nested\");").unwrap();
            writeln!(temp_file, "        }}").unwrap();
            writeln!(temp_file, "    }}").unwrap();
        }
        writeln!(temp_file, "}}").unwrap();
        temp_file.flush().unwrap();

        let result = tool.execute(json!({
            "path": temp_file.path().to_str().unwrap(),
            "max_recommendations": 3
        })).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        assert_eq!(response["status"], "completed");
        assert!(response["recommendations"].is_array());
        assert!(response["total_recommendations"].as_u64().unwrap() > 0);
    }

    /// RED TEST: Severity filtering should work correctly
    #[test]
    fn red_severity_filtering() {
        assert!(should_include_severity("critical", "low"));
        assert!(should_include_severity("high", "medium"));
        assert!(!should_include_severity("low", "high"));
        assert!(should_include_severity("medium", "medium"));
    }

    /// RED TEST: Suggestion generation should be contextual
    #[test]
    fn red_suggestion_generation() {
        let complexity_suggestion = generate_suggestion("High cyclomatic complexity: 20", "StructuralComplexity");
        assert!(complexity_suggestion.contains("smaller"));
        assert!(complexity_suggestion.contains("functions"));

        let nesting_suggestion = generate_suggestion("Deep nesting: 5 levels", "SemanticComplexity");
        assert!(nesting_suggestion.contains("nesting"));
        assert!(nesting_suggestion.contains("early returns") || nesting_suggestion.contains("guard clauses"));

        let duplication_suggestion = generate_suggestion("Code duplication: 15.2%", "Duplication");
        assert!(duplication_suggestion.contains("reusable"));
    }
}
