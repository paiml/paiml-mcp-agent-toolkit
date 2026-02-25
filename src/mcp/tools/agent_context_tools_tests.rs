// Tests for agent_context_tools
// Split from agent_context_tools.rs for maintainability

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_code_schema() {
        let schema = QueryCodeTool::schema();
        assert_eq!(schema["name"], "pmat_query_code");
        assert!(schema["parameters"]["properties"]["query"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["query"]));
    }

    #[test]
    fn test_get_function_schema() {
        let schema = GetFunctionTool::schema();
        assert_eq!(schema["name"], "pmat_get_function");
        assert!(schema["parameters"]["properties"]["function_id"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["function_id"]));
    }

    #[test]
    fn test_find_similar_schema() {
        let schema = FindSimilarTool::schema();
        assert_eq!(schema["name"], "pmat_find_similar");
        assert!(schema["parameters"]["properties"]["function_id"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["function_id"]));
    }

    #[test]
    fn test_index_stats_schema() {
        let schema = IndexStatsTool::schema();
        assert_eq!(schema["name"], "pmat_index_stats");
        assert!(schema["parameters"]["properties"]["rebuild"].is_object());
    }

    #[test]
    fn test_all_tool_names() {
        assert_eq!(QueryCodeTool::schema()["name"], "pmat_query_code");
        assert_eq!(GetFunctionTool::schema()["name"], "pmat_get_function");
        assert_eq!(FindSimilarTool::schema()["name"], "pmat_find_similar");
        assert_eq!(IndexStatsTool::schema()["name"], "pmat_index_stats");
    }

    #[test]
    fn test_parse_function_id_valid() {
        let (file, func) = parse_function_id("src/handlers/auth.rs::handle_login").unwrap();
        assert_eq!(file, "src/handlers/auth.rs");
        assert_eq!(func, "handle_login");
    }

    #[test]
    fn test_parse_function_id_nested() {
        let (file, func) = parse_function_id("src/foo/bar.rs::baz::qux").unwrap();
        assert_eq!(file, "src/foo/bar.rs::baz");
        assert_eq!(func, "qux");
    }

    #[test]
    fn test_parse_function_id_invalid_no_separator() {
        let result = parse_function_id("no_separator");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid function_id format"));
    }

    #[test]
    fn test_parse_function_id_invalid_empty_parts() {
        let result = parse_function_id("::function_only");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_index_manager_new() {
        let manager = IndexManager::new(PathBuf::from("/tmp/test"));
        let guard = manager.index.read().await;
        assert!(guard.is_none());
    }
}
