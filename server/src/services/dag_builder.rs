use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use crate::services::context::{AstItem, FileContext, ProjectContext};
use std::collections::HashMap;

pub struct DagBuilder {
    graph: DependencyGraph,
    // Track functions by name for call resolution
    function_map: HashMap<String, String>, // function_name -> full_id
    // Track types for inheritance resolution
    type_map: HashMap<String, String>, // type_name -> full_id
}

impl DagBuilder {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            function_map: HashMap::new(),
            type_map: HashMap::new(),
        }
    }

    pub fn build_from_project(project: &ProjectContext) -> DependencyGraph {
        let mut builder = Self::new();

        // First pass: collect all nodes and build lookup maps
        for file in &project.files {
            builder.collect_nodes(file);
        }

        // Second pass: create edges based on relationships
        for file in &project.files {
            builder.process_relationships(file);
        }

        builder.graph
    }

    fn collect_nodes(&mut self, file: &FileContext) {
        for item in &file.items {
            match item {
                AstItem::Function {
                    name,
                    line,
                    visibility: _,
                    is_async: _,
                } => {
                    let id = format!("{}::{}", self.normalize_path(&file.path), name);
                    self.add_node(NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Function,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: self.calculate_complexity(item),
                    });
                    self.function_map.insert(name.clone(), id);
                }
                AstItem::Struct {
                    name,
                    line,
                    fields_count: _,
                    derives: _,
                    visibility: _,
                } => {
                    let id = format!("{}::{}", self.normalize_path(&file.path), name);
                    self.add_node(NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Class,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: self.calculate_complexity(item),
                    });
                    self.type_map.insert(name.clone(), id);
                }
                AstItem::Trait {
                    name,
                    line,
                    visibility: _,
                } => {
                    let id = format!("{}::{}", self.normalize_path(&file.path), name);
                    self.add_node(NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Trait,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: 1,
                    });
                    self.type_map.insert(name.clone(), id);
                }
                AstItem::Module {
                    name,
                    line,
                    visibility: _,
                } => {
                    let id = format!("{}::{}", self.normalize_path(&file.path), name);
                    self.add_node(NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Module,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: 1,
                    });
                }
                _ => {}
            }
        }
    }

    fn process_relationships(&mut self, file: &FileContext) {
        // Create module node for the file itself
        let file_module_id = self.normalize_path(&file.path);
        self.add_node(NodeInfo {
            id: file_module_id.clone(),
            label: self.extract_module_name(&file.path),
            node_type: NodeType::Module,
            file_path: file.path.clone(),
            line_number: 0,
            complexity: 1,
        });

        for item in &file.items {
            match item {
                AstItem::Use { path, line: _ } => {
                    // Create import edges from the file module to imported items
                    if let Some(target_id) = self.resolve_import_path(path) {
                        self.add_edge(Edge {
                            from: file_module_id.clone(),
                            to: target_id,
                            edge_type: EdgeType::Imports,
                            weight: 1,
                        });
                    }
                }
                AstItem::Impl {
                    type_name,
                    trait_name,
                    ..
                } => {
                    // Create inheritance edges for trait implementations
                    if let (Some(trait_name), Some(struct_id)) =
                        (trait_name.as_ref(), self.type_map.get(type_name))
                    {
                        if let Some(trait_id) = self.type_map.get(trait_name) {
                            self.add_edge(Edge {
                                from: struct_id.clone(),
                                to: trait_id.clone(),
                                edge_type: EdgeType::Inherits,
                                weight: 1,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
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

    fn normalize_path(&self, path: &str) -> String {
        // Convert file path to a module-like identifier
        path.trim_start_matches("./")
            .trim_start_matches("/")
            .trim_end_matches(".rs")
            .trim_end_matches(".ts")
            .trim_end_matches(".py")
            .trim_end_matches(".js")
            .trim_end_matches(".tsx")
            .trim_end_matches(".jsx")
            .replace(['/', '.', '-'], "_")
    }

    fn extract_module_name(&self, path: &str) -> String {
        // Extract just the file name without extension
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path)
            .to_string()
    }

    fn resolve_import_path(&self, import_path: &str) -> Option<String> {
        // Try to resolve the import to a known node
        // First check if it's a direct type reference
        if let Some(type_id) = self.type_map.get(import_path) {
            return Some(type_id.clone());
        }

        // Check if it's a function reference
        if let Some(func_id) = self.function_map.get(import_path) {
            return Some(func_id.clone());
        }

        // For module paths like "crate::models::dag", try to find a matching module
        let parts: Vec<&str> = import_path.split("::").collect();
        if let Some(last_part) = parts.last() {
            // Try as type
            if let Some(type_id) = self.type_map.get(*last_part) {
                return Some(type_id.clone());
            }
            // Try as function
            if let Some(func_id) = self.function_map.get(*last_part) {
                return Some(func_id.clone());
            }
        }

        // If nothing found, create a module node for the import
        Some(import_path.replace("::", "_"))
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
