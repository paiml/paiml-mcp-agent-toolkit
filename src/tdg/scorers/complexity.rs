use anyhow::Result;
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, count_nodes_of_kind, max_depth};

pub struct StructuralComplexityScorer;

impl StructuralComplexityScorer {
    pub fn new() -> Self {
        Self
    }
    
    fn calculate_cyclomatic_complexity(&self, root: Node) -> u32 {
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
        let mut max_nesting = 0;
        
        fn check_nesting(node: Node, current_depth: usize, max: &mut usize) {
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
        count_nodes_of_kind(root, "if_statement") +
        count_nodes_of_kind(root, "match_expression") +
        count_nodes_of_kind(root, "while_statement") +
        count_nodes_of_kind(root, "for_statement")
    }

    /// Extract all function nodes from the tree
    fn extract_functions(&self, root: Node) -> Vec<Node> {
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
    fn score_file_level(&self, root: Node, config: &TdgConfig, tracker: &mut PenaltyTracker, mut points: f32) -> Result<f32> {
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
    fn score(&self, tree: &Tree, _source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
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
        MetricCategory::StructuralComplexity
    }
}

pub struct SemanticComplexityScorer;

impl SemanticComplexityScorer {
    pub fn new() -> Self {
        Self
    }
    
    fn calculate_cognitive_complexity(&self, root: Node) -> u32 {
        let mut complexity = 0;
        let mut nesting_stack = Vec::new();
        
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

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    
    fn parse_rust(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        parser.parse(source, None).unwrap()
    }
    
    #[test]
    fn test_cyclomatic_complexity() {
        let source = r#"
            fn test() {
                if x > 0 {
                    if y > 0 {
                        return 1;
                    }
                }
                while z < 10 {
                    z += 1;
                }
            }
        "#;
        
        let tree = parse_rust(source);
        let scorer = StructuralComplexityScorer::new();
        let complexity = scorer.calculate_cyclomatic_complexity(tree.root_node());
        assert!(complexity >= 3);
    }
    
    #[test]
    fn test_cognitive_complexity() {
        let source = r#"
            fn test() {
                if x > 0 {
                    if y > 0 {
                        while z < 10 {
                            if w == 0 {
                                break;
                            }
                        }
                    }
                }
            }
        "#;

        let tree = parse_rust(source);
        let scorer = SemanticComplexityScorer::new();
        let complexity = scorer.calculate_cognitive_complexity(tree.root_node());
        assert!(complexity > 0);
    }

    #[test]
    fn test_function_extraction() {
        let source = r#"
            fn helper_one() {
                println!("simple");
            }

            fn helper_two() {
                if x > 0 {
                    return 1;
                }
                return 0;
            }

            fn main() {
                helper_one();
                helper_two();
            }
        "#;

        let tree = parse_rust(source);
        let scorer = StructuralComplexityScorer::new();
        let functions = scorer.extract_functions(tree.root_node());

        // Should find 3 functions
        assert_eq!(functions.len(), 3);
    }

    #[test]
    fn test_per_function_scoring() {
        // Code similar to bug report: refactored with many small functions
        let source = r#"
            fn emit_memory_section() -> Option<u32> {
                if needs_memory() {
                    Some(1)
                } else {
                    None
                }
            }

            fn lower_while() -> Vec<u32> {
                let mut v = vec![];
                if condition() {
                    v.push(1);
                }
                v
            }

            fn lower_expression() -> u32 {
                match get_type() {
                    1 => lower_while().len() as u32,
                    _ => 0,
                }
            }

            fn needs_memory() -> bool { true }
            fn condition() -> bool { false }
            fn get_type() -> u32 { 1 }
        "#;

        let tree = parse_rust(source);
        let scorer = StructuralComplexityScorer::new();
        let functions = scorer.extract_functions(tree.root_node());

        // Should find 6 functions
        assert_eq!(functions.len(), 6);

        // All functions should have low complexity
        for func in &functions {
            let complexity = scorer.calculate_cyclomatic_complexity(*func);
            assert!(complexity <= 10, "Function complexity {} exceeds Toyota Way limit", complexity);
        }
    }

    #[test]
    fn test_toyota_way_compliance() {
        // Many small functions (Toyota Way: <10 complexity)
        let good_source = r#"
            fn a() { return 1; }
            fn b() { return 2; }
            fn c() { if x > 0 { return 3; } return 0; }
            fn d() { return 4; }
            fn e() { return 5; }
            fn f() { return 6; }
            fn g() { return 7; }
            fn h() { return 8; }
            fn i() { return 9; }
            fn j() { return 10; }
            fn k() { return 11; }
            fn l() { return 12; }
        "#;

        let tree = parse_rust(good_source);
        let scorer = StructuralComplexityScorer::new();
        let config = TdgConfig::default();
        let mut tracker = PenaltyTracker::new();

        let score = scorer.score(&tree, good_source, Language::Rust, &config, &mut tracker).unwrap();

        // Should get bonus points for good decomposition (>10 functions, avg complexity <8)
        // Score should be close to max (25.0)
        assert!(score >= 20.0, "Expected high score for well-decomposed code, got {}", score);
    }
}