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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    Calls,
    Imports,
    Inherits,
    Implements,
    Uses,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: NodeInfo) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

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
