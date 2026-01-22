//! Runner tests - Part 1
//! DemoStep, DemoReport, DemoAnalysisResult, Component, and Repository tests

use super::*;
use std::collections::HashMap;
use tempfile::TempDir;

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

// === Repository Resolution Tests ===

#[test]
fn test_try_local_path_exists() {
    let temp_dir = TempDir::new().unwrap();
    let path_str = temp_dir.path().to_string_lossy().to_string();

    let result = try_local_path(&path_str);
    // try_local_path returns Some when path exists, but detect_repository
    // returns Err if not a git repository
    assert!(result.is_some());
    // The path exists but isn't a git repo, so detect_repository returns Err
    assert!(result.unwrap().is_err());
}

#[test]
fn test_try_local_path_not_exists() {
    let result = try_local_path("/nonexistent/path/that/doesnt/exist/at/all");
    assert!(result.is_none());
}

#[test]
fn test_try_github_shorthand() {
    let result = try_github_shorthand("gh:owner/repo");
    assert!(result.is_some());
    let path = result.unwrap().unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
    assert!(path.to_string_lossy().contains("owner/repo"));
}

#[test]
fn test_try_github_shorthand_not_shorthand() {
    let result = try_github_shorthand("owner/repo");
    assert!(result.is_none());
}

#[test]
fn test_try_github_url_https() {
    let result = try_github_url("https://github.com/owner/repo");
    assert!(result.is_some());
    let path = result.unwrap().unwrap();
    assert_eq!(path.to_string_lossy(), "https://github.com/owner/repo");
}

#[test]
fn test_try_github_url_git() {
    let result = try_github_url("git@github.com:owner/repo");
    assert!(result.is_some());
    let path = result.unwrap().unwrap();
    assert_eq!(path.to_string_lossy(), "git@github.com:owner/repo");
}

#[test]
fn test_try_github_url_not_github() {
    let result = try_github_url("https://gitlab.com/owner/repo");
    assert!(result.is_none());
}

#[test]
fn test_try_owner_repo_format() {
    let result = try_owner_repo_format("owner/repo");
    assert!(result.is_some());
    let path = result.unwrap().unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
    assert!(path.to_string_lossy().contains("owner/repo"));
}

#[test]
fn test_try_owner_repo_format_with_dot() {
    let result = try_owner_repo_format("owner.name/repo");
    assert!(result.is_none());
}

#[test]
fn test_try_owner_repo_format_no_slash() {
    let result = try_owner_repo_format("owner-repo");
    assert!(result.is_none());
}

// === find_git_root Tests ===

#[test]
fn test_find_git_root_direct() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    let result = find_git_root(temp_dir.path());
    assert!(result.is_some());
    assert_eq!(result.unwrap(), temp_dir.path());
}

#[test]
fn test_find_git_root_parent() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();

    let result = find_git_root(&sub_dir);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), temp_dir.path());
}

#[test]
fn test_find_git_root_not_found() {
    let temp_dir = TempDir::new().unwrap();
    // No .git directory created

    let result = find_git_root(temp_dir.path());
    assert!(result.is_none());
}

// === get_canonical_path Tests ===

#[test]
fn test_get_canonical_path_some() {
    let temp_dir = TempDir::new().unwrap();
    let result = get_canonical_path(Some(temp_dir.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn test_get_canonical_path_none() {
    let result = get_canonical_path(None);
    // Should return current directory
    assert!(result.is_ok());
}

#[test]
fn test_get_canonical_path_nonexistent() {
    let result = get_canonical_path(Some(PathBuf::from("/nonexistent/path/xyz")));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

// === resolve_repository Tests ===

#[test]
fn test_resolve_repository_with_url() {
    let result = resolve_repository(
        None,
        Some("https://github.com/owner/repo".to_string()),
        None,
    );
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
}

#[test]
fn test_resolve_repository_with_repo_shorthand() {
    let result = resolve_repository(None, None, Some("gh:owner/repo".to_string()));
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
}

#[test]
fn test_resolve_repository_with_local_path() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

    let result = resolve_repository(Some(temp_dir.path().to_path_buf()), None, None);
    assert!(result.is_ok());
}

// === is_interactive_environment Tests ===

#[test]
fn test_is_interactive_environment_in_ci() {
    // In CI, this should return false (CI env var is typically set)
    // We can't easily control the environment, but we can check it runs
    let _result = is_interactive_environment();
    // Just verify it doesn't panic
}
