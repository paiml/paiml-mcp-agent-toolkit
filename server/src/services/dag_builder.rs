use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
use crate::services::context::{AstItem, FileContext, ProjectContext};
use crate::services::semantic_naming::SemanticNamer;
use std::collections::{HashMap, HashSet};

pub struct DagBuilder {
    graph: DependencyGraph,
    // Track functions by name for call resolution
    function_map: HashMap<String, String>, // function_name -> full_id
    // Track types for inheritance resolution
    type_map: HashMap<String, String>, // type_name -> full_id
    // Semantic namer for deterministic display names
    namer: SemanticNamer,
}

impl DagBuilder {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            function_map: HashMap::new(),
            type_map: HashMap::new(),
            namer: SemanticNamer::new(),
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

        builder.finalize_graph()
    }

    const EDGE_BUDGET: usize = 400; // Empirically derived Mermaid limit

    fn finalize_graph(mut self) -> DependencyGraph {
        // First, remove edges that reference non-existent nodes
        let valid_nodes: HashSet<&String> = self.graph.nodes.keys().collect();
        self.graph
            .edges
            .retain(|edge| valid_nodes.contains(&edge.from) && valid_nodes.contains(&edge.to));

        if self.graph.edges.len() > Self::EDGE_BUDGET {
            // Priority-based edge sorting (Inherits > Uses > Implements > Call > Import)
            let priority = |edge_type: &EdgeType| -> u8 {
                match edge_type {
                    EdgeType::Inherits => 0,
                    EdgeType::Uses => 1,
                    EdgeType::Implements => 2,
                    EdgeType::Calls => 3,
                    EdgeType::Imports => 4,
                }
            };

            // Sort edges by priority (lower number = higher priority)
            self.graph
                .edges
                .sort_unstable_by_key(|e| priority(&e.edge_type));

            // Truncate to budget limit
            self.graph.edges.truncate(Self::EDGE_BUDGET);

            // Maintain node consistency - only keep nodes referenced in remaining edges
            let retained_nodes: HashSet<String> = self
                .graph
                .edges
                .iter()
                .flat_map(|e| [e.from.clone(), e.to.clone()])
                .collect();

            self.graph.nodes.retain(|id, _| retained_nodes.contains(id));
        }

        self.graph
    }

    pub fn build_from_project_with_limit(
        project: &ProjectContext,
        max_nodes: usize,
    ) -> DependencyGraph {
        let graph = Self::build_from_project(project);
        if graph.edges.len() > 400 {
            // Safety margin for Mermaid
            prune_graph_pagerank(&graph, max_nodes)
        } else {
            graph
        }
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
                    let node = NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Function,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: file
                            .complexity_metrics
                            .as_ref()
                            .and_then(|m| {
                                m.functions
                                    .iter()
                                    .find(|f| f.name == *name)
                                    .map(|f| f.metrics.cognitive as u32)
                            })
                            .unwrap_or_else(|| self.calculate_complexity(item)),
                        metadata: HashMap::new(),
                    };
                    self.add_node(self.enrich_node(node));
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
                    let node = NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Class,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: file
                            .complexity_metrics
                            .as_ref()
                            .and_then(|m| {
                                m.classes
                                    .iter()
                                    .find(|c| c.name == *name)
                                    .map(|c| c.metrics.cognitive as u32)
                            })
                            .unwrap_or_else(|| self.calculate_complexity(item)),
                        metadata: HashMap::new(),
                    };
                    self.add_node(self.enrich_node(node));
                    self.type_map.insert(name.clone(), id);
                }
                AstItem::Trait {
                    name,
                    line,
                    visibility: _,
                } => {
                    let id = format!("{}::{}", self.normalize_path(&file.path), name);
                    let node = NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Trait,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: 1,
                        metadata: HashMap::new(),
                    };
                    self.add_node(self.enrich_node(node));
                    self.type_map.insert(name.clone(), id);
                }
                AstItem::Module {
                    name,
                    line,
                    visibility: _,
                } => {
                    let id = format!("{}::{}", self.normalize_path(&file.path), name);
                    let node = NodeInfo {
                        id: id.clone(),
                        label: name.clone(),
                        node_type: NodeType::Module,
                        file_path: file.path.clone(),
                        line_number: *line,
                        complexity: 1,
                        metadata: HashMap::new(),
                    };
                    self.add_node(self.enrich_node(node));
                }
                _ => {}
            }
        }
    }

    fn process_relationships(&mut self, file: &FileContext) {
        // Create module node for the file itself
        let file_module_id = self.normalize_path(&file.path);
        let node = NodeInfo {
            id: file_module_id.clone(),
            label: self.extract_module_name(&file.path),
            node_type: NodeType::Module,
            file_path: file.path.clone(),
            line_number: 0,
            complexity: file
                .complexity_metrics
                .as_ref()
                .map(|m| m.total_complexity.cognitive as u32)
                .unwrap_or(1),
            metadata: HashMap::new(),
        };
        self.add_node(self.enrich_node(node));

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

    /// Enrich node with semantic naming and metadata
    fn enrich_node(&self, mut node: NodeInfo) -> NodeInfo {
        // Apply semantic naming
        let semantic_name = self.namer.get_semantic_name(&node.id, &node);
        if semantic_name != node.id && !semantic_name.is_empty() {
            node.label = semantic_name;
        }

        // Add comprehensive metadata as specified in the bug report
        node.metadata
            .insert("file_path".to_string(), node.file_path.clone());
        node.metadata.insert(
            "module_path".to_string(),
            self.path_to_module(&node.file_path),
        );
        node.metadata
            .insert("display_name".to_string(), node.label.clone());
        node.metadata
            .insert("node_type".to_string(), format!("{:?}", node.node_type));
        node.metadata
            .insert("line_number".to_string(), node.line_number.to_string());
        node.metadata
            .insert("complexity".to_string(), node.complexity.to_string());

        // Add language-specific metadata
        let ext = std::path::Path::new(&node.file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = match ext {
            "rs" => "rust",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            _ => "unknown",
        };
        node.metadata
            .insert("language".to_string(), language.to_string());

        node
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

    fn path_to_module(&self, path: &str) -> String {
        // Convert file path to module notation using semantic namer
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = SemanticNamer::detect_language(ext);

        // Use the semantic namer's path_to_module logic indirectly
        let clean_path = path
            .trim_start_matches("./")
            .trim_start_matches("/")
            .trim_start_matches("src/")
            .trim_start_matches("lib/")
            .trim_start_matches("app/");

        let without_ext = std::path::Path::new(clean_path)
            .with_extension("")
            .to_string_lossy()
            .into_owned();

        let separator = match language {
            "rust" => "::",
            "python" => ".",
            "typescript" | "javascript" => ".",
            "go" => "/",
            "java" => ".",
            _ => "::",
        };

        without_ext.replace(['/', '\\'], separator)
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

/// Prune graph using PageRank algorithm to keep only the most important nodes
pub fn prune_graph_pagerank(graph: &DependencyGraph, max_nodes: usize) -> DependencyGraph {
    if graph.nodes.len() <= max_nodes {
        return graph.clone();
    }

    // Build adjacency for PageRank
    let node_ids: Vec<&String> = graph.nodes.keys().collect();
    let mut scores = vec![1.0f32; node_ids.len()];
    let node_idx: HashMap<&String, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    // 10 iterations sufficient for ranking
    for _ in 0..10 {
        let mut new_scores = vec![0.15f32; scores.len()];
        for edge in &graph.edges {
            if let (Some(&from), Some(&to)) = (node_idx.get(&edge.from), node_idx.get(&edge.to)) {
                let out_degree = graph.edges.iter().filter(|e| e.from == edge.from).count() as f32;
                if out_degree > 0.0 {
                    new_scores[to] += 0.85 * scores[from] / out_degree;
                }
            }
        }
        scores = new_scores;
    }

    // Select top-k nodes
    let mut ranked: Vec<_> = scores.iter().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let keep: HashSet<&String> = ranked
        .iter()
        .take(max_nodes)
        .map(|(i, _)| node_ids[*i])
        .collect();

    DependencyGraph {
        nodes: graph
            .nodes
            .iter()
            .filter(|(id, _)| keep.contains(id))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        edges: graph
            .edges
            .iter()
            .filter(|e| keep.contains(&e.from) && keep.contains(&e.to))
            .cloned()
            .collect(),
    }
}
