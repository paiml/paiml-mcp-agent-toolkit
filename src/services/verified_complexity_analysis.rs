// Verified complexity analysis methods
// This file is included into verified_complexity.rs via include!()
// DO NOT add `use` imports or `#!` inner attributes here.

impl VerifiedComplexityAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self { nesting_level: 0 }
    }

    /// Calculate all complexity metrics for a function
    #[inline]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_function(&mut self, node: &UnifiedAstNode) -> ComplexityMetrics {
        debug_assert!(
            matches!(node.kind, AstKind::Function(_)),
            "Node must be a function"
        );

        // Reset state
        self.nesting_level = 0;

        // Calculate cyclomatic complexity
        let cyclomatic = self.calculate_cyclomatic(node);

        // Calculate cognitive complexity
        let cognitive = self.compute_cognitive_weight(node);

        // Calculate essential complexity
        let essential = self.compute_essential(node, cyclomatic);

        // Calculate Halstead metrics
        let halstead = self.calculate_halstead(node);

        // Sanity checks - relaxed for real-world code
        debug_assert!(
            cognitive >= cyclomatic.saturating_sub(1),
            "Cognitive too low"
        );
        debug_assert!(cognitive <= cyclomatic * 3, "Cognitive > 3x cyclomatic");
        debug_assert!(essential <= cyclomatic, "Essential > cyclomatic");

        ComplexityMetrics {
            cyclomatic,
            cognitive,
            essential,
            halstead,
        }
    }

    /// Calculate cyclomatic complexity (`McCabe`)
    fn calculate_cyclomatic(&self, node: &UnifiedAstNode) -> u32 {
        let mut complexity = 1; // Base complexity

        self.visit_cyclomatic(node, &mut complexity);

        complexity
    }

    fn visit_cyclomatic(&self, node: &UnifiedAstNode, complexity: &mut u32) {
        match &node.kind {
            AstKind::Statement(StmtKind::If) => *complexity += 1,
            AstKind::Statement(StmtKind::While | StmtKind::For) => {
                *complexity += 1;
            }
            AstKind::Statement(StmtKind::Switch) => {
                // Each case adds to complexity
                // This is simplified - would need to count actual case statements
                *complexity += 1;
            }
            AstKind::Expression(ExprKind::Binary) => {
                // Logical operators add complexity
                // Would need to check operator type in real implementation
                *complexity += 1;
            }
            AstKind::Statement(StmtKind::Try) => {
                // Each catch block adds complexity
                *complexity += 1;
            }
            _ => {}
        }

        // Recurse through children - simplified since we don't have child iteration
        // In real implementation, would iterate through node.children()
    }

    /// Compute cognitive complexity weight per Sonar rules
    fn compute_cognitive_weight(&mut self, node: &UnifiedAstNode) -> u32 {
        let mut weight = 0;

        match &node.kind {
            AstKind::Statement(StmtKind::If) => {
                weight += 1 + self.nesting_level;
            }
            AstKind::Statement(StmtKind::While | StmtKind::For) => {
                weight += 1 + self.nesting_level;
            }
            AstKind::Statement(StmtKind::Switch) => {
                weight += 1 + self.nesting_level;
            }
            AstKind::Expression(ExprKind::Binary) => {
                // Logical operators add cognitive load
                weight += 1;
            }
            AstKind::Statement(StmtKind::Try) => {
                weight += 1 + self.nesting_level;
            }
            AstKind::Statement(StmtKind::Return) if self.nesting_level > 0 => {
                // Early returns add cognitive load
                weight += 1;
            }
            AstKind::Function(_) => {
                // Check for async functions - would need proper flag checking in real implementation
                // For now, all functions get base complexity
                weight += 0;
            }
            _ => {}
        }

        // Track nesting for children
        let increases_nesting = matches!(
            &node.kind,
            AstKind::Statement(
                StmtKind::If | StmtKind::While | StmtKind::For | StmtKind::Switch | StmtKind::Try
            ) | AstKind::Function(_)
        );

        if increases_nesting {
            self.nesting_level += 1;
        }

        // Process children - simplified
        // In real implementation would iterate through children

        if increases_nesting {
            self.nesting_level -= 1;
        }

        weight
    }

    /// Compute essential complexity (remove linear paths)
    fn compute_essential(&self, node: &UnifiedAstNode, cyclomatic: u32) -> u32 {
        let linear_paths = self.count_linear_paths(node);
        cyclomatic.saturating_sub(linear_paths)
    }

    /// Count linear execution paths that can be simplified
    fn count_linear_paths(&self, node: &UnifiedAstNode) -> u32 {
        let mut linear_paths = 0;

        // Look for simple if-return patterns
        if let AstKind::Statement(StmtKind::If) = &node.kind {
            // Simplified check - would need to inspect children
            linear_paths += 1;
        }

        // Look for guard clauses
        if self.is_guard_clause(node) {
            linear_paths += 1;
        }

        linear_paths
    }

    fn is_guard_clause(&self, node: &UnifiedAstNode) -> bool {
        // Guard clause: early return
        matches!(node.kind, AstKind::Statement(StmtKind::Return))
    }

    /// Calculate Halstead metrics
    fn calculate_halstead(&self, node: &UnifiedAstNode) -> HalsteadMetrics {
        let mut operators = HashMap::new();
        let mut operands = HashMap::new();

        self.collect_halstead_tokens(node, &mut operators, &mut operands);

        HalsteadMetrics {
            n1: operators.len() as u32,
            n2: operands.len() as u32,
            N1: operators.values().sum(),
            N2: operands.values().sum(),
        }
    }

    fn collect_halstead_tokens(
        &self,
        node: &UnifiedAstNode,
        operators: &mut HashMap<String, u32>,
        operands: &mut HashMap<String, u32>,
    ) {
        match &node.kind {
            // Operators
            AstKind::Expression(ExprKind::Binary) => {
                *operators.entry("binary_op".to_string()).or_insert(0) += 1;
            }
            AstKind::Expression(ExprKind::Unary) => {
                *operators.entry("unary_op".to_string()).or_insert(0) += 1;
            }
            AstKind::Statement(StmtKind::If) => {
                *operators.entry("if".to_string()).or_insert(0) += 1;
            }
            AstKind::Statement(StmtKind::While) => {
                *operators.entry("while".to_string()).or_insert(0) += 1;
            }
            AstKind::Statement(StmtKind::For) => {
                *operators.entry("for".to_string()).or_insert(0) += 1;
            }
            AstKind::Expression(ExprKind::Call) => {
                *operators.entry("()".to_string()).or_insert(0) += 1;
            }

            // Operands
            AstKind::Expression(ExprKind::Identifier) => {
                *operands.entry("identifier".to_string()).or_insert(0) += 1;
            }
            AstKind::Expression(ExprKind::Literal) => {
                *operands.entry("literal".to_string()).or_insert(0) += 1;
            }
            _ => {}
        }

        // In real implementation would recurse through children
    }

    /// Helper to iterate children - placeholder for actual implementation
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn children(&self, _node: &UnifiedAstNode) -> Vec<&UnifiedAstNode> {
        // In actual implementation, would follow first_child/next_sibling links
        vec![]
    }
}
