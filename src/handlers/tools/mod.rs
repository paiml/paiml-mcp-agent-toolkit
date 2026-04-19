#![cfg_attr(coverage_nightly, coverage(off))]
// Tools handlers - split for file health (CB-040)
include!("core_tools.rs");
include!("extended_tools.rs");

#[cfg(test)]
mod tests {
    // Tests from original file
}

// ============================================================================
// R22-1 / D101 regression tests — cwd-exfiltration parity fix
// ============================================================================
//
// Proves that the live `src/handlers/tools/` dispatcher now rejects missing,
// null, and empty/whitespace `project_path` with JSON-RPC `-32602` across
// every analysis handler that previously fell back to
// `std::env::current_dir()`. Mirrors the R21-5 / D99 test layout in
// `src/agent/mcp_server_tests.rs` (PR #371).

#[cfg(test)]
mod r22_1_d101_cwd_guard_tests {
    use super::*;
    use serde_json::json;

    fn assert_invalid_params(response: &McpResponse, context: &str) {
        assert!(
            response.error.is_some(),
            "{context}: expected an error response, got success: {response:?}"
        );
        let err = response.error.as_ref().unwrap();
        assert_eq!(
            err.code, -32602,
            "{context}: expected JSON-RPC -32602 (Invalid params), got {}: message={}",
            err.code, err.message
        );
        assert!(
            err.message.contains("project_path"),
            "{context}: error message should name the offending field: {}",
            err.message
        );
    }

    // --- handle_analyze_complexity ----------------------------------------

    #[tokio::test]
    async fn analyze_complexity_rejects_missing_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_complexity / missing");
    }

    #[tokio::test]
    async fn analyze_complexity_rejects_null_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({ "project_path": null })).await;
        assert_invalid_params(&response, "analyze_complexity / null");
    }

    #[tokio::test]
    async fn analyze_complexity_rejects_empty_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_complexity / empty");
    }

    #[tokio::test]
    async fn analyze_complexity_rejects_whitespace_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({ "project_path": "   " })).await;
        assert_invalid_params(&response, "analyze_complexity / whitespace");
    }

    // --- handle_analyze_code_churn ----------------------------------------

    #[tokio::test]
    async fn analyze_code_churn_rejects_missing_project_path() {
        let response = handle_analyze_code_churn(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_code_churn / missing");
    }

    #[tokio::test]
    async fn analyze_code_churn_rejects_empty_project_path() {
        let response = handle_analyze_code_churn(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_code_churn / empty");
    }

    // --- handle_analyze_dag -----------------------------------------------

    #[tokio::test]
    async fn analyze_dag_rejects_missing_project_path() {
        let response = handle_analyze_dag(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_dag / missing");
    }

    #[tokio::test]
    async fn analyze_dag_rejects_empty_project_path() {
        let response = handle_analyze_dag(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_dag / empty");
    }

    // --- handle_generate_context ------------------------------------------

    #[tokio::test]
    async fn generate_context_rejects_missing_project_path() {
        let response = handle_generate_context(json!(1), json!({})).await;
        assert_invalid_params(&response, "generate_context / missing");
    }

    #[tokio::test]
    async fn generate_context_rejects_empty_project_path() {
        let response = handle_generate_context(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "generate_context / empty");
    }

    // --- handle_analyze_system_architecture -------------------------------

    #[tokio::test]
    async fn analyze_system_architecture_rejects_missing_project_path() {
        let response = handle_analyze_system_architecture(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_system_architecture / missing");
    }

    #[tokio::test]
    async fn analyze_system_architecture_rejects_empty_project_path() {
        let response =
            handle_analyze_system_architecture(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_system_architecture / empty");
    }

    // --- require_project_path helper --------------------------------------

    #[test]
    fn require_project_path_accepts_nonempty() {
        let out = require_project_path(Some("/tmp/x".to_string())).unwrap();
        assert_eq!(out, std::path::PathBuf::from("/tmp/x"));
    }

    #[test]
    fn require_project_path_rejects_none() {
        let err = require_project_path(None).unwrap_err();
        assert!(err.contains("project_path"), "{err}");
        assert!(err.contains("D101"), "{err}");
    }

    #[test]
    fn require_project_path_rejects_empty() {
        let err = require_project_path(Some(String::new())).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }

    #[test]
    fn require_project_path_rejects_whitespace() {
        let err = require_project_path(Some("  \n\t ".to_string())).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }
}
