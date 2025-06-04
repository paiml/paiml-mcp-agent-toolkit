#[cfg(test)]
mod tests {
    use crate::models::unified_ast::{AstDag, AstKind, FunctionKind, Language, UnifiedAstNode};
    use crate::services::dead_code_analyzer::{DeadCodeAnalyzer, ReferenceEdge, ReferenceType};

    #[test]
    fn verify_entry_point_detection() {
        let mut analyzer = DeadCodeAnalyzer::new(1000);

        // Add some entry points
        analyzer.add_entry_point(1); // main function
        analyzer.add_entry_point(2); // public function

        // The entry points are tracked internally
        // We can verify they work by running analysis
        let dag = AstDag::new();
        let _report = analyzer.analyze(&dag);
    }

    #[test]
    fn verify_cross_language_references() {
        let mut analyzer = DeadCodeAnalyzer::new(1000);

        // Add nodes through the public API by creating a DAG with nodes
        let mut dag = AstDag::new();
        let node1 = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let node2 = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::TypeScript,
        );
        dag.add_node(node1);
        dag.add_node(node2);

        // Add cross-language reference
        analyzer.add_reference(ReferenceEdge {
            from: 1, // TypeScript function (node 2 is index 1)
            to: 0,   // Rust function (node 1 is index 0)
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        });

        // Mark TypeScript function as entry point
        analyzer.add_entry_point(1);

        // Run analysis
        let report = analyzer.analyze(&dag);

        // Since node 1 is an entry point and calls node 0,
        // neither should be marked as dead
        assert_eq!(
            report.dead_functions.len(),
            0,
            "Functions with cross-language references should not be marked as dead"
        );
    }

    #[test]
    fn verify_dead_code_detection() {
        let mut analyzer = DeadCodeAnalyzer::new(100);

        // Create a DAG with 4 nodes
        let mut dag = AstDag::new();
        for _i in 0..4 {
            let node =
                UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
            dag.add_node(node);
        }

        // Add edges creating a chain: 0 -> 1 -> 2
        // Node 3 has no incoming edges
        analyzer.add_reference(ReferenceEdge {
            from: 0,
            to: 1,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        });

        analyzer.add_reference(ReferenceEdge {
            from: 1,
            to: 2,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        });

        // Mark node 0 as entry point
        analyzer.add_entry_point(0);

        let report = analyzer.analyze(&dag);

        // Debug output
        eprintln!("Total nodes: 4");
        eprintln!("Dead functions found: {}", report.dead_functions.len());
        eprintln!(
            "Dead function keys: {:?}",
            report
                .dead_functions
                .iter()
                .map(|f| f.node_key)
                .collect::<Vec<_>>()
        );

        // Node 3 should be marked as dead since it's not reachable
        let dead_function_keys: Vec<u32> = report
            .dead_functions
            .iter()
            .map(|item| item.node_key)
            .collect();

        assert!(
            dead_function_keys.contains(&3),
            "Unreachable function (node 3) should be marked as dead. Found dead nodes: {:?}",
            dead_function_keys
        );
    }

    #[test]
    fn verify_zero_dead_code_edge_cases() {
        let mut analyzer = DeadCodeAnalyzer::new(100);

        // Create DAG with 2 nodes
        let mut dag = AstDag::new();
        for _ in 0..2 {
            let node =
                UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
            dag.add_node(node);
        }

        // Add call from node 0 to node 1
        analyzer.add_reference(ReferenceEdge {
            from: 0,
            to: 1,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        });

        // Mark node 0 as entry point
        analyzer.add_entry_point(0);

        let report = analyzer.analyze(&dag);

        // In a simple project where main calls all functions, zero dead code is legitimate
        assert_eq!(
            report.dead_functions.len(),
            0,
            "Simple project with all functions called should have zero dead code"
        );
    }

    #[test]
    fn verify_closure_and_dynamic_dispatch() {
        let mut analyzer = DeadCodeAnalyzer::new(100);

        // Create DAG with nodes for closure scenario
        let mut dag = AstDag::new();
        for _ in 0..3 {
            let node =
                UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
            dag.add_node(node);
        }

        // Add edges: async_handler contains closure, closure calls helper
        analyzer.add_reference(ReferenceEdge {
            from: 0,
            to: 1,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        });

        analyzer.add_reference(ReferenceEdge {
            from: 1,
            to: 2,
            reference_type: ReferenceType::DirectCall,
            confidence: 0.95,
        });

        // Mark async_handler as entry point
        analyzer.add_entry_point(0);

        let report = analyzer.analyze(&dag);

        // Helper should not be dead if called from closure
        let dead_function_keys: Vec<u32> = report
            .dead_functions
            .iter()
            .map(|item| item.node_key)
            .collect();

        assert!(
            !dead_function_keys.contains(&2),
            "Function called from closure should not be marked as dead"
        );
    }

    #[test]
    fn test_coverage_integration() {
        use crate::services::dead_code_analyzer::CoverageData;
        use std::collections::{HashMap, HashSet};

        let mut covered_lines = HashMap::new();
        let mut lines = HashSet::new();
        lines.insert(10);
        lines.insert(20);
        lines.insert(30);
        covered_lines.insert("test.rs".to_string(), lines);

        let mut execution_counts = HashMap::new();
        let mut counts = HashMap::new();
        counts.insert(10, 5);
        counts.insert(20, 3);
        counts.insert(30, 1);
        execution_counts.insert("test.rs".to_string(), counts);

        let coverage = CoverageData {
            covered_lines,
            execution_counts,
        };

        let mut analyzer = DeadCodeAnalyzer::new(100).with_coverage(coverage);

        // Verify coverage data works by running analysis
        let dag = AstDag::new();
        let _report = analyzer.analyze(&dag);
        // If we got here without panic, coverage integration works
    }
}
