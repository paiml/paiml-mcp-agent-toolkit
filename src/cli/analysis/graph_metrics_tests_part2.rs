
#[test]
fn test_calculate_metrics_all_types() {
    let graph = create_simple_graph();
    let result = calculate_metrics(
        &graph,
        vec![
            GraphMetricType::Centrality,
            GraphMetricType::Betweenness,
            GraphMetricType::Closeness,
            GraphMetricType::PageRank,
        ],
        vec![],
        0.85,
        100,
        1e-6,
    )
    .unwrap();

    for node in &result.nodes {
        assert!(node.degree_centrality >= 0.0);
        // Every one of these was explicitly selected above, so each must be
        // Some: `None` here would mean the selection computed nothing.
        assert!(node.betweenness_centrality.expect("betweenness selected") >= 0.0);
        assert!(node.closeness_centrality.expect("closeness selected") >= 0.0);
        assert!(node.pagerank.expect("page-rank selected") >= 0.0);
    }
}

#[test]
fn test_calculate_metrics_empty_graph() {
    let graph = create_empty_graph();
    let result = calculate_metrics(
        &graph,
        vec![GraphMetricType::Centrality],
        vec![],
        0.85,
        100,
        1e-6,
    )
    .unwrap();

    assert_eq!(result.total_nodes, 0);
    assert_eq!(result.total_edges, 0);
    assert!(result.nodes.is_empty());
}

#[test]
fn test_calculate_metrics_single_node() {
    let graph = create_single_node_graph();
    let result = calculate_metrics(
        &graph,
        vec![GraphMetricType::Centrality],
        vec![],
        0.85,
        100,
        1e-6,
    )
    .unwrap();

    assert_eq!(result.total_nodes, 1);
    assert_eq!(result.total_edges, 0);
    assert_eq!(result.density, 0.0);
}

#[test]
fn test_calculate_metrics_disconnected_graph() {
    let graph = create_disconnected_graph();
    let result = calculate_metrics(
        &graph,
        vec![GraphMetricType::Centrality],
        vec![],
        0.85,
        100,
        1e-6,
    )
    .unwrap();

    assert_eq!(result.total_nodes, 4);
    assert_eq!(result.connected_components, 2);
}

// calculate_betweenness tests

#[test]
fn test_calculate_betweenness_linear_graph() {
    let graph = create_linear_graph();
    let node_indices: Vec<_> = graph.node_indices().collect();

    // Middle node should have higher betweenness
    let betweenness_n2 = calculate_betweenness(&graph, node_indices[1]);
    assert!(betweenness_n2 >= 0.0);
}

#[test]
fn test_calculate_betweenness_star_graph() {
    let graph = create_star_graph();
    let center_idx = graph.node_indices().next().unwrap();

    let betweenness = calculate_betweenness(&graph, center_idx);
    assert!(betweenness >= 0.0);
}

#[test]
fn test_calculate_betweenness_two_node_graph() {
    let mut graph = SimpleGraph::new();
    let a = graph.add_node("A".to_string());
    let b = graph.add_node("B".to_string());
    graph.add_edge(a, b);

    // With only 2 nodes, betweenness should be 0
    let betweenness = calculate_betweenness(&graph, a);
    assert_eq!(betweenness, 0.0);
}

// calculate_closeness tests

#[test]
fn test_calculate_closeness_simple_graph() {
    let graph = create_simple_graph();
    let node = graph.node_indices().next().unwrap();

    let closeness = calculate_closeness(&graph, node);
    assert!(closeness >= 0.0);
}

#[test]
fn test_calculate_closeness_disconnected_node() {
    let mut graph = SimpleGraph::new();
    graph.add_node("isolated".to_string());

    let node = graph.node_indices().next().unwrap();
    let closeness = calculate_closeness(&graph, node);
    assert_eq!(closeness, 0.0);
}

#[test]
fn test_calculate_closeness_star_center() {
    let graph = create_star_graph();
    let center = graph.node_indices().next().unwrap();

    let closeness = calculate_closeness(&graph, center);
    assert!(closeness > 0.0);
}

// calculate_pagerank tests

#[test]
fn test_calculate_pagerank_simple() {
    let graph = create_simple_graph();
    let pageranks = calculate_pagerank(&graph, &[], 0.85, 100, 1e-6).unwrap();

    assert_eq!(pageranks.len(), 3);
    let total: f64 = pageranks.iter().sum();
    assert!((total - 1.0).abs() < 0.1);
}

#[test]
fn test_calculate_pagerank_with_seeds() {
    let graph = create_star_graph();
    let pageranks = calculate_pagerank(&graph, &["center".to_string()], 0.85, 100, 1e-6).unwrap();

    assert_eq!(pageranks.len(), 5);
}

#[test]
fn test_calculate_pagerank_damping_variations() {
    let graph = create_simple_graph();

    // Test different damping factors
    let pr_high = calculate_pagerank(&graph, &[], 0.99, 100, 1e-6).unwrap();
    let pr_low = calculate_pagerank(&graph, &[], 0.5, 100, 1e-6).unwrap();

    // Both should have valid pageranks
    assert!(!pr_high.is_empty());
    assert!(!pr_low.is_empty());
}

#[test]
fn test_calculate_pagerank_convergence() {
    let graph = create_simple_graph();

    // Test with tight convergence threshold
    let pr = calculate_pagerank(&graph, &[], 0.85, 1000, 1e-10).unwrap();
    assert!(!pr.is_empty());
}

#[test]
fn test_calculate_pagerank_dangling_nodes() {
    // Graph with dangling node (no outgoing edges)
    let mut graph = SimpleGraph::new();
    let a = graph.add_node("A".to_string());
    let b = graph.add_node("B".to_string());
    let _c = graph.add_node("C".to_string()); // dangling

    graph.add_edge(a, b);

    let pageranks = calculate_pagerank(&graph, &[], 0.85, 100, 1e-6).unwrap();
    assert_eq!(pageranks.len(), 3);
}

// is_on_shortest_path tests

#[test]
fn test_is_on_shortest_path_linear() {
    let graph = create_linear_graph();
    let indices: Vec<_> = graph.node_indices().collect();

    // n2 is on path from n1 to n3
    let on_path = is_on_shortest_path(&graph, indices[0], indices[2], indices[1]);
    assert!(on_path);
}

#[test]
fn test_is_on_shortest_path_not_on_path() {
    let mut graph = SimpleGraph::new();
    let a = graph.add_node("A".to_string());
    let b = graph.add_node("B".to_string());
    let c = graph.add_node("C".to_string());

    graph.add_edge(a, b);
    graph.add_edge(a, c);

    // c is not on path from a to b (direct edge)
    let on_path = is_on_shortest_path(&graph, a, b, c);
    assert!(!on_path);
}

#[test]
fn test_is_on_shortest_path_no_path() {
    let graph = create_disconnected_graph();
    let indices: Vec<_> = graph.node_indices().collect();

    // No path between disconnected components
    let on_path = is_on_shortest_path(&graph, indices[0], indices[2], indices[1]);
    assert!(!on_path);
}

// filter_results tests

fn create_mock_result() -> GraphMetricsResult {
    GraphMetricsResult {
        nodes: vec![
            NodeMetrics {
                name: "high".to_string(),
                degree_centrality: 0.9,
                betweenness_centrality: Some(0.8),
                closeness_centrality: Some(0.7),
                pagerank: Some(0.3),
                clustering_coefficient: None,
                component_id: None,
                in_degree: 5,
                out_degree: 4,
            },
            NodeMetrics {
                name: "medium".to_string(),
                degree_centrality: 0.5,
                betweenness_centrality: Some(0.4),
                closeness_centrality: Some(0.3),
                pagerank: Some(0.2),
                clustering_coefficient: None,
                component_id: None,
                in_degree: 2,
                out_degree: 2,
            },
            NodeMetrics {
                name: "low".to_string(),
                degree_centrality: 0.1,
                betweenness_centrality: Some(0.05),
                closeness_centrality: Some(0.08),
                pagerank: Some(0.1),
                clustering_coefficient: None,
                component_id: None,
                in_degree: 1,
                out_degree: 0,
            },
        ],
        total_nodes: 3,
        total_edges: 5,
        density: 0.5,
        average_degree: 3.33,
        max_degree: 9,
        connected_components: 1,
    }
}

#[test]
fn test_filter_results_top_k() {
    let result = create_mock_result();
    let filtered = filter_results(result, 2, 0.0);

    assert_eq!(filtered.nodes.len(), 2);
    assert_eq!(filtered.nodes[0].name, "high");
}

#[test]
fn test_filter_results_min_centrality() {
    let result = create_mock_result();
    let filtered = filter_results(result, 10, 0.2);

    // Only nodes with centrality >= 0.2 should remain
    assert!(filtered.nodes.iter().all(|n| n.degree_centrality >= 0.2
        || n.betweenness_centrality.is_some_and(|v| v >= 0.2)
        || n.closeness_centrality.is_some_and(|v| v >= 0.2)));
}

#[test]
fn test_filter_results_combined() {
    let result = create_mock_result();
    let filtered = filter_results(result, 1, 0.0);

    assert_eq!(filtered.nodes.len(), 1);
    assert_eq!(filtered.nodes[0].name, "high");
}

#[test]
fn test_filter_results_large_top_k() {
    let result = create_mock_result();
    let filtered = filter_results(result, 100, 0.0);

    assert_eq!(filtered.nodes.len(), 3);
}

// GraphML export tests

#[test]
fn test_write_graphml_header() {
    let mut output = String::new();
    write_graphml_header(&mut output).unwrap();

    assert!(output.contains("<?xml version"));
    assert!(output.contains("graphml"));
    assert!(output.contains("graph id=\"G\""));
}

#[test]
fn test_write_graphml_nodes() {
    let mut graph = SimpleGraph::new();
    graph.add_node("node1".to_string());
    graph.add_node("node2".to_string());

    let mut output = String::new();
    write_graphml_nodes(&mut output, &graph).unwrap();

    assert!(output.contains("node1"));
    assert!(output.contains("node2"));
    assert!(output.contains("<node id="));
}

#[test]
fn test_write_graphml_edges() {
    let graph = create_simple_graph();
    let mut output = String::new();
    write_graphml_edges(&mut output, &graph).unwrap();

    assert!(output.contains("<edge source="));
    assert!(output.contains("target="));
}

/// A file name may contain `&` or `<`. Interpolated raw, the export was not
/// even well-formed XML.
#[test]
fn test_graphml_escapes_xml_metacharacters_in_names() {
    let mut graph = SimpleGraph::new();
    graph.add_node("a&b<c>\"d\".rs".to_string());

    let doc = render_graphml(&graph).expect("render");

    assert!(doc.contains("a&amp;b&lt;c&gt;&quot;d&quot;.rs"), "{doc}");
    assert!(
        !doc.contains("a&b<c>"),
        "raw metacharacters leaked into the document: {doc}"
    );
}

#[test]
fn test_write_graphml_footer() {
    let mut output = String::new();
    write_graphml_footer(&mut output).unwrap();

    assert!(output.contains("</graph>"));
    assert!(output.contains("</graphml>"));
}

/// Pull every `id=` / `source=` / `target=` value out of a `GraphML` document.
fn graphml_ids(doc: &str) -> (Vec<String>, Vec<(String, String)>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for line in doc.lines() {
        let attr = |name: &str| -> Option<String> {
            let key = format!("{name}=\"");
            let start = line.find(&key)? + key.len();
            let rest = &line[start..];
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        };
        if line.trim_start().starts_with("<node ") {
            nodes.push(attr("id").expect("node needs an id"));
        } else if line.trim_start().starts_with("<edge ") {
            edges.push((
                attr("source").expect("edge needs a source"),
                attr("target").expect("edge needs a target"),
            ));
        }
    }
    (nodes, edges)
}

/// The export used to declare only the `--top-k`-filtered nodes while emitting
/// every edge in the graph: on the 124-node corpus, 20 `<node>` elements and
/// 119 edges pointing at 100 ids that were never declared. No `GraphML` reader
/// can load that.
///
/// RED on the old code: `write_graphml_nodes` took `&result.nodes`.
#[test]
fn test_graphml_declares_every_edge_endpoint() {
    let mut graph = SimpleGraph::new();
    let ids: Vec<_> = (0..6).map(|i| graph.add_node(format!("f{i}.rs"))).collect();
    for w in ids.windows(2) {
        graph.add_edge(w[0], w[1]);
    }

    let doc = render_graphml(&graph).expect("render");
    let (nodes, edges) = graphml_ids(&doc);

    assert_eq!(nodes.len(), 6, "every graph node must be declared: {doc}");
    assert_eq!(edges.len(), 5, "{doc}");
    let declared: std::collections::HashSet<&String> = nodes.iter().collect();
    for (s, t) in &edges {
        assert!(declared.contains(s), "undeclared edge source {s}: {doc}");
        assert!(declared.contains(t), "undeclared edge target {t}: {doc}");
    }
}

/// Node names are bare file names, so any repository with two `mod.rs` files
/// produced two `<node id="mod.rs">` elements — duplicate ids that silently
/// merge two distinct nodes.
///
/// RED on the old code: the id was `node.name`.
#[test]
fn test_graphml_ids_are_unique_when_basenames_collide() {
    let mut graph = SimpleGraph::new();
    let a = graph.add_node("mod.rs".to_string());
    let b = graph.add_node("mod.rs".to_string());
    graph.add_edge(a, b);

    let doc = render_graphml(&graph).expect("render");
    let (nodes, _) = graphml_ids(&doc);

    assert_eq!(nodes.len(), 2, "{doc}");
    let unique: std::collections::HashSet<&String> = nodes.iter().collect();
    assert_eq!(unique.len(), 2, "duplicate node ids in {doc}");
    assert_eq!(doc.matches("mod.rs").count(), 2, "labels preserved: {doc}");
}

/// `--export-graphml` with no `-o` used to bail: the flag turned a working
/// command (exit 0) into a failing one with an empty stdout. The document has
/// a destination now — stdout — so the command still works.
///
/// RED on the old code: `write_graphml_file` returned `Err` for `&None`.
#[tokio::test]
async fn test_export_graphml_without_output_path_still_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.rs"), "use b;\n").expect("write a");
    std::fs::write(dir.path().join("b.rs"), "pub fn x() {}\n").expect("write b");

    handle_analyze_graph_metrics(
        dir.path().to_path_buf(),
        vec![crate::cli::GraphMetricType::All],
        vec![],
        0.85,
        100,
        0.001,
        true,
        crate::cli::GraphMetricsOutputFormat::Summary,
        None,
        None,
        None,
        false,
        20,
        0.0,
    )
    .await
    .expect("--export-graphml with no -o must not fail the command");
}

/// `--export-graphml -o out.graphml` — the spelling the old error message told
/// you to use — wrote the XML to `<PATH>.graphml` and then overwrote it with
/// the metrics summary going to `<PATH>`, while printing
/// "✅ GraphML exported to:" over the wreckage.
///
/// RED on the old code: `out.graphml` held "Graph Metrics Analysis…".
#[tokio::test]
async fn test_export_graphml_output_file_holds_graphml() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.rs"), "use b;\n").expect("write a");
    std::fs::write(dir.path().join("b.rs"), "pub fn x() {}\n").expect("write b");
    let out = dir.path().join("out.graphml");

    handle_analyze_graph_metrics(
        dir.path().to_path_buf(),
        vec![crate::cli::GraphMetricType::All],
        vec![],
        0.85,
        100,
        0.001,
        true,
        crate::cli::GraphMetricsOutputFormat::Summary,
        None,
        None,
        Some(out.clone()),
        false,
        20,
        0.0,
    )
    .await
    .expect("export");

    let content = std::fs::read_to_string(&out).expect("out.graphml must exist");
    assert!(content.starts_with("<?xml version"), "got: {content}");
    assert!(content.contains("</graphml>"), "got: {content}");
    assert!(
        !content.contains("Graph Metrics Analysis"),
        "the metrics summary overwrote the export: {content}"
    );
}
