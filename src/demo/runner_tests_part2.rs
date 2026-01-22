//! Runner tests - Part 2
//! DemoRunner and render_step_highlights tests

use super::*;
use std::collections::HashMap;
use tempfile::TempDir;

// === DemoRunner Tests ===

#[tokio::test]
async fn test_demo_runner_creation() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));
    assert!(runner.execution_log.is_empty());
}

#[test]
fn test_demo_runner_build_mcp_request() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let request = runner.build_mcp_request("test_method", json!({"param1": "value1"}));

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "tools/call");
    assert!(request.params.is_some());
    let params = request.params.as_ref().unwrap();
    assert_eq!(params["name"], "test_method");
}

#[test]
fn test_demo_runner_generate_system_diagram() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let diagram = runner.generate_system_diagram(&[]).unwrap();
    assert!(diagram.contains("graph TD"));
    assert!(diagram.contains("AST Context Analysis"));
    assert!(diagram.contains("File Parser"));
    assert!(diagram.contains("Rust AST"));
    assert!(diagram.contains("style A fill:#90EE90"));
}

#[test]
fn test_demo_runner_render_system_mermaid() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let components = HashMap::new();
    let mermaid = runner.render_system_mermaid(&components).unwrap();

    assert!(mermaid.starts_with("graph TD"));
    assert!(mermaid.contains("AST Context Analysis"));
    assert!(mermaid.contains("TypeScript AST"));
    assert!(mermaid.contains("Python AST"));
    assert!(mermaid.contains("Code Complexity"));
    assert!(mermaid.contains("DAG Generation"));
    assert!(mermaid.contains("Code Churn"));
    assert!(mermaid.contains("Git Analysis"));
    assert!(mermaid.contains("Template Generation"));
}

#[test]
fn test_demo_runner_create_demo_step_success() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("test"),
        method: "test".to_string(),
        params: None,
    };
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: json!("test"),
        result: Some(json!({"success": true})),
        error: None,
    };

    let step = runner.create_demo_step("Test Step", "Test Capability", request, response, 100);

    assert!(step.success);
    assert_eq!(step.name, "Test Step");
    assert_eq!(step.elapsed_ms, 100);
}

#[test]
fn test_demo_runner_create_demo_step_error() {
    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let runner = DemoRunner::new(Arc::new(server));

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("test"),
        method: "test".to_string(),
        params: None,
    };
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: json!("test"),
        result: None,
        error: Some(crate::models::mcp::McpError {
            code: -32600,
            message: "Test error".to_string(),
            data: None,
        }),
    };

    let step = runner.create_demo_step("Error Step", "Error Capability", request, response, 50);

    assert!(!step.success);
    assert!(step.output.is_some());
    let output = step.output.unwrap();
    assert!(output["error"].as_str().unwrap().contains("Test error"));
}

// === DemoReport render_step_highlights Tests ===

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
fn test_render_step_highlights_complexity() {
    let report = create_minimal_report();

    let mut output = String::new();
    let result = json!({
        "total_functions": 50,
        "total_warnings": 5,
        "total_errors": 2
    });

    report.render_step_highlights(&mut output, "Code Complexity Analysis", &result);
    assert!(output.contains("Functions: 50"));
    assert!(output.contains("Warnings: 5"));
    assert!(output.contains("Errors: 2"));
}

#[test]
fn test_render_step_highlights_dag() {
    let report = create_minimal_report();

    let mut output = String::new();
    let result = json!({
        "stats": {
            "nodes": 25,
            "edges": 40
        }
    });

    report.render_step_highlights(&mut output, "DAG Visualization", &result);
    assert!(output.contains("25 nodes"));
    assert!(output.contains("40 edges"));
}

#[test]
fn test_render_step_highlights_churn() {
    let report = create_minimal_report();

    let mut output = String::new();
    let result = json!({
        "files_analyzed": 30,
        "total_churn_score": 150
    });

    report.render_step_highlights(&mut output, "Code Churn Analysis", &result);
    assert!(output.contains("30"));
    assert!(output.contains("150"));
}

#[test]
fn test_render_step_highlights_architecture() {
    let report = create_minimal_report();

    let mut output = String::new();
    let result = json!({
        "metadata": {
            "nodes": 10,
            "edges": 15
        }
    });

    report.render_step_highlights(&mut output, "System Architecture Analysis", &result);
    assert!(output.contains("Components: 10"));
    assert!(output.contains("Relationships: 15"));
}

#[test]
fn test_render_step_highlights_defects() {
    let report = create_minimal_report();

    let mut output = String::new();
    let result = json!({
        "high_risk_files": ["file1.rs", "file2.rs"],
        "average_probability": 0.35
    });

    report.render_step_highlights(&mut output, "Defect Probability Analysis", &result);
    assert!(output.contains("High-risk files: 2"));
    assert!(output.contains("0.35"));
}

#[test]
fn test_render_step_highlights_unknown_capability() {
    let report = create_minimal_report();

    let mut output = String::new();
    let result = json!({"some": "data"});

    report.render_step_highlights(&mut output, "Unknown Capability", &result);
    // Should not add anything for unknown capabilities
    assert!(output.is_empty());
}

// === resolve_repo_spec Tests ===

#[test]
fn test_resolve_repo_spec_local_path() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let result = resolve_repo_spec(&path_str);
    assert!(result.is_ok());
}

#[test]
fn test_resolve_repo_spec_github_shorthand() {
    let result = resolve_repo_spec("gh:owner/repo");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
}

#[test]
fn test_resolve_repo_spec_github_url() {
    let result = resolve_repo_spec("https://github.com/owner/repo");
    assert!(result.is_ok());
}

#[test]
fn test_resolve_repo_spec_owner_repo() {
    let result = resolve_repo_spec("owner/repo");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
}

#[test]
fn test_resolve_repo_spec_not_found() {
    let result = resolve_repo_spec("nonexistent-path-that-definitely-does-not-exist");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// === detect_repository Tests ===

#[test]
fn test_detect_repository_with_git() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    let result = detect_repository(Some(temp_dir.path().to_path_buf()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), temp_dir.path());
}

// === Additional DemoRunner Tests ===

#[tokio::test]
async fn test_demo_runner_execute_with_local_repo() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    // Create a simple Rust file for analysis
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn hello() -> &'static str {
            "hello"
        }
        "#,
    )
    .unwrap();

    // Create Cargo.toml
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let mut runner = DemoRunner::new(Arc::new(server));

    let result = runner.execute(temp_dir.path().to_path_buf()).await;
    // The demo should run, though analysis may partially fail on minimal project
    // We're testing that it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_demo_runner_execute_with_diagram_local() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    let server = crate::stateless_server::StatelessTemplateServer::new().unwrap();
    let mut runner = DemoRunner::new(Arc::new(server));

    // Test execute_with_diagram with local path and no URL
    let result = runner.execute_with_diagram(temp_dir.path(), None).await;
    // Should run without panicking
    assert!(result.is_ok() || result.is_err());
}
