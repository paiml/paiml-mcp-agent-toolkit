//! Graph metrics analysis - calculates centrality and other graph metrics
//! Uses a local SimpleGraph implementation (no petgraph dependency)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Local SimpleGraph implementation (replaces petgraph::Graph)

/// Node index for the simple graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeIndex(usize);

impl NodeIndex {
    #[allow(dead_code)]
    fn index(self) -> usize {
        self.0
    }
}

/// A simple directed graph with String nodes and unit edges
struct SimpleGraph {
    nodes: Vec<String>,
    /// Adjacency list: outgoing edges
    outgoing: Vec<Vec<usize>>,
    /// Adjacency list: incoming edges
    incoming: Vec<Vec<usize>>,
}

impl SimpleGraph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }

    fn add_node(&mut self, name: String) -> NodeIndex {
        let idx = self.nodes.len();
        self.nodes.push(name);
        self.outgoing.push(Vec::new());
        self.incoming.push(Vec::new());
        NodeIndex(idx)
    }

    fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.outgoing[from.0].push(to.0);
        self.incoming[to.0].push(from.0);
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.outgoing.iter().map(|v| v.len()).sum()
    }

    fn node_indices(&self) -> impl Iterator<Item = NodeIndex> {
        (0..self.nodes.len()).map(NodeIndex)
    }

    fn get_node(&self, idx: NodeIndex) -> &String {
        &self.nodes[idx.0]
    }

    fn out_degree(&self, idx: NodeIndex) -> usize {
        self.outgoing[idx.0].len()
    }

    fn in_degree(&self, idx: NodeIndex) -> usize {
        self.incoming[idx.0].len()
    }

    fn outgoing_edges(&self, idx: NodeIndex) -> &[usize] {
        &self.outgoing[idx.0]
    }

    /// Dijkstra's algorithm for shortest paths
    fn dijkstra(&self, source: NodeIndex, target: Option<NodeIndex>) -> HashMap<NodeIndex, i32> {
        use std::collections::BinaryHeap;

        let mut distances: HashMap<NodeIndex, i32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(source, 0);
        heap.push(std::cmp::Reverse((0, source)));

        while let Some(std::cmp::Reverse((dist, node))) = heap.pop() {
            if let Some(&best) = distances.get(&node) {
                if dist > best {
                    continue;
                }
            }

            // Early exit if we found the target
            if let Some(t) = target {
                if node == t {
                    return distances;
                }
            }

            for &neighbor_idx in &self.outgoing[node.0] {
                let neighbor = NodeIndex(neighbor_idx);
                let new_dist = dist + 1;

                let is_better = distances.get(&neighbor).map_or(true, |&d| new_dist < d);

                if is_better {
                    distances.insert(neighbor, new_dist);
                    heap.push(std::cmp::Reverse((new_dist, neighbor)));
                }
            }
        }

        distances
    }

    /// Connected components using BFS/DFS (treats graph as undirected)
    fn connected_components(&self) -> usize {
        let n = self.node_count();
        if n == 0 {
            return 0;
        }

        let mut visited = vec![false; n];
        let mut count = 0;

        for start in 0..n {
            if !visited[start] {
                self.dfs_undirected(start, &mut visited);
                count += 1;
            }
        }

        count
    }

    fn dfs_undirected(&self, node: usize, visited: &mut [bool]) {
        if visited[node] {
            return;
        }
        visited[node] = true;

        // Follow outgoing edges
        for &neighbor in &self.outgoing[node] {
            if !visited[neighbor] {
                self.dfs_undirected(neighbor, visited);
            }
        }

        // Follow incoming edges (treat as undirected)
        for &neighbor in &self.incoming[node] {
            if !visited[neighbor] {
                self.dfs_undirected(neighbor, visited);
            }
        }
    }

    /// Get edge endpoints for GraphML export
    fn edge_endpoints(&self) -> Vec<(NodeIndex, NodeIndex)> {
        let mut edges = Vec::new();
        for (from_idx, targets) in self.outgoing.iter().enumerate() {
            for &to_idx in targets {
                edges.push((NodeIndex(from_idx), NodeIndex(to_idx)));
            }
        }
        edges
    }
}

// Public types and functions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub name: String,
    pub degree_centrality: f64,
    pub betweenness_centrality: f64,
    pub closeness_centrality: f64,
    pub pagerank: f64,
    pub in_degree: usize,
    pub out_degree: usize,
}

#[derive(Debug, Serialize)]
pub struct GraphMetricsResult {
    pub nodes: Vec<NodeMetrics>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub average_degree: f64,
    pub max_degree: usize,
    pub connected_components: usize,
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_graph_metrics(
    project_path: PathBuf,
    metrics: Vec<crate::cli::GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    export_graphml: bool,
    format: crate::cli::GraphMetricsOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    _perf: bool,
    top_k: usize,
    min_centrality: f64,
) -> Result<()> {
    eprintln!("📊 Analyzing graph metrics...");

    // Build dependency graph
    let graph = build_dependency_graph(&project_path, &include, &exclude).await?;
    eprintln!(
        "✅ Built graph with {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    // Calculate metrics
    let metrics_result = calculate_metrics(
        &graph,
        metrics,
        pagerank_seeds,
        damping_factor,
        max_iterations,
        convergence_threshold,
    )?;

    // Filter results
    let filtered = filter_results(metrics_result, top_k, min_centrality);

    // Export GraphML if requested
    if export_graphml {
        export_to_graphml(&graph, &filtered, &output)?;
    }

    // Format output
    let content = format_output(filtered, format)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

// Build dependency graph from project
async fn build_dependency_graph(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<SimpleGraph> {
    let mut graph = SimpleGraph::new();
    let mut node_indices = HashMap::new();

    // Collect source files
    let files = collect_files(project_path, include, exclude).await?;

    // Add nodes for each file
    for file in &files {
        let name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let idx = graph.add_node(name.clone());
        node_indices.insert(name, idx);
    }

    // Add edges based on imports/dependencies
    for file in &files {
        let content = tokio::fs::read_to_string(file).await?;
        let file_name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if let Some(&from_idx) = node_indices.get(&file_name) {
            let deps = extract_dependencies(&content, file)?;
            for dep in deps {
                if let Some(&to_idx) = node_indices.get(&dep) {
                    graph.add_edge(from_idx, to_idx);
                }
            }
        }
    }

    Ok(graph)
}

// Collect files based on patterns
async fn collect_files(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    collect_files_recursive(project_path, &mut files, include, exclude).await?;

    Ok(files)
}

// Sprint 85 GREEN Phase: Refactored recursive file collection
// BEFORE: Complexity 14 (High entropy, mixed concerns)
// AFTER: Complexity 7 (A+ standard, single responsibility)
async fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Early exit for excluded paths - extracted logic
        if should_exclude_path_sprint85(&path.to_string_lossy(), exclude) {
            continue;
        }

        // Delegate entry processing to extracted function
        Box::pin(process_directory_entry_sprint85(
            path, files, include, exclude,
        ))
        .await?;
    }

    Ok(())
}

// Sprint 85 GREEN Phase: NEW EXTRACTED FUNCTIONS (A+ ≤10 complexity each)

/// Check if path should be excluded - EXTRACTED FUNCTION
/// Complexity: 3 (A+ standard)
fn should_exclude_path_sprint85(path_str: &str, exclude_pattern: &Option<String>) -> bool {
    if let Some(excl) = exclude_pattern {
        path_str.contains(excl)
    } else {
        false
    }
}

/// Check if path should be included - EXTRACTED FUNCTION\
/// Complexity: 3 (A+ standard)
fn should_include_path_sprint85(path_str: &str, include_pattern: &Option<String>) -> bool {
    if let Some(incl) = include_pattern {
        path_str.contains(incl)
    } else {
        true // Include all if no pattern specified
    }
}

/// Check if directory should be traversed - EXTRACTED FUNCTION
/// Complexity: 5 (A+ standard)
fn should_traverse_directory_sprint85(dir_name: &str) -> bool {
    !dir_name.starts_with('.') && dir_name != "node_modules" && dir_name != "target"
}

/// Process individual directory entry - EXTRACTED FUNCTION
/// Complexity: 8 (A+ standard)
async fn process_directory_entry_sprint85(
    path: PathBuf,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    if path.is_dir() {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if should_traverse_directory_sprint85(&name) {
            collect_files_recursive(&path, files, include, exclude).await?;
        }
    } else if is_source_file(&path) {
        let path_str = path.to_string_lossy();
        if should_include_path_sprint85(&path_str, include) {
            files.push(path);
        }
    }
    Ok(())
}

// Check if file is source
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java")
    )
}

// Extract dependencies from file
fn extract_dependencies(content: &str, file_path: &Path) -> Result<Vec<String>> {
    use regex::Regex;

    let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut deps = Vec::new();

    let patterns = match ext {
        "rs" => vec![Regex::new(r"use\s+(\w+)")?, Regex::new(r"mod\s+(\w+)")?],
        "js" | "ts" => vec![
            Regex::new(r#"import\s+.*from\s+['"]\./(\w+)"#)?,
            Regex::new(r#"require\(['"]\./(\w+)"#)?,
        ],
        "py" => vec![
            Regex::new(r"from\s+(\w+)\s+import")?,
            Regex::new(r"import\s+(\w+)")?,
        ],
        _ => vec![],
    };

    for pattern in patterns {
        for cap in pattern.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                deps.push(format!("{}.{}", name.as_str(), ext));
            }
        }
    }

    Ok(deps)
}

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

// Sprint 89 GREEN Phase: Refactored export_to_graphml function
// BEFORE: Complexity 14 (High entropy, mixed concerns)
// AFTER: Complexity 6 (A+ standard, single responsibility)
fn export_to_graphml(
    graph: &SimpleGraph,
    result: &GraphMetricsResult,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut graphml = String::new();

    // Delegate XML generation to extracted functions
    write_graphml_header(&mut graphml)?;
    write_graphml_nodes(&mut graphml, &result.nodes)?;
    write_graphml_edges(&mut graphml, graph)?;
    write_graphml_footer(&mut graphml)?;

    // Delegate file writing to extracted function
    write_graphml_file(&graphml, output)?;

    Ok(())
}

// Sprint 89 GREEN Phase: NEW EXTRACTED FUNCTIONS (A+ ≤10 complexity each)

/// Write `GraphML` XML header - EXTRACTED FUNCTION
/// Complexity: 3 (A+ standard)
fn write_graphml_header(graphml: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(graphml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        graphml,
        r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns">"#
    )?;
    writeln!(graphml, r#"  <graph id="G" edgedefault="directed">"#)?;
    Ok(())
}

/// Write `GraphML` nodes section - EXTRACTED FUNCTION\
/// Complexity: 4 (A+ standard)
fn write_graphml_nodes(graphml: &mut String, nodes: &[NodeMetrics]) -> Result<()> {
    use std::fmt::Write;
    for node in nodes {
        writeln!(graphml, r#"    <node id="{}" />"#, node.name)?;
    }
    Ok(())
}

/// Write `GraphML` edges section - EXTRACTED FUNCTION
/// Complexity: 7 (A+ standard)
fn write_graphml_edges(graphml: &mut String, graph: &SimpleGraph) -> Result<()> {
    use std::fmt::Write;

    // Write edges
    for (source, target) in graph.edge_endpoints() {
        let source_name = graph.get_node(source);
        let target_name = graph.get_node(target);
        writeln!(
            graphml,
            r#"    <edge source="{source_name}" target="{target_name}" />"#
        )?;
    }
    Ok(())
}

/// Write `GraphML` XML footer - EXTRACTED FUNCTION
/// Complexity: 2 (A+ standard)
fn write_graphml_footer(graphml: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(graphml, "  </graph>")?;
    writeln!(graphml, "</graphml>")?;
    Ok(())
}

/// Write `GraphML` to file - EXTRACTED FUNCTION
/// Complexity: 4 (A+ standard)
fn write_graphml_file(graphml: &str, output: &Option<PathBuf>) -> Result<()> {
    if let Some(path) = output {
        let graphml_path = path.with_extension("graphml");
        std::fs::write(&graphml_path, graphml)?;
        eprintln!("✅ GraphML exported to: {}", graphml_path.display());
    }
    Ok(())
}

// Format output
// Refactored format_output with reduced complexity
fn format_output(
    result: GraphMetricsResult,
    format: crate::cli::GraphMetricsOutputFormat,
) -> Result<String> {
    match format {
        crate::cli::GraphMetricsOutputFormat::Json => format_gm_as_json(result),
        crate::cli::GraphMetricsOutputFormat::Human
        | crate::cli::GraphMetricsOutputFormat::Summary
        | crate::cli::GraphMetricsOutputFormat::Detailed => format_gm_as_human(result),
        crate::cli::GraphMetricsOutputFormat::Csv => format_gm_as_csv(result),
        crate::cli::GraphMetricsOutputFormat::GraphML => {
            Ok("GraphML export handled separately.".to_string())
        }
        crate::cli::GraphMetricsOutputFormat::Markdown => format_gm_as_markdown(result),
    }
}

// Helper: Format as JSON
fn format_gm_as_json(result: GraphMetricsResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(&result)?)
}

// Helper: Format as human-readable
fn format_gm_as_human(result: GraphMetricsResult) -> Result<String> {
    let mut output = String::new();

    write_gm_human_header(&mut output)?;
    write_gm_statistics(&mut output, &result)?;
    write_gm_top_nodes(&mut output, &result)?;

    Ok(output)
}

// Helper: Write human header
fn write_gm_human_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Graph Metrics Analysis\n")?;
    writeln!(output, "## Graph Statistics")?;
    Ok(())
}

// Helper: Write statistics
fn write_gm_statistics(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "- Total nodes: {}", result.total_nodes)?;
    writeln!(output, "- Total edges: {}", result.total_edges)?;
    writeln!(output, "- Density: {:.3}", result.density)?;
    writeln!(output, "- Average degree: {:.2}", result.average_degree)?;
    writeln!(output, "- Max degree: {}", result.max_degree)?;
    writeln!(
        output,
        "- Connected components: {}",
        result.connected_components
    )?;
    Ok(())
}

// Helper: Write top nodes
fn write_gm_top_nodes(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## Top Nodes by Centrality\n")?;

    for (i, node) in result.nodes.iter().enumerate() {
        write_gm_node_details(output, i + 1, node)?;
    }

    Ok(())
}

// Helper: Write node details
fn write_gm_node_details(output: &mut String, index: usize, node: &NodeMetrics) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "{}. {} ", index, node.name)?;
    writeln!(
        output,
        "   - Degree: {:.3} (in: {}, out: {})",
        node.degree_centrality, node.in_degree, node.out_degree
    )?;
    writeln!(
        output,
        "   - Betweenness: {:.3}",
        node.betweenness_centrality
    )?;
    writeln!(output, "   - Closeness: {:.3}", node.closeness_centrality)?;
    writeln!(output, "   - PageRank: {:.3}", node.pagerank)?;
    writeln!(output)?;
    Ok(())
}

// Helper: Format as CSV
fn format_gm_as_csv(result: GraphMetricsResult) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    // Write header
    writeln!(
        output,
        "name,degree_centrality,betweenness,closeness,pagerank,in_degree,out_degree"
    )?;

    // Write data rows
    for node in result.nodes {
        writeln!(
            output,
            "{},{:.3},{:.3},{:.3},{:.3},{},{}",
            node.name,
            node.degree_centrality,
            node.betweenness_centrality,
            node.closeness_centrality,
            node.pagerank,
            node.in_degree,
            node.out_degree
        )?;
    }

    Ok(output)
}

// Helper: Format as Markdown
fn format_gm_as_markdown(result: GraphMetricsResult) -> Result<String> {
    let mut output = String::new();

    write_gm_markdown_header(&mut output)?;
    write_gm_markdown_summary(&mut output, &result)?;
    write_gm_markdown_top_nodes(&mut output, &result)?;

    Ok(output)
}

// Helper: Write Markdown header
fn write_gm_markdown_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Graph Metrics Report\n")?;
    writeln!(output, "## Summary\n")?;
    Ok(())
}

// Helper: Write Markdown summary table
fn write_gm_markdown_summary(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "| Metric | Value |")?;
    writeln!(output, "|--------|-------|")?;
    writeln!(output, "| Total Nodes | {} |", result.total_nodes)?;
    writeln!(output, "| Total Edges | {} |", result.total_edges)?;
    writeln!(output, "| Density | {:.3} |", result.density)?;
    writeln!(output, "| Average Degree | {:.2} |", result.average_degree)?;
    writeln!(output, "| Max Degree | {} |", result.max_degree)?;
    writeln!(
        output,
        "| Connected Components | {} |",
        result.connected_components
    )?;
    Ok(())
}

// Helper: Write Markdown top nodes table
fn write_gm_markdown_top_nodes(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## Top Nodes\n")?;
    writeln!(
        output,
        "| Node | Degree | Betweenness | Closeness | PageRank |"
    )?;
    writeln!(
        output,
        "|------|--------|-------------|-----------|----------|"
    )?;

    for node in result.nodes.iter().take(10) {
        writeln!(
            output,
            "| {} | {:.3} | {:.3} | {:.3} | {:.3} |",
            node.name,
            node.degree_centrality,
            node.betweenness_centrality,
            node.closeness_centrality,
            node.pagerank
        )?;
    }

    Ok(())
}

// Tests extracted to graph_metrics_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "graph_metrics_tests.rs"]
mod tests;
