// GraphML export and output formatting

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
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}{}Graph Metrics Analysis{}\n", c::BOLD, c::UNDERLINE, c::RESET)?;
    writeln!(output, "{}Graph Statistics{}", c::BOLD, c::RESET)?;
    Ok(())
}

// Helper: Write statistics
fn write_gm_statistics(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "  {}Total nodes:{} {}{}{}", c::BOLD, c::RESET, c::BOLD_WHITE, result.total_nodes, c::RESET)?;
    writeln!(output, "  {}Total edges:{} {}{}{}", c::BOLD, c::RESET, c::BOLD_WHITE, result.total_edges, c::RESET)?;
    writeln!(output, "  {}Density:{} {}{:.3}{}", c::BOLD, c::RESET, c::BOLD_WHITE, result.density, c::RESET)?;
    writeln!(output, "  {}Average degree:{} {}{:.2}{}", c::BOLD, c::RESET, c::BOLD_WHITE, result.average_degree, c::RESET)?;
    writeln!(output, "  {}Max degree:{} {}{}{}", c::BOLD, c::RESET, c::BOLD_WHITE, result.max_degree, c::RESET)?;
    writeln!(
        output,
        "  {}Connected components:{} {}{}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, result.connected_components, c::RESET
    )?;
    Ok(())
}

// Helper: Write top nodes
fn write_gm_top_nodes(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "\n{}Top Nodes by Centrality{}\n", c::BOLD, c::RESET)?;

    for (i, node) in result.nodes.iter().enumerate() {
        write_gm_node_details(output, i + 1, node)?;
    }

    Ok(())
}

// Helper: Write node details
fn write_gm_node_details(output: &mut String, index: usize, node: &NodeMetrics) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "  {}. {}{}{}", index, c::CYAN, node.name, c::RESET)?;
    writeln!(
        output,
        "     {}Degree:{} {}{:.3}{} (in: {}, out: {})",
        c::BOLD, c::RESET, c::BOLD_WHITE, node.degree_centrality, c::RESET, node.in_degree, node.out_degree
    )?;
    writeln!(
        output,
        "     {}Betweenness:{} {}{:.3}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, node.betweenness_centrality, c::RESET
    )?;
    writeln!(output, "     {}Closeness:{} {}{:.3}{}", c::BOLD, c::RESET, c::BOLD_WHITE, node.closeness_centrality, c::RESET)?;
    writeln!(output, "     {}PageRank:{} {}{:.3}{}", c::BOLD, c::RESET, c::BOLD_WHITE, node.pagerank, c::RESET)?;
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
