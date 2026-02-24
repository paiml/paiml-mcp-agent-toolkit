#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_complexity() {
        let code = r#"
            fn test(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        return x * 2;
                    }
                    return x + 1;
                }
                return 0;
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);

        assert!(cfg.cyclomatic_complexity() >= 3);
    }

    #[test]
    fn test_halstead_metrics() {
        let code = "let x = a + b * c;";
        let analyzer = ComplexityAnalyzer::new();
        let metrics = analyzer.calculate_halstead_metrics(code);

        assert!(metrics.vocabulary > 0);
        assert!(metrics.volume > 0.0);
    }

    #[test]
    fn test_maintainability_index() {
        let mi = calculate_maintainability_index(100.0, 5, 50);
        assert!((0.0..=100.0).contains(&mi));
    }

    #[test]
    fn test_kosaraju_scc_empty_graph() {
        let graph: SimpleDiGraph<&str, &str> = SimpleDiGraph::new();
        let sccs = graph.kosaraju_scc();
        assert!(sccs.is_empty());
    }

    // === CfgNode Tests ===

    #[test]
    fn test_cfg_node_entry() {
        let node = CfgNode::Entry;
        let debug = format!("{:?}", node);
        assert!(debug.contains("Entry"));
    }

    #[test]
    fn test_cfg_node_exit() {
        let node = CfgNode::Exit;
        let debug = format!("{:?}", node);
        assert!(debug.contains("Exit"));
    }

    #[test]
    fn test_cfg_node_statement() {
        let node = CfgNode::Statement("test_stmt".to_string());
        let debug = format!("{:?}", node);
        assert!(debug.contains("Statement"));
        assert!(debug.contains("test_stmt"));
    }

    #[test]
    fn test_cfg_node_condition() {
        let node = CfgNode::Condition("if_cond".to_string());
        let debug = format!("{:?}", node);
        assert!(debug.contains("Condition"));
    }

    #[test]
    fn test_cfg_node_branch() {
        let node = CfgNode::Branch("else_branch".to_string());
        let debug = format!("{:?}", node);
        assert!(debug.contains("Branch"));
    }

    #[test]
    fn test_cfg_node_clone() {
        let node = CfgNode::Condition("test".to_string());
        let cloned = node.clone();
        assert_eq!(format!("{:?}", node), format!("{:?}", cloned));
    }

    // === CfgEdge Tests ===

    #[test]
    fn test_cfg_edge_sequential() {
        let edge = CfgEdge::Sequential;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("Sequential"));
    }

    #[test]
    fn test_cfg_edge_true() {
        let edge = CfgEdge::True;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("True"));
    }

    #[test]
    fn test_cfg_edge_false() {
        let edge = CfgEdge::False;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("False"));
    }

    #[test]
    fn test_cfg_edge_jump() {
        let edge = CfgEdge::Jump;
        let debug = format!("{:?}", edge);
        assert!(debug.contains("Jump"));
    }

    #[test]
    fn test_cfg_edge_clone() {
        let edge = CfgEdge::True;
        let cloned = edge.clone();
        assert_eq!(format!("{:?}", edge), format!("{:?}", cloned));
    }

    // === ControlFlowGraph Tests ===

    #[test]
    fn test_cfg_simple_function() {
        let code = r#"
            fn simple() -> i32 {
                42
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.node_count() >= 2);
        assert!(cfg.edge_count() >= 1);
    }

    #[test]
    fn test_cfg_node_count() {
        let code = r#"
            fn test() {
                if true {
                    println!("yes");
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.node_count() > 2);
    }

    #[test]
    fn test_cfg_edge_count() {
        let code = r#"
            fn test() {
                if true {
                    println!("yes");
                } else {
                    println!("no");
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.edge_count() > 2);
    }

    #[test]
    fn test_cfg_essential_complexity() {
        let code = r#"
            fn test() {
                loop {
                    break;
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        let essential = cfg.essential_complexity();
        let _ = essential;
    }

    #[test]
    fn test_cfg_with_loop() {
        let code = r#"
            fn test() {
                loop {
                    if true {
                        break;
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let cfg = ControlFlowGraph::from_ast(&ast);
        assert!(cfg.cyclomatic_complexity() >= 1);
    }

    // === ComplexityMetrics Tests ===

    #[test]
    fn test_complexity_metrics_struct() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 8,
            max_nesting: 3,
            essential: 2,
        };
        assert_eq!(metrics.cyclomatic, 5);
        assert_eq!(metrics.cognitive, 8);
        assert_eq!(metrics.max_nesting, 3);
        assert_eq!(metrics.essential, 2);
    }

    #[test]
    fn test_complexity_metrics_clone() {
        let metrics = ComplexityMetrics {
            cyclomatic: 10,
            cognitive: 15,
            max_nesting: 4,
            essential: 3,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.cyclomatic, 10);
    }

    #[test]
    fn test_complexity_metrics_debug() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 8,
            max_nesting: 3,
            essential: 2,
        };
        let debug = format!("{:?}", metrics);
        assert!(debug.contains("ComplexityMetrics"));
        assert!(debug.contains("cyclomatic"));
    }

    // === FunctionMetrics Tests ===

    #[test]
    fn test_function_metrics_struct() {
        let metrics = FunctionMetrics {
            name: "test_func".to_string(),
            complexity: 5,
            lines: 20,
            parameters: 3,
        };
        assert_eq!(metrics.name, "test_func");
        assert_eq!(metrics.complexity, 5);
        assert_eq!(metrics.lines, 20);
        assert_eq!(metrics.parameters, 3);
    }

    #[test]
    fn test_function_metrics_clone() {
        let metrics = FunctionMetrics {
            name: "my_func".to_string(),
            complexity: 8,
            lines: 50,
            parameters: 2,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.name, "my_func");
    }

    // === HalsteadMetrics Tests ===

    #[test]
    fn test_halstead_metrics_struct() {
        let metrics = HalsteadMetrics {
            vocabulary: 10,
            length: 25,
            volume: 83.0,
            difficulty: 5.0,
            effort: 415.0,
        };
        assert_eq!(metrics.vocabulary, 10);
        assert_eq!(metrics.length, 25);
    }

    #[test]
    fn test_halstead_empty_code() {
        let analyzer = ComplexityAnalyzer::new();
        let metrics = analyzer.calculate_halstead_metrics("");
        assert_eq!(metrics.vocabulary, 0);
        assert_eq!(metrics.length, 0);
    }

    #[test]
    fn test_halstead_simple_expression() {
        let analyzer = ComplexityAnalyzer::new();
        let metrics = analyzer.calculate_halstead_metrics("x + y");
        assert!(metrics.vocabulary > 0);
    }

    // === is_operator Tests ===

    #[test]
    fn test_is_operator_arithmetic() {
        assert!(is_operator("+"));
        assert!(is_operator("-"));
        assert!(is_operator("*"));
        assert!(is_operator("/"));
        assert!(is_operator("%"));
    }

    #[test]
    fn test_is_operator_comparison() {
        assert!(is_operator("=="));
        assert!(is_operator("!="));
        assert!(is_operator("<"));
        assert!(is_operator(">"));
        assert!(is_operator("<="));
        assert!(is_operator(">="));
    }

    #[test]
    fn test_is_operator_logical() {
        assert!(is_operator("&&"));
        assert!(is_operator("||"));
        assert!(is_operator("!"));
    }

    #[test]
    fn test_is_operator_control_flow() {
        assert!(is_operator("if"));
        assert!(is_operator("else"));
        assert!(is_operator("match"));
        assert!(is_operator("for"));
        assert!(is_operator("while"));
        assert!(is_operator("loop"));
    }

    #[test]
    fn test_is_not_operator() {
        assert!(!is_operator("x"));
        assert!(!is_operator("variable"));
        assert!(!is_operator("123"));
    }

    // === is_operand Tests ===

    #[test]
    fn test_is_operand() {
        assert!(is_operand("x"));
        assert!(is_operand("variable"));
        assert!(is_operand("123"));
    }

    #[test]
    fn test_is_not_operand() {
        assert!(!is_operand("+"));
        assert!(!is_operand("if"));
    }

    // === tokenize Tests ===

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("a + b");
        assert!(tokens.contains(&"a".to_string()));
        assert!(tokens.contains(&"+".to_string()));
        assert!(tokens.contains(&"b".to_string()));
    }

    #[test]
    fn test_tokenize_identifier() {
        let tokens = tokenize("my_variable");
        assert!(tokens.contains(&"my_variable".to_string()));
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    // === calculate_maintainability_index Tests ===

    #[test]
    fn test_maintainability_index_perfect() {
        let mi = calculate_maintainability_index(10.0, 1, 10);
        assert!(mi > 0.0);
        assert!(mi <= 100.0);
    }

    #[test]
    fn test_maintainability_index_complex() {
        let mi = calculate_maintainability_index(1000.0, 50, 500);
        assert!(mi >= 0.0);
    }

    #[test]
    fn test_maintainability_index_bounds() {
        // Very high volume should clamp to lower bound
        let mi_low = calculate_maintainability_index(1000000.0, 100, 10000);
        assert!(mi_low >= 0.0);

        // Very low should still be in bounds
        let mi_high = calculate_maintainability_index(1.0, 1, 1);
        assert!(mi_high <= 100.0);
    }

    // === analyze_functions Tests ===

    #[test]
    fn test_analyze_functions_empty() {
        let code = r#"
            struct MyStruct {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let analyzer = ComplexityAnalyzer::new();
        let functions = analyzer.analyze_functions(&ast);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_analyze_functions_single() {
        let code = r#"
            fn my_function(x: i32, y: i32) -> i32 {
                x + y
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let analyzer = ComplexityAnalyzer::new();
        let functions = analyzer.analyze_functions(&ast);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "my_function");
        assert_eq!(functions[0].parameters, 2);
    }

    #[test]
    fn test_analyze_functions_multiple() {
        let code = r#"
            fn func1() {}
            fn func2(x: i32) {}
            fn func3(a: i32, b: i32, c: i32) {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let analyzer = ComplexityAnalyzer::new();
        let functions = analyzer.analyze_functions(&ast);
        assert_eq!(functions.len(), 3);
    }
}
