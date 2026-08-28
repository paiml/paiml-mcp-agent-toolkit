// Handler and dependency-graph construction

#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    crate::status_eprintln!("📊 Analyzing graph metrics...");

    // Build dependency graph
    let graph = build_dependency_graph(&project_path, &include, &exclude).await?;
    crate::status_eprintln!(
        "✅ Built graph with {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    // #1015: a node count of 0 means no source file was parsed. `DagBuilder`
    // registers the file's OWN module node as the first statement of
    // `collect_nodes` (src/services/dag_builder_node_collection.rs), so any file
    // that parsed contributes at least one node; the only other way to reach 0
    // is an `--include`/`--exclude` pair that admits nothing. This used to go on
    // to print "Total nodes: 0 / Density: 0.000 / Average degree: 0.00 /
    // Connected components: 0" and exit 0. Density and average degree are ratios
    // over the node count; at 0 they are undefined, not 0.000. A graph that WAS
    // built and simply has no edges still measures something, so the refusal is
    // on nodes, not edges.
    crate::cli::ensure_source_files_were_analyzed(
        "graph-metrics",
        &project_path,
        graph.node_count(),
    )?;

    // #1087: nodes but no edges makes every centrality below 0 BY CONSTRUCTION,
    // and nothing in the document distinguishes that from a measured verdict
    // that no file is central. The refusal above deliberately stays on nodes —
    // an edgeless graph over files that did parse is a real measurement, a tree
    // of standalone scripts has no dependencies — so this states the condition
    // rather than failing on it.
    if graph.edge_count() == 0 {
        crate::status_eprintln!(
            "⚠️  No dependency edge was resolved under {}: every centrality below is 0 \
             by construction, not a finding that nothing is central.",
            project_path.display()
        );
    }

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

    // `--export-graphml` says "Export graph as GraphML format", which is the
    // same request `-f graph-ml` makes. Folding the switch into the format
    // leaves one document per run and one writer below; modelling it as a
    // second, side-channel document is what gave it no destination without
    // `-o` (exit 1) and a colliding one with it (see `render_graphml`).
    let format = if export_graphml {
        crate::cli::GraphMetricsOutputFormat::GraphML
    } else {
        format
    };

    // Format output
    let content = format_output(filtered, format, &graph)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// The dependency graph `analyze dag` builds, mapped onto [`SimpleGraph`].
///
/// #1087: this used to build its OWN graph — one node per source file, and one
/// edge per `use X` / `mod X` / `import X` whose first captured word matched
/// some other collected file's BASE NAME. Both halves of that are wrong, and
/// both were measured on this tree with the pre-fix binary:
///
/// * a dependency spelled `use crate::printer::print;` captures `crate`, is
///   looked up as a file named `crate.rs`, and resolves to nothing. On a
///   two-file fixture whose only dependency has that shape, `pmat analyze dag
///   -p .` answers "4 nodes and 2 edges" and `pmat analyze graph-metrics -p .`
///   answers "2 nodes and 0 edges" over the identical tree. Every centrality is
///   then 0 by construction and the command prints that as a verdict.
/// * the lookup table was keyed by base name, so the 193 tracked files named
///   `mod.rs` in this repository all shared one entry. `analyze graph-metrics
///   -p . --format csv --metrics centrality` put a single node called `mod.rs`
///   at in-degree 1 / out-degree 1773 — an artefact of the aliasing, not a hub.
///
/// The graph now comes from the builder `analyze dag` uses, so the two commands
/// cannot answer differently about the same tree.
async fn build_dependency_graph(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<SimpleGraph> {
    let toolchain =
        crate::cli::detect_primary_language(project_path).unwrap_or_else(|| "rust".to_string());
    let project = crate::services::context::analyze_project(project_path, &toolchain).await?;

    // Steps 1 and 2 of `services::dag_pipeline::build_typed_dag` for a
    // full-dependency graph: the complete node set, then the call edges that
    // only the sources carry. Its remaining steps are deliberately NOT run here:
    //
    // * `filter_by_edge_types` selects a `--dag-type`. graph-metrics has no such
    //   flag; it wants every dependency.
    // * `enforce_edge_budget` is the Mermaid rendering budget, and its own
    //   documentation calls it "a PRESENTATION step ... destructive twice over":
    //   it truncates to EDGE_BUDGET (400) edges, dropping the lowest-priority
    //   type — `Imports` — first, and then deletes every node the survivors do
    //   not touch. `pmat analyze dag -p src/cli` reports exactly "400 edges"
    //   over 1,133 files for that reason. Centralities computed over that slice
    //   would describe the renderer's budget, not the tree.
    // * `add_pagerank_scores` and `annotate_function_complexity` write node
    //   fields `SimpleGraph` does not carry, so here they would be pure cost.
    let mut dependencies =
        crate::services::dag_builder::DagBuilder::build_from_project_unbudgeted(&project);
    crate::services::dag_call_edges::add_call_edges(&mut dependencies, project_path);

    Ok(simple_graph_from_dependencies(
        &dependencies,
        include,
        exclude,
    ))
}

/// Project a [`crate::models::dag::DependencyGraph`] onto the `SimpleGraph` the
/// metric algorithms take, honouring `--include` / `--exclude`.
///
/// DETERMINISM (CB-1335): `DependencyGraph::nodes` is an `FxHashMap`, and its
/// iteration order is not stable between runs. Numbering the nodes in that order
/// would move every node's index — and with it every ranking that breaks ties on
/// index or name — from run to run over an unchanged tree. The ids are therefore
/// sorted before an index is assigned to any of them, and the edge list is
/// sorted before it is inserted, for the reason given at the sort itself.
fn simple_graph_from_dependencies(
    dependencies: &crate::models::dag::DependencyGraph,
    include: &Option<String>,
    exclude: &Option<String>,
) -> SimpleGraph {
    // `--include`/`--exclude` were substring tests over the walked file path, and
    // they stay substring tests over the same string: `NodeInfo::file_path` is
    // the path the project walk recorded, which is what the old collector
    // matched on.
    let mut ids: Vec<&String> = dependencies
        .nodes
        .values()
        .filter(|node| {
            should_include_path_sprint85(&node.file_path, include)
                && !should_exclude_path_sprint85(&node.file_path, exclude)
        })
        .map(|node| &node.id)
        .collect();
    ids.sort_unstable();

    let mut graph = SimpleGraph::new();
    let mut node_indices: HashMap<&str, NodeIndex> = HashMap::new();
    for id in ids {
        let idx = graph.add_node(id.clone());
        node_indices.insert(id.as_str(), idx);
    }

    let mut edges: Vec<(NodeIndex, NodeIndex)> = Vec::with_capacity(dependencies.edges.len());
    for edge in &dependencies.edges {
        let (Some(&from), Some(&to)) = (
            node_indices.get(edge.from.as_str()),
            node_indices.get(edge.to.as_str()),
        ) else {
            // An endpoint filtered out by --include/--exclude takes its edges
            // with it.
            continue;
        };
        edges.push((from, to));
    }

    // Sorted, then deduplicated, for two separate reasons.
    //
    // DEDUP: the dependency graph records one edge per RELATIONSHIP, so three
    // `use b;` lines in one file are three `Imports` edges from `a` to `b`.
    // `SimpleGraph` carries no edge type, and its `out_degree` feeds
    // `degree_centrality`, which is defined over neighbours — keeping the
    // duplicates there would count import statements, the same inflation the
    // regex builder had.
    //
    // SORT: `add_edge` appends to the adjacency lists, so their order is the
    // order edges arrive in, and `dag_call_edges::add_call_edges` appends its
    // call edges after the declared ones from an index of its own. Brandes'
    // betweenness walks `outgoing_edges` to build each node's predecessor list
    // and then accumulates `delta` over that list; floating-point addition is
    // not associative, so a reordering moves the result in its last bits, and
    // `filter_results` ranks with `total_cmp`, which sees those bits. Sorting
    // makes the adjacency lists a function of the tree alone.
    edges.sort_unstable();
    edges.dedup();
    for (from, to) in edges {
        graph.add_edge(from, to);
    }

    graph
}

// What follows is the file walker the pre-#1087 builder used.
// `should_include_path_sprint85` and `should_exclude_path_sprint85` are still
// live — `simple_graph_from_dependencies` applies them to `--include` /
// `--exclude`. `should_traverse_directory_sprint85`, `is_source_file`,
// `extract_dependencies` and `DEP_PATTERNS` take no further part in building the
// graph; they are still here because graph_metrics_tests.rs and
// regex_hoisting_tests.rs pin them by name, and they should be removed together
// with those tests in one change rather than left half-deleted here.

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

// Check if file is source
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java")
    )
}

// Extract dependencies from file
/// Import patterns per language, compiled ONCE for the process.
///
/// They were built inside the per-file extractor, so every file in the tree paid
/// to recompile its language's two patterns. `analyze graph-metrics --path
/// src/services` took 6.37s over 1,387 files before this hoist. A controlled
/// same-bytes experiment put 93% of the command's runtime in a constant charged
/// per FILE rather than per byte, and `analyze complexity` over the same
/// fixtures showed no such gap — regex compilation was the only difference.
///
/// That measurement is recorded history: since #1087 the extractor below no
/// longer builds the graph, so nothing in this command calls it per file.
///
/// `expect` rather than `?`: compile-time literals, so a failure is a bug here
/// and not something a caller can act on.
struct DepPatterns {
    rust: Vec<regex::Regex>,
    js: Vec<regex::Regex>,
    py: Vec<regex::Regex>,
}

static DEP_PATTERNS: std::sync::LazyLock<DepPatterns> = std::sync::LazyLock::new(|| {
    use regex::Regex;
    let re = |p: &str| Regex::new(p).expect("static dependency pattern must compile");
    DepPatterns {
        rust: vec![re(r"use\s+(\w+)"), re(r"mod\s+(\w+)")],
        js: vec![
            re(r#"import\s+.*from\s+['"]\./(\w+)"#),
            re(r#"require\(['"]\./(\w+)"#),
        ],
        py: vec![re(r"from\s+(\w+)\s+import"), re(r"import\s+(\w+)")],
    }
});

fn extract_dependencies(content: &str, file_path: &Path) -> Result<Vec<String>> {
    let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut deps = Vec::new();

    let patterns: &[regex::Regex] = match ext {
        "rs" => &DEP_PATTERNS.rust,
        "js" | "ts" => &DEP_PATTERNS.js,
        "py" => &DEP_PATTERNS.py,
        _ => &[],
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

/// Regression tests for the graph source itself (#1087).
#[cfg(test)]
mod dependency_graph_edge_tests {
    use super::*;

    /// The pre-#1087 builder pushed one edge per `use` LINE, so `out_degree`
    /// counted import statements rather than dependencies. The dependency graph
    /// records the same three relationships (`pmat analyze dag -p .` over this
    /// fixture draws `a -.-> b` three times), so the collapse has to happen on
    /// the way into `SimpleGraph`.
    #[tokio::test]
    async fn repeated_imports_of_one_module_produce_one_edge() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("a.rs"), "use b;\nuse b;\nuse b;\n").expect("write a");
        std::fs::write(dir.path().join("b.rs"), "pub fn x() {}\n").expect("write b");

        let graph = build_dependency_graph(dir.path(), &None, &None)
            .await
            .expect("graph");

        // `a`, `b`, and `b::x`: the file module node for each file plus the one
        // declared function. `pmat analyze dag -p .` over this fixture reports
        // "3 nodes and 3 edges".
        assert_eq!(graph.node_count(), 3);
        assert_eq!(
            graph.edge_count(),
            1,
            "three `use b` lines are one dependency, not three edges"
        );
    }

    /// #1087, the defect itself: a dependency spelled the way Rust actually
    /// spells them resolved to nothing, because the regex captured `crate` and
    /// looked for a file named `crate.rs`.
    ///
    /// RED on the pre-fix code, measured rather than assumed: this is the
    /// fixture `pmat analyze dag -p .` answers "4 nodes and 2 edges" for and
    /// `pmat analyze graph-metrics -p .` answers "2 nodes and 0 edges" for, so
    /// the old builder fails both assertions below (2 nodes, 0 edges).
    #[tokio::test]
    async fn a_use_crate_dependency_produces_edges() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        std::fs::write(
            src.join("parser.rs"),
            "use crate::printer::print;\n\npub fn parse() {\n    print();\n}\n",
        )
        .expect("write parser.rs");
        std::fs::write(src.join("printer.rs"), "pub fn print() {}\n").expect("write printer.rs");

        let graph = build_dependency_graph(dir.path(), &None, &None)
            .await
            .expect("graph");

        assert!(
            graph.edge_count() > 0,
            "`use crate::printer::print` is a dependency and must produce an edge; \
             got {} nodes and {} edges",
            graph.node_count(),
            graph.edge_count()
        );
        assert!(
            graph.node_count() > 2,
            "the graph is the dependency graph's, which carries the declarations \
             too, not one node per file; got {} nodes",
            graph.node_count()
        );
    }

    /// `--exclude` must still remove a subtree, and it must take that subtree's
    /// edges with it.
    #[tokio::test]
    async fn exclude_drops_the_nodes_and_their_edges() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        std::fs::write(
            src.join("parser.rs"),
            "use crate::printer::print;\n\npub fn parse() {\n    print();\n}\n",
        )
        .expect("write parser.rs");
        std::fs::write(src.join("printer.rs"), "pub fn print() {}\n").expect("write printer.rs");

        let all = build_dependency_graph(dir.path(), &None, &None)
            .await
            .expect("graph");
        let without_printer =
            build_dependency_graph(dir.path(), &None, &Some("printer".to_string()))
                .await
                .expect("graph");

        // `pmat analyze dag` over this fixture reports "4 nodes and 2 edges",
        // and both edges cross into printer.rs.
        assert!(
            without_printer.node_count() < all.node_count(),
            "--exclude printer must drop nodes: {} vs {}",
            without_printer.node_count(),
            all.node_count()
        );
        assert!(
            all.edge_count() > 0,
            "the unfiltered fixture has dependencies"
        );
        assert!(
            without_printer.edge_count() < all.edge_count(),
            "an edge whose endpoint was excluded must go with it: {} vs {}",
            without_printer.edge_count(),
            all.edge_count()
        );
        for idx in without_printer.node_indices() {
            assert!(
                !without_printer.get_node(idx).contains("printer"),
                "excluded node survived: {}",
                without_printer.get_node(idx)
            );
        }
    }
}
