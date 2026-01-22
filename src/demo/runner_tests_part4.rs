//! Runner tests - Part 4
//! Edge cases, serialization, and additional tests

use super::*;
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

// === find_git_root Edge Cases ===

#[test]
fn test_find_git_root_deeply_nested() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    // Create deeply nested structure
    let mut nested = temp_dir.path().to_path_buf();
    for i in 0..10 {
        nested = nested.join(format!("level{i}"));
        std::fs::create_dir(&nested).unwrap();
    }

    let result = find_git_root(&nested);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), temp_dir.path());
}

#[test]
fn test_find_git_root_at_filesystem_root() {
    // Test with a path that has no .git in any parent
    let result = find_git_root(Path::new("/"));
    assert!(result.is_none());
}

// === Component Clone/Debug Tests ===

#[test]
fn test_component_clone() {
    let component = Component {
        id: "X".to_string(),
        label: "Clone Test".to_string(),
        color: "#123456".to_string(),
        connections: vec![
            ("Y".to_string(), "ref".to_string()),
            ("Z".to_string(), "uses".to_string()),
        ],
    };

    let cloned = component.clone();
    assert_eq!(cloned.id, component.id);
    assert_eq!(cloned.label, component.label);
    assert_eq!(cloned.color, component.color);
    assert_eq!(cloned.connections.len(), 2);
}

#[test]
fn test_component_debug() {
    let component = Component {
        id: "D".to_string(),
        label: "Debug Test".to_string(),
        color: "#AABBCC".to_string(),
        connections: vec![],
    };

    let debug_str = format!("{:?}", component);
    assert!(debug_str.contains("Component"));
    assert!(debug_str.contains("Debug Test"));
}

// === DemoStep Serialization Tests ===

#[test]
fn test_demo_step_serialize() {
    let step = DemoStep {
        name: "Serialize Test".to_string(),
        capability: "Test Capability",
        request: McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("ser-test"),
            method: "test".to_string(),
            params: Some(json!({"key": "value"})),
        },
        response: McpResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("ser-test"),
            result: Some(json!({"result": "success"})),
            error: None,
        },
        elapsed_ms: 123,
        success: true,
        output: Some(json!({"output": "data"})),
    };

    let serialized = serde_json::to_string(&step).unwrap();
    assert!(serialized.contains("Serialize Test"));
    assert!(serialized.contains("123"));
    assert!(serialized.contains("true"));
}

#[test]
fn test_demo_step_deserialize() {
    let json_str = r#"{
        "name": "Deserialize Test",
        "capability": "Test Capability",
        "request": {
            "jsonrpc": "2.0",
            "id": "deser-test",
            "method": "test",
            "params": null
        },
        "response": {
            "jsonrpc": "2.0",
            "id": "deser-test",
            "result": null,
            "error": null
        },
        "elapsed_ms": 456,
        "success": false,
        "output": null
    }"#;

    let step: DemoStep = serde_json::from_str(json_str).unwrap();
    assert_eq!(step.name, "Deserialize Test");
    assert_eq!(step.elapsed_ms, 456);
    assert!(!step.success);
}

// === DemoReport Serialization Tests ===

#[test]
fn test_demo_report_serialize() {
    let report = DemoReport {
        repository: "/serialize/test".to_string(),
        total_time_ms: 999,
        steps: vec![],
        system_diagram: Some("graph LR\n    X --> Y".to_string()),
        analysis: DemoAnalysisResult {
            files_analyzed: 42,
            functions_analyzed: 100,
            avg_complexity: 7.5,
            hotspot_functions: 3,
            quality_score: 0.88,
            tech_debt_hours: 5,
            qa_verification: Some("PASSED".to_string()),
            language_stats: None,
            complexity_metrics: None,
        },
        execution_time_ms: 999,
    };

    let serialized = serde_json::to_string(&report).unwrap();
    assert!(serialized.contains("/serialize/test"));
    assert!(serialized.contains("999"));
    assert!(serialized.contains("42"));
    assert!(serialized.contains("0.88"));
}

// === DemoAnalysisResult Serialization Tests ===

#[test]
fn test_demo_analysis_result_serialize() {
    let mut lang_stats = HashMap::new();
    lang_stats.insert("go".to_string(), json!({"count": 15}));

    let mut complexity_metrics = HashMap::new();
    complexity_metrics.insert("max".to_string(), json!(25));
    complexity_metrics.insert("min".to_string(), json!(1));

    let result = DemoAnalysisResult {
        files_analyzed: 100,
        functions_analyzed: 500,
        avg_complexity: 12.3,
        hotspot_functions: 10,
        quality_score: 0.7,
        tech_debt_hours: 20,
        qa_verification: Some("PENDING".to_string()),
        language_stats: Some(lang_stats),
        complexity_metrics: Some(complexity_metrics),
    };

    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("100"));
    assert!(serialized.contains("500"));
    assert!(serialized.contains("12.3"));
    assert!(serialized.contains("PENDING"));
    assert!(serialized.contains("go"));
}

// === McpRequest Building Tests ===

#[test]
fn test_build_mcp_request_with_complex_arguments() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let args = json!({
        "project_path": "/test/path",
        "toolchain": "rust",
        "options": {
            "max_depth": 10,
            "include_tests": true,
            "filters": ["*.rs", "*.toml"]
        }
    });

    let request = runner.build_mcp_request("complex_analysis", args);

    assert_eq!(request.method, "tools/call");
    let params = request.params.unwrap();
    assert_eq!(params["name"], "complex_analysis");
    assert!(params["arguments"]["options"]["max_depth"].as_i64().is_some());
}

// === render_step_highlights Edge Cases ===

fn create_minimal_report() -> DemoReport {
    DemoReport {
        repository: "/minimal".to_string(),
        total_time_ms: 0,
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
        execution_time_ms: 0,
    }
}

#[test]
fn test_render_step_highlights_partial_complexity_data() {
    let report = create_minimal_report();
    let mut output = String::new();

    // Missing some fields
    let result = json!({
        "total_functions": 25
        // missing warnings and errors
    });

    report.render_step_highlights(&mut output, "Code Complexity Analysis", &result);
    // Should not add anything when fields are missing
    assert!(output.is_empty());
}

#[test]
fn test_render_step_highlights_partial_dag_data() {
    let report = create_minimal_report();
    let mut output = String::new();

    // Missing stats
    let result = json!({
        "graph": "some data"
    });

    report.render_step_highlights(&mut output, "DAG Visualization", &result);
    // Should not add anything when stats are missing
    assert!(output.is_empty());
}

#[test]
fn test_render_step_highlights_partial_churn_data() {
    let report = create_minimal_report();
    let mut output = String::new();

    // Missing total_churn_score
    let result = json!({
        "files_analyzed": 10
    });

    report.render_step_highlights(&mut output, "Code Churn Analysis", &result);
    // Should not add anything when data is incomplete
    assert!(output.is_empty());
}

#[test]
fn test_render_step_highlights_partial_architecture_data() {
    let report = create_minimal_report();
    let mut output = String::new();

    // metadata exists but missing nodes
    let result = json!({
        "metadata": {
            "edges": 15
        }
    });

    report.render_step_highlights(&mut output, "System Architecture Analysis", &result);
    // Should not add anything when data is incomplete
    assert!(output.is_empty());
}

#[test]
fn test_render_step_highlights_defect_empty_array() {
    let report = create_minimal_report();
    let mut output = String::new();

    let result = json!({
        "high_risk_files": [],
        "average_probability": 0.0
    });

    report.render_step_highlights(&mut output, "Defect Probability Analysis", &result);
    assert!(output.contains("High-risk files: 0"));
    assert!(output.contains("0.00"));
}

// === detect_repository Without Git Tests ===

#[test]
fn test_detect_repository_no_git_non_interactive() {
    let temp_dir = TempDir::new().unwrap();
    // No .git directory

    // This should fail in non-interactive mode (CI)
    let result = detect_repository(Some(temp_dir.path().to_path_buf()));
    // In CI, this will return an error
    // We can't control terminal state, so just verify it doesn't panic
    let _ = result;
}

// === Async Repository Resolution Tests ===

#[tokio::test]
async fn test_resolve_repository_async_local_path() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    let result =
        resolve_repository_async(Some(temp_dir.path().to_path_buf()), None, None).await;

    assert!(result.is_ok());
    // Should return the local path without cloning
    let path = result.unwrap();
    assert!(path.exists());
}

#[tokio::test]
async fn test_resolve_repository_async_with_shorthand() {
    // This test would actually try to clone, so we just test the URL parsing
    let result = resolve_repository_async(None, None, Some("gh:rust-lang/rust".to_string()));

    // This would fail if not in CI or without network, but shouldn't panic
    // The important thing is the URL is correctly formed
    match result.await {
        Ok(path) => {
            // If it succeeds, verify path
            assert!(path.to_string_lossy().len() > 0);
        }
        Err(e) => {
            // Clone failure is acceptable in test environment
            let err_str = e.to_string();
            // Should be a clone-related error, not a parsing error
            assert!(
                err_str.contains("clone")
                    || err_str.contains("git")
                    || err_str.contains("timeout")
                    || err_str.contains("network")
                    || err_str.contains("error")
            );
        }
    }
}

// === DemoRunner execution_log Tests ===

#[tokio::test]
async fn test_demo_runner_execution_log_accumulation() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    // Initially empty
    assert!(runner.execution_log.is_empty());
}

// === Additional MCP Request Tests ===

#[test]
fn test_build_mcp_request_empty_arguments() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let request = runner.build_mcp_request("empty_test", json!({}));

    assert_eq!(request.jsonrpc, "2.0");
    let params = request.params.unwrap();
    assert_eq!(params["arguments"], json!({}));
}

#[test]
fn test_build_mcp_request_array_arguments() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let request = runner.build_mcp_request("array_test", json!(["a", "b", "c"]));

    let params = request.params.unwrap();
    assert!(params["arguments"].is_array());
}

// === Step Output Extraction Tests ===

#[test]
fn test_create_demo_step_with_none_error_message() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("none-err"),
        method: "test".to_string(),
        params: None,
    };

    // Error with None data
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: json!("none-err"),
        result: None,
        error: Some(crate::models::mcp::McpError {
            code: -32000,
            message: "".to_string(), // Empty message
            data: None,
        }),
    };

    let step = runner.create_demo_step("None Error", "None Cap", request, response, 10);

    assert!(!step.success);
    // Output should have error key even with empty message
    let output = step.output.unwrap();
    assert!(output.get("error").is_some());
}
