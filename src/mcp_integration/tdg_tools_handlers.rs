// Included from tdg_tools.rs — MCP tool implementations for TDG analysis

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

        let analysis_type = params["analysis_type"].as_str().unwrap_or("auto");

        let include_penalties = params["include_penalties"].as_bool().unwrap_or(true);

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

#[async_trait]
impl McpTool for GetQualityRecommendationsTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "get_quality_recommendations".to_string(),
            description: "Get actionable quality improvement recommendations based on TDG analysis"
                .to_string(),
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

        let max_recommendations = params["max_recommendations"].as_u64().unwrap_or(5) as usize;

        let min_severity = params["min_severity"].as_str().unwrap_or("medium");

        // Create TDG analyzer
        let analyzer = TdgAnalyzer::new().map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to create TDG analyzer: {}", e),
            data: None,
        })?;

        // Analyze file or project
        let score = if path.is_dir() {
            analyzer
                .analyze_project(&path)
                .map_err(|e| McpError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("Analysis failed: {}", e),
                    data: None,
                })?
                .average()
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
