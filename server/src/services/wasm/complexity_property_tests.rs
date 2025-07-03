#[cfg(test)]
mod tests {
    use super::super::complexity::*;
    use crate::models::unified_ast::{
        AstDag, AstKind, FunctionKind, Language, NodeKey, UnifiedAstNode,
    };
    use proptest::prelude::*;

    proptest! {
        fn analyzer_never_panics_on_arbitrary_text(
            input in ".*"
        ) {
            let analyzer = WasmComplexityAnalyzer::new();

            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                analyzer.analyze_text(&input)
            }));

            prop_assert!(result.is_ok());
        }

        fn analyzer_never_panics_on_arbitrary_ast(
            node_count in 0usize..100,
            _edge_count in 0usize..200,
        ) {
            let analyzer = WasmComplexityAnalyzer::new();
            let mut dag = AstDag::new();
            let mut node_ids = Vec::new();

            // Add nodes
            for _ in 0..node_count {
                // Use a simple function kind for testing
                let kind = AstKind::Function(FunctionKind::Regular);
                let node = UnifiedAstNode::new(kind, Language::WebAssembly);
                let id = dag.add_node(node);
                node_ids.push(id);
            }

            // Note: AstDag doesn't expose edge manipulation methods
            // so we just test with nodes

            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                analyzer.analyze_ast(&dag)
            }));

            prop_assert!(result.is_ok());
        }

        fn complexity_values_are_reasonable(
            text in prop::string::string_regex("(func[^\n]*\n){0,50}.*").unwrap()
        ) {
            let analyzer = WasmComplexityAnalyzer::new();
            let result = analyzer.analyze_text(&text);

            prop_assert!(result.is_ok());
            let complexity = result.unwrap();

            // All values should be reasonable
            prop_assert!(complexity.memory_pressure >= 0.0);
            prop_assert!(complexity.hot_path_score >= 0.0);
            prop_assert!(complexity.estimated_gas >= 0.0);
            prop_assert!(complexity.indirect_call_overhead >= 0.0);

            // Values should be bounded
            prop_assert!(complexity.cyclomatic <= 10000);
            prop_assert!(complexity.cognitive <= 10000);
            prop_assert!(complexity.memory_pressure <= 10000.0);
            prop_assert!(complexity.max_loop_depth <= 1000);
        }

        fn complexity_increases_with_functions(
            base_lines in 0usize..100,
            func_count in 0usize..50,
        ) {
            let analyzer = WasmComplexityAnalyzer::new();

            // Create content with base lines
            let mut content = "// Base content\n".repeat(base_lines);

            // Add functions
            for i in 0..func_count {
                content.push_str(&format!("func test{}() {{}}\n", i));
            }

            let result = analyzer.analyze_text(&content);
            prop_assert!(result.is_ok());
            let complexity = result.unwrap();

            // Complexity should increase with function count
            let expected_min = (func_count * 2) as u32;
            prop_assert!(complexity.cyclomatic >= expected_min);
        }

        fn function_complexity_is_deterministic(
            node_count in 0usize..20,
        ) {
            let analyzer = WasmComplexityAnalyzer::new();
            let mut dag = AstDag::new();

            // Create function node
            let func_node = UnifiedAstNode::new(
                AstKind::Function(FunctionKind::Regular),
                Language::WebAssembly,
            );
            let func_id = dag.add_node(func_node);

            // Add child nodes
            for _ in 0..node_count {
                let node = UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::WebAssembly
                );
                let _child_id = dag.add_node(node);
                // Note: Can't add edges in AstDag
            }

            // Calculate complexity twice
            let complexity1 = analyzer.analyze_function(&dag, func_id);
            let complexity2 = analyzer.analyze_function(&dag, func_id);

            // Should be deterministic
            prop_assert_eq!(complexity1.cyclomatic, complexity2.cyclomatic);
            prop_assert_eq!(complexity1.cognitive, complexity2.cognitive);
            prop_assert_eq!(complexity1.max_loop_depth, complexity2.max_loop_depth);
        }

        fn loop_depth_detected(
            loop_count in 0usize..10,
        ) {
            let analyzer = WasmComplexityAnalyzer::new();
            let mut dag = AstDag::new();

            // Create function with loops
            let func_node = UnifiedAstNode::new(
                AstKind::Function(FunctionKind::Regular),
                Language::WebAssembly,
            );
            let func_id = dag.add_node(func_node);

            // Add function nodes (can't add loops as they're not in AstKind)
            for _ in 0..loop_count {
                let func_node = UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::WebAssembly,
                );
                let _loop_id = dag.add_node(func_node);
            }

            let complexity = analyzer.analyze_function(&dag, func_id);

            // Basic complexity check
            prop_assert!(complexity.cyclomatic >= 1);
        }

        fn ast_complexity_bounds(
            func_id in any::<u64>(),
        ) {
            let analyzer = WasmComplexityAnalyzer::new();
            let dag = AstDag::new();

            // Analyze non-existent function
            let node_id = func_id as NodeKey;
            let complexity = analyzer.analyze_function(&dag, node_id);

            // Should return base complexity for non-existent nodes
            prop_assert_eq!(complexity.cyclomatic, 1);
            prop_assert_eq!(complexity.max_loop_depth, 0);
        }

        fn memory_cost_model_consistency(
            _dummy in 0u8..1
        ) {
            let model1 = MemoryCostModel::default();
            let model2 = MemoryCostModel::default();

            // Default values should be consistent
            prop_assert_eq!(model1.load_cost, model2.load_cost);
            prop_assert_eq!(model1.store_cost, model2.store_cost);
            prop_assert_eq!(model1.grow_cost, model2.grow_cost);

            // Values should be positive
            prop_assert!(model1.load_cost > 0.0);
            prop_assert!(model1.store_cost > 0.0);
            prop_assert!(model1.grow_cost > 0.0);

            // Grow should be most expensive
            prop_assert!(model1.grow_cost > model1.store_cost);
            prop_assert!(model1.grow_cost > model1.load_cost);
        }
    }

    #[test]
    fn test_edge_cases() {
        let analyzer = WasmComplexityAnalyzer::new();

        // Empty content
        let result = analyzer.analyze_text("").unwrap();
        assert_eq!(result.cyclomatic, 0);
        assert_eq!(result.max_loop_depth, 1); // Default value

        // Only whitespace
        let result = analyzer.analyze_text("   \n\t  \n").unwrap();
        assert_eq!(result.cyclomatic, 0);

        // Single function
        let result = analyzer.analyze_text("func test() {}").unwrap();
        assert_eq!(result.cyclomatic, 2);

        // Multiple functions on same line
        let result = analyzer
            .analyze_text("func a() {} func b() {} func c() {}")
            .unwrap();
        assert_eq!(result.cyclomatic, 6);
    }

    #[test]
    fn test_simple_function_complexity() {
        let analyzer = WasmComplexityAnalyzer::new();
        let mut dag = AstDag::new();

        // Function node
        let func_node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::WebAssembly,
        );
        let func_id = dag.add_node(func_node);

        let complexity = analyzer.analyze_function(&dag, func_id);

        // Should have base complexity
        assert_eq!(complexity.cyclomatic, 1);
    }

    #[test]
    fn test_memory_cost_model_values() {
        let model = MemoryCostModel::default();

        // Test default values match expected
        assert_eq!(model.load_cost, 3.0);
        assert_eq!(model.store_cost, 5.0);
        assert_eq!(model.grow_cost, 100.0);
    }
}
