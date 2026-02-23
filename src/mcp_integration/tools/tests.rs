#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use crate::agents::registry::AgentRegistry;
    use crate::mcp::tools::agent_context_tools::IndexManager;
    use crate::mcp_integration::McpTool;
    use crate::mcp_integration::tools::{
        AnalyzeTool, FindSimilarToolAdapter, GetFunctionToolAdapter, IndexStatsToolAdapter,
        OrchestrateTool, QualityGateTool, QueryCodeToolAdapter, TransformTool, ValidateTool,
    };
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn test_analyze_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze");
        assert!(metadata.description.contains("quality metrics"));
    }

    #[test]
    fn test_transform_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "transform");
        assert!(metadata.description.contains("AST manipulation"));
    }

    #[test]
    fn test_validate_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "validate");
        assert!(metadata.description.contains("quality standards"));
    }

    #[test]
    fn test_orchestrate_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "orchestrate");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_quality_gate_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "quality_gate");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_analyze_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);
        assert!(tool.analyzer.is_none());
    }

    #[test]
    fn test_transform_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);
        assert!(tool.transformer.is_none());
    }

    #[test]
    fn test_validate_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);
        assert!(tool.validator.is_none());
    }

    #[test]
    fn test_orchestrate_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);
        // Just verify construction works
        assert!(!tool.metadata().name.is_empty());
    }

    #[test]
    fn test_quality_gate_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);
        assert!(!tool.metadata().name.is_empty());
    }

    #[test]
    fn test_analyze_tool_schema_has_required_fields() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["language"].is_object());
        assert_eq!(schema["required"], json!(["code", "language"]));
    }

    #[test]
    fn test_transform_tool_schema_has_required_fields() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["transformation"].is_object());
    }

    #[test]
    fn test_validate_tool_schema_has_required_fields() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
    }

    #[test]
    fn test_orchestrate_tool_schema() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["workflow"].is_object());
    }

    #[test]
    fn test_quality_gate_tool_schema() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["language"].is_object());
    }

    #[actix_rt::test]
    async fn test_analyze_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);

        let result = tool.execute(json!({ "language": "rust" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_analyze_tool_missing_language() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);

        let result = tool.execute(json!({ "code": "fn main() {}" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("language"));
    }

    #[actix_rt::test]
    async fn test_transform_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);

        let result = tool.execute(json!({ "transform": "refactor" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_transform_tool_missing_transform() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);

        let result = tool.execute(json!({ "code": "fn main() {}" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("transform"));
    }

    #[actix_rt::test]
    async fn test_validate_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_orchestrate_tool_missing_workflow() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("workflow"));
    }

    #[actix_rt::test]
    async fn test_quality_gate_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_analyze_tool_no_actor() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);

        let result = tool
            .execute(json!({
                "code": "fn main() {}",
                "language": "rust"
            }))
            .await;

        // Should fail because no actor is initialized
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not initialized"));
    }

    #[actix_rt::test]
    async fn test_transform_tool_no_actor() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);

        let result = tool
            .execute(json!({
                "code": "fn main() {}",
                "transformation": "refactor"
            }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not initialized"));
    }

    #[actix_rt::test]
    async fn test_validate_tool_no_actor() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);

        let result = tool.execute(json!({ "code": "fn main() {}" })).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not initialized"));
    }

    #[actix_rt::test]
    async fn test_orchestrate_tool_missing_workflow_name() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);

        // Workflow without name should still work (uses default)
        let result = tool
            .execute(json!({
                "workflow": {
                    "steps": []
                }
            }))
            .await;

        // Should either succeed or fail with workflow-related error
        // The orchestrator doesn't check for actor initialization
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(
                err.message.contains("workflow") || err.message.contains("Workflow"),
                "Error should be workflow-related: {}",
                err.message
            );
        }
    }

    #[test]
    fn test_tool_metadata_descriptions_not_empty() {
        let registry = Arc::new(AgentRegistry::new());

        let analyze = AnalyzeTool::new(registry.clone());
        assert!(!analyze.metadata().description.is_empty());

        let transform = TransformTool::new(registry.clone());
        assert!(!transform.metadata().description.is_empty());

        let validate = ValidateTool::new(registry.clone());
        assert!(!validate.metadata().description.is_empty());

        let orchestrate = OrchestrateTool::new(registry.clone());
        assert!(!orchestrate.metadata().description.is_empty());

        let quality_gate = QualityGateTool::new(registry);
        assert!(!quality_gate.metadata().description.is_empty());
    }

    #[test]
    fn test_all_tools_have_object_schema() {
        let registry = Arc::new(AgentRegistry::new());

        let tools: Vec<Box<dyn McpTool + Send + Sync>> = vec![
            Box::new(AnalyzeTool::new(registry.clone())),
            Box::new(TransformTool::new(registry.clone())),
            Box::new(ValidateTool::new(registry.clone())),
            Box::new(OrchestrateTool::new(registry.clone())),
            Box::new(QualityGateTool::new(registry)),
        ];

        for tool in tools {
            let metadata = tool.metadata();
            assert_eq!(metadata.input_schema["type"], "object");
        }
    }

    // ============================================================
    // Agent Context Tool Adapter Tests (PMAT-470)
    // ============================================================

    #[test]
    fn test_query_code_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = QueryCodeToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_query_code");
        assert!(metadata.description.contains("Search code functions"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_get_function_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = GetFunctionToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_get_function");
        assert!(metadata.description.contains("function"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_find_similar_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = FindSimilarToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_find_similar");
        assert!(metadata.description.contains("similar"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_index_stats_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = IndexStatsToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_index_stats");
        assert!(metadata.description.contains("statistics"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_agent_context_adapters_have_object_schema() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));

        let tools: Vec<Box<dyn McpTool + Send + Sync>> = vec![
            Box::new(QueryCodeToolAdapter::new(manager.clone())),
            Box::new(GetFunctionToolAdapter::new(manager.clone())),
            Box::new(FindSimilarToolAdapter::new(manager.clone())),
            Box::new(IndexStatsToolAdapter::new(manager)),
        ];

        for tool in tools {
            let metadata = tool.metadata();
            assert_eq!(metadata.input_schema["type"], "object");
            assert!(!metadata.name.is_empty());
            assert!(!metadata.description.is_empty());
        }
    }
}
