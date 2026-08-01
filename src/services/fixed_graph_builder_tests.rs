#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;

    fn create_test_graph() -> DependencyGraph {
        let mut nodes = FxHashMap::default();

        // Add some test nodes
        nodes.insert(
            "node1".to_string(),
            NodeInfo {
                id: "node1".to_string(),
                label: "foo".to_string(),
                node_type: NodeType::Module,
                file_path: "src/services/foo.rs".to_string(),
                line_number: 1,
                complexity: 10,
                metadata: FxHashMap::default(),
            },
        );

        nodes.insert(
            "node2".to_string(),
            NodeInfo {
                id: "node2".to_string(),
                label: "bar".to_string(),
                node_type: NodeType::Module,
                file_path: "src/models/bar.rs".to_string(),
                line_number: 1,
                complexity: 5,
                metadata: FxHashMap::default(),
            },
        );

        let edges = vec![Edge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: crate::models::dag::EdgeType::Imports,
            weight: 1,
        }];

        DependencyGraph { nodes, edges }
    }

    /// Regression test for #653: two nodes that render with the same display name
    /// (the module `main` and the function `main`) both have to survive. The map was
    /// keyed by display name, so one silently overwrote the other and `analyze dag`
    /// announced more nodes than it drew ("6 nodes", 4 node lines).
    #[test]
    fn test_nodes_with_identical_display_names_are_both_kept() {
        let mut nodes = FxHashMap::default();
        nodes.insert(
            "src_main".to_string(),
            NodeInfo {
                id: "src_main".to_string(),
                label: "main".to_string(),
                node_type: NodeType::Module,
                file_path: "src/main.rs".to_string(),
                line_number: 0,
                complexity: 1,
                metadata: FxHashMap::default(),
            },
        );
        nodes.insert(
            "src_main::main".to_string(),
            NodeInfo {
                id: "src_main::main".to_string(),
                label: "main".to_string(),
                node_type: NodeType::Function,
                file_path: "src/main.rs".to_string(),
                line_number: 4,
                complexity: 2,
                metadata: FxHashMap::default(),
            },
        );
        let graph = DependencyGraph {
            nodes,
            edges: vec![],
        };

        let builder = FixedGraphBuilder::new(GraphConfig {
            max_nodes: 50,
            max_edges: 400,
            grouping: GroupingStrategy::Module,
        });
        let fixed = builder.build(&graph).unwrap();

        assert_eq!(
            fixed.nodes.len(),
            2,
            "distinct nodes must not be merged by display name"
        );
    }

    #[test]
    fn test_deterministic_build() {
        let config = GraphConfig::default();
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        // Multiple runs should produce identical output
        let result1 = builder.build(&graph).unwrap();
        let result2 = builder.build(&graph).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_node_limit() {
        let config = GraphConfig {
            max_nodes: 1,
            max_edges: 10,
            grouping: GroupingStrategy::Module,
        };
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        let result = builder.build(&graph).unwrap();
        assert!(result.nodes.len() <= 1);
    }

    #[test]
    fn test_graph_config_default() {
        let config = GraphConfig::default();
        assert_eq!(config.max_nodes, 20);
        assert_eq!(config.max_edges, 60);
        assert_eq!(config.grouping, GroupingStrategy::Module);
    }

    #[test]
    fn test_graph_config_clone() {
        let config = GraphConfig {
            max_nodes: 50,
            max_edges: 100,
            grouping: GroupingStrategy::Directory,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_nodes, 50);
        assert_eq!(cloned.max_edges, 100);
        assert_eq!(cloned.grouping, GroupingStrategy::Directory);
    }

    #[test]
    fn test_graph_config_debug() {
        let config = GraphConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("GraphConfig"));
        assert!(debug.contains("max_nodes"));
    }

    #[test]
    fn test_grouping_strategy_clone() {
        let strategy = GroupingStrategy::Module;
        let cloned = strategy;
        assert_eq!(cloned, GroupingStrategy::Module);
    }

    #[test]
    fn test_grouping_strategy_variants() {
        assert_ne!(GroupingStrategy::Module, GroupingStrategy::Directory);
        assert_ne!(GroupingStrategy::Module, GroupingStrategy::None);
        assert_ne!(GroupingStrategy::Directory, GroupingStrategy::None);
    }

    #[test]
    fn test_grouping_strategy_debug() {
        let module = GroupingStrategy::Module;
        let directory = GroupingStrategy::Directory;
        let none = GroupingStrategy::None;

        assert!(format!("{:?}", module).contains("Module"));
        assert!(format!("{:?}", directory).contains("Directory"));
        assert!(format!("{:?}", none).contains("None"));
    }

    #[test]
    fn test_fixed_graph_struct() {
        let graph = FixedGraph {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        };
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_fixed_graph_clone() {
        let mut nodes = BTreeMap::new();
        nodes.insert(
            "node1".to_string(),
            FixedNode {
                id: "node1".to_string(),
                display_name: "Node 1".to_string(),
                node_type: NodeType::Module,
                complexity: 10,
            },
        );
        let graph = FixedGraph {
            nodes,
            edges: vec![],
        };
        let cloned = graph.clone();
        assert_eq!(cloned.nodes.len(), 1);
    }

    #[test]
    fn test_fixed_node_struct() {
        let node = FixedNode {
            id: "test".to_string(),
            display_name: "Test Node".to_string(),
            node_type: NodeType::Function,
            complexity: 5,
        };
        assert_eq!(node.id, "test");
        assert_eq!(node.display_name, "Test Node");
        assert_eq!(node.complexity, 5);
    }

    #[test]
    fn test_fixed_node_clone() {
        let node = FixedNode {
            id: "n1".to_string(),
            display_name: "Node".to_string(),
            node_type: NodeType::Module,
            complexity: 3,
        };
        let cloned = node.clone();
        assert_eq!(cloned.id, node.id);
        assert_eq!(cloned.complexity, node.complexity);
    }

    #[test]
    fn test_fixed_node_debug() {
        let node = FixedNode {
            id: "n1".to_string(),
            display_name: "Node".to_string(),
            node_type: NodeType::Module,
            complexity: 3,
        };
        let debug = format!("{:?}", node);
        assert!(debug.contains("FixedNode"));
        assert!(debug.contains("n1"));
    }

    #[test]
    fn test_fixed_graph_builder_new() {
        let config = GraphConfig::default();
        let _builder = FixedGraphBuilder::new(config);
    }

    #[test]
    fn test_fixed_graph_builder_custom_config() {
        let config = GraphConfig {
            max_nodes: 100,
            max_edges: 500,
            grouping: GroupingStrategy::None,
        };
        let _builder = FixedGraphBuilder::new(config);
    }

    #[test]
    fn test_build_empty_graph() {
        let config = GraphConfig::default();
        let builder = FixedGraphBuilder::new(config);
        let graph = DependencyGraph {
            nodes: FxHashMap::default(),
            edges: vec![],
        };
        let result = builder.build(&graph).unwrap();
        assert!(result.nodes.is_empty());
        assert!(result.edges.is_empty());
    }

    #[test]
    fn test_edge_limit() {
        let config = GraphConfig {
            max_nodes: 10,
            max_edges: 1,
            grouping: GroupingStrategy::Module,
        };
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        let result = builder.build(&graph).unwrap();
        // Edge limit should be respected
        assert!(result.edges.len() <= 1);
    }

    #[test]
    fn test_grouping_strategy_directory() {
        let config = GraphConfig {
            max_nodes: 10,
            max_edges: 20,
            grouping: GroupingStrategy::Directory,
        };
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        let result = builder.build(&graph).unwrap();
        assert!(result.nodes.len() <= 10);
    }

    #[test]
    fn test_grouping_strategy_none() {
        let config = GraphConfig {
            max_nodes: 10,
            max_edges: 20,
            grouping: GroupingStrategy::None,
        };
        let builder = FixedGraphBuilder::new(config);
        let graph = create_test_graph();

        let result = builder.build(&graph).unwrap();
        assert!(result.nodes.len() <= 10);
    }
}

