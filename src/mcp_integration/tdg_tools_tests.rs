// Included from tdg_tools.rs — tests for TDG tools

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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

        let result = tool
            .execute(json!({
                "path": "/nonexistent/path/to/file.rs"
            }))
            .await;

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

        let result = tool
            .execute(json!({
                "path": temp_file.path().to_str().unwrap(),
                "analysis_type": "file"
            }))
            .await;

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

        let result = tool
            .execute(json!({
                "path": temp_file.path().to_str().unwrap(),
                "max_recommendations": 3
            }))
            .await;

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
        let complexity_suggestion =
            generate_suggestion("High cyclomatic complexity: 20", "StructuralComplexity");
        assert!(complexity_suggestion.contains("smaller"));
        assert!(complexity_suggestion.contains("functions"));

        let nesting_suggestion =
            generate_suggestion("Deep nesting: 5 levels", "SemanticComplexity");
        assert!(nesting_suggestion.contains("nesting"));
        assert!(
            nesting_suggestion.contains("early returns")
                || nesting_suggestion.contains("guard clauses")
        );

        let duplication_suggestion = generate_suggestion("Code duplication: 15.2%", "Duplication");
        assert!(duplication_suggestion.contains("reusable"));
    }

    #[test]
    fn test_analyze_technical_debt_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);
        assert_eq!(tool.metadata().name, "analyze_technical_debt");
    }

    #[test]
    fn test_get_quality_recommendations_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = GetQualityRecommendationsTool::new(registry);
        assert_eq!(tool.metadata().name, "get_quality_recommendations");
    }

    #[test]
    fn test_analyze_technical_debt_schema_analysis_type() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);
        let schema = tool.metadata().input_schema;

        let analysis_type = &schema["properties"]["analysis_type"];
        assert!(analysis_type["enum"].is_array());
        let enum_vals: Vec<&str> = analysis_type["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(enum_vals.contains(&"file"));
        assert!(enum_vals.contains(&"project"));
        assert!(enum_vals.contains(&"auto"));
    }

    #[test]
    fn test_get_quality_recommendations_schema_defaults() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = GetQualityRecommendationsTool::new(registry);
        let schema = tool.metadata().input_schema;

        // Check default values exist
        assert!(schema["properties"]["max_recommendations"]["default"].is_number());
        assert!(schema["properties"]["min_severity"]["default"].is_string());
    }

    #[tokio::test]
    async fn test_analyze_technical_debt_auto_type() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTechnicalDebtTool::new(registry);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn test() {{ let x = 1; }}").unwrap();
        temp_file.flush().unwrap();

        // Default analysis_type should be "auto"
        let result = tool
            .execute(json!({
                "path": temp_file.path().to_str().unwrap()
            }))
            .await;

        // Should succeed with default options
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_quality_recommendations_nonexistent_path() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = GetQualityRecommendationsTool::new(registry);

        let result = tool
            .execute(json!({
                "path": "/nonexistent/path/file.rs"
            }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist"));
    }

    #[test]
    fn test_severity_ordering() {
        // Test all severity combinations
        assert!(should_include_severity("critical", "critical"));
        assert!(should_include_severity("critical", "high"));
        assert!(should_include_severity("critical", "medium"));
        assert!(should_include_severity("critical", "low"));

        assert!(!should_include_severity("high", "critical"));
        assert!(should_include_severity("high", "high"));
        assert!(should_include_severity("high", "medium"));
        assert!(should_include_severity("high", "low"));

        assert!(!should_include_severity("medium", "critical"));
        assert!(!should_include_severity("medium", "high"));
        assert!(should_include_severity("medium", "medium"));
        assert!(should_include_severity("medium", "low"));

        assert!(!should_include_severity("low", "critical"));
        assert!(!should_include_severity("low", "high"));
        assert!(!should_include_severity("low", "medium"));
        assert!(should_include_severity("low", "low"));
    }

    #[test]
    fn test_generate_suggestion_unknown_category() {
        let suggestion = generate_suggestion("Some unknown issue", "UnknownCategory");
        // Should still produce some suggestion
        assert!(!suggestion.is_empty());
    }

    #[test]
    fn test_tool_descriptions_are_meaningful() {
        let registry = Arc::new(AgentRegistry::new());

        let analyze_tool = AnalyzeTechnicalDebtTool::new(registry.clone());
        let analyze_desc = analyze_tool.metadata().description;
        assert!(analyze_desc.len() > 30);
        assert!(analyze_desc.contains("technical debt") || analyze_desc.contains("TDG"));

        let recommend_tool = GetQualityRecommendationsTool::new(registry);
        let recommend_desc = recommend_tool.metadata().description;
        assert!(recommend_desc.len() > 30);
        assert!(
            recommend_desc.contains("recommendations") || recommend_desc.contains("actionable")
        );
    }
}
