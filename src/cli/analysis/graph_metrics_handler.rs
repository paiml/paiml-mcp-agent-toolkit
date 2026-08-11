// Handler and file collection logic

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
            // One dependency edge per distinct neighbour. Every `use foo` line
            // in a file used to push another copy of the same edge, so
            // out_degree counted import STATEMENTS, not dependencies, and
            // degree_centrality was inflated for files that import one module
            // repeatedly.
            let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
            for dep in deps {
                if let Some(&to_idx) = node_indices.get(&dep) {
                    if seen.insert(to_idx.index()) {
                        graph.add_edge(from_idx, to_idx);
                    }
                }
            }
        }
    }

    Ok(graph)
}

/// Regression test for the duplicate-edge half of the graph-metrics defect:
/// repeated imports of the same module used to push one edge each.
#[cfg(test)]
mod dependency_graph_edge_tests {
    use super::*;

    #[tokio::test]
    async fn repeated_imports_of_one_module_produce_one_edge() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("a.rs"), "use b;\nuse b;\nuse b;\n").expect("write a");
        std::fs::write(dir.path().join("b.rs"), "pub fn x() {}\n").expect("write b");

        let graph = build_dependency_graph(dir.path(), &None, &None)
            .await
            .expect("graph");

        assert_eq!(graph.node_count(), 2);
        assert_eq!(
            graph.edge_count(),
            1,
            "three `use b` lines are one dependency, not three edges"
        );
    }
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
