// GraphML export and output formatting

/// Render the dependency graph as a `GraphML` document.
///
/// `--export-graphml` (and its `-f graph-ml` spelling) used to be modelled as a
/// *second* document written on the side, which left it with no destination of
/// its own. Two defects fell straight out of that one modelling error:
///
/// * with no `-o` there was nowhere to put it, so the flag bailed — adding
///   `--export-graphml` to a working invocation turned exit 0 into exit 1 with
///   an empty stdout, which is a worse defect than a flag that does nothing;
/// * with `-o out.graphml` — the exact spelling that bail told you to use —
///   the XML was written to `<PATH>.graphml` and then immediately overwritten
///   by the metrics summary going to `<PATH>`, under a green
///   "✅ GraphML exported to:". The success message survived; the export did
///   not.
///
/// `GraphML` is now simply *the document this run produces*, delivered through
/// the command's one existing output channel: `-o` when given, stdout
/// otherwise. One document per invocation, one writer, no sidecar.
fn render_graphml(graph: &SimpleGraph) -> Result<String> {
    let mut graphml = String::new();
    write_graphml_header(&mut graphml)?;
    write_graphml_nodes(&mut graphml, graph)?;
    write_graphml_edges(&mut graphml, graph)?;
    write_graphml_footer(&mut graphml)?;
    Ok(graphml)
}

/// Escape the XML predefined entities.
///
/// Node labels are file names, and a file name may legally contain `&` or `<`.
/// Interpolating one raw into an attribute or element produced a document that
/// is not even well-formed XML, which is the one thing an exporter must never
/// emit.
fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Write `GraphML` XML header.
fn write_graphml_header(graphml: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(graphml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        graphml,
        r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns">"#
    )?;
    writeln!(
        graphml,
        r#"  <key id="d0" for="node" attr.name="label" attr.type="string"/>"#
    )?;
    writeln!(graphml, r#"  <graph id="G" edgedefault="directed">"#)?;
    Ok(())
}

/// Write every node in the graph.
///
/// This used to emit `result.nodes` — the list `--top-k` and `--min-centrality`
/// had already truncated — while the edge section emitted the whole graph. On
/// the 124-node corpus that meant 20 declared nodes against 119 edges pointing
/// at 100 ids that were never declared: valid XML, but not a graph any
/// `GraphML` reader can load. The graph is what `--help` promises to export, so
/// the graph is what gets declared.
///
/// Identity is the node index, with the name carried as a `label`, because
/// names are bare file names: any repository with two `mod.rs` files would
/// otherwise emit duplicate `id`s and silently merge two distinct nodes.
fn write_graphml_nodes(graphml: &mut String, graph: &SimpleGraph) -> Result<()> {
    use std::fmt::Write;
    for idx in graph.node_indices() {
        writeln!(
            graphml,
            r#"    <node id="n{}"><data key="d0">{}</data></node>"#,
            idx.index(),
            escape_xml(graph.get_node(idx))
        )?;
    }
    Ok(())
}

/// Write `GraphML` edges section.
fn write_graphml_edges(graphml: &mut String, graph: &SimpleGraph) -> Result<()> {
    use std::fmt::Write;

    for (source, target) in graph.edge_endpoints() {
        writeln!(
            graphml,
            r#"    <edge source="n{}" target="n{}" />"#,
            source.index(),
            target.index()
        )?;
    }
    Ok(())
}

/// Write `GraphML` XML footer.
fn write_graphml_footer(graphml: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(graphml, "  </graph>")?;
    writeln!(graphml, "</graphml>")?;
    Ok(())
}

// Format output
// Refactored format_output with reduced complexity
fn format_output(
    result: GraphMetricsResult,
    format: crate::cli::GraphMetricsOutputFormat,
    graph: &SimpleGraph,
) -> Result<String> {
    match format {
        crate::cli::GraphMetricsOutputFormat::Json => format_gm_as_json(result),
        // These three used to be one arm, so `--format summary`,
        // `--format detailed` and `--format human` produced byte-identical
        // output — `-f detailed` was documented as "Detailed metrics with
        // rankings" and `-f summary` as "Summary statistics only", and both
        // printed the same middle rendering. They now differ by what they
        // contain, not just by name.
        crate::cli::GraphMetricsOutputFormat::Summary => format_gm_as_summary(&result),
        crate::cli::GraphMetricsOutputFormat::Human => format_gm_as_human(result),
        crate::cli::GraphMetricsOutputFormat::Detailed => format_gm_as_detailed(&result),
        crate::cli::GraphMetricsOutputFormat::Csv => format_gm_as_csv(result),
        // `-f graph-ml` first returned a 34-byte developer note ("GraphML
        // export handled separately") as the *document*, and was then changed
        // to refuse outright on the grounds that only `--export-graphml` held
        // the `SimpleGraph` needed to emit edges. It holds it now: the graph is
        // a parameter, so `-f graph-ml` and `--export-graphml` are one request
        // spelled two ways and render the identical document.
        crate::cli::GraphMetricsOutputFormat::GraphML => render_graphml(graph),
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

/// `-f summary`: the graph-level statistics and nothing else, which is what
/// "Summary statistics only" in `--help` promises.
fn format_gm_as_summary(result: &GraphMetricsResult) -> Result<String> {
    let mut output = String::new();

    write_gm_human_header(&mut output)?;
    write_gm_statistics(&mut output, result)?;

    Ok(output)
}

/// `-f detailed`: everything `human` shows, plus the per-measure rankings
/// `--help` promises. The node set is the same one `--top-k` already selected;
/// what this adds is the order each centrality measure puts it in, which the
/// single combined listing cannot show.
fn format_gm_as_detailed(result: &GraphMetricsResult) -> Result<String> {
    let mut output = String::new();

    write_gm_human_header(&mut output)?;
    write_gm_statistics(&mut output, result)?;
    write_gm_top_nodes(&mut output, result)?;
    write_gm_rankings(&mut output, result)?;

    Ok(output)
}

/// A centrality measure: its heading and how to read it off a node.
type CentralityMeasure = (&'static str, fn(&NodeMetrics) -> f64);

/// One ranked list per centrality measure.
fn write_gm_rankings(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let measures: [CentralityMeasure; 4] = [
        ("PageRank", |n| n.pagerank),
        ("Betweenness", |n| n.betweenness_centrality),
        ("Closeness", |n| n.closeness_centrality),
        ("Degree", |n| n.degree_centrality),
    ];

    for (label, key) in measures {
        writeln!(output, "\n{}Ranked by {}{}\n", c::BOLD, label, c::RESET)?;
        let mut ranked: Vec<&NodeMetrics> = result.nodes.iter().collect();
        // DETERMINISM: name breaks ties, so nodes with equal scores (which is
        // most of them on a sparse graph) rank in a fixed order across runs.
        ranked.sort_by(|a, b| {
            key(b)
                .partial_cmp(&key(a))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name))
        });
        for (i, node) in ranked.iter().enumerate() {
            writeln!(
                output,
                "  {}. {}{}{} {}{:.3}{}",
                i + 1,
                c::CYAN,
                node.name,
                c::RESET,
                c::BOLD_WHITE,
                key(node),
                c::RESET
            )?;
        }
    }

    Ok(())
}

// Helper: Write human header
fn write_gm_human_header(output: &mut String) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(
        output,
        "{}{}Graph Metrics Analysis{}\n",
        c::BOLD,
        c::UNDERLINE,
        c::RESET
    )?;
    writeln!(output, "{}Graph Statistics{}", c::BOLD, c::RESET)?;
    Ok(())
}

// Helper: Write statistics
fn write_gm_statistics(output: &mut String, result: &GraphMetricsResult) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(
        output,
        "  {}Total nodes:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.total_nodes,
        c::RESET
    )?;
    writeln!(
        output,
        "  {}Total edges:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.total_edges,
        c::RESET
    )?;
    writeln!(
        output,
        "  {}Density:{} {}{:.3}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.density,
        c::RESET
    )?;
    writeln!(
        output,
        "  {}Average degree:{} {}{:.2}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.average_degree,
        c::RESET
    )?;
    writeln!(
        output,
        "  {}Max degree:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.max_degree,
        c::RESET
    )?;
    writeln!(
        output,
        "  {}Connected components:{} {}{}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        result.connected_components,
        c::RESET
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
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        node.degree_centrality,
        c::RESET,
        node.in_degree,
        node.out_degree
    )?;
    writeln!(
        output,
        "     {}Betweenness:{} {}{:.3}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        node.betweenness_centrality,
        c::RESET
    )?;
    writeln!(
        output,
        "     {}Closeness:{} {}{:.3}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        node.closeness_centrality,
        c::RESET
    )?;
    writeln!(
        output,
        "     {}PageRank:{} {}{:.3}{}",
        c::BOLD,
        c::RESET,
        c::BOLD_WHITE,
        node.pagerank,
        c::RESET
    )?;
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
