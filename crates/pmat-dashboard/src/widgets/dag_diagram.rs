//! DagDiagram Widget - Replaces Mermaid.js
//!
//! A pure Presentar flow diagram with:
//! - Node and edge rendering
//! - Pan/zoom support
//! - Mermaid syntax parsing
//! - Click event handling

use crate::state::ZoomConfig;
use std::collections::HashMap;

/// DAG node
#[derive(Debug, Clone)]
pub struct DagNode {
    id: String,
    label: String,
    node_type: NodeType,
}

impl DagNode {
    /// Create a new node
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            label: id.to_string(),
            node_type: NodeType::Rectangle,
        }
    }

    /// Set node label
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    /// Set node type
    pub fn node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }
}

/// Node visual type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Rectangle,
    Rounded,
    Circle,
    Diamond,
}

/// DAG edge
#[derive(Debug, Clone)]
pub struct DagEdge {
    /// Source node ID (used for graph traversal)
    #[allow(dead_code)]
    from: String,
    /// Target node ID (used for graph traversal)
    #[allow(dead_code)]
    to: String,
    label: Option<String>,
    edge_type: EdgeType,
}

impl DagEdge {
    /// Create a new edge
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            edge_type: EdgeType::Arrow,
        }
    }

    /// Set edge label
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set edge type
    pub fn edge_type(mut self, edge_type: EdgeType) -> Self {
        self.edge_type = edge_type;
        self
    }
}

/// Edge visual type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Arrow,
    Dotted,
    Thick,
}

/// DAG diagram widget
#[derive(Debug, Clone)]
pub struct DagDiagram {
    nodes: HashMap<String, DagNode>,
    edges: Vec<DagEdge>,
    zoom_config: Option<ZoomConfig>,
    current_zoom: f32,
    /// Pan offset for viewport scrolling (reserved for rendering)
    #[allow(dead_code)]
    pan_offset: (f32, f32),
    click_handler: Option<fn(&str)>,
}

impl DagDiagram {
    /// Create a new diagram
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            zoom_config: None,
            current_zoom: 1.0,
            pan_offset: (0.0, 0.0),
            click_handler: None,
        }
    }

    /// Add a node
    pub fn add_node(mut self, node: DagNode) -> Self {
        self.nodes.insert(node.id.clone(), node);
        self
    }

    /// Add an edge
    pub fn add_edge(mut self, edge: DagEdge) -> Self {
        self.edges.push(edge);
        self
    }

    /// Configure zoom
    pub fn with_zoom(mut self, config: ZoomConfig) -> Self {
        self.zoom_config = Some(config);
        self
    }

    /// Set zoom level
    pub fn zoom_to(&mut self, level: f32) {
        if let Some(config) = &self.zoom_config {
            self.current_zoom = level.clamp(config.min, config.max);
        } else {
            self.current_zoom = level;
        }
    }

    /// Get current zoom level
    pub fn current_zoom(&self) -> f32 {
        self.current_zoom
    }

    /// Set click handler
    pub fn on_node_click(mut self, handler: fn(&str)) -> Self {
        self.click_handler = Some(handler);
        self
    }

    /// Simulate a click event (for testing)
    pub fn simulate_click(&self, node_id: &str) {
        if let Some(handler) = self.click_handler {
            handler(node_id);
        }
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Parse Mermaid syntax
    pub fn from_mermaid(mmd: &str) -> Result<Self, MermaidParseError> {
        let mut diagram = Self::new();

        for line in mmd.lines() {
            let line = line.trim();

            // Skip empty lines and graph declaration
            if line.is_empty() || line.starts_with("graph") || line.starts_with("flowchart") {
                continue;
            }

            // Parse edge: A --> B or A[Label] --> B[Label]
            if line.contains("-->") {
                let parts: Vec<&str> = line.split("-->").collect();
                if parts.len() == 2 {
                    let from = parse_node_id(parts[0].trim());
                    let to = parse_node_id(parts[1].trim());

                    // Add nodes if not exists
                    if !diagram.nodes.contains_key(&from.0) {
                        diagram
                            .nodes
                            .insert(from.0.clone(), DagNode::new(&from.0).label(&from.1));
                    }
                    if !diagram.nodes.contains_key(&to.0) {
                        diagram
                            .nodes
                            .insert(to.0.clone(), DagNode::new(&to.0).label(&to.1));
                    }

                    // Add edge
                    diagram.edges.push(DagEdge::new(&from.0, &to.0));
                }
            }
        }

        if diagram.nodes.is_empty() {
            return Err(MermaidParseError::EmptyDiagram);
        }

        Ok(diagram)
    }
}

impl Default for DagDiagram {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse node id and label from Mermaid syntax
/// Examples: "A", "A[Label]", "A[Label Text]"
fn parse_node_id(s: &str) -> (String, String) {
    if let Some(bracket_start) = s.find('[') {
        if let Some(bracket_end) = s.find(']') {
            let id = s[..bracket_start].trim().to_string();
            let label = s[bracket_start + 1..bracket_end].trim().to_string();
            return (id, label);
        }
    }
    (s.to_string(), s.to_string())
}

/// Mermaid parse error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MermaidParseError {
    EmptyDiagram,
    InvalidSyntax(String),
}

impl std::fmt::Display for MermaidParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDiagram => write!(f, "Empty diagram"),
            Self::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
        }
    }
}

impl std::error::Error for MermaidParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_nodes() {
        let dag = DagDiagram::new()
            .add_node(DagNode::new("a").label("Node A"))
            .add_node(DagNode::new("b").label("Node B"));
        assert_eq!(dag.node_count(), 2);
    }

    #[test]
    fn test_dag_edges() {
        let dag = DagDiagram::new()
            .add_node(DagNode::new("a"))
            .add_node(DagNode::new("b"))
            .add_edge(DagEdge::new("a", "b").label("connects"));
        assert_eq!(dag.edge_count(), 1);
    }

    #[test]
    fn test_dag_zoom() {
        let mut dag = DagDiagram::new().with_zoom(ZoomConfig { min: 0.5, max: 2.0 });
        dag.zoom_to(1.5);
        assert_eq!(dag.current_zoom(), 1.5);

        // Test clamping
        dag.zoom_to(3.0);
        assert_eq!(dag.current_zoom(), 2.0);
    }

    #[test]
    fn test_parse_mermaid_simple() {
        let mmd = r#"
            graph TD
                A[Parser] --> B[Analyzer]
                B --> C[Generator]
        "#;
        let dag = DagDiagram::from_mermaid(mmd).unwrap();
        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);
    }

    #[test]
    fn test_parse_mermaid_labels() {
        let mmd = "A[Node A] --> B[Node B]";
        let dag = DagDiagram::from_mermaid(mmd).unwrap();
        assert_eq!(dag.node_count(), 2);
    }

    #[test]
    fn test_parse_node_id() {
        assert_eq!(parse_node_id("A"), ("A".to_string(), "A".to_string()));
        assert_eq!(
            parse_node_id("A[Label]"),
            ("A".to_string(), "Label".to_string())
        );
        assert_eq!(
            parse_node_id("A[Multi Word]"),
            ("A".to_string(), "Multi Word".to_string())
        );
    }
}
