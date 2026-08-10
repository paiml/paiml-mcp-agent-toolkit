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
    // `--metrics all` is the DEFAULT, and `GraphMetricType::All` matched neither
    // the per-node arms below nor the `contains(&PageRank)` guard further down.
    // The default run therefore reported pagerank frozen at its 1/N initializer
    // (0.00021939447125932427 for every node of a 4558-node repo, hub and leaf
    // alike) and betweenness/closeness at their 0.0 initializers, while
    // `--metrics page-rank` on the same repo produced real, distinct values.
    // Expand All into the concrete metrics it promises before anything reads it.
    let metric_types = expand_all_metric_types(metric_types);

    let node_count = graph.node_count();
    let edge_count = graph.edge_count();

    // Betweenness is computed once for the whole graph (Brandes) rather than
    // per node: the per-node probe below is O(V^2) shortest-path queries *per
    // node*, which cannot finish on a real repo, so wiring All through to it
    // would have swapped a fabricated number for a hang.
    let betweenness = if metric_types.contains(&crate::cli::GraphMetricType::Betweenness) {
        Some(calculate_betweenness_all(graph))
    } else {
        None
    };

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
                    if let Some(bc) = &betweenness {
                        metrics.betweenness_centrality = bc[node_idx.index()];
                    }
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

// Expand `GraphMetricType::All` into the concrete per-node metrics it stands for.
// Anything else is passed through untouched.
fn expand_all_metric_types(
    metric_types: Vec<crate::cli::GraphMetricType>,
) -> Vec<crate::cli::GraphMetricType> {
    if metric_types.contains(&crate::cli::GraphMetricType::All) {
        vec![
            crate::cli::GraphMetricType::Centrality,
            crate::cli::GraphMetricType::Betweenness,
            crate::cli::GraphMetricType::Closeness,
            crate::cli::GraphMetricType::PageRank,
        ]
    } else {
        metric_types
    }
}

// Betweenness centrality for every node at once (Brandes 2001, unweighted).
// Runs in O(V*(V+E)) so the default `--metrics all` path terminates on a repo
// sized graph; `calculate_betweenness` below is the original per-node probe and
// is kept only for the small-graph unit tests that pin its behaviour.
fn calculate_betweenness_all(graph: &SimpleGraph) -> Vec<f64> {
    use std::collections::VecDeque;

    let n = graph.node_count();
    let mut centrality = vec![0.0f64; n];
    if n < 3 {
        return centrality;
    }

    for s in 0..n {
        let mut stack: Vec<usize> = Vec::with_capacity(n);
        let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut sigma = vec![0.0f64; n];
        let mut distance = vec![-1i64; n];

        sigma[s] = 1.0;
        distance[s] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in graph.outgoing_edges(NodeIndex(v)) {
                if distance[w] < 0 {
                    distance[w] = distance[v] + 1;
                    queue.push_back(w);
                }
                if distance[w] == distance[v] + 1 {
                    sigma[w] += sigma[v];
                    predecessors[w].push(v);
                }
            }
        }

        let mut delta = vec![0.0f64; n];
        while let Some(w) = stack.pop() {
            for &v in &predecessors[w] {
                if sigma[w] > 0.0 {
                    delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                }
            }
            if w != s {
                centrality[w] += delta[w];
            }
        }
    }

    // Normalise by the number of ordered pairs that could route through a node.
    let denominator = ((n - 1) * (n - 2)) as f64;
    for c in &mut centrality {
        *c /= denominator;
    }

    centrality
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

// Closeness centrality, Wasserman-Faust normalised.
//
// This used to be `(node_count - 1) / sum_of_distances` where the sum ran over
// the REACHABLE set only, so a node that reached exactly one neighbour scored
// (N-1)/1 = N-1 — the maximum — while a hub that reached hundreds of nodes
// scored a small fraction of it. On this repo that put 4557.0 on
// near-isolated files, and since `filter_results` ranks by the SUM of the four
// centralities, the unbounded value dominated the sort and inverted the top-k.
// Scaling by reachable/(N-1) both bounds the value in [0,1] and charges a node
// for the part of the graph it cannot reach.
fn calculate_closeness(graph: &SimpleGraph, node: NodeIndex) -> f64 {
    let distances = graph.dijkstra(node, None);
    let total_distance: i32 = distances.values().sum();
    // `dijkstra` seeds the source at distance 0; it is not a reachable *other*.
    let reachable = distances.len().saturating_sub(1);
    let node_count = graph.node_count();

    if total_distance <= 0 || reachable == 0 || node_count < 2 {
        return 0.0;
    }

    let inverse_mean_distance = reachable as f64 / f64::from(total_distance);
    let reachable_fraction = reachable as f64 / (node_count - 1) as f64;
    inverse_mean_distance * reachable_fraction
}

// Calculate PageRank
fn calculate_pagerank(
    graph: &SimpleGraph,
    seeds: &[String],
    damping: f32,
    max_iter: usize,
    threshold: f64,
) -> Result<Vec<f64>> {
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

/// Regression tests for the default `--metrics all` path.
///
/// `GraphMetricType::All` used to fall through the `_ => {}` arm and the
/// `contains(&PageRank)` guard, so the DEFAULT invocation of
/// `pmat analyze graph-metrics` reported the pagerank initializer (1/N, the same
/// number for every node) and betweenness/closeness of 0.0.
#[cfg(test)]
mod metrics_all_expansion_regression_tests {
    use super::*;
    use crate::cli::GraphMetricType;

    /// n1 -> n2 -> n3 -> n4 -> n5
    fn chain_graph(len: usize) -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        let nodes: Vec<_> = (0..len).map(|i| graph.add_node(format!("n{i}"))).collect();
        for pair in nodes.windows(2) {
            graph.add_edge(pair[0], pair[1]);
        }
        graph
    }

    fn distinct_count(values: impl Iterator<Item = f64>) -> usize {
        let mut v: Vec<f64> = values.collect();
        v.sort_by(f64::total_cmp);
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        v.len()
    }

    #[test]
    fn metrics_all_computes_pagerank_betweenness_and_closeness() {
        let graph = chain_graph(5);
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::All],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();

        assert_eq!(result.nodes.len(), 5);

        let pagerank_values = distinct_count(result.nodes.iter().map(|n| n.pagerank));
        assert!(
            pagerank_values > 1,
            "--metrics all left every node at the 1/N pagerank initializer: {:?}",
            result.nodes.iter().map(|n| n.pagerank).collect::<Vec<_>>()
        );

        assert!(
            result.nodes.iter().any(|n| n.betweenness_centrality > 0.0),
            "--metrics all reported betweenness 0.0 for every node of a chain"
        );
        assert!(
            result.nodes.iter().any(|n| n.closeness_centrality > 0.0),
            "--metrics all reported closeness 0.0 for every node of a chain"
        );
    }

    /// Closeness used to be unbounded and computed over the reachable subset
    /// only, so the LAST reachable node of a chain (one neighbour, distance 1)
    /// scored N-1 while the head that reaches everything scored 0.4.
    #[test]
    fn closeness_is_bounded_and_ranks_a_hub_above_a_near_isolated_node() {
        let graph = chain_graph(5);
        let head = calculate_closeness(&graph, NodeIndex(0));
        let next_to_last = calculate_closeness(&graph, NodeIndex(3));
        let last = calculate_closeness(&graph, NodeIndex(4));

        for (name, value) in [("head", head), ("n3", next_to_last), ("tail", last)] {
            assert!(
                (0.0..=1.0).contains(&value),
                "closeness must be normalised into [0,1]; {name} = {value}"
            );
        }
        assert!(
            head > next_to_last,
            "the head reaches 4 nodes, n3 reaches 1: {head} vs {next_to_last}"
        );
        // The tail reaches nothing, so it has no closeness at all.
        assert_eq!(last, 0.0);
    }

    /// The top-k sort key is the SUM of the centralities, so an unbounded
    /// closeness inverted the whole ranking.
    #[test]
    fn top_k_ranking_is_not_dominated_by_a_near_isolated_node() {
        let graph = chain_graph(5);
        let result = calculate_metrics(
            &graph,
            vec![GraphMetricType::All],
            vec![],
            0.85,
            100,
            1e-6,
        )
        .unwrap();
        let ranked = filter_results(result, 5, 0.0);
        let order: Vec<&str> = ranked.nodes.iter().map(|n| n.name.as_str()).collect();
        let rank_of = |name: &str| {
            order
                .iter()
                .position(|n| *n == name)
                .unwrap_or_else(|| panic!("{name} missing from {order:?}"))
        };

        assert_ne!(
            order[0], "n3",
            "a node reaching a single neighbour led the ranking: {order:?}"
        );
        assert!(
            rank_of("n2") < rank_of("n3"),
            "the middle of the chain must outrank a near-isolated node: {order:?}"
        );
    }

    #[test]
    fn betweenness_all_ranks_the_middle_of_a_chain_above_its_ends() {
        let graph = chain_graph(5);
        let centrality = calculate_betweenness_all(&graph);

        assert!(
            centrality[2] > centrality[0],
            "middle of a chain must route more shortest paths than its head: {centrality:?}"
        );
        assert_eq!(
            centrality[0], 0.0,
            "the head of a chain lies on no shortest path between other nodes"
        );
    }
}
