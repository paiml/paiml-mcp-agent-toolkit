mod tests {
    use super::*;
    use serde_json::json;

    // ===============================================
    // Tests for is_vectorized_tool
    // ===============================================

    #[test]
    fn test_is_vectorized_tool_duplicates() {
        assert!(is_vectorized_tool("analyze_duplicates_vectorized"));
    }

    #[test]
    fn test_is_vectorized_tool_graph_metrics() {
        assert!(is_vectorized_tool("analyze_graph_metrics_vectorized"));
    }

    #[test]
    fn test_is_vectorized_tool_name_similarity() {
        assert!(is_vectorized_tool("analyze_name_similarity_vectorized"));
    }

    #[test]
    fn test_is_vectorized_tool_symbol_table() {
        assert!(is_vectorized_tool("analyze_symbol_table_vectorized"));
    }

    #[test]
    fn test_is_vectorized_tool_incremental_coverage() {
        assert!(is_vectorized_tool(
            "analyze_incremental_coverage_vectorized"
        ));
    }

    #[test]
    fn test_is_vectorized_tool_big_o() {
        assert!(is_vectorized_tool("analyze_big_o_vectorized"));
    }

    #[test]
    fn test_is_vectorized_tool_enhanced_report() {
        assert!(is_vectorized_tool("generate_enhanced_report"));
    }

    #[test]
    fn test_is_vectorized_tool_unknown() {
        assert!(!is_vectorized_tool("unknown_tool"));
    }

    #[test]
    fn test_is_vectorized_tool_empty_string() {
        assert!(!is_vectorized_tool(""));
    }

    #[test]
    fn test_is_vectorized_tool_partial_match() {
        // Should not match partial names
        assert!(!is_vectorized_tool("analyze_duplicates"));
        assert!(!is_vectorized_tool("duplicates_vectorized"));
        assert!(!is_vectorized_tool("vectorized"));
    }

    #[test]
    fn test_is_vectorized_tool_case_sensitive() {
        assert!(!is_vectorized_tool("ANALYZE_DUPLICATES_VECTORIZED"));
        assert!(!is_vectorized_tool("Analyze_Duplicates_Vectorized"));
    }

    // ===============================================
    // Tests for VECTORIZED_TOOLS constant
    // ===============================================

    #[test]
    fn test_vectorized_tools_count() {
        assert_eq!(VECTORIZED_TOOLS.len(), 7);
    }

    #[test]
    fn test_vectorized_tools_all_unique() {
        let mut seen = std::collections::HashSet::new();
        for tool in VECTORIZED_TOOLS {
            assert!(seen.insert(*tool), "Duplicate tool name: {}", tool);
        }
    }

    // ===============================================
    // Tests for get_vectorized_tools_info
    // ===============================================

    #[test]
    fn test_get_vectorized_tools_info_count() {
        let tools = get_vectorized_tools_info();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_get_vectorized_tools_info_has_name() {
        let tools = get_vectorized_tools_info();
        for tool in &tools {
            assert!(tool.get("name").is_some(), "Tool missing 'name' field");
            assert!(tool["name"].is_string(), "Tool 'name' should be a string");
        }
    }

    #[test]
    fn test_get_vectorized_tools_info_has_description() {
        let tools = get_vectorized_tools_info();
        for tool in &tools {
            assert!(
                tool.get("description").is_some(),
                "Tool missing 'description' field"
            );
            assert!(
                tool["description"].is_string(),
                "Tool 'description' should be a string"
            );
        }
    }

    #[test]
    fn test_get_vectorized_tools_info_has_input_schema() {
        let tools = get_vectorized_tools_info();
        for tool in &tools {
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool missing 'inputSchema' field"
            );
            assert!(
                tool["inputSchema"].is_object(),
                "Tool 'inputSchema' should be an object"
            );
        }
    }

    #[test]
    fn test_get_vectorized_tools_info_input_schema_structure() {
        let tools = get_vectorized_tools_info();
        for tool in &tools {
            let schema = &tool["inputSchema"];
            assert_eq!(schema["type"], "object");
            assert!(
                schema.get("properties").is_some(),
                "inputSchema missing 'properties'"
            );
            assert!(
                schema.get("required").is_some(),
                "inputSchema missing 'required'"
            );
        }
    }

    #[test]
    fn test_get_vectorized_tools_info_all_require_project_path() {
        let tools = get_vectorized_tools_info();
        for tool in &tools {
            let required = tool["inputSchema"]["required"].as_array().unwrap();
            let required_strs: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
            assert!(
                required_strs.contains(&"project_path"),
                "Tool {} should require 'project_path'",
                tool["name"]
            );
        }
    }

    #[test]
    fn test_get_vectorized_tools_info_duplicates_tool_schema() {
        let tools = get_vectorized_tools_info();
        let duplicates_tool = tools
            .iter()
            .find(|t| t["name"] == "analyze_duplicates_vectorized")
            .expect("analyze_duplicates_vectorized not found");

        let props = &duplicates_tool["inputSchema"]["properties"];
        assert!(props.get("project_path").is_some());
        assert!(props.get("detection_type").is_some());
        assert!(props.get("threshold").is_some());
        assert!(props.get("parallel_threads").is_some());
        assert!(props.get("use_simd").is_some());
    }

    #[test]
    fn test_get_vectorized_tools_info_name_similarity_requires_query() {
        let tools = get_vectorized_tools_info();
        let similarity_tool = tools
            .iter()
            .find(|t| t["name"] == "analyze_name_similarity_vectorized")
            .expect("analyze_name_similarity_vectorized not found");

        let required = similarity_tool["inputSchema"]["required"]
            .as_array()
            .unwrap();
        let required_strs: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(required_strs.contains(&"query"));
    }

    // ===============================================
    // Tests for handle_vectorized_tools
    // ===============================================

    #[tokio::test]
    async fn test_handle_vectorized_tools_unknown_tool() {
        let request_id = json!(1);
        let tool_params = ToolCallParams {
            name: "unknown_vectorized_tool".to_string(),
            arguments: json!({}),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Unknown vectorized tool"));
    }

    // ===============================================
    // Tests for handle_duplicates_vectorized
    // ===============================================

    #[tokio::test]
    async fn test_handle_duplicates_vectorized_success() {
        let request_id = json!(1);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert!(result.get("summary").is_some());
        assert!(result.get("duplicates").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_duplicates_vectorized_with_all_options() {
        let request_id = json!(2);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "detection_type": "semantic",
                "threshold": 0.85,
                "min_lines": 5,
                "max_tokens": 1000,
                "parallel_threads": 4,
                "use_simd": false
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["summary"]["simd_enabled"], false);
        assert_eq!(result["summary"]["parallel_threads"], 4);
    }

    #[tokio::test]
    async fn test_handle_duplicates_vectorized_missing_params() {
        let request_id = json!(3);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({}), // Missing required project_path
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Invalid parameters"));
    }

    #[tokio::test]
    async fn test_handle_duplicates_vectorized_invalid_params() {
        let request_id = json!(4);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": 123 // Should be a string
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_some());
    }

    // ===============================================
    // Tests for handle_graph_metrics_vectorized
    // ===============================================

    #[tokio::test]
    async fn test_handle_graph_metrics_vectorized_success() {
        let request_id = json!(10);
        let tool_params = ToolCallParams {
            name: "analyze_graph_metrics_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert!(result.get("graph_stats").is_some());
        assert!(result.get("centrality_metrics").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_graph_metrics_vectorized_with_gpu() {
        let request_id = json!(11);
        let tool_params = ToolCallParams {
            name: "analyze_graph_metrics_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "metrics": ["pagerank", "betweenness"],
                "pagerank_damping": 0.85,
                "max_iterations": 100,
                "convergence_threshold": 0.0001,
                "use_gpu": true
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["performance"]["gpu_acceleration"], true);
    }

    #[tokio::test]
    async fn test_handle_graph_metrics_vectorized_missing_params() {
        let request_id = json!(12);
        let tool_params = ToolCallParams {
            name: "analyze_graph_metrics_vectorized".to_string(),
            arguments: json!({}),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_some());
    }

    // ===============================================
    // Tests for handle_name_similarity_vectorized
    // ===============================================

    #[tokio::test]
    async fn test_handle_name_similarity_vectorized_success() {
        let request_id = json!(20);
        let tool_params = ToolCallParams {
            name: "analyze_name_similarity_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "query": "process"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert_eq!(result["query"], "process");
        assert!(result.get("matches").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_name_similarity_vectorized_with_options() {
        let request_id = json!(21);
        let tool_params = ToolCallParams {
            name: "analyze_name_similarity_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "query": "test_function",
                "top_k": 10,
                "threshold": 0.7,
                "phonetic": true,
                "fuzzy": true,
                "use_simd": false
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["performance"]["simd_enabled"], false);
    }

    #[tokio::test]
    async fn test_handle_name_similarity_vectorized_missing_query() {
        let request_id = json!(22);
        let tool_params = ToolCallParams {
            name: "analyze_name_similarity_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
                // Missing required "query"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_some());
    }

    // ===============================================
    // Tests for handle_symbol_table_vectorized
    // ===============================================

    #[tokio::test]
    async fn test_handle_symbol_table_vectorized_success() {
        let request_id = json!(30);
        let tool_params = ToolCallParams {
            name: "analyze_symbol_table_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert!(result.get("summary").is_some());
        assert!(result.get("symbols").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_symbol_table_vectorized_with_options() {
        let request_id = json!(31);
        let tool_params = ToolCallParams {
            name: "analyze_symbol_table_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "filter": "function",
                "query": "process",
                "show_unreferenced": true,
                "show_references": true,
                "parallel_parsing": false
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        // When parallel_parsing is false, then_some returns None
        assert!(result["performance"]["parallel_threads"].is_null());
    }

    #[tokio::test]
    async fn test_handle_symbol_table_vectorized_with_parallel() {
        let request_id = json!(32);
        let tool_params = ToolCallParams {
            name: "analyze_symbol_table_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "parallel_parsing": true
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        // When parallel_parsing is true, then_some returns Some(8)
        assert_eq!(result["performance"]["parallel_threads"], 8);
    }

    // ===============================================
    // Tests for handle_incremental_coverage_vectorized
    // ===============================================

    #[tokio::test]
    async fn test_handle_incremental_coverage_vectorized_success() {
        let request_id = json!(40);
        let tool_params = ToolCallParams {
            name: "analyze_incremental_coverage_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert!(result.get("coverage_summary").is_some());
        assert!(result.get("file_coverage").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_incremental_coverage_vectorized_with_branches() {
        let request_id = json!(41);
        let tool_params = ToolCallParams {
            name: "analyze_incremental_coverage_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "base_branch": "main",
                "target_branch": "feature-branch",
                "changed_files_only": true,
                "parallel_diff": false
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["performance"]["parallel_enabled"], false);
    }

    // ===============================================
    // Tests for handle_big_o_vectorized
    // ===============================================

    #[tokio::test]
    async fn test_handle_big_o_vectorized_success() {
        let request_id = json!(50);
        let tool_params = ToolCallParams {
            name: "analyze_big_o_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert!(result.get("summary").is_some());
        assert!(result.get("complexity_distribution").is_some());
        assert!(result.get("high_complexity_functions").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_big_o_vectorized_with_options() {
        let request_id = json!(51);
        let tool_params = ToolCallParams {
            name: "analyze_big_o_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "confidence_threshold": 80,
                "analyze_space": true,
                "high_complexity_only": true,
                "parallel_analysis": false
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        // When parallel_analysis is false, then_some returns None
        assert!(result["performance"]["parallel_threads"].is_null());
    }

    #[tokio::test]
    async fn test_handle_big_o_vectorized_with_parallel() {
        let request_id = json!(52);
        let tool_params = ToolCallParams {
            name: "analyze_big_o_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path",
                "parallel_analysis": true
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["performance"]["parallel_threads"], 8);
    }

    // ===============================================
    // Tests for handle_enhanced_report
    // ===============================================

    #[tokio::test]
    async fn test_handle_enhanced_report_success() {
        let request_id = json!(60);
        let tool_params = ToolCallParams {
            name: "generate_enhanced_report".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "success");
        assert!(result.get("report").is_some());
        assert!(result.get("performance").is_some());
    }

    #[tokio::test]
    async fn test_handle_enhanced_report_with_options() {
        let request_id = json!(61);
        let tool_params = ToolCallParams {
            name: "generate_enhanced_report".to_string(),
            arguments: json!({
                "project_path": "/test/project",
                "output_format": "markdown",
                "analyses": ["complexity", "duplicates", "coverage"],
                "include_visualizations": true,
                "include_recommendations": true,
                "confidence_threshold": 90
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["performance"]["analyses_performed"], 3);
    }

    #[tokio::test]
    async fn test_handle_enhanced_report_metadata() {
        let request_id = json!(62);
        let tool_params = ToolCallParams {
            name: "generate_enhanced_report".to_string(),
            arguments: json!({
                "project_path": "/test/my-project"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let metadata = &result["report"]["metadata"];

        assert_eq!(metadata["project_name"], "my-project");
        assert!(metadata.get("report_date").is_some());
        assert!(metadata.get("tool_version").is_some());
    }

    #[tokio::test]
    async fn test_handle_enhanced_report_executive_summary() {
        let request_id = json!(63);
        let tool_params = ToolCallParams {
            name: "generate_enhanced_report".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let summary = &result["report"]["executive_summary"];

        assert!(summary.get("health_score").is_some());
        assert!(summary.get("risk_level").is_some());
        assert!(summary.get("critical_issues").is_some());
        assert!(summary.get("high_priority_issues").is_some());
        assert!(summary.get("key_findings").is_some());
    }

    // ===============================================
    // Tests for response structure
    // ===============================================

    #[tokio::test]
    async fn test_response_jsonrpc_version() {
        let request_id = json!(100);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert_eq!(response.jsonrpc, "2.0");
    }

    #[tokio::test]
    async fn test_response_preserves_request_id_number() {
        let request_id = json!(42);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert_eq!(response.id, json!(42));
    }

    #[tokio::test]
    async fn test_response_preserves_request_id_string() {
        let request_id = json!("request-uuid-12345");
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert_eq!(response.id, json!("request-uuid-12345"));
    }

    #[tokio::test]
    async fn test_response_error_has_correct_structure() {
        let request_id = json!(999);
        let tool_params = ToolCallParams {
            name: "unknown_tool".to_string(),
            arguments: json!({}),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(!error.message.is_empty());
    }

    // ===============================================
    // Tests for edge cases
    // ===============================================

    #[tokio::test]
    async fn test_handle_with_null_request_id() {
        let request_id = Value::Null;
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.id.is_null());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_with_array_request_id() {
        let request_id = json!([1, 2, 3]);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        // JSON-RPC allows any JSON value as ID
        assert_eq!(response.id, json!([1, 2, 3]));
    }

    #[tokio::test]
    async fn test_handle_with_special_characters_in_path() {
        let request_id = json!(1);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/path/with spaces/and-dashes/and_underscores"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_with_unicode_path() {
        let request_id = json!(1);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/path/with/unicode/\u{1F600}/emoji"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_all_vectorized_tools_handle_missing_args() {
        let request_id = json!(1);

        for tool_name in VECTORIZED_TOOLS {
            let tool_params = ToolCallParams {
                name: tool_name.to_string(),
                arguments: json!({}),
            };

            let response = handle_vectorized_tools(request_id.clone(), tool_params).await;

            // All tools should return an error when required params are missing
            assert!(
                response.error.is_some(),
                "Tool {} should error on missing params",
                tool_name
            );
        }
    }

    // ===============================================
    // Tests for performance fields
    // ===============================================

    #[tokio::test]
    async fn test_duplicates_performance_fields() {
        let request_id = json!(1);
        let tool_params = ToolCallParams {
            name: "analyze_duplicates_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;
        let result = response.result.unwrap();
        let perf = &result["performance"];

        assert!(perf.get("files_per_second").is_some());
        assert!(perf.get("mb_per_second").is_some());
        assert!(perf.get("vectorization_speedup").is_some());
    }

    #[tokio::test]
    async fn test_graph_metrics_performance_fields() {
        let request_id = json!(1);
        let tool_params = ToolCallParams {
            name: "analyze_graph_metrics_vectorized".to_string(),
            arguments: json!({
                "project_path": "/test/path"
            }),
        };

        let response = handle_vectorized_tools(request_id.clone(), tool_params).await;
        let result = response.result.unwrap();
        let perf = &result["performance"];

        assert!(perf.get("computation_time_ms").is_some());
        assert!(perf.get("vectorization_enabled").is_some());
        assert!(perf.get("speedup_factor").is_some());
    }

    // ===============================================
    // Debug trait tests
    // ===============================================

    #[test]
    fn test_duplicates_args_debug() {
        let args: DuplicatesVectorizedArgs = serde_json::from_value(json!({
            "project_path": "/test",
            "detection_type": "exact",
            "threshold": 0.9
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("DuplicatesVectorizedArgs"));
    }

    #[test]
    fn test_graph_metrics_args_debug() {
        let args: GraphMetricsVectorizedArgs = serde_json::from_value(json!({
            "project_path": "/test"
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("GraphMetricsVectorizedArgs"));
    }

    #[test]
    fn test_name_similarity_args_debug() {
        let args: NameSimilarityVectorizedArgs = serde_json::from_value(json!({
            "project_path": "/test",
            "query": "test"
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("NameSimilarityVectorizedArgs"));
    }

    #[test]
    fn test_symbol_table_args_debug() {
        let args: SymbolTableVectorizedArgs = serde_json::from_value(json!({
            "project_path": "/test"
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("SymbolTableVectorizedArgs"));
    }

    #[test]
    fn test_incremental_coverage_args_debug() {
        let args: IncrementalCoverageVectorizedArgs = serde_json::from_value(json!({
            "project_path": "/test"
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("IncrementalCoverageVectorizedArgs"));
    }

    #[test]
    fn test_big_o_args_debug() {
        let args: BigOVectorizedArgs = serde_json::from_value(json!({
            "project_path": "/test"
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("BigOVectorizedArgs"));
    }

    #[test]
    fn test_enhanced_report_args_debug() {
        let args: EnhancedReportArgs = serde_json::from_value(json!({
            "project_path": "/test"
        }))
        .unwrap();

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("EnhancedReportArgs"));
    }
}
