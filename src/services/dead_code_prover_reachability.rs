// ReachabilityAnalyzer implementation methods
// Included from dead_code_prover.rs - do NOT add `use` imports or `#!` attributes here.

impl ReachabilityAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
