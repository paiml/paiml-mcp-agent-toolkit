// === Quality Gates, Complexity, and Format Tests ===

#[tokio::test]
async fn test_mcp_server_handle_run_quality_gates() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let params = json!({
        "target_path": path_str
    });

    let result = server.handle_run_quality_gates(&params).await.unwrap();
    assert!(result.get("content").is_some());
}

#[tokio::test]
async fn test_handle_run_quality_gates_with_default_path() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    // When target_path is not provided, it defaults to "."
    let params = json!({});

    // This should not error, but use default "." path
    let result = server.handle_run_quality_gates(&params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mcp_server_handle_analyze_complexity() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let params = json!({
        "file_path": "/test/file.rs"
    });

    let result = server.handle_analyze_complexity(&params).await.unwrap();
    assert!(result.get("content").is_some());

    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty());

    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("Complexity Analysis"));
}

#[tokio::test]
async fn test_handle_analyze_complexity_with_default_path() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    // When file_path is not provided, it defaults to "."
    let params = json!({});

    let result = server.handle_analyze_complexity(&params).await.unwrap();
    let content = result["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("Complexity Analysis for ."));
}

#[tokio::test]
async fn test_mcp_server_format_complexity_analysis_results() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let result = server.format_complexity_analysis_results("/test/path");
    assert!(result.contains("Complexity Analysis"));
    assert!(result.contains("/test/path"));
    assert!(result.contains("Files analyzed"));
    assert!(result.contains("Toyota Way standards"));
}

#[tokio::test]
async fn test_mcp_server_format_quality_gate_results_passed() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let output = QualityGateOutput {
        passed: true,
        results: vec![crate::services::quality_gate_service::QualityCheckResult {
            check: "complexity".to_string(),
            passed: true,
            message: "All good".to_string(),
            violations: vec![],
        }],
        summary: crate::services::quality_gate_service::QualitySummary {
            total_checks: 1,
            passed_checks: 1,
            failed_checks: 0,
            total_violations: 0,
            error_count: 0,
            warning_count: 0,
        },
    };

    let result = server.format_quality_gate_results("/test/path", &output);
    assert!(result.contains("Quality Gate Results"));
    assert!(result.contains("PASSED"));
    assert!(result.contains("/test/path"));
}

#[tokio::test]
async fn test_mcp_server_format_quality_gate_results_failed() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let output = QualityGateOutput {
        passed: false,
        results: vec![crate::services::quality_gate_service::QualityCheckResult {
            check: "complexity".to_string(),
            passed: false,
            message: "Complexity too high".to_string(),
            violations: vec![],
        }],
        summary: crate::services::quality_gate_service::QualitySummary {
            total_checks: 1,
            passed_checks: 0,
            failed_checks: 1,
            total_violations: 0,
            error_count: 0,
            warning_count: 0,
        },
    };

    let result = server.format_quality_gate_results("/test/path", &output);
    assert!(result.contains("Quality Gate Results"));
    assert!(result.contains("FAILED"));
    assert!(result.contains("Failed Checks"));
}

#[tokio::test]
async fn test_format_failed_checks_with_multiple_failures() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let output = QualityGateOutput {
        passed: false,
        results: vec![
            crate::services::quality_gate_service::QualityCheckResult {
                check: "complexity".to_string(),
                passed: false,
                message: "High complexity detected".to_string(),
                violations: vec![],
            },
            crate::services::quality_gate_service::QualityCheckResult {
                check: "satd".to_string(),
                passed: false,
                message: "Too many TODOs".to_string(),
                violations: vec![],
            },
            crate::services::quality_gate_service::QualityCheckResult {
                check: "lint".to_string(),
                passed: true,
                message: "No lint issues".to_string(),
                violations: vec![],
            },
        ],
        summary: crate::services::quality_gate_service::QualitySummary {
            total_checks: 3,
            passed_checks: 1,
            failed_checks: 2,
            total_violations: 0,
            error_count: 0,
            warning_count: 0,
        },
    };

    let mut result_text = String::new();
    server.format_failed_checks(&mut result_text, &output);

    assert!(result_text.contains("Failed Checks: 2/3"));
    assert!(result_text.contains("complexity"));
    assert!(result_text.contains("satd"));
}

#[tokio::test]
async fn test_format_failed_checks_with_no_failures() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let output = QualityGateOutput {
        passed: true,
        results: vec![crate::services::quality_gate_service::QualityCheckResult {
            check: "complexity".to_string(),
            passed: true,
            message: "All good".to_string(),
            violations: vec![],
        }],
        summary: crate::services::quality_gate_service::QualitySummary {
            total_checks: 1,
            passed_checks: 1,
            failed_checks: 0,
            total_violations: 0,
            error_count: 0,
            warning_count: 0,
        },
    };

    let mut result_text = String::new();
    server.format_failed_checks(&mut result_text, &output);

    // Should not contain failure message
    assert!(!result_text.contains("Failed Checks"));
}

#[tokio::test]
async fn test_format_quality_summary() {
    let config = AgentConfig::default();
    let server = ClaudeCodeAgentMcpServer::new(config);

    let output = QualityGateOutput {
        passed: true,
        results: vec![],
        summary: crate::services::quality_gate_service::QualitySummary {
            total_checks: 5,
            passed_checks: 4,
            failed_checks: 1,
            total_violations: 3,
            error_count: 1,
            warning_count: 2,
        },
    };

    let mut result_text = String::new();
    server.format_quality_summary(&mut result_text, &output);

    assert!(result_text.contains("Summary"));
    assert!(result_text.contains("Total Checks: 5"));
    assert!(result_text.contains("Passed: 4"));
    assert!(result_text.contains("Failed: 1"));
}
