// Project summary building, context graph construction, item counting,
// and gitignore configuration.

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn build_gitignore(root_path: &Path) -> Result<ignore::gitignore::Gitignore, TemplateError> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    let mut gitignore = GitignoreBuilder::new(root_path);

    // Add default ignores
    let default_ignores = [".git", "target", "node_modules", ".venv", "__pycache__"];
    for pattern in &default_ignores {
        gitignore.add_line(None, pattern).ok();
    }

    if let Ok(gi_path) = root_path.join(".gitignore").canonicalize() {
        gitignore.add(&gi_path);
    }

    gitignore
        .build()
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn build_project_summary(
    files: &[FileContext],
    root_path: &Path,
    toolchain: &str,
) -> ProjectSummary {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    debug_assert!(!toolchain.is_empty(), "toolchain must not be empty");
    debug_assert!(!files.is_empty(), "files must not be empty");
    let mut summary = ProjectSummary {
        total_files: files.len(),
        total_functions: 0,
        total_structs: 0,
        total_enums: 0,
        total_traits: 0,
        total_impls: 0,
        dependencies: Vec::new(),
    };

    // Calculate item counts
    calculate_item_counts(&mut summary, files);

    // Read dependencies
    summary.dependencies = read_dependencies(root_path, toolchain).await;

    summary
}

/// Build O(1) context graph for symbol lookups and PageRank
///
/// Extracts all functions/structs/etc from files and builds a trueno-graph CSR
/// for O(1) symbol lookups and PageRank-based importance scoring.
fn build_context_graph(
    files: &[FileContext],
) -> Result<crate::services::context_graph::ProjectContextGraph, TemplateError> {
    debug_assert!(!files.is_empty(), "files must not be empty");
    use crate::services::context_graph::ProjectContextGraph;

    let mut graph = ProjectContextGraph::new();

    // Phase 1: Add all symbols as nodes
    for file in files {
        for item in &file.items {
            let symbol_name = item.display_name();

            // Skip if already added (duplicates from multiple files)
            if graph.get_item(symbol_name).is_some() {
                continue;
            }

            // Add node to graph
            if let Err(e) = graph.add_item(symbol_name.to_string(), item.clone()) {
                eprintln!("Warning: Failed to add item to graph: {}", e);
            }
        }
    }

    // Phase 2: Edge extraction deferred; graph provides O(1) node lookups

    // Phase 3: Run PageRank to identify "hot" symbols
    if let Err(e) = graph.update_hotness() {
        eprintln!("Warning: Failed to compute PageRank: {}", e);
    }

    Ok(graph)
}

fn calculate_item_counts(summary: &mut ProjectSummary, files: &[FileContext]) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    for file in files {
        for item in &file.items {
            match item {
                AstItem::Function { .. } => summary.total_functions += 1,
                AstItem::Struct { .. } => summary.total_structs += 1,
                AstItem::Enum { .. } => summary.total_enums += 1,
                AstItem::Trait { .. } => summary.total_traits += 1,
                AstItem::Impl { .. } => summary.total_impls += 1,
                _ => {}
            }
        }
    }
}
