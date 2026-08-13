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

    // Every measure is computed HERE or not at all, and "not at all" is `None`
    // all the way to the output. The previous shape — seed the struct with an
    // initializer, then overwrite it in a `match` over the selection — is what
    // published 1/N pagerank and 0.0 closeness as measurements whenever the
    // selection did not include them.
    let selected = |m: &crate::cli::GraphMetricType| metric_types.contains(m);

    // Betweenness is computed once for the whole graph (Brandes) rather than
    // per node: the per-node probe below is O(V^2) shortest-path queries *per
    // node*, which cannot finish on a real repo, so wiring All through to it
    // would have swapped a fabricated number for a hang.
    let betweenness = selected(&crate::cli::GraphMetricType::Betweenness)
        .then(|| calculate_betweenness_all(graph));
    let clustering =
        selected(&crate::cli::GraphMetricType::Clustering).then(|| calculate_clustering_all(graph));
    let component_ids =
        selected(&crate::cli::GraphMetricType::Components).then(|| graph.component_ids());
    let pageranks = if selected(&crate::cli::GraphMetricType::PageRank) {
        Some(calculate_pagerank(
            graph,
            &pagerank_seeds,
            damping_factor,
            max_iterations,
            convergence_threshold,
        )?)
    } else {
        None
    };
    let want_closeness = selected(&crate::cli::GraphMetricType::Closeness);

    let mut node_metrics = Vec::new();

    // Calculate metrics for each node
    for node_idx in graph.node_indices() {
        let name = graph.get_node(node_idx);
        let in_degree = graph.in_degree(node_idx);
        let out_degree = graph.out_degree(node_idx);
        let i = node_idx.index();

        node_metrics.push(NodeMetrics {
            name: name.clone(),
            degree_centrality: if node_count > 1 {
                (in_degree + out_degree) as f64 / (node_count - 1) as f64
            } else {
                0.0
            },
            betweenness_centrality: betweenness.as_ref().and_then(|b| b.get(i).copied()),
            closeness_centrality: want_closeness.then(|| calculate_closeness(graph, node_idx)),
            pagerank: pageranks.as_ref().and_then(|p| p.get(i).copied()),
            clustering_coefficient: clustering.as_ref().and_then(|c| c.get(i).copied()),
            component_id: component_ids.as_ref().and_then(|c| c.get(i).copied()),
            in_degree,
            out_degree,
        });
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
//
// `All` is "All available metrics" in `--help`, so every metric this command can
// compute has to be in this list — leaving one out is how `All` came to publish
// initializers in the first place.
fn expand_all_metric_types(
    metric_types: Vec<crate::cli::GraphMetricType>,
) -> Vec<crate::cli::GraphMetricType> {
    if metric_types.contains(&crate::cli::GraphMetricType::All) {
        vec![
            crate::cli::GraphMetricType::Centrality,
            crate::cli::GraphMetricType::Betweenness,
            crate::cli::GraphMetricType::Closeness,
            crate::cli::GraphMetricType::PageRank,
            crate::cli::GraphMetricType::Clustering,
            crate::cli::GraphMetricType::Components,
        ]
    } else {
        metric_types
    }
}

/// Local clustering coefficient for every node.
///
/// `C(v) = 2 * |edges among N(v)| / (k * (k-1))` over the UNDIRECTED
/// neighbourhood, i.e. the fraction of a node's neighbour pairs that are
/// themselves connected. `0.0` for a node with fewer than two neighbours: there
/// is no pair to close, which is a measured answer, not a missing one.
///
/// `--metrics clustering` was accepted, documented as "Clustering coefficient",
/// and fell through the per-node `_ => {}` arm: it returned degree centrality
/// under another name.
fn calculate_clustering_all(graph: &SimpleGraph) -> Vec<f64> {
    let edges = graph.undirected_edge_set();
    let mut coefficients = Vec::with_capacity(graph.node_count());

    for node_idx in graph.node_indices() {
        let neighbors = graph.undirected_neighbors(node_idx);
        let k = neighbors.len();
        if k < 2 {
            coefficients.push(0.0);
            continue;
        }

        let mut linked_pairs = 0usize;
        for (i, &a) in neighbors.iter().enumerate() {
            for &b in &neighbors[i + 1..] {
                if edges.contains(&(a.min(b), a.max(b))) {
                    linked_pairs += 1;
                }
            }
        }

        let possible_pairs = k * (k - 1) / 2;
        coefficients.push(linked_pairs as f64 / possible_pairs as f64);
    }

    coefficients
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
    // Filter by minimum centrality.
    //
    // Only a metric this run actually COMPUTED may admit or reject a node. This
    // compared `closeness_centrality >= min_centrality` unconditionally, so on
    // any selection that does not include closeness the default
    // `--min-centrality 0.001` was being applied to a 0.0 initializer — a filter
    // decision taken on a number nothing measured.
    result.nodes.retain(|n| {
        n.degree_centrality >= min_centrality
            || n.betweenness_centrality
                .is_some_and(|v| v >= min_centrality)
            || n.closeness_centrality.is_some_and(|v| v >= min_centrality)
    });

    // Sort by combined score and take top K. An uncomputed metric contributes
    // nothing to the sum rather than a stand-in value, and the node name breaks
    // ties so the selection is stable across runs (most nodes tie on a sparse
    // graph, and with only degree computed they tie in bulk).
    result.nodes.sort_by(|a, b| {
        score_for_ranking(b)
            .total_cmp(&score_for_ranking(a))
            .then_with(|| a.name.cmp(&b.name))
    });

    result.nodes.truncate(top_k);

    result
}

/// Combined centrality score used to pick the top-k. Uncomputed measures are
/// worth 0 here — the sum is a ranking key, not a published number.
fn score_for_ranking(n: &NodeMetrics) -> f64 {
    n.degree_centrality
        + n.betweenness_centrality.unwrap_or(0.0)
        + n.closeness_centrality.unwrap_or(0.0)
        + n.pagerank.unwrap_or(0.0)
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

        let pagerank_values =
            distinct_count(result.nodes.iter().filter_map(|n| n.pagerank));
        assert!(
            pagerank_values > 1,
            "--metrics all left every node at the 1/N pagerank initializer: {:?}",
            result.nodes.iter().map(|n| n.pagerank).collect::<Vec<_>>()
        );

        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.betweenness_centrality.unwrap_or(0.0) > 0.0),
            "--metrics all reported betweenness 0.0 for every node of a chain"
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.closeness_centrality.unwrap_or(0.0) > 0.0),
            "--metrics all reported closeness 0.0 for every node of a chain"
        );
        // `All` is "All available metrics": every measure this command can
        // compute must be present, not just the four it used to expand into.
        assert!(
            result.nodes.iter().all(|n| n.clustering_coefficient.is_some()),
            "--metrics all did not compute the clustering coefficient"
        );
        assert!(
            result.nodes.iter().all(|n| n.component_id.is_some()),
            "--metrics all did not compute component membership"
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

/// Regression tests for the metrics a run did NOT compute (#932) and for the
/// `--metrics` values that computed nothing at all (#933).
#[cfg(test)]
mod unselected_metrics_are_not_published_tests {
    use super::*;
    use crate::cli::GraphMetricType;

    /// n0 -> n1 -> n2 -> n3 -> n4
    fn chain_graph(len: usize) -> SimpleGraph {
        let mut graph = SimpleGraph::new();
        let nodes: Vec<_> = (0..len).map(|i| graph.add_node(format!("n{i}"))).collect();
        for pair in nodes.windows(2) {
            graph.add_edge(pair[0], pair[1]);
        }
        graph
    }

    fn metrics(graph: &SimpleGraph, selection: GraphMetricType) -> GraphMetricsResult {
        calculate_metrics(graph, vec![selection], vec![], 0.85, 100, 1e-6).unwrap()
    }

    /// The defect exactly: `--metrics centrality` published `1/N` for every node
    /// as `pagerank` and `0.0` as `closeness_centrality`.
    #[test]
    fn a_metric_the_selection_omits_is_none_not_its_initializer() {
        let graph = chain_graph(5);
        let result = metrics(&graph, GraphMetricType::Centrality);

        for node in &result.nodes {
            assert_eq!(
                node.pagerank, None,
                "--metrics centrality published a pagerank for {}: {:?} (1/N = {})",
                node.name,
                node.pagerank,
                1.0 / 5.0
            );
            assert_eq!(node.closeness_centrality, None);
            assert_eq!(node.betweenness_centrality, None);
            assert_eq!(node.clustering_coefficient, None);
            assert_eq!(node.component_id, None);
        }
    }

    /// Each selection computes ITS metric and leaves the others `None`.
    #[test]
    fn each_selection_computes_exactly_what_it_names() {
        let graph = chain_graph(5);

        let pr = metrics(&graph, GraphMetricType::PageRank);
        assert!(pr.nodes.iter().all(|n| n.pagerank.is_some()));
        assert!(pr.nodes.iter().all(|n| n.closeness_centrality.is_none()));

        let cl = metrics(&graph, GraphMetricType::Closeness);
        assert!(cl.nodes.iter().all(|n| n.closeness_centrality.is_some()));
        assert!(cl.nodes.iter().all(|n| n.pagerank.is_none()));

        let bt = metrics(&graph, GraphMetricType::Betweenness);
        assert!(bt.nodes.iter().all(|n| n.betweenness_centrality.is_some()));
        assert!(bt.nodes.iter().all(|n| n.pagerank.is_none()));
    }

    /// `--min-centrality` must not be decided by a metric nothing measured:
    /// `filter_results` compared `closeness_centrality >= min_centrality` on the
    /// 0.0 initializer, so the default `0.001` was being applied to a
    /// non-measurement.
    ///
    /// HONESTY NOTE: this one is a contract test, not a proven regression. It
    /// passes on the pre-fix code too, because the initializer was 0.0 and every
    /// threshold a user can pass is either above it (rejects both ways) or 0.0
    /// (accepts both ways, since degree is never negative). The change is still
    /// required — the comparison must not be *reachable* — but the difference is
    /// not observable from the outside on any input.
    #[test]
    fn min_centrality_ignores_metrics_that_were_never_computed() {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_edge(a, b);
        // Third node, no edges: degree 0, and nothing else computed.
        graph.add_node("island".to_string());

        let result = metrics(&graph, GraphMetricType::Centrality);
        let filtered = filter_results(result, 100, 0.001);

        assert!(
            !filtered.nodes.iter().any(|n| n.name == "island"),
            "a node with no measured centrality above the threshold was retained"
        );
        assert_eq!(filtered.nodes.len(), 2, "{:?}", filtered.nodes);
    }

    /// `--metrics clustering` was byte-identical to `--metrics centrality`: the
    /// value was accepted, documented as "Clustering coefficient", and fell
    /// through the per-node `_ => {}` arm.
    #[test]
    fn clustering_is_computed_and_is_a_real_coefficient() {
        // A triangle plus a pendant: the triangle members' neighbourhoods are
        // fully connected, the pendant's is not.
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        let d = graph.add_node("d".to_string());
        graph.add_edge(a, b);
        graph.add_edge(b, c);
        graph.add_edge(c, a);
        graph.add_edge(a, d);

        let coefficients = calculate_clustering_all(&graph);

        // b's neighbours are a and c, and a-c is an edge => 1 of 1 pair.
        assert_eq!(coefficients[b.index()], 1.0);
        // a's neighbours are b, c and d: only b-c is closed => 1 of 3 pairs.
        assert!((coefficients[a.index()] - 1.0 / 3.0).abs() < 1e-12);
        // d has one neighbour: no pair exists to close.
        assert_eq!(coefficients[d.index()], 0.0);
    }

    /// `--metrics components` was byte-identical to `--metrics centrality` too.
    #[test]
    fn components_selection_labels_each_node_with_its_component() {
        let mut graph = SimpleGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_edge(a, b);
        let island = graph.add_node("island".to_string());

        let result = metrics(&graph, GraphMetricType::Components);
        let by_name = |name: &str| {
            result
                .nodes
                .iter()
                .find(|n| n.name == name)
                .unwrap()
                .component_id
        };

        assert_eq!(by_name("a"), by_name("b"));
        assert_ne!(by_name("a"), by_name("island"));
        assert_eq!(result.connected_components, 2);
        assert_eq!(graph.component_ids()[island.index()], 1);
    }

    /// Three different `--metrics` selections must not produce the same
    /// document. Comparing the rendered JSON is the same check the issue's
    /// `sha256sum` loop made.
    #[test]
    fn centrality_clustering_and_components_render_different_documents() {
        let graph = chain_graph(5);
        let render = |selection: GraphMetricType| {
            format_gm_as_json(filter_results(metrics(&graph, selection), 100, 0.0)).unwrap()
        };

        let centrality = render(GraphMetricType::Centrality);
        let clustering = render(GraphMetricType::Clustering);
        let components = render(GraphMetricType::Components);

        assert_ne!(centrality, clustering);
        assert_ne!(centrality, components);
        assert_ne!(clustering, components);
        assert!(clustering.contains("\"clustering_coefficient\": 0.0"));
        assert!(components.contains("\"component_id\": 0"));
    }

    /// The uncomputed metric has to survive as "uncomputed" all the way through
    /// the renderers: JSON `null`, an empty CSV field, `n/a` in the text
    /// formats. `0.000` in any of them is the fabrication again.
    #[test]
    fn renderers_report_an_uncomputed_metric_as_uncomputed() {
        let graph = chain_graph(3);
        let result = || filter_results(metrics(&graph, GraphMetricType::Centrality), 100, 0.0);

        let json = format_gm_as_json(result()).unwrap();
        assert!(json.contains("\"pagerank\": null"), "{json}");

        let csv = format_gm_as_csv(result()).unwrap();
        assert!(
            csv.lines().next().unwrap().contains("clustering,component_id"),
            "{csv}"
        );
        for row in csv.lines().skip(1) {
            // name,degree,betweenness,closeness,pagerank,clustering,component,in,out
            let fields: Vec<&str> = row.split(',').collect();
            assert_eq!(fields.len(), 9, "{row}");
            assert_eq!(&fields[2..7], &["", "", "", "", ""], "{row}");
        }

        let markdown = format_gm_as_markdown(result()).unwrap();
        assert!(markdown.contains("n/a"), "{markdown}");
    }
}
