    // === DemoReport with Steps Tests ===

    #[test]
    fn test_demo_report_render_cli_with_steps() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("test"),
            method: "test".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("test"),
            result: Some(json!({
                "total_functions": 100,
                "total_warnings": 10,
                "total_errors": 2
            })),
            error: None,
        };

        let step = DemoStep {
            name: "Complexity Analysis".to_string(),
            capability: "Code Complexity Analysis",
            request,
            response,
            elapsed_ms: 250,
            success: true,
            output: Some(json!({"status": "done"})),
        };

        let report = DemoReport {
            repository: "/test/repo".to_string(),
            total_time_ms: 1500,
            steps: vec![step],
            system_diagram: Some("graph TD\n    A --> B".to_string()),
            analysis: DemoAnalysisResult {
                files_analyzed: 20,
                functions_analyzed: 100,
                avg_complexity: 6.5,
                hotspot_functions: 5,
                quality_score: 0.8,
                tech_debt_hours: 10,
                qa_verification: Some("PASSED".to_string()),
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 1500,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("Code Complexity Analysis"));
        assert!(output.contains("250 ms"));
        assert!(output.contains("Functions: 100"));
        assert!(output.contains("Warnings: 10"));
        assert!(output.contains("Errors: 2"));
    }

    #[test]
    fn test_demo_report_render_cli_with_dag_step() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("dag-test"),
            method: "analyze_dag".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("dag-test"),
            result: Some(json!({
                "stats": {
                    "nodes": 50,
                    "edges": 75
                }
            })),
            error: None,
        };

        let step = DemoStep {
            name: "DAG Generation".to_string(),
            capability: "DAG Visualization",
            request,
            response,
            elapsed_ms: 300,
            success: true,
            output: None,
        };

        let report = DemoReport {
            repository: "/test/dag-repo".to_string(),
            total_time_ms: 500,
            steps: vec![step],
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 10,
                functions_analyzed: 50,
                avg_complexity: 4.0,
                hotspot_functions: 2,
                quality_score: 0.9,
                tech_debt_hours: 3,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 500,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("DAG Visualization"));
        assert!(output.contains("50 nodes"));
        assert!(output.contains("75 edges"));
    }

    #[test]
    fn test_demo_report_render_cli_with_churn_step() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("churn-test"),
            method: "analyze_churn".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("churn-test"),
            result: Some(json!({
                "files_analyzed": 45,
                "total_churn_score": 200
            })),
            error: None,
        };

        let step = DemoStep {
            name: "Churn Analysis".to_string(),
            capability: "Code Churn Analysis",
            request,
            response,
            elapsed_ms: 150,
            success: true,
            output: None,
        };

        let report = DemoReport {
            repository: "/test/churn-repo".to_string(),
            total_time_ms: 200,
            steps: vec![step],
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 45,
                functions_analyzed: 0,
                avg_complexity: 0.0,
                hotspot_functions: 0,
                quality_score: 0.0,
                tech_debt_hours: 0,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 200,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("Code Churn Analysis"));
        assert!(output.contains("45"));
        assert!(output.contains("200"));
    }

    #[test]
    fn test_demo_report_render_cli_with_architecture_step() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("arch-test"),
            method: "analyze_architecture".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("arch-test"),
            result: Some(json!({
                "metadata": {
                    "nodes": 20,
                    "edges": 30
                }
            })),
            error: None,
        };

        let step = DemoStep {
            name: "Architecture Analysis".to_string(),
            capability: "System Architecture Analysis",
            request,
            response,
            elapsed_ms: 400,
            success: true,
            output: None,
        };

        let report = DemoReport {
            repository: "/test/arch-repo".to_string(),
            total_time_ms: 500,
            steps: vec![step],
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 0,
                functions_analyzed: 0,
                avg_complexity: 0.0,
                hotspot_functions: 0,
                quality_score: 0.0,
                tech_debt_hours: 0,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 500,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("System Architecture Analysis"));
        assert!(output.contains("Components: 20"));
        assert!(output.contains("Relationships: 30"));
    }

    #[test]
    fn test_demo_report_render_cli_with_defect_step() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("defect-test"),
            method: "analyze_defects".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("defect-test"),
            result: Some(json!({
                "high_risk_files": ["file1.rs", "file2.rs", "file3.rs"],
                "average_probability": 0.42
            })),
            error: None,
        };

        let step = DemoStep {
            name: "Defect Analysis".to_string(),
            capability: "Defect Probability Analysis",
            request,
            response,
            elapsed_ms: 350,
            success: true,
            output: None,
        };

        let report = DemoReport {
            repository: "/test/defect-repo".to_string(),
            total_time_ms: 400,
            steps: vec![step],
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 0,
                functions_analyzed: 0,
                avg_complexity: 0.0,
                hotspot_functions: 0,
                quality_score: 0.0,
                tech_debt_hours: 0,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 400,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("Defect Probability Analysis"));
        assert!(output.contains("High-risk files: 3"));
        assert!(output.contains("0.42"));
    }

    #[test]
    fn test_demo_report_render_cli_without_diagram() {
        let report = DemoReport {
            repository: "/test/no-diagram".to_string(),
            total_time_ms: 100,
            steps: vec![],
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 0,
                functions_analyzed: 0,
                avg_complexity: 0.0,
                hotspot_functions: 0,
                quality_score: 0.0,
                tech_debt_hours: 0,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 100,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(!output.contains("```mermaid"));
        assert!(output.contains("PAIML MCP Agent Toolkit Demo Complete"));
    }

    #[test]
    fn test_demo_report_render_cli_multiple_steps() {
        let steps = vec![
            DemoStep {
                name: "Step 1".to_string(),
                capability: "AST Context Analysis",
                request: McpRequest {
                    jsonrpc: "2.0".to_string(),
                    id: json!("1"),
                    method: "context".to_string(),
                    params: None,
                },
                response: McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: json!("1"),
                    result: Some(json!({})),
                    error: None,
                },
                elapsed_ms: 100,
                success: true,
                output: None,
            },
            DemoStep {
                name: "Step 2".to_string(),
                capability: "Template Generation",
                request: McpRequest {
                    jsonrpc: "2.0".to_string(),
                    id: json!("2"),
                    method: "template".to_string(),
                    params: None,
                },
                response: McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: json!("2"),
                    result: Some(json!({})),
                    error: None,
                },
                elapsed_ms: 50,
                success: true,
                output: None,
            },
        ];

        let report = DemoReport {
            repository: "/test/multi".to_string(),
            total_time_ms: 150,
            steps,
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 0,
                functions_analyzed: 0,
                avg_complexity: 0.0,
                hotspot_functions: 0,
                quality_score: 0.0,
                tech_debt_hours: 0,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 150,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("1. AST Context Analysis"));
        assert!(output.contains("2. Template Generation"));
        assert!(output.contains("100 ms"));
        assert!(output.contains("50 ms"));
    }

