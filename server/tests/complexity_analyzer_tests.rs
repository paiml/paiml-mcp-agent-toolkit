use pmat::quality::complexity::{ComplexityAnalyzer, ComplexityMetrics};
use pmat::quality::complexity_enhanced::ControlFlowGraph;
use proptest::prelude::*;

#[cfg(test)]
mod complexity_analyzer_tests {
    use super::*;

    #[test]
    fn test_mccabe_complexity_calculation() {
        let analyzer = ComplexityAnalyzer::new();

        // Simple function - complexity 1
        let simple = r#"
            fn simple() {
                println!("Hello");
            }
        "#;

        let ast = syn::parse_file(simple).unwrap();
        assert_eq!(analyzer.calculate_cyclomatic(&ast), 1);

        // Function with one if - complexity 2
        let with_if = r#"
            fn with_if(x: i32) {
                if x > 0 {
                    println!("Positive");
                }
            }
        "#;

        let ast = syn::parse_file(with_if).unwrap();
        assert_eq!(analyzer.calculate_cyclomatic(&ast), 2);

        // Function with nested loops - higher complexity
        let nested = r#"
            fn nested(data: &[i32]) {
                for i in data {
                    for j in data {
                        if i > j {
                            println!("{} > {}", i, j);
                        }
                    }
                }
            }
        "#;

        let ast = syn::parse_file(nested).unwrap();
        assert!(analyzer.calculate_cyclomatic(&ast) >= 4);
    }

    #[test]
    fn test_cognitive_complexity_rules() {
        let analyzer = ComplexityAnalyzer::new();

        // Nesting increases cognitive complexity
        let nested = r#"
            fn nested(x: i32, y: i32) {
                if x > 0 {                    // +1
                    if y > 0 {                // +2 (nested)
                        println!("Both positive");
                    }
                }
            }
        "#;

        let ast = syn::parse_file(nested).unwrap();
        let cognitive = analyzer.calculate_cognitive(&ast);
        assert_eq!(cognitive, 3);
    }

    #[test]
    fn test_cfg_construction() {
        let code = r#"
            fn test(x: i32) -> i32 {
                if x > 0 {
                    return x * 2;
                } else {
                    return x * 3;
                }
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);

        assert_eq!(cfg.node_count(), 5); // Entry, condition, true branch, false branch, exit
        assert_eq!(cfg.edge_count(), 5); // Edges connecting nodes
    }

    proptest! {
        #[test]
        fn proptest_complexity_monotonicity(
            num_ifs in 0..10usize,
            num_loops in 0..5usize,
            num_matches in 0..3usize,
        ) {
            let analyzer = ComplexityAnalyzer::new();

            let code1 = generate_code(num_ifs, num_loops, num_matches);
            let code2 = generate_code(num_ifs + 1, num_loops, num_matches);
            let code3 = generate_code(num_ifs, num_loops + 1, num_matches);

            let c1 = analyzer.analyze_string(&code1).unwrap().cyclomatic;
            let c2 = analyzer.analyze_string(&code2).unwrap().cyclomatic;
            let c3 = analyzer.analyze_string(&code3).unwrap().cyclomatic;

            prop_assert!(c2 >= c1, "Adding if should not decrease complexity");
            prop_assert!(c3 >= c1, "Adding loop should not decrease complexity");
        }
    }

    #[test]
    fn test_nesting_depth_calculation() {
        let analyzer = ComplexityAnalyzer::new();

        let deeply_nested = r#"
            fn deeply_nested(a: i32, b: i32, c: i32) {
                if a > 0 {
                    if b > 0 {
                        if c > 0 {
                            for i in 0..10 {
                                println!("{}", i);
                            }
                        }
                    }
                }
            }
        "#;

        let metrics = analyzer.analyze_string(deeply_nested).unwrap();
        assert_eq!(metrics.max_nesting, 4);
    }

    #[test]
    fn test_function_metrics() {
        let analyzer = ComplexityAnalyzer::new();

        let code = r#"
            fn short() { }

            fn medium(x: i32, y: i32) -> i32 {
                if x > y {
                    x
                } else {
                    y
                }
            }

            fn long(data: &[i32]) -> i32 {
                let mut sum = 0;
                for val in data {
                    sum += val;
                    if sum > 100 {
                        break;
                    }
                }
                sum
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let functions = analyzer.analyze_functions(&ast);

        assert_eq!(functions.len(), 3);
        assert!(functions[0].complexity < functions[2].complexity);
    }

    #[test]
    fn test_halstead_metrics() {
        let analyzer = ComplexityAnalyzer::new();

        let code = r#"
            fn calculate(x: i32, y: i32) -> i32 {
                let sum = x + y;
                let product = x * y;
                if sum > product {
                    sum
                } else {
                    product
                }
            }
        "#;

        let metrics = analyzer.calculate_halstead_metrics(code);

        assert!(metrics.vocabulary > 0);
        assert!(metrics.length > 0);
        assert!(metrics.volume > 0.0);
        assert!(metrics.difficulty > 0.0);
        assert!(metrics.effort > 0.0);
    }

    // Benchmark test
    #[test]
    #[ignore] // Run with --ignored flag
    fn bench_complexity_analysis() {
        use std::time::Instant;

        let analyzer = ComplexityAnalyzer::new();
        let large_code = generate_large_code(100, 50, 20);

        let start = Instant::now();
        let result = analyzer.analyze_string(&large_code);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed.as_millis() < 100,
            "Analysis took too long: {:?}",
            elapsed
        );
    }

    fn generate_code(num_ifs: usize, num_loops: usize, num_matches: usize) -> String {
        let mut code = String::from("fn test() {\n");

        for i in 0..num_ifs {
            code.push_str(&format!("    if x{} > 0 {{\n", i));
            code.push_str(&format!("        println!(\"{}\");\n", i));
            code.push_str("    }\n");
        }

        for i in 0..num_loops {
            code.push_str(&format!("    for i{} in 0..10 {{\n", i));
            code.push_str(&format!("        println!(\"{{}}\", i{});\n", i));
            code.push_str("    }\n");
        }

        for i in 0..num_matches {
            code.push_str(&format!("    match x{} {{\n", i));
            code.push_str("        0 => println!(\"zero\"),\n");
            code.push_str("        1 => println!(\"one\"),\n");
            code.push_str("        _ => println!(\"other\"),\n");
            code.push_str("    }\n");
        }

        code.push_str("}\n");
        code
    }

    fn generate_large_code(functions: usize, lines_per_fn: usize, complexity: usize) -> String {
        let mut code = String::new();

        for f in 0..functions {
            code.push_str(&format!("fn func_{}() {{\n", f));

            for _ in 0..lines_per_fn {
                code.push_str("    let x = 42;\n");
            }

            for c in 0..complexity {
                code.push_str(&format!("    if condition_{} {{\n", c));
                code.push_str("        action();\n");
                code.push_str("    }\n");
            }

            code.push_str("}\n\n");
        }

        code
    }
}
