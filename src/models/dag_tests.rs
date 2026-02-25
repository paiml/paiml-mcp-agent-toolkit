#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, node_type: NodeType) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            label: id.split("::").last().unwrap_or(id).to_string(),
            node_type,
            file_path: "test.rs".to_string(),
            line_number: 1,
            complexity: 5,
            metadata: FxHashMap::default(),
        }
    }

    // ========================================================================
    // NodeType Tests
    // ========================================================================

    #[test]
    fn test_node_type_variants() {
        let function = NodeType::Function;
        let class = NodeType::Class;
        let module = NodeType::Module;
        let trait_type = NodeType::Trait;
        let interface = NodeType::Interface;

        assert_eq!(function, NodeType::Function);
        assert_ne!(function, class);
        assert_ne!(module, trait_type);
        assert_ne!(class, interface);
    }

    #[test]
    fn test_node_type_serialization() {
        let node_type = NodeType::Function;
        let json = serde_json::to_string(&node_type).unwrap();
        let deserialized: NodeType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, node_type);
    }

    // ========================================================================
    // EdgeType Tests
    // ========================================================================

    #[test]
    fn test_edge_type_variants() {
        let calls = EdgeType::Calls;
        let imports = EdgeType::Imports;
        let inherits = EdgeType::Inherits;
        let implements = EdgeType::Implements;
        let uses = EdgeType::Uses;

        assert_eq!(calls, EdgeType::Calls);
        assert_ne!(calls, imports);
        assert_ne!(inherits, implements);
        assert_ne!(uses, calls);
    }

    #[test]
    fn test_edge_type_hash() {
        let mut set = FxHashSet::default();
        set.insert(EdgeType::Calls);
        set.insert(EdgeType::Imports);
        set.insert(EdgeType::Calls); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&EdgeType::Calls));
        assert!(set.contains(&EdgeType::Imports));
    }

    #[test]
    fn test_edge_type_serialization() {
        let edge_type = EdgeType::Inherits;
        let json = serde_json::to_string(&edge_type).unwrap();
        let deserialized: EdgeType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, edge_type);
    }

    // ========================================================================
    // NodeInfo Tests
    // ========================================================================

    #[test]
    fn test_node_info_creation() {
        let node = create_test_node("main::hello", NodeType::Function);

        assert_eq!(node.id, "main::hello");
        assert_eq!(node.label, "hello");
        assert_eq!(node.node_type, NodeType::Function);
        assert_eq!(node.file_path, "test.rs");
        assert_eq!(node.line_number, 1);
        assert_eq!(node.complexity, 5);
    }

    #[test]
    fn test_node_info_with_metadata() {
        let mut metadata = FxHashMap::default();
        metadata.insert("visibility".to_string(), "public".to_string());
        metadata.insert("is_async".to_string(), "true".to_string());

        let node = NodeInfo {
            id: "mod::func".to_string(),
            label: "func".to_string(),
            node_type: NodeType::Function,
            file_path: "lib.rs".to_string(),
            line_number: 42,
            complexity: 10,
            metadata,
        };

        assert_eq!(node.metadata.get("visibility"), Some(&"public".to_string()));
        assert_eq!(node.metadata.get("is_async"), Some(&"true".to_string()));
    }

    #[test]
    fn test_node_info_serialization() {
        let node = create_test_node("test::node", NodeType::Class);
        let json = serde_json::to_string(&node).unwrap();
        let deserialized: NodeInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, node.id);
        assert_eq!(deserialized.node_type, node.node_type);
    }

    // ========================================================================
    // Edge Tests
    // ========================================================================

    #[test]
    fn test_edge_creation() {
        let edge = Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        };

        assert_eq!(edge.from, "a");
        assert_eq!(edge.to, "b");
        assert_eq!(edge.edge_type, EdgeType::Calls);
        assert_eq!(edge.weight, 1);
    }

    #[test]
    fn test_edge_serialization() {
        let edge = Edge {
            from: "module::func1".to_string(),
            to: "module::func2".to_string(),
            edge_type: EdgeType::Imports,
            weight: 5,
        };

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: Edge = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, edge);
    }

    // ========================================================================
    // DependencyGraph Tests
    // ========================================================================

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_dependency_graph_default() {
        let graph = DependencyGraph::default();

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_dependency_graph_add_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("func1", NodeType::Function));
        graph.add_node(create_test_node("func2", NodeType::Function));

        assert_eq!(graph.node_count(), 2);
        assert!(graph.nodes.contains_key("func1"));
        assert!(graph.nodes.contains_key("func2"));
    }

    #[test]
    fn test_dependency_graph_add_duplicate_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("func1", NodeType::Function));
        graph.add_node(create_test_node("func1", NodeType::Class)); // same id, different type

        // Should overwrite with new node
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.nodes.get("func1").unwrap().node_type, NodeType::Class);
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_dependency_graph_add_multiple_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });
        graph.add_edge(Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            edge_type: EdgeType::Imports,
            weight: 2,
        });
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "c".to_string(),
            edge_type: EdgeType::Uses,
            weight: 1,
        });

        assert_eq!(graph.edge_count(), 3);
    }

    #[test]
    fn test_filter_by_edge_type_calls() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("a", NodeType::Function));
        graph.add_node(create_test_node("b", NodeType::Function));
        graph.add_node(create_test_node("c", NodeType::Module));

        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });
        graph.add_edge(Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            edge_type: EdgeType::Imports,
            weight: 1,
        });

        let calls_only = graph.filter_by_edge_type(EdgeType::Calls);

        assert_eq!(calls_only.edge_count(), 1);
        assert_eq!(calls_only.edges[0].edge_type, EdgeType::Calls);
        assert_eq!(calls_only.node_count(), 2); // only nodes connected by Calls edges
        assert!(calls_only.nodes.contains_key("a"));
        assert!(calls_only.nodes.contains_key("b"));
        assert!(!calls_only.nodes.contains_key("c"));
    }

    #[test]
    fn test_filter_by_edge_type_no_match() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("a", NodeType::Function));
        graph.add_edge(Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let inherits_only = graph.filter_by_edge_type(EdgeType::Inherits);

        assert_eq!(inherits_only.edge_count(), 0);
        assert_eq!(inherits_only.node_count(), 0); // no nodes since no edges match
    }

    #[test]
    fn test_filter_by_edge_type_empty_graph() {
        let graph = DependencyGraph::new();
        let filtered = graph.filter_by_edge_type(EdgeType::Calls);

        assert_eq!(filtered.edge_count(), 0);
        assert_eq!(filtered.node_count(), 0);
    }

    #[test]
    fn test_filter_by_edge_type_nodes_no_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("a", NodeType::Function));
        graph.add_node(create_test_node("b", NodeType::Function));
        // No edges added

        let filtered = graph.filter_by_edge_type(EdgeType::Calls);

        // Should return all nodes since no edges exist
        assert_eq!(filtered.edge_count(), 0);
        assert_eq!(filtered.node_count(), 2);
    }

    #[test]
    fn test_dependency_graph_serialization() {
        let mut graph = DependencyGraph::new();
        graph.add_node(create_test_node("func1", NodeType::Function));
        graph.add_edge(Edge {
            from: "func1".to_string(),
            to: "func2".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: DependencyGraph = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.node_count(), 1);
        assert_eq!(deserialized.edge_count(), 1);
    }

    // ========================================================================
    // DagType Tests
    // ========================================================================

    #[test]
    fn test_dag_type_variants() {
        let call_graph = DagType::CallGraph;
        let import_graph = DagType::ImportGraph;
        let inheritance = DagType::Inheritance;
        let full = DagType::FullDependency;

        assert_eq!(call_graph, DagType::CallGraph);
        assert_ne!(call_graph, import_graph);
        assert_ne!(inheritance, full);
    }

    #[test]
    fn test_dag_type_hash() {
        let mut set = FxHashSet::default();
        set.insert(DagType::CallGraph);
        set.insert(DagType::ImportGraph);
        set.insert(DagType::CallGraph); // duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_dag_type_serialization() {
        let dag_type = DagType::FullDependency;
        let json = serde_json::to_string(&dag_type).unwrap();
        let deserialized: DagType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dag_type);
    }
}
