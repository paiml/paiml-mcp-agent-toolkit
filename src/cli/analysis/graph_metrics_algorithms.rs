// Graph algorithms: metrics calculation, centrality, PageRank, filtering

// Calculate graph metrics
fn calculate_metrics(
    graph: &SimpleGraph,
    metric_types: Vec<crate::cli::GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
) -> Result<GraphMetricsResult> {
    let node_count = graph.node_count();
    let edge_count = graph.edge_count();

    let mut node_metrics = Vec::new();

    // Calculate metrics for each node
    for node_idx in graph.node_indices() {
        let name = graph.get_node(node_idx);
        let in_degree = graph.in_degree(node_idx);
        let out_degree = graph.out_degree(node_idx);

        let mut metrics = NodeMetrics {
            name: name.clone(),
            degree_centrality: if node_count > 1 {
                (in_degree + out_degree) as f64 / (node_count - 1) as f64
            } else {
                0.0
            },
            betweenness_centrality: 0.0,
            closeness_centrality: 0.0,
            pagerank: 1.0 / node_count.max(1) as f64,
            in_degree,
            out_degree,
        };

        // Calculate additional metrics if requested
        for metric_type in &metric_types {
            match metric_type {
                crate::cli::GraphMetricType::Betweenness => {
                    metrics.betweenness_centrality = calculate_betweenness(graph, node_idx);
                }
                crate::cli::GraphMetricType::Closeness => {
                    metrics.closeness_centrality = calculate_closeness(graph, node_idx);
                }
                crate::cli::GraphMetricType::PageRank => {
                    // PageRank calculated separately below
                }
                _ => {}
            }
        }

        node_metrics.push(metrics);
    }

    // Calculate PageRank if requested
    if metric_types.contains(&crate::cli::GraphMetricType::PageRank) {
        let pageranks = calculate_pagerank(
            graph,
            &pagerank_seeds,
            damping_factor,
            max_iterations,
            convergence_threshold,
        )?;

        for (i, pr) in pageranks.iter().enumerate() {
            if i < node_metrics.len() {
                node_metrics[i].pagerank = *pr;
            }
        }
    }

    // Calculate graph-wide metrics
    let total_degree: usize = node_metrics
        .iter()
        .map(|n| n.in_degree + n.out_degree)
        .sum();
    let max_degree = node_metrics
        .iter()
        .map(|n| n.in_degree + n.out_degree)
        .max()
        .unwrap_or(0);

    Ok(GraphMetricsResult {
        nodes: node_metrics,
        total_nodes: node_count,
        total_edges: edge_count,
        density: if node_count > 1 {
            2.0 * edge_count as f64 / (node_count * (node_count - 1)) as f64
        } else {
            0.0
        },
        average_degree: if node_count > 0 {
            total_degree as f64 / node_count as f64
        } else {
            0.0
        },
        max_degree,
        connected_components: graph.connected_components(),
    })
}

// Calculate betweenness centrality (simplified)
fn calculate_betweenness(graph: &SimpleGraph, node: NodeIndex) -> f64 {
    // Simplified betweenness - count paths through node
    let mut count = 0;
    for source in graph.node_indices() {
        for target in graph.node_indices() {
            if source != target && source != node && target != node {
                // Check if node is on shortest path
                if is_on_shortest_path(graph, source, target, node) {
                    count += 1;
                }
            }
        }
    }

    let n = graph.node_count();
    if n > 2 {
        f64::from(count) / ((n - 1) * (n - 2)) as f64
    } else {
        0.0
    }
}

// Check if node is on shortest path
fn is_on_shortest_path(
    graph: &SimpleGraph,
    source: NodeIndex,
    target: NodeIndex,
    node: NodeIndex,
) -> bool {
    let from_source = graph.dijkstra(source, Some(target));
    let from_node = graph.dijkstra(node, Some(target));
    let to_node = graph.dijkstra(source, Some(node));

    if let (Some(&dist_st), Some(&dist_nt), Some(&dist_sn)) = (
        from_source.get(&target),
        from_node.get(&target),
        to_node.get(&node),
    ) {
        dist_sn + dist_nt == dist_st
    } else {
        false
    }
}

// Calculate closeness centrality
fn calculate_closeness(graph: &SimpleGraph, node: NodeIndex) -> f64 {
    let distances = graph.dijkstra(node, None);
    let total_distance: i32 = distances.values().sum();

    if total_distance > 0 {
        (graph.node_count() - 1) as f64 / f64::from(total_distance)
    } else {
        0.0
    }
}

// Calculate PageRank
fn calculate_pagerank(
    graph: &SimpleGraph,
    seeds: &[String],
    damping: f32,
    max_iter: usize,
    threshold: f64,
) -> Result<Vec<f64>> {
    debug_assert!(max_iter > 0, "max_iter must be positive");
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
    let n = graph.node_count();
    if n == 0 {
        return Ok(Vec::new());
    }
    let mut pagerank = vec![1.0 / n as f64; n];

    // Boost seed nodes
    for (i, node_idx) in graph.node_indices().enumerate() {
        if seeds.contains(graph.get_node(node_idx)) {
            pagerank[i] = 2.0 / n as f64;
        }
    }

    // Power iteration
    for _ in 0..max_iter {
        let mut new_pagerank = vec![(1.0 - f64::from(damping)) / n as f64; n];

        for (i, node_idx) in graph.node_indices().enumerate() {
            let out_edges = graph.out_degree(node_idx);
            if out_edges > 0 {
                let contrib = f64::from(damping) * pagerank[i] / out_edges as f64;
                for &target_idx in graph.outgoing_edges(node_idx) {
                    new_pagerank[target_idx] += contrib;
                }
            } else {
                // Distribute to all nodes
                let contrib = f64::from(damping) * pagerank[i] / n as f64;
                for pr in &mut new_pagerank {
                    *pr += contrib;
                }
            }
        }

        // Check convergence
        let diff: f64 = pagerank
            .iter()
            .zip(&new_pagerank)
            .map(|(old, new)| (old - new).abs())
            .sum();

        pagerank = new_pagerank;

        if diff < threshold {
            break;
        }
    }

    Ok(pagerank)
}

// Filter results
fn filter_results(
    mut result: GraphMetricsResult,
    top_k: usize,
    min_centrality: f64,
) -> GraphMetricsResult {
    // Filter by minimum centrality
    result.nodes.retain(|n| {
        n.degree_centrality >= min_centrality
            || n.betweenness_centrality >= min_centrality
            || n.closeness_centrality >= min_centrality
    });

    // Sort by combined score and take top K
    result.nodes.sort_by(|a, b| {
        let score_a =
            a.degree_centrality + a.betweenness_centrality + a.closeness_centrality + a.pagerank;
        let score_b =
            b.degree_centrality + b.betweenness_centrality + b.closeness_centrality + b.pagerank;
        score_b.total_cmp(&score_a)
    });

    result.nodes.truncate(top_k);

    result
}
