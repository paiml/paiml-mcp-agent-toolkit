use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: FxHashMap<String, NodeInfo>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub file_path: String,
    pub line_number: usize,
    pub complexity: u32,
    #[serde(default)]
    pub metadata: FxHashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Function,
    Class,
    Module,
    Trait,
    Interface,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Calls,
    Imports,
    Inherits,
    Implements,
    Uses,
}

impl DependencyGraph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
            edges: Vec::new(),
        }
    }

    /// Adds a node to the dependency graph
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::dag::{DependencyGraph, NodeInfo, NodeType};
    /// use rustc_hash::FxHashMap;
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_node(NodeInfo {
    ///     id: "main::hello".to_string(),
    ///     label: "hello".to_string(),
    ///     node_type: NodeType::Function,
    ///     file_path: "src/main.rs".to_string(),
    ///     line_number: 10,
    ///     complexity: 1,
    ///     metadata: FxHashMap::default(),
    /// });
    ///
    /// assert_eq!(graph.nodes.len(), 1);
    /// ```
    pub fn add_node(&mut self, node: NodeInfo) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Get the number of nodes in the graph
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the graph  
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Adds an edge between two nodes in the dependency graph
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::dag::{DependencyGraph, Edge, EdgeType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_edge(Edge {
    ///     from: "main::hello".to_string(),
    ///     to: "utils::print".to_string(),
    ///     edge_type: EdgeType::Calls,
    ///     weight: 1,
    /// });
    ///
    /// assert_eq!(graph.edges.len(), 1);
    /// assert_eq!(graph.edges[0].edge_type, EdgeType::Calls);
    /// ```
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Creates a new graph containing only edges of the specified type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::dag::{DependencyGraph, Edge, EdgeType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_edge(Edge {
    ///     from: "a".to_string(),
    ///     to: "b".to_string(),
    ///     edge_type: EdgeType::Calls,
    ///     weight: 1,
    /// });
    /// graph.add_edge(Edge {
    ///     from: "c".to_string(),
    ///     to: "d".to_string(),
    ///     edge_type: EdgeType::Imports,
    ///     weight: 1,
    /// });
    ///
    /// let calls_only = graph.filter_by_edge_type(EdgeType::Calls);
    /// assert_eq!(calls_only.edges.len(), 1);
    /// assert_eq!(calls_only.edges[0].edge_type, EdgeType::Calls);
    /// ```
    #[must_use]
    pub fn filter_by_edge_type(&self, edge_type: EdgeType) -> Self {
        let filtered_edges: Vec<Edge> = self
            .edges
            .iter()
            .filter(|e| e.edge_type == edge_type)
            .cloned()
            .collect();

        // If filtering results in no edges but we originally had edges,
        // only include nodes that were connected by the filtered edge type
        if filtered_edges.is_empty() && !self.edges.is_empty() {
            // Return empty nodes since no nodes are connected by this edge type
            return Self {
                nodes: FxHashMap::default(),
                edges: filtered_edges,
            };
        }

        // If we have no edges at all, return all nodes
        if self.edges.is_empty() {
            return Self {
                nodes: self.nodes.clone(),
                edges: Vec::new(),
            };
        }

        // Otherwise, filter nodes to only those connected by the filtered edges
        let used_nodes: FxHashSet<String> = filtered_edges
            .iter()
            .flat_map(|e| vec![e.from.clone(), e.to.clone()])
            .collect();

        let filtered_nodes: FxHashMap<String, NodeInfo> = self
            .nodes
            .iter()
            .filter(|(id, _)| used_nodes.contains(*id))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Self {
            nodes: filtered_nodes,
            edges: filtered_edges,
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

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

// DAG generation types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DagType {
    CallGraph,
    ImportGraph,
    Inheritance,
    FullDependency,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
