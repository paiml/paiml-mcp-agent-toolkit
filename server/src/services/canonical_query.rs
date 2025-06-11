use anyhow::Result;
use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core trait for canonical analysis queries
pub trait CanonicalQuery: Send + Sync {
    fn query_id(&self) -> &'static str;
    fn execute(&self, ctx: &AnalysisContext) -> Result<QueryResult>;
    fn cache_key(&self, project_path: &Path) -> String {
        format!("{}:{}", self.query_id(), project_path.display())
    }
}

/// Analysis context containing all data needed for queries
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub project_path: std::path::PathBuf,
    pub ast_dag: crate::models::dag::DependencyGraph,
    pub call_graph: CallGraph,
    pub complexity_map: FxHashMap<String, crate::services::complexity::ComplexityMetrics>,
    pub churn_analysis: Option<crate::models::churn::CodeChurnAnalysis>,
}

/// Call graph representation for component relationship analysis
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    pub nodes: Vec<CallNode>,
    pub edges: Vec<CallEdge>,
}

#[derive(Debug, Clone)]
pub struct CallNode {
    pub id: String,
    pub name: String,
    pub module_path: String,
    pub node_type: CallNodeType,
}

#[derive(Debug, Clone)]
pub enum CallNodeType {
    Function,
    Method,
    Struct,
    Module,
    Trait,
}

#[derive(Debug, Clone)]
pub struct CallEdge {
    pub from: String,
    pub to: String,
    pub edge_type: CallEdgeType,
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub enum CallEdgeType {
    FunctionCall,
    MethodCall,
    StructInstantiation,
    TraitImpl,
    ModuleImport,
}

/// Query result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub diagram: String,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub nodes: usize,
    pub edges: usize,
    pub max_depth: usize,
    pub timestamp: DateTime<Utc>,
    pub query_version: String,
    pub analysis_time_ms: u64,
}

/// System architecture analysis query
pub struct SystemArchitectureQuery;

impl CanonicalQuery for SystemArchitectureQuery {
    fn query_id(&self) -> &'static str {
        "system-architecture-v1"
    }

    fn execute(&self, ctx: &AnalysisContext) -> Result<QueryResult> {
        let start = std::time::Instant::now();

        // 1. Component detection via module hierarchy
        let components = detect_architectural_components(&ctx.ast_dag)?;

        // 2. Edge inference from imports/calls
        let edges = infer_component_relationships(&components, &ctx.call_graph)?;

        // 3. Complexity aggregation per component
        let metrics = aggregate_component_metrics(&components, &ctx.complexity_map)?;

        // 4. Mermaid generation with styling
        let mermaid = generate_styled_architecture_diagram(&components, &edges, &metrics)?;

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            diagram: mermaid,
            metadata: GraphMetadata {
                nodes: components.len(),
                edges: edges.len(),
                max_depth: calculate_graph_diameter(&components, &edges),
                timestamp: Utc::now(),
                query_version: self.query_id().to_string(),
                analysis_time_ms: elapsed,
            },
        })
    }
}

/// Architectural component representation
#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub label: String,
    pub nodes: Vec<String>,
    pub complexity: f64,
    pub loc: usize,
    pub functions: usize,
}

/// Component relationship edge
#[derive(Debug, Clone)]
pub struct ComponentEdge {
    pub from: String,
    pub to: String,
    pub edge_type: ComponentEdgeType,
    pub weight: u32,
}

#[derive(Debug)]
pub enum ComponentEdgeType {
    Import,
    Call,
    Inheritance,
    Composition,
}

/// Component metrics aggregated from individual functions/modules
#[derive(Debug, Clone)]
pub struct ComponentMetrics {
    pub total_complexity: f64,
    pub avg_complexity: f64,
    pub max_complexity: f64,
    pub total_loc: usize,
    pub function_count: usize,
}

// Implementation functions

fn detect_architectural_components(
    dag: &crate::models::dag::DependencyGraph,
) -> Result<Vec<Component>> {
    let mut components = Vec::new();

    // Extract top-level modules as initial components
    for (node_id, node_info) in &dag.nodes {
        // Focus on top-level modules (depth <= 2)
        if node_info.file_path.matches('/').count() <= 2
            && node_info.node_type == crate::models::dag::NodeType::Module
        {
            let component = Component {
                id: sanitize_component_id(node_id),
                label: humanize_component_name(&node_info.label),
                nodes: collect_component_nodes(dag, node_id),
                complexity: 0.0, // Will be populated by aggregate_component_metrics
                loc: 0,
                functions: 0,
            };
            components.push(component);
        }
    }

    // Merge tightly coupled modules (>80% bidirectional edges)
    merge_coupled_components(&mut components, dag);

    // Remove empty components
    components.retain(|c| !c.nodes.is_empty());

    Ok(components)
}

fn infer_component_relationships(
    components: &[Component],
    call_graph: &CallGraph,
) -> Result<Vec<ComponentEdge>> {
    let mut edges = Vec::new();
    let mut component_map = FxHashMap::default();

    // Build component lookup map
    for component in components {
        for node in &component.nodes {
            component_map.insert(node.clone(), component.id.clone());
        }
    }

    // Analyze call graph edges to infer component relationships
    for edge in &call_graph.edges {
        if let (Some(from_component), Some(to_component)) =
            (component_map.get(&edge.from), component_map.get(&edge.to))
        {
            if from_component != to_component {
                // Cross-component edge found
                edges.push(ComponentEdge {
                    from: from_component.clone(),
                    to: to_component.clone(),
                    edge_type: match edge.edge_type {
                        CallEdgeType::FunctionCall | CallEdgeType::MethodCall => {
                            ComponentEdgeType::Call
                        }
                        CallEdgeType::ModuleImport => ComponentEdgeType::Import,
                        CallEdgeType::TraitImpl => ComponentEdgeType::Inheritance,
                        CallEdgeType::StructInstantiation => ComponentEdgeType::Composition,
                    },
                    weight: edge.weight,
                });
            }
        }
    }

    // Deduplicate and aggregate weights
    let mut aggregated_edges = FxHashMap::default();
    for edge in edges {
        let key = (edge.from.clone(), edge.to.clone(), edge.edge_type.clone());
        let weight = aggregated_edges.get(&key).unwrap_or(&0) + edge.weight;
        aggregated_edges.insert(key, weight);
    }

    let final_edges = aggregated_edges
        .into_iter()
        .map(|((from, to, edge_type), weight)| ComponentEdge {
            from,
            to,
            edge_type,
            weight,
        })
        .collect();

    Ok(final_edges)
}

fn aggregate_component_metrics(
    components: &[Component],
    complexity_map: &FxHashMap<String, crate::services::complexity::ComplexityMetrics>,
) -> Result<FxHashMap<String, ComponentMetrics>> {
    let mut metrics = FxHashMap::default();

    for component in components {
        let mut total_complexity = 0.0;
        let mut total_loc = 0;
        let mut function_count = 0;
        let mut max_complexity: f64 = 0.0;

        for node in &component.nodes {
            if let Some(node_metrics) = complexity_map.get(node) {
                total_complexity += node_metrics.cyclomatic as f64;
                total_loc += node_metrics.lines as usize;
                function_count += 1;
                max_complexity = max_complexity.max(node_metrics.cyclomatic as f64);
            }
        }

        let avg_complexity = if function_count > 0 {
            total_complexity / function_count as f64
        } else {
            0.0
        };

        metrics.insert(
            component.id.clone(),
            ComponentMetrics {
                total_complexity,
                avg_complexity,
                max_complexity,
                total_loc,
                function_count,
            },
        );
    }

    Ok(metrics)
}

fn generate_styled_architecture_diagram(
    components: &[Component],
    edges: &[ComponentEdge],
    metrics: &FxHashMap<String, ComponentMetrics>,
) -> Result<String> {
    let mut diagram = String::from("graph TD\n");

    // Add components with styling based on complexity
    for component in components {
        let complexity_class = if let Some(m) = metrics.get(&component.id) {
            match m.avg_complexity {
                x if x > 15.0 => "high-complexity",
                x if x > 10.0 => "medium-complexity",
                _ => "low-complexity",
            }
        } else {
            "unknown-complexity"
        };

        diagram.push_str(&format!("    {}[\"{}\"]\n", component.id, component.label));
        diagram.push_str(&format!(
            "    class {} {}\n",
            component.id, complexity_class
        ));
    }

    // Add edges with styling based on weight
    for edge in edges {
        let edge_class = match edge.weight {
            w if w > 10 => "strong-coupling",
            w if w > 5 => "medium-coupling",
            _ => "weak-coupling",
        };

        let arrow_style = match edge.edge_type {
            ComponentEdgeType::Import => "-->",
            ComponentEdgeType::Call => "-.->",
            ComponentEdgeType::Inheritance => "==>>",
            ComponentEdgeType::Composition => "--o",
        };

        diagram.push_str(&format!("    {} {} {}\n", edge.from, arrow_style, edge.to));
        diagram.push_str(&format!(
            "    linkStyle {} stroke:{}\n",
            edges.len(),
            match edge_class {
                "strong-coupling" => "#ff0000",
                "medium-coupling" => "#ff8800",
                _ => "#888888",
            }
        ));
    }

    // Add class definitions
    diagram.push('\n');
    diagram.push_str("    classDef high-complexity fill:#ff9999,stroke:#ff0000,stroke-width:3px\n");
    diagram
        .push_str("    classDef medium-complexity fill:#ffcc99,stroke:#ff8800,stroke-width:2px\n");
    diagram.push_str("    classDef low-complexity fill:#99ff99,stroke:#00aa00,stroke-width:1px\n");
    diagram
        .push_str("    classDef unknown-complexity fill:#cccccc,stroke:#888888,stroke-width:1px\n");

    Ok(diagram)
}

// Helper functions

fn sanitize_component_id(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn humanize_component_name(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn collect_component_nodes(
    dag: &crate::models::dag::DependencyGraph,
    module_name: &str,
) -> Vec<String> {
    dag.nodes
        .iter()
        .filter(|(node_id, _)| node_id.starts_with(module_name))
        .map(|(node_id, _)| node_id.clone())
        .collect()
}

fn merge_coupled_components(
    _components: &mut [Component],
    _dag: &crate::models::dag::DependencyGraph,
) {
    // TRACKED: Implement coupling analysis and merge highly coupled components
    // For now, this is a placeholder
}

fn calculate_graph_diameter(_components: &[Component], _edges: &[ComponentEdge]) -> usize {
    // TRACKED: Implement graph diameter calculation
    // For now, return a placeholder value
    5
}

impl Clone for ComponentEdgeType {
    fn clone(&self) -> Self {
        match self {
            ComponentEdgeType::Import => ComponentEdgeType::Import,
            ComponentEdgeType::Call => ComponentEdgeType::Call,
            ComponentEdgeType::Inheritance => ComponentEdgeType::Inheritance,
            ComponentEdgeType::Composition => ComponentEdgeType::Composition,
        }
    }
}

impl std::hash::Hash for ComponentEdgeType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for ComponentEdgeType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for ComponentEdgeType {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_call_graph_creation() {
        let mut graph = CallGraph::default();

        graph.nodes.push(CallNode {
            id: "1".to_string(),
            name: "main".to_string(),
            module_path: "app".to_string(),
            node_type: CallNodeType::Function,
        });

        graph.nodes.push(CallNode {
            id: "2".to_string(),
            name: "helper".to_string(),
            module_path: "app::utils".to_string(),
            node_type: CallNodeType::Function,
        });

        graph.edges.push(CallEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: CallEdgeType::FunctionCall,
            weight: 1,
        });

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_call_node_types() {
        let node_types = vec![
            CallNodeType::Function,
            CallNodeType::Method,
            CallNodeType::Struct,
            CallNodeType::Module,
            CallNodeType::Trait,
        ];

        for node_type in node_types {
            let node = CallNode {
                id: "test".to_string(),
                name: "test".to_string(),
                module_path: "test".to_string(),
                node_type,
            };

            match node.node_type {
                CallNodeType::Function => {}
                CallNodeType::Method => {}
                CallNodeType::Struct => {}
                CallNodeType::Module => {}
                CallNodeType::Trait => {}
            }
        }
    }

    #[test]
    fn test_call_edge_types() {
        let edge_types = vec![
            CallEdgeType::FunctionCall,
            CallEdgeType::MethodCall,
            CallEdgeType::StructInstantiation,
            CallEdgeType::TraitImpl,
            CallEdgeType::ModuleImport,
        ];

        for edge_type in edge_types {
            let edge = CallEdge {
                from: "a".to_string(),
                to: "b".to_string(),
                edge_type,
                weight: 1,
            };

            assert_eq!(edge.from, "a");
            assert_eq!(edge.to, "b");
            assert_eq!(edge.weight, 1);
        }
    }

    #[test]
    fn test_graph_metadata() {
        let metadata = GraphMetadata {
            nodes: 10,
            edges: 15,
            max_depth: 5,
            timestamp: Utc::now(),
            query_version: "1.0".to_string(),
            analysis_time_ms: 100,
        };

        assert_eq!(metadata.nodes, 10);
        assert_eq!(metadata.edges, 15);
        assert_eq!(metadata.max_depth, 5);
        assert_eq!(metadata.query_version, "1.0");
        assert_eq!(metadata.analysis_time_ms, 100);
    }

    #[test]
    fn test_query_result() {
        let result = QueryResult {
            diagram: "graph TD\n  A --> B".to_string(),
            metadata: GraphMetadata {
                nodes: 2,
                edges: 1,
                max_depth: 2,
                timestamp: Utc::now(),
                query_version: "1.0".to_string(),
                analysis_time_ms: 50,
            },
        };

        assert!(result.diagram.contains("graph TD"));
        assert_eq!(result.metadata.nodes, 2);
    }

    #[test]
    fn test_analysis_context() {
        let context = AnalysisContext {
            project_path: PathBuf::from("/test/project"),
            ast_dag: crate::models::dag::DependencyGraph::new(),
            call_graph: CallGraph::default(),
            complexity_map: FxHashMap::default(),
            churn_analysis: None,
        };

        assert_eq!(context.project_path, PathBuf::from("/test/project"));
        assert!(context.churn_analysis.is_none());
    }

    // Test for the trait implementation
    struct TestQuery;

    impl CanonicalQuery for TestQuery {
        fn query_id(&self) -> &'static str {
            "test_query"
        }

        fn execute(&self, _ctx: &AnalysisContext) -> Result<QueryResult> {
            Ok(QueryResult {
                diagram: "test".to_string(),
                metadata: GraphMetadata {
                    nodes: 0,
                    edges: 0,
                    max_depth: 0,
                    timestamp: Utc::now(),
                    query_version: "1.0".to_string(),
                    analysis_time_ms: 10,
                },
            })
        }
    }

    #[test]
    fn test_canonical_query_trait() {
        let query = TestQuery;
        let path = Path::new("/test/project");

        assert_eq!(query.query_id(), "test_query");
        assert_eq!(query.cache_key(path), "test_query:/test/project");
    }

    #[test]
    fn test_component_edge_type_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let edge_type = ComponentEdgeType::Import;
        let mut hasher = DefaultHasher::new();
        edge_type.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = DefaultHasher::new();
        ComponentEdgeType::Import.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_component_edge_type_equality() {
        assert_eq!(ComponentEdgeType::Import, ComponentEdgeType::Import);
        assert_ne!(ComponentEdgeType::Import, ComponentEdgeType::Composition);
        assert_eq!(
            ComponentEdgeType::Inheritance,
            ComponentEdgeType::Inheritance
        );
    }
}
