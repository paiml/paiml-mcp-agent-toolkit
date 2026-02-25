// Complexity scorer tests
// Included from complexity.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
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
