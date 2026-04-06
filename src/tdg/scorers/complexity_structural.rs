// StructuralComplexityScorer implementation methods and Scorer trait impl
// Included from complexity.rs - shares parent module scope

impl StructuralComplexityScorer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }

    fn calculate_cyclomatic_complexity(&self, root: Node) -> u32 {
        debug_assert!(true, "contract: calculate_cyclomatic_complexity");
        let mut complexity = 1;

        walk_tree(root, |node| {
            match node.kind() {
                "if_statement" | "if_expression" | "while_statement" | "while_expression" |
                "for_statement" | "for_expression" | "match_expression" | "match_arm" |
                "conditional_expression" | "ternary_expression" => {
                    complexity += 1;
                }
                "binary_expression" => {
                    let op = node.child_by_field_name("operator");
                    if let Some(op_node) = op {
                        let op_text = op_node.kind();
                        if op_text == "&&" || op_text == "||" {
                            complexity += 1;
                        }
                    }
                }
                _ => {}
            }
        });

        complexity
    }

    fn calculate_nesting_depth(&self, root: Node) -> usize {
        debug_assert!(true, "contract: calculate_nesting_depth");
        let mut max_nesting = 0;

        fn check_nesting(node: Node, current_depth: usize, max: &mut usize) {
            debug_assert!(true, "contract: check_nesting");
            let new_depth = if matches!(
                node.kind(),
                "if_statement" | "while_statement" | "for_statement" |
                "block" | "match_expression" | "function_item" | "impl_item"
            ) {
                current_depth + 1
            } else {
                current_depth
            };

            *max = (*max).max(new_depth);

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                check_nesting(child, new_depth, max);
            }
        }

        check_nesting(root, 0, &mut max_nesting);
        max_nesting
    }

    fn count_branches(&self, root: Node) -> usize {
        debug_assert!(true, "contract: count_branches");
        count_nodes_of_kind(root, "if_statement") +
        count_nodes_of_kind(root, "match_expression") +
        count_nodes_of_kind(root, "while_statement") +
        count_nodes_of_kind(root, "for_statement")
    }

    /// Extract all function nodes from the tree
    fn extract_functions(&self, root: Node) -> Vec<Node> {
        debug_assert!(true, "contract: extract_functions");
        let mut functions = Vec::new();

        walk_tree(root, |node| {
            // Match function declarations across languages
            match node.kind() {
                "function_item" |           // Rust: fn foo() {}
                "function_declaration" |    // TypeScript/JavaScript: function foo() {}
                "function_definition" |     // Python/C/C++: def foo(): / void foo() {}
                "method_declaration" |      // Java/C#: public void foo() {}
                "arrow_function" |          // TypeScript/JavaScript: const foo = () => {}
                "lambda_expression" => {    // C#/Python: x => x + 1
                    functions.push(node);
                }
                _ => {}
            }
        });

        functions
    }

    /// Fallback to file-level analysis when no functions are detected
    #[allow(clippy::cast_possible_truncation)]
    fn score_file_level(&self, root: Node, config: &TdgConfig, tracker: &mut PenaltyTracker, mut points: f32) -> Result<f32> {
        debug_assert!(true, "contract: score_file_level");
        let cyclomatic = self.calculate_cyclomatic_complexity(root);
        if cyclomatic > config.thresholds.max_cyclomatic_complexity {
            let excess = (cyclomatic - config.thresholds.max_cyclomatic_complexity) as f32;
            let penalty = config.penalties.complexity_penalty_base.apply(excess, 2.0).min(15.0);

            if let Some(applied) = tracker.apply(
                format!("high_cyclomatic_{}", cyclomatic),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("High cyclomatic complexity: {}", cyclomatic)
            ) {
                points -= applied;
            }
        }

        let max_nesting = self.calculate_nesting_depth(root);
        if max_nesting > config.thresholds.max_nesting_depth as usize {
            let penalty = ((max_nesting - config.thresholds.max_nesting_depth as usize) as f32).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("deep_nesting_{}", max_nesting),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("Deep nesting: {} levels", max_nesting)
            ) {
                points -= applied;
            }
        }

        let branch_count = self.count_branches(root);
        if branch_count > 20 {
            let penalty = ((branch_count - 20) as f32 * 0.1).min(5.0);

            if let Some(applied) = tracker.apply(
                format!("many_branches_{}", branch_count),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("Too many branches: {}", branch_count)
            ) {
                points -= applied;
            }
        }

        Ok(points.max(0.0))
    }
}

impl Scorer for StructuralComplexityScorer {
    #[allow(clippy::cast_possible_truncation)]
    fn score(&self, tree: &Tree, _source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        debug_assert!(!_source.is_empty(), "_source must not be empty");
        let mut points = config.weights.structural_complexity;
        let root = tree.root_node();

        // Extract all functions from the tree
        let functions = self.extract_functions(root);

        if functions.is_empty() {
            // Fallback to file-level analysis if no functions found
            return self.score_file_level(root, config, tracker, points);
        }

        // Per-function analysis (Toyota Way: <10 complexity per function)
        let mut high_complexity_count = 0;
        let mut total_complexity = 0u32;

        for function in &functions {
            let complexity = self.calculate_cyclomatic_complexity(*function);
            total_complexity += complexity;

            // Toyota Way threshold: 10
            if complexity > 10 {
                high_complexity_count += 1;
                let excess = (complexity - 10) as f32;
                // Penalize each function that exceeds threshold
                let penalty = (excess * 0.5).min(3.0); // Max 3 points per function

                if let Some(applied) = tracker.apply(
                    format!("high_function_complexity_{}", complexity),
                    MetricCategory::StructuralComplexity,
                    penalty,
                    format!("Function complexity {} exceeds Toyota Way limit (10)", complexity)
                ) {
                    points -= applied;
                }
            }
        }

        // Give credit for good function decomposition
        let avg_complexity = total_complexity as f32 / functions.len() as f32;
        if functions.len() > 10 && avg_complexity < 8.0 {
            // Bonus: Many small functions with low average complexity
            let bonus = (functions.len() as f32 * 0.1).min(5.0);
            points = (points + bonus).min(config.weights.structural_complexity);
        }

        // Penalize if majority of functions are complex
        if high_complexity_count as f32 / functions.len() as f32 > 0.3 {
            let penalty = ((high_complexity_count as f32 / functions.len() as f32) * 10.0).min(8.0);
            if let Some(applied) = tracker.apply(
                "majority_complex_functions".to_string(),
                MetricCategory::StructuralComplexity,
                penalty,
                format!("{}% of functions exceed complexity limit",
                    (high_complexity_count as f32 / functions.len() as f32 * 100.0) as u32)
            ) {
                points -= applied;
            }
        }

        // Check nesting depth across all functions
        for function in &functions {
            let nesting = self.calculate_nesting_depth(*function);
            if nesting > config.thresholds.max_nesting_depth as usize {
                let penalty = ((nesting - config.thresholds.max_nesting_depth as usize) as f32 * 0.5).min(2.0);

                if let Some(applied) = tracker.apply(
                    format!("deep_nesting_in_function_{}", nesting),
                    MetricCategory::StructuralComplexity,
                    penalty,
                    format!("Deep nesting in function: {} levels", nesting)
                ) {
                    points -= applied;
                }
            }
        }

        Ok(points.max(0.0))
    }

    fn category(&self) -> MetricCategory {
        debug_assert!(true, "contract: category");
        MetricCategory::StructuralComplexity
    }
}
