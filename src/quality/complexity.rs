#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;
use syn::{self, visit::Visit};

pub struct ComplexityAnalyzer {
    _current_complexity: u32,
    _cognitive_complexity: u32,
    _nesting_depth: u32,
    _max_nesting: u32,
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            _current_complexity: 1, // Base complexity
            _cognitive_complexity: 0,
            _nesting_depth: 0,
            _max_nesting: 0,
        }
    }

    pub fn calculate_cyclomatic(&self, ast: &syn::File) -> u32 {
        let mut visitor = ComplexityVisitor {
            complexity: 1,
            nesting_depth: 0,
        };
        visitor.visit_file(ast);
        visitor.complexity
    }

    pub fn calculate_cognitive(&self, ast: &syn::File) -> u32 {
        let mut visitor = CognitiveComplexityVisitor {
            complexity: 0,
            nesting_depth: 0,
        };
        visitor.visit_file(ast);
        visitor.complexity
    }

    pub fn analyze_string(&self, code: &str) -> Result<ComplexityMetrics, syn::Error> {
        let ast = syn::parse_file(code)?;
        Ok(ComplexityMetrics {
            cyclomatic: self.calculate_cyclomatic(&ast),
            cognitive: self.calculate_cognitive(&ast),
        })
    }

    pub fn calculate_shannon_entropy(&self, code: &str) -> f64 {
        let mut char_counts = HashMap::new();
        let total = code.len() as f64;

        for ch in code.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in char_counts.values() {
            let probability = *count as f64 / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
}

struct ComplexityVisitor {
    complexity: u32,
    nesting_depth: u32,
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.complexity += 1; // Each if adds a path
        self.nesting_depth += 1;
        syn::visit::visit_expr_if(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        // Each arm except the first adds a path
        if node.arms.len() > 1 {
            self.complexity += (node.arms.len() - 1) as u32;
        }
        self.nesting_depth += 1;
        syn::visit::visit_expr_match(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.complexity += 1;
        self.nesting_depth += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1;
        self.nesting_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.complexity += 1;
        self.nesting_depth += 1;
        syn::visit::visit_expr_loop(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        use syn::BinOp;
        match node.op {
            BinOp::And(_) | BinOp::Or(_) => {
                self.complexity += 1;
            }
            _ => {}
        }
        syn::visit::visit_expr_binary(self, node);
    }
}

struct CognitiveComplexityVisitor {
    complexity: u32,
    nesting_depth: u32,
}

impl<'ast> Visit<'ast> for CognitiveComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.complexity += 1 + self.nesting_depth; // Nesting adds cognitive load
        self.nesting_depth += 1;
        syn::visit::visit_expr_if(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.complexity += 1 + self.nesting_depth;
        self.nesting_depth += 1;
        syn::visit::visit_expr_match(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.complexity += 1 + self.nesting_depth;
        self.nesting_depth += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1 + self.nesting_depth;
        self.nesting_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.nesting_depth -= 1;
    }

    fn visit_expr_break(&mut self, _node: &'ast syn::ExprBreak) {
        self.complexity += 1; // Breaks add cognitive complexity
    }

    fn visit_expr_continue(&mut self, _node: &'ast syn::ExprContinue) {
        self.complexity += 1; // Continues add cognitive complexity
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ComplexityAnalyzer tests
    #[test]
    fn test_complexity_analyzer_default() {
        let analyzer = ComplexityAnalyzer::default();
        let _ = analyzer;
    }

    #[test]
    fn test_complexity_analyzer_new() {
        let analyzer = ComplexityAnalyzer::new();
        let _ = analyzer;
    }

    #[test]
    fn test_calculate_cyclomatic_simple_function() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn simple() { let x = 1; }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 1); // Base complexity
    }

    #[test]
    fn test_calculate_cyclomatic_with_if() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_if() { if true { } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 2); // Base + 1 for if
    }

    #[test]
    fn test_calculate_cyclomatic_with_match() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn with_match() {
                match x {
                    1 => {},
                    2 => {},
                    _ => {},
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 3); // Base + 2 for arms (3-1)
    }

    #[test]
    fn test_calculate_cyclomatic_with_for_loop() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_for() { for i in 0..10 { } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 2); // Base + 1 for loop
    }

    #[test]
    fn test_calculate_cyclomatic_with_while() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_while() { while true { } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 2); // Base + 1 for while
    }

    #[test]
    fn test_calculate_cyclomatic_with_loop() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_loop() { loop { break; } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 2); // Base + 1 for loop
    }

    #[test]
    fn test_calculate_cyclomatic_with_and_or() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_logic() { let x = a && b || c; }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(complexity, 3); // Base + 2 for && and ||
    }

    #[test]
    fn test_calculate_cognitive_simple() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn simple() { let x = 1; }";
        let ast = syn::parse_file(code).unwrap();
        let cognitive = analyzer.calculate_cognitive(&ast);
        assert_eq!(cognitive, 0); // No control flow
    }

    #[test]
    fn test_calculate_cognitive_with_if() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_if() { if true { } }";
        let ast = syn::parse_file(code).unwrap();
        let cognitive = analyzer.calculate_cognitive(&ast);
        assert_eq!(cognitive, 1); // +1 for if at nesting 0
    }

    #[test]
    fn test_calculate_cognitive_nested_if() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn nested() {
                if true {
                    if false { }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cognitive = analyzer.calculate_cognitive(&ast);
        // First if: +1 (nesting 0)
        // Second if: +1 + 1 = +2 (nesting 1)
        assert_eq!(cognitive, 3);
    }

    #[test]
    fn test_calculate_cognitive_with_break() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_break() { loop { break; } }";
        let ast = syn::parse_file(code).unwrap();
        let cognitive = analyzer.calculate_cognitive(&ast);
        // +1 for break
        assert!(cognitive >= 1);
    }

    #[test]
    fn test_calculate_cognitive_with_continue() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_continue() { for i in 0..10 { continue; } }";
        let ast = syn::parse_file(code).unwrap();
        let cognitive = analyzer.calculate_cognitive(&ast);
        // +1 for for at nesting 0, +1 for continue
        assert!(cognitive >= 2);
    }

    #[test]
    fn test_analyze_string_success() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn test() { if true { } }";
        let result = analyzer.analyze_string(code);
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.cyclomatic, 2);
        assert_eq!(metrics.cognitive, 1);
    }

    #[test]
    fn test_analyze_string_parse_error() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "this is not valid rust code {{{{";
        let result = analyzer.analyze_string(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_shannon_entropy_uniform() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "aaaa"; // All same characters
        let entropy = analyzer.calculate_shannon_entropy(code);
        assert_eq!(entropy, 0.0); // Zero entropy for uniform distribution
    }

    #[test]
    fn test_calculate_shannon_entropy_diverse() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn calculate_prime(n: u64) -> bool { if n <= 1 { false } else { true } }";
        let entropy = analyzer.calculate_shannon_entropy(code);
        assert!(entropy > 3.0); // High entropy for diverse code
    }

    #[test]
    fn test_calculate_shannon_entropy_empty() {
        let analyzer = ComplexityAnalyzer::new();
        let entropy = analyzer.calculate_shannon_entropy("");
        assert!(entropy.is_nan() || entropy == 0.0); // Handle empty string
    }

    // ComplexityMetrics tests
    #[test]
    fn test_complexity_metrics_default() {
        let metrics = ComplexityMetrics::default();
        assert_eq!(metrics.cyclomatic, 0);
        assert_eq!(metrics.cognitive, 0);
    }

    #[test]
    fn test_complexity_metrics_clone() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 3,
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.cyclomatic, cloned.cyclomatic);
        assert_eq!(metrics.cognitive, cloned.cognitive);
    }

    #[test]
    fn test_complexity_metrics_serialization() {
        let metrics = ComplexityMetrics {
            cyclomatic: 10,
            cognitive: 7,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: ComplexityMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.cyclomatic, deserialized.cyclomatic);
        assert_eq!(metrics.cognitive, deserialized.cognitive);
    }

    // Complex code tests
    #[test]
    fn test_complex_nested_loops() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn complex() {
                for i in 0..10 {
                    for j in 0..10 {
                        for k in 0..10 {
                            if i == j && j == k {
                                break;
                            }
                        }
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cyclomatic = analyzer.calculate_cyclomatic(&ast);
        let cognitive = analyzer.calculate_cognitive(&ast);

        // High cyclomatic: 3 loops + 1 if + 1 && + base = 6
        assert!(cyclomatic >= 5);
        // High cognitive due to nesting
        assert!(cognitive >= 10);
    }

    #[test]
    fn test_match_with_many_arms() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn many_arms(x: i32) {
                match x {
                    0 => {},
                    1 => {},
                    2 => {},
                    3 => {},
                    4 => {},
                    5 => {},
                    _ => {},
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cyclomatic = analyzer.calculate_cyclomatic(&ast);
        // 7 arms - 1 = 6, + base = 7
        assert_eq!(cyclomatic, 7);
    }

    #[test]
    fn test_multiple_functions() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn func1() { if true { } }
            fn func2() { for i in 0..1 { } }
            fn func3() { while false { } }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cyclomatic = analyzer.calculate_cyclomatic(&ast);
        // Base + if + for + while = 1 + 1 + 1 + 1 = 4
        assert_eq!(cyclomatic, 4);
    }

    #[test]
    fn test_empty_file() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "";
        let ast = syn::parse_file(code).unwrap();
        let cyclomatic = analyzer.calculate_cyclomatic(&ast);
        assert_eq!(cyclomatic, 1); // Base complexity
    }

    #[test]
    fn test_only_struct_definition() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "struct Foo { x: i32, y: String }";
        let ast = syn::parse_file(code).unwrap();
        let cyclomatic = analyzer.calculate_cyclomatic(&ast);
        let cognitive = analyzer.calculate_cognitive(&ast);
        assert_eq!(cyclomatic, 1); // No control flow
        assert_eq!(cognitive, 0);
    }
}
