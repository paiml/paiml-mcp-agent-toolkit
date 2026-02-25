    // === DemoStep Tests ===

    #[test]
    fn test_demo_step_creation() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("test-1"),
            method: "test".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("test-1"),
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        let step = DemoStep {
            name: "Test Step".to_string(),
            capability: "Test Capability",
            request,
            response,
            elapsed_ms: 100,
            success: true,
            output: Some(json!({"test": "data"})),
        };

        assert_eq!(step.name, "Test Step");
        assert_eq!(step.capability, "Test Capability");
        assert_eq!(step.elapsed_ms, 100);
        assert!(step.success);
        assert!(step.output.is_some());
    }

    #[test]
    fn test_demo_step_with_error() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("test-error"),
            method: "failing_test".to_string(),
            params: None,
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("test-error"),
            result: None,
            error: Some(crate::models::mcp::McpError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
        };

        let step = DemoStep {
            name: "Error Step".to_string(),
            capability: "Error Capability",
            request,
            response,
            elapsed_ms: 50,
            success: false,
            output: Some(json!({"error": "Invalid request"})),
        };

        assert!(!step.success);
        assert_eq!(step.name, "Error Step");
    }

    // === DemoReport Tests ===

    #[test]
    fn test_demo_report_creation() {
        let report = DemoReport {
            repository: "/test/repo".to_string(),
            total_time_ms: 5000,
            steps: Vec::new(),
            system_diagram: Some("graph TD\n    A --> B".to_string()),
            analysis: DemoAnalysisResult {
                files_analyzed: 10,
                functions_analyzed: 50,
                avg_complexity: 5.5,
                hotspot_functions: 2,
                quality_score: 0.9,
                tech_debt_hours: 4,
                qa_verification: Some("PASSED".to_string()),
                language_stats: Some(HashMap::new()),
                complexity_metrics: Some(HashMap::new()),
            },
            execution_time_ms: 5000,
        };

        assert_eq!(report.repository, "/test/repo");
        assert_eq!(report.total_time_ms, 5000);
        assert!(report.system_diagram.is_some());
        assert_eq!(report.analysis.files_analyzed, 10);
    }

    #[test]
    fn test_demo_report_render_cli() {
        let report = DemoReport {
            repository: "/test/repo".to_string(),
            total_time_ms: 1000,
            steps: vec![],
            system_diagram: Some("graph TD\n    A --> B".to_string()),
            analysis: DemoAnalysisResult {
                files_analyzed: 5,
                functions_analyzed: 20,
                avg_complexity: 4.0,
                hotspot_functions: 1,
                quality_score: 0.85,
                tech_debt_hours: 2,
                qa_verification: None,
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 1000,
        };

        let output = report.render(ExecutionMode::Cli);
        assert!(output.contains("PAIML MCP Agent Toolkit Demo Complete"));
        assert!(output.contains("/test/repo"));
        assert!(output.contains("1000 ms"));
        assert!(output.contains("mermaid"));
    }

    #[test]
    fn test_demo_report_render_mcp() {
        let report = DemoReport {
            repository: "/test/repo".to_string(),
            total_time_ms: 500,
            steps: vec![],
            system_diagram: None,
            analysis: DemoAnalysisResult {
                files_analyzed: 3,
                functions_analyzed: 10,
                avg_complexity: 3.0,
                hotspot_functions: 0,
                quality_score: 0.95,
                tech_debt_hours: 1,
                qa_verification: Some("PASSED".to_string()),
                language_stats: None,
                complexity_metrics: None,
            },
            execution_time_ms: 500,
        };

        let output = report.render(ExecutionMode::Mcp);
        // MCP mode should produce JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["repository"], "/test/repo");
    }

    // === DemoAnalysisResult Tests ===

    #[test]
    fn test_demo_analysis_result_defaults() {
        let result = DemoAnalysisResult {
            files_analyzed: 0,
            functions_analyzed: 0,
            avg_complexity: 0.0,
            hotspot_functions: 0,
            quality_score: 0.0,
            tech_debt_hours: 0,
            qa_verification: None,
            language_stats: None,
            complexity_metrics: None,
        };

        assert_eq!(result.files_analyzed, 0);
        assert_eq!(result.quality_score, 0.0);
        assert!(result.qa_verification.is_none());
    }

    #[test]
    fn test_demo_analysis_result_with_stats() {
        let mut lang_stats = HashMap::new();
        lang_stats.insert("rust".to_string(), json!({"files": 10, "lines": 1000}));
        lang_stats.insert("python".to_string(), json!({"files": 5, "lines": 500}));

        let result = DemoAnalysisResult {
            files_analyzed: 15,
            functions_analyzed: 100,
            avg_complexity: 8.5,
            hotspot_functions: 5,
            quality_score: 0.75,
            tech_debt_hours: 12,
            qa_verification: Some("PASSED".to_string()),
            language_stats: Some(lang_stats),
            complexity_metrics: Some(HashMap::new()),
        };

        assert_eq!(result.files_analyzed, 15);
        assert!(result.language_stats.is_some());
        assert_eq!(result.language_stats.as_ref().unwrap().len(), 2);
    }

    // === Component Tests ===

    #[test]
    fn test_component_structure() {
        let component = Component {
            id: "A".to_string(),
            label: "Test Component".to_string(),
            color: "#FF0000".to_string(),
            connections: vec![("B".to_string(), "uses".to_string())],
        };

        assert_eq!(component.id, "A");
        assert_eq!(component.label, "Test Component");
        assert_eq!(component.color, "#FF0000");
        assert_eq!(component.connections.len(), 1);
    }
