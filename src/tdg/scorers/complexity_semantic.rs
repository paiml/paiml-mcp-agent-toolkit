// SemanticComplexityScorer implementation methods and Scorer trait impl
// Included from complexity.rs - shares parent module scope

impl SemanticComplexityScorer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_cognitive_complexity(&self, root: Node) -> u32 {
        let mut complexity = 0;
        let mut nesting_stack = Vec::new();

        #[allow(clippy::cast_possible_truncation)]
        fn process_node(node: Node, complexity: &mut u32, nesting_stack: &mut Vec<&str>) {
            let nesting_level = nesting_stack.len() as u32;

            match node.kind() {
                "if_statement" | "if_expression" => {
                    *complexity += 1 + nesting_level;
                    nesting_stack.push("if");
                }
                "else_clause" => {
                    *complexity += 1;
                }
                "while_statement" | "while_expression" | "for_statement" | "for_expression" => {
                    *complexity += 1 + nesting_level;
                    nesting_stack.push("loop");
                }
                "break_expression" | "continue_expression" => {
                    *complexity += 1 + nesting_level;
                }
                "match_expression" => {
                    *complexity += 1 + nesting_level;
                    nesting_stack.push("match");
                }
                "try_expression" => {
                    *complexity += 1 + nesting_level;
                    nesting_stack.push("try");
                }
                "binary_expression" => {
                    if let Some(op) = node.child_by_field_name("operator") {
                        let op_kind = op.kind();
                        if op_kind == "&&" || op_kind == "||" {
                            *complexity += 1;
                        }
                    }
                }
                _ => {}
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                process_node(child, complexity, nesting_stack);
            }

            if matches!(
                node.kind(),
                "if_statement" | "if_expression" | "while_statement" | "while_expression" |
                "for_statement" | "for_expression" | "match_expression" | "try_expression"
            ) {
                nesting_stack.pop();
            }
        }

        process_node(root, &mut complexity, &mut nesting_stack);
        complexity
    }

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_type_complexity(&self, root: Node) -> f32 {
        let mut complexity = 0.0;

        walk_tree(root, |node| {
            match node.kind() {
                "generic_type" | "generic_arguments" => {
                    complexity += 0.1;
                    let depth = self.generic_depth(node);
                    if depth > 2 {
                        complexity += (depth - 2) as f32 * 0.2;
                    }
                }
                "trait_bounds" | "where_clause" => {
                    complexity += 0.2;
                }
                "impl_item" => {
                    if node.child_by_field_name("trait").is_some() {
                        complexity += 0.15;
                    }
                }
                _ => {}
            }
        });

        complexity.min(2.5)
    }

    fn generic_depth(&self, node: Node) -> usize {
        if !matches!(node.kind(), "generic_type" | "generic_arguments") {
            return 0;
        }

        let mut max_child_depth = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            max_child_depth = max_child_depth.max(self.generic_depth(child));
        }

        max_child_depth + 1
    }

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_expression_complexity(&self, root: Node) -> f32 {
        let mut complexity = 0.0;

        walk_tree(root, |node| {
            match node.kind() {
                "closure_expression" | "lambda_expression" | "arrow_function" => {
                    complexity += 0.15;
                }
                "conditional_expression" | "ternary_expression" => {
                    complexity += 0.2;
                    if self.is_nested_ternary(node) {
                        complexity += 0.3;
                    }
                }
                "call_expression" => {
                    let chain_length = self.call_chain_length(node);
                    if chain_length > 3 {
                        complexity += (chain_length - 3) as f32 * 0.1;
                    }
                }
                _ => {}
            }
        });

        complexity.min(2.5)
    }

    fn is_nested_ternary(&self, node: Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "conditional_expression" | "ternary_expression") {
                return true;
            }
        }
        false
    }

    fn call_chain_length(&self, node: Node) -> usize {
        if node.kind() != "call_expression" {
            return 0;
        }

        let mut length = 1;
        if let Some(function) = node.child_by_field_name("function") {
            if function.kind() == "field_expression" {
                if let Some(value) = function.child_by_field_name("value") {
                    length += self.call_chain_length(value);
                }
            }
        }

        length
    }
}

impl Scorer for SemanticComplexityScorer {
    #[allow(clippy::cast_possible_truncation)]
    fn score(&self, tree: &Tree, _source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        let mut points = config.weights.semantic_complexity;
        let root = tree.root_node();

        let cognitive = self.calculate_cognitive_complexity(root);
        if cognitive > config.thresholds.max_cognitive_complexity {
            let excess = (cognitive - config.thresholds.max_cognitive_complexity) as f32;
            let penalty = (excess * 0.5).min(10.0);

            if let Some(applied) = tracker.apply(
                format!("high_cognitive_{}", cognitive),
                MetricCategory::SemanticComplexity,
                penalty,
                format!("High cognitive complexity: {}", cognitive)
            ) {
                points -= applied;
            }
        }

        let type_complexity = self.analyze_type_complexity(root);
        let type_penalty = (type_complexity * 2.0).min(5.0);
        if type_penalty > 0.0 {
            if let Some(applied) = tracker.apply(
                "complex_types".to_string(),
                MetricCategory::SemanticComplexity,
                type_penalty,
                format!("Complex type signatures: {:.1}", type_complexity)
            ) {
                points -= applied;
            }
        }

        let expr_complexity = self.analyze_expression_complexity(root);
        let expr_penalty = (expr_complexity * 2.0).min(5.0);
        if expr_penalty > 0.0 {
            if let Some(applied) = tracker.apply(
                "complex_expressions".to_string(),
                MetricCategory::SemanticComplexity,
                expr_penalty,
                format!("Complex expressions: {:.1}", expr_complexity)
            ) {
                points -= applied;
            }
        }

        Ok(points.max(0.0))
    }

    fn category(&self) -> MetricCategory {
        MetricCategory::SemanticComplexity
    }
}
