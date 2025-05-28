use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use crate::services::context::{AstItem, FileContext, ProjectContext};

pub struct DagBuilder {
    graph: DependencyGraph,
}

impl DagBuilder {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    pub fn build_from_project(project: &ProjectContext) -> DependencyGraph {
        let mut builder = Self::new();

        for file in &project.files {
            builder.process_file_context(file);
        }

        builder.graph
    }

    fn process_file_context(&mut self, file: &FileContext) {
        for item in &file.items {
            match item {
                AstItem::Function {
                    name,
                    line,
                    visibility: _,
                    is_async: _,
                } => {
                    self.add_node(NodeInfo {
                        id: format!("{}::{}", file.path, name),
                        label: name.clone(),
                        node_type: NodeType::Function,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: self.calculate_complexity(item),
                    });
                }
                AstItem::Struct {
                    name,
                    line,
                    fields_count: _,
                    derives: _,
                    visibility: _,
                } => {
                    self.add_node(NodeInfo {
                        id: format!("{}::{}", file.path, name),
                        label: name.clone(),
                        node_type: NodeType::Class,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: self.calculate_complexity(item),
                    });
                }
                AstItem::Trait {
                    name,
                    line,
                    visibility: _,
                } => {
                    self.add_node(NodeInfo {
                        id: format!("{}::{}", file.path, name),
                        label: name.clone(),
                        node_type: NodeType::Trait,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: 1,
                    });
                }
                AstItem::Module {
                    name,
                    line,
                    visibility: _,
                } => {
                    self.add_node(NodeInfo {
                        id: format!("{}::{}", file.path, name),
                        label: name.clone(),
                        node_type: NodeType::Module,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: 1,
                    });
                }
                AstItem::Use { path, line: _ } => {
                    // Create edges for imports
                    let from_module = self.get_current_module(&file.path);
                    self.add_edge(Edge {
                        from: from_module.clone(),
                        to: path.clone(),
                        edge_type: EdgeType::Imports,
                        weight: 1,
                    });
                }
                _ => {}
            }
        }

        // Process function calls within the file
        self.process_function_calls(file);
    }

    fn add_node(&mut self, node: NodeInfo) {
        self.graph.add_node(node);
    }

    fn add_edge(&mut self, edge: Edge) {
        self.graph.add_edge(edge);
    }

    fn calculate_complexity(&self, item: &AstItem) -> u32 {
        match item {
            AstItem::Function { .. } => {
                // Simple complexity calculation - could be enhanced with actual AST analysis
                2
            }
            AstItem::Struct { fields_count, .. } => *fields_count as u32 + 1,
            _ => 1,
        }
    }

    fn get_current_module(&self, file_path: &str) -> String {
        // Extract module name from file path
        file_path.to_string()
    }

    fn process_function_calls(&mut self, _file: &FileContext) {
        // This would require deeper AST analysis to track function calls
        // For now, we'll leave this as a placeholder
    }
}

impl Default for DagBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for filtering graphs
pub fn filter_call_edges(graph: DependencyGraph) -> DependencyGraph {
    graph.filter_by_edge_type(EdgeType::Calls)
}

pub fn filter_import_edges(graph: DependencyGraph) -> DependencyGraph {
    graph.filter_by_edge_type(EdgeType::Imports)
}

pub fn filter_inheritance_edges(graph: DependencyGraph) -> DependencyGraph {
    graph.filter_by_edge_type(EdgeType::Inherits)
}
