// Implementation of CanonicalQuery for SystemArchitectureQuery
// Core query execution: component detection, relationship inference, metric aggregation.
// This file is include!()'d into canonical_query.rs and shares its scope.

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
                total_complexity += f64::from(node_metrics.cyclomatic);
                total_loc += node_metrics.lines as usize;
                function_count += 1;
                max_complexity = max_complexity.max(f64::from(node_metrics.cyclomatic));
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
