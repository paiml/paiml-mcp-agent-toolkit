//! Graph metrics analysis - calculates centrality and other graph metrics

use anyhow::Result;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
        println!("{}", content);
    }

    Ok(())
}

// Build dependency graph from project
async fn build_dependency_graph(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<petgraph::Graph<String, ()>> {
    use petgraph::Graph;

    let mut graph = Graph::new();
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
                    graph.add_edge(from_idx, to_idx, ());
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

// Recursively collect files
async fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        if let Some(excl) = exclude {
            if path_str.contains(excl) {
                continue;
            }
        }

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "node_modules" && name != "target" {
                Box::pin(collect_files_recursive(&path, files, include, exclude)).await?;
            }
        } else if is_source_file(&path) {
            if let Some(incl) = include {
                if path_str.contains(incl) {
                    files.push(path);
                }
            } else {
                files.push(path);
            }
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
    graph: &petgraph::Graph<String, ()>,
    metric_types: Vec<crate::cli::GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
) -> Result<GraphMetricsResult> {
    use petgraph::algo::connected_components;

    let node_count = graph.node_count();
    let edge_count = graph.edge_count();

    let mut node_metrics = Vec::new();

    // Calculate metrics for each node
    for node_idx in graph.node_indices() {
        let name = &graph[node_idx];
        let in_degree = graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
            .count();
        let out_degree = graph
            .edges_directed(node_idx, petgraph::Direction::Outgoing)
            .count();

        let mut metrics = NodeMetrics {
            name: name.clone(),
            degree_centrality: (in_degree + out_degree) as f64 / (node_count - 1) as f64,
            betweenness_centrality: 0.0,
            closeness_centrality: 0.0,
            pagerank: 1.0 / node_count as f64,
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
        average_degree: total_degree as f64 / node_count as f64,
        max_degree,
        connected_components: connected_components(graph),
    })
}

// Calculate betweenness centrality (simplified)
fn calculate_betweenness(
    graph: &petgraph::Graph<String, ()>,
    node: petgraph::graph::NodeIndex,
) -> f64 {
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
        count as f64 / ((n - 1) * (n - 2)) as f64
    } else {
        0.0
    }
}

// Check if node is on shortest path
fn is_on_shortest_path(
    graph: &petgraph::Graph<String, ()>,
    source: petgraph::graph::NodeIndex,
    target: petgraph::graph::NodeIndex,
    node: petgraph::graph::NodeIndex,
) -> bool {
    use petgraph::algo::dijkstra;

    let from_source = dijkstra(graph, source, Some(target), |_| 1);
    let from_node = dijkstra(graph, node, Some(target), |_| 1);
    let to_node = dijkstra(graph, source, Some(node), |_| 1);

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
fn calculate_closeness(
    graph: &petgraph::Graph<String, ()>,
    node: petgraph::graph::NodeIndex,
) -> f64 {
    use petgraph::algo::dijkstra;

    let distances = dijkstra(graph, node, None, |_| 1);
    let total_distance: i32 = distances.values().sum();

    if total_distance > 0 {
        (graph.node_count() - 1) as f64 / total_distance as f64
    } else {
        0.0
    }
}

// Calculate PageRank
fn calculate_pagerank(
    graph: &petgraph::Graph<String, ()>,
    seeds: &[String],
    damping: f32,
    max_iter: usize,
    threshold: f64,
) -> Result<Vec<f64>> {
    let n = graph.node_count();
    let mut pagerank = vec![1.0 / n as f64; n];

    // Boost seed nodes
    for (i, node) in graph.node_indices().enumerate() {
        if seeds.contains(&graph[node]) {
            pagerank[i] = 2.0 / n as f64;
        }
    }

    // Power iteration
    for _ in 0..max_iter {
        let mut new_pagerank = vec![(1.0 - damping as f64) / n as f64; n];

        for (i, node) in graph.node_indices().enumerate() {
            let out_edges = graph.edges(node).count();
            if out_edges > 0 {
                let contrib = damping as f64 * pagerank[i] / out_edges as f64;
                for edge_ref in graph.edges(node) {
                    let target_idx = edge_ref.target();
                    new_pagerank[target_idx.index()] += contrib;
                }
            } else {
                // Distribute to all nodes
                let contrib = damping as f64 * pagerank[i] / n as f64;
                for pr in new_pagerank.iter_mut() {
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
        score_b.partial_cmp(&score_a).unwrap()
    });

    result.nodes.truncate(top_k);

    result
}

// Export to GraphML
fn export_to_graphml(
    graph: &petgraph::Graph<String, ()>,
    result: &GraphMetricsResult,
    output: &Option<PathBuf>,
) -> Result<()> {
    use std::fmt::Write;

    let mut graphml = String::new();
    writeln!(&mut graphml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        &mut graphml,
        r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns">"#
    )?;
    writeln!(&mut graphml, r#"  <graph id="G" edgedefault="directed">"#)?;

    // Add nodes
    for node in &result.nodes {
        writeln!(&mut graphml, r#"    <node id="{}" />"#, node.name)?;
    }

    // Add edges
    let node_names: HashMap<_, _> = graph
        .node_indices()
        .map(|idx| (idx, graph[idx].clone()))
        .collect();

    for edge in graph.edge_indices() {
        if let Some((source, target)) = graph.edge_endpoints(edge) {
            if let (Some(source_name), Some(target_name)) =
                (node_names.get(&source), node_names.get(&target))
            {
                writeln!(
                    &mut graphml,
                    r#"    <edge source="{}" target="{}" />"#,
                    source_name, target_name
                )?;
            }
        }
    }

    writeln!(&mut graphml, "  </graph>")?;
    writeln!(&mut graphml, "</graphml>")?;

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
    writeln!(output, "- Connected components: {}", result.connected_components)?;
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
    writeln!(output, "   - Degree: {:.3} (in: {}, out: {})",
        node.degree_centrality, node.in_degree, node.out_degree)?;
    writeln!(output, "   - Betweenness: {:.3}", node.betweenness_centrality)?;
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
    writeln!(output, "name,degree_centrality,betweenness,closeness,pagerank,in_degree,out_degree")?;
    
    // Write data rows
    for node in result.nodes {
        writeln!(output, "{},{:.3},{:.3},{:.3},{:.3},{},{}",
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
    writeln!(output, "| Connected Components | {} |", result.connected_components)?;
    Ok(())
}

// Helper: Write Markdown top nodes table
fn write_gm_markdown_top_nodes(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## Top Nodes\n")?;
    writeln!(output, "| Node | Degree | Betweenness | Closeness | PageRank |")?;
    writeln!(output, "|------|--------|-------------|-----------|----------|")?;
    
    for node in result.nodes.iter().take(10) {
        writeln!(output, "| {} | {:.3} | {:.3} | {:.3} | {:.3} |",
            node.name,
            node.degree_centrality,
            node.betweenness_centrality,
            node.closeness_centrality,
            node.pagerank
        )?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.rs")));
        assert!(is_source_file(Path::new("test.js")));
        assert!(!is_source_file(Path::new("test.txt")));
    }

    #[test]
    fn test_extract_dependencies() {
        let content = "use std::collections::HashMap;\nmod utils;";
        let deps = extract_dependencies(content, Path::new("main.rs")).unwrap();
        assert!(deps.contains(&"utils.rs".to_string()));
    }

    #[test]
    fn test_graph_metrics_result() {
        let result = GraphMetricsResult {
            nodes: vec![],
            total_nodes: 5,
            total_edges: 8,
            density: 0.4,
            average_degree: 3.2,
            max_degree: 5,
            connected_components: 1,
        };

        assert_eq!(result.total_nodes, 5);
        assert_eq!(result.connected_components, 1);
    }
}
