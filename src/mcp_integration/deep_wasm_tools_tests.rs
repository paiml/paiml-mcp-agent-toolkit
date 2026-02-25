// Tests for Deep WASM tools

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_wasm_analyze_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = DeepWasmAnalyzeTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "deep_wasm_analyze");
        assert!(metadata.description.contains("pipeline"));
    }

    #[test]
    fn test_deep_wasm_query_mapping_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = DeepWasmQueryMappingTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "deep_wasm_query_mapping");
        assert!(metadata.description.contains("bidirectional"));
    }

    #[test]
    fn test_all_tools_have_schemas() {
        let registry = Arc::new(AgentRegistry::new());

        let tools: Vec<Box<dyn McpTool>> = vec![
            Box::new(DeepWasmAnalyzeTool::new(registry.clone())),
            Box::new(DeepWasmQueryMappingTool::new(registry.clone())),
            Box::new(DeepWasmTraceExecutionTool::new(registry.clone())),
            Box::new(DeepWasmCompareOptimizationsTool::new(registry.clone())),
            Box::new(DeepWasmDetectIssuesTool::new(registry)),
        ];

        for tool in tools {
            let metadata = tool.metadata();
            assert!(!metadata.name.is_empty());
            assert!(!metadata.description.is_empty());
            assert!(metadata.input_schema.is_object());
        }
    }
}
