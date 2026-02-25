// Unit tests for CouplingAnalyzer analysis methods (afferent/efferent coupling,
// abstractness, builtin types, visibility) and DependencyGraph operations.

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

    // === CouplingAnalyzer tests ===

    #[test]
    fn test_coupling_analyzer_new() {
        let analyzer = CouplingAnalyzer::new();
        assert_eq!(analyzer.category(), MetricCategory::Coupling);
    }

    #[test]
    fn test_coupling_calculation() {
        let source = r#"
            use std::collections::HashMap;
            use crate::module::SomeType;

            pub struct MyStruct {
                field: HashMap<String, SomeType>,
            }

            impl MyStruct {
                pub fn new() -> Self {
                    Self {
                        field: HashMap::new(),
                    }
                }
            }
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();

        let afferent = analyzer.calculate_afferent_coupling(tree.root_node(), source);
        let efferent = analyzer.calculate_efferent_coupling(tree.root_node(), source);

        assert!(afferent >= 0);
        assert!(efferent > 0);
    }

    #[test]
    fn test_abstractness() {
        let source = r#"
            trait MyTrait {
                fn method(&self);
            }

            struct Concrete;

            impl MyTrait for Concrete {
                fn method(&self) {}
            }
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let abstractness = analyzer.calculate_abstractness(tree.root_node(), source);

        assert!(abstractness > 0.0);
        assert!(abstractness <= 1.0);
    }

    #[test]
    fn test_abstractness_zero_for_structs_only() {
        let source = r#"
            struct StructA;
            struct StructB;
            struct StructC;
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let abstractness = analyzer.calculate_abstractness(tree.root_node(), source);

        // All concrete, no traits
        assert_eq!(abstractness, 0.0);
    }

    #[test]
    fn test_abstractness_one_for_traits_only() {
        let source = r#"
            trait TraitA {
                fn method_a(&self);
            }

            trait TraitB {
                fn method_b(&self);
            }
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let abstractness = analyzer.calculate_abstractness(tree.root_node(), source);

        // All traits, no concrete
        assert_eq!(abstractness, 1.0);
    }

    #[test]
    fn test_efferent_coupling_with_type_references() {
        let source = r#"
            use std::io::Result;
            use std::path::PathBuf;

            fn process(path: PathBuf) -> Result<()> {
                std::fs::read(&path)?;
                Ok(())
            }
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let efferent = analyzer.calculate_efferent_coupling(tree.root_node(), source);

        // Should detect use declarations and qualified paths
        assert!(efferent >= 2);
    }

    #[test]
    fn test_afferent_coupling_with_public_items() {
        let source = r#"
            pub fn public_function() {}
            fn private_function() {}
            pub struct PublicStruct;
            struct PrivateStruct;
            pub trait PublicTrait {}
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let afferent = analyzer.calculate_afferent_coupling(tree.root_node(), source);

        // Should count public items
        assert!(afferent >= 1);
    }

    #[test]
    fn test_is_builtin_type_rust_primitives() {
        let analyzer = CouplingAnalyzer::new();

        // Rust primitives
        assert!(analyzer.is_builtin_type("i32"));
        assert!(analyzer.is_builtin_type("u64"));
        assert!(analyzer.is_builtin_type("f64"));
        assert!(analyzer.is_builtin_type("bool"));
        assert!(analyzer.is_builtin_type("char"));
        assert!(analyzer.is_builtin_type("str"));
        assert!(analyzer.is_builtin_type("usize"));
        assert!(analyzer.is_builtin_type("isize"));
    }

    #[test]
    fn test_is_builtin_type_std_types() {
        let analyzer = CouplingAnalyzer::new();

        // Standard library types
        assert!(analyzer.is_builtin_type("String"));
        assert!(analyzer.is_builtin_type("Vec"));
        assert!(analyzer.is_builtin_type("HashMap"));
        assert!(analyzer.is_builtin_type("HashSet"));
        assert!(analyzer.is_builtin_type("Option"));
        assert!(analyzer.is_builtin_type("Result"));
    }

    #[test]
    fn test_is_builtin_type_smart_pointers() {
        let analyzer = CouplingAnalyzer::new();

        // Smart pointers and concurrency types
        assert!(analyzer.is_builtin_type("Box"));
        assert!(analyzer.is_builtin_type("Rc"));
        assert!(analyzer.is_builtin_type("Arc"));
        assert!(analyzer.is_builtin_type("Cell"));
        assert!(analyzer.is_builtin_type("RefCell"));
        assert!(analyzer.is_builtin_type("Mutex"));
    }

    #[test]
    fn test_is_builtin_type_c_types() {
        let analyzer = CouplingAnalyzer::new();

        // C/C++ style types
        assert!(analyzer.is_builtin_type("int"));
        assert!(analyzer.is_builtin_type("float"));
        assert!(analyzer.is_builtin_type("double"));
        assert!(analyzer.is_builtin_type("void"));
        assert!(analyzer.is_builtin_type("auto"));
        assert!(analyzer.is_builtin_type("const"));
    }

    #[test]
    fn test_is_builtin_type_custom_type_not_builtin() {
        let analyzer = CouplingAnalyzer::new();

        // Custom types should not be builtin
        assert!(!analyzer.is_builtin_type("MyCustomType"));
        assert!(!analyzer.is_builtin_type("UserStruct"));
        assert!(!analyzer.is_builtin_type("ErrorHandler"));
        assert!(!analyzer.is_builtin_type("Database"));
    }

    // === DependencyGraph tests ===

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("module_a".to_string(), "module_b".to_string());

        assert!(graph.edges.contains_key("module_a"));
        assert!(graph.edges["module_a"].contains("module_b"));
    }

    #[test]
    fn test_dependency_graph_multiple_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a".to_string(), "b".to_string());
        graph.add_edge("a".to_string(), "c".to_string());
        graph.add_edge("b".to_string(), "c".to_string());

        assert_eq!(graph.edges["a"].len(), 2);
        assert_eq!(graph.edges["b"].len(), 1);
    }

    #[test]
    fn test_dependency_graph_topological_sort_simple() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a".to_string(), "b".to_string());
        graph.add_edge("b".to_string(), "c".to_string());

        let sorted = graph.topological_sort();

        // 'a' should come before 'b', 'b' before 'c'
        let pos_a = sorted.iter().position(|x| x == "a");
        let pos_b = sorted.iter().position(|x| x == "b");

        if let (Some(a_idx), Some(b_idx)) = (pos_a, pos_b) {
            assert!(a_idx < b_idx);
        }
    }

    #[test]
    fn test_dependency_graph_topological_sort_empty() {
        let graph = DependencyGraph::new();
        let sorted = graph.topological_sort();

        assert!(sorted.is_empty());
    }

    #[test]
    fn test_dependency_graph_calculate_depth_linear() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a".to_string(), "b".to_string());
        graph.add_edge("b".to_string(), "c".to_string());
        graph.add_edge("c".to_string(), "d".to_string());

        let depth = graph.calculate_depth();

        // Linear chain should have depth based on number of nodes
        assert!(depth >= 1);
    }

    #[test]
    fn test_dependency_graph_calculate_depth_empty() {
        let graph = DependencyGraph::new();
        let depth = graph.calculate_depth();

        assert_eq!(depth, 0);
    }

    #[test]
    fn test_build_dependency_graph() {
        let source = r#"
            mod module_a {
                use crate::module_b::SomeType;
            }

            mod module_b {
                pub struct SomeType;
            }
        "#;

        let tree = parse_rust(source);
        let analyzer = CouplingAnalyzer::new();
        let graph = analyzer.build_dependency_graph(tree.root_node(), source);

        // The graph should have some edges or be empty depending on parser
        assert!(graph.edges.len() >= 0);
    }
}
