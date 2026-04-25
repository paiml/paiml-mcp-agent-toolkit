// ReachabilityAnalyzer implementation methods
// Included from dead_code_prover.rs - do NOT add `use` imports or `#!` attributes here.

impl ReachabilityAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            entry_points: HashSet::new(),
            reachable: HashSet::new(),
            ffi_exports: HashSet::new(),
            dynamic_targets: HashSet::new(),
        }
    }

    /// Find entry points in AST
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_entry_points(&mut self, ast: &UnifiedAstNode, file_path: &str) {
        self.visit_for_entry_points(ast, file_path);
    }

    fn visit_for_entry_points(&mut self, node: &UnifiedAstNode, file_path: &str) {
        if let AstKind::Function(FunctionKind::Regular) = &node.kind {
            // Check for main function
            if let Some(name) = self.extract_function_name(node) {
                if name == "main" {
                    self.entry_points.insert(SymbolId {
                        file_path: file_path.to_string(),
                        function_name: name.clone(),
                        line_number: node.source_range.start as usize,
                    });
                }

                // Check for test functions
                if name.starts_with("test_") || self.has_test_attribute(node) {
                    self.entry_points.insert(SymbolId {
                        file_path: file_path.to_string(),
                        function_name: name.clone(),
                        line_number: node.source_range.start as usize,
                    });
                }

                // Check for benchmark functions
                if name.starts_with("bench_") || self.has_benchmark_attribute(node) {
                    self.entry_points.insert(SymbolId {
                        file_path: file_path.to_string(),
                        function_name: name,
                        line_number: node.source_range.start as usize,
                    });
                }
            }
        }

        // Would recursively visit children in full implementation
    }

    fn extract_function_name(&self, _node: &UnifiedAstNode) -> Option<String> {
        // The new UnifiedAstNode doesn't have a direct name field
        // Would need to extract from metadata or use a different approach
        None
    }

    fn has_test_attribute(&self, _node: &UnifiedAstNode) -> bool {
        // Would check for #[test] attribute in real implementation
        false
    }

    fn has_benchmark_attribute(&self, _node: &UnifiedAstNode) -> bool {
        // Would check for #[bench] attribute in real implementation
        false
    }
}

#[cfg(test)]
mod reachability_analyzer_tests {
    //! Covers the placeholder ReachabilityAnalyzer methods in
    //! dead_code_prover_reachability.rs (36 uncov on broad, 0% cov).
    use super::*;
    use crate::models::unified_ast::{
        AstKind, FunctionKind, Language, UnifiedAstNode,
    };

    #[test]
    fn test_new_returns_empty_state() {
        let analyzer = ReachabilityAnalyzer::new();
        assert!(analyzer.entry_points.is_empty());
        assert!(analyzer.reachable.is_empty());
        assert!(analyzer.ffi_exports.is_empty());
        assert!(analyzer.dynamic_targets.is_empty());
    }

    #[test]
    fn test_default_matches_new() {
        let from_default = ReachabilityAnalyzer::default();
        let from_new = ReachabilityAnalyzer::new();
        // Both should produce empty sets — compare by emptiness.
        assert_eq!(
            from_default.entry_points.len(),
            from_new.entry_points.len()
        );
        assert_eq!(from_default.reachable.len(), from_new.reachable.len());
    }

    #[test]
    fn test_extract_function_name_returns_none_placeholder() {
        // The current implementation always returns None — placeholder method.
        let analyzer = ReachabilityAnalyzer::new();
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(analyzer.extract_function_name(&node).is_none());
    }

    #[test]
    fn test_has_test_attribute_returns_false_placeholder() {
        let analyzer = ReachabilityAnalyzer::new();
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(!analyzer.has_test_attribute(&node));
    }

    #[test]
    fn test_has_benchmark_attribute_returns_false_placeholder() {
        let analyzer = ReachabilityAnalyzer::new();
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        assert!(!analyzer.has_benchmark_attribute(&node));
    }

    #[test]
    fn test_find_entry_points_function_node_no_entry_points_due_to_placeholder() {
        // visit_for_entry_points only adds entry_points when extract_function_name
        // returns Some(name). Since the placeholder returns None, no entries get
        // added even for Function nodes.
        let mut analyzer = ReachabilityAnalyzer::new();
        let node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        );
        analyzer.find_entry_points(&node, "src/a.rs");
        assert!(analyzer.entry_points.is_empty());
    }

    #[test]
    fn test_find_entry_points_non_function_node_no_entry_points() {
        // Non-function AST kinds should not add entry points either.
        use crate::models::unified_ast::ClassKind;
        let mut analyzer = ReachabilityAnalyzer::new();
        let node = UnifiedAstNode::new(
            AstKind::Class(ClassKind::Regular),
            Language::Rust,
        );
        analyzer.find_entry_points(&node, "src/a.rs");
        assert!(analyzer.entry_points.is_empty());
    }
}
