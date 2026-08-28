
fn build_gitignore(root_path: &Path) -> Result<ignore::gitignore::Gitignore, TemplateError> {
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

/// Whether `path` is excluded by the project's gitignore.
///
/// The context/dag walk used the non-recursive `gitignore.matched(path, false)`.
/// A directory pattern like `.claude/worktrees/` never matches a file nested
/// inside it, so every file under an ignored directory survived the filter:
/// `analyze dag` on this repo scanned past 10000 files, hit the
/// "Large project detected" cap, and rendered nodes rooted in
/// `.claude/worktrees/...` — while `analyze defects` on the same tree saw 4260
/// files. `matched_path_or_any_parents` consults the parent directories, which
/// is what a `dir/` pattern actually means.
///
/// The path is made relative to the walk root first: `matched_path_or_any_parents`
/// asserts its argument is under the matcher root, and a relative path can
/// never trip that assert.
fn is_gitignored(
    gitignore: &ignore::gitignore::Gitignore,
    root_path: &Path,
    path: &Path,
    is_dir: bool,
) -> bool {
    let relative = path.strip_prefix(root_path).unwrap_or(path);
    if relative.as_os_str().is_empty() {
        return false; // the walk root itself is never "ignored"
    }
    gitignore
        .matched_path_or_any_parents(relative, is_dir)
        .is_ignore()
}

/// Optimized file scanner for dead code - only scans Rust source files
async fn scan_rust_files_only(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    const MAX_DEPTH: usize = 5; // Shallower search for performance
    const MAX_FILES: usize = 100; // Even lower limit for fast dead code analysis
    const BATCH_SIZE: usize = 20; // Smaller batches for responsiveness

    // Only scan Rust source files, skip tests and examples
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH)
        .into_iter()
        // Prune ignored directories instead of walking into them and
        // discarding their files one by one.
        .filter_entry(|entry| {
            !is_gitignored(
                gitignore,
                root_path,
                entry.path(),
                entry.file_type().is_dir(),
            )
        })
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return false;
            }
            // Only Rust files
            if path.extension().is_none_or(|ext| ext != "rs") {
                return false;
            }
            // Skip test and example files for dead code analysis
            let path_str = path.to_string_lossy();
            !path_str.contains("/tests/")
                && !path_str.contains("/test/")
                && !path_str.contains("/examples/")
                && !path_str.contains("/benches/")
                && !path_str.contains("_test.rs")
                && !path_str.ends_with("/build.rs")
        })
        .take(MAX_FILES)
        .map(|entry| entry.path().to_path_buf())
        .collect();

    eprintln!(
        "🎯 Dead code analysis: scanning {} Rust source files (max {})",
        paths.len(),
        MAX_FILES
    );

    let mut all_results = Vec::new();
    for chunk in paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let toolchain = toolchain.to_string();
                let cache_manager = cache_manager.clone();
                tokio::spawn(async move {
                    let timeout_duration = tokio::time::Duration::from_secs(2);
                    tokio::time::timeout(timeout_duration, async move {
                        analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
                    })
                    .await
                    .ok()
                    .flatten()
                })
            })
            .collect();

        let batch_results = join_all(batch_tasks).await;
        all_results.extend(
            batch_results
                .into_iter()
                .filter_map(std::result::Result::ok)
                .flatten(),
        );
    }

    all_results
}

async fn scan_and_analyze_files(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    // FIXED: Add depth limit and file count limit to prevent hanging
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    const MAX_FILES: usize = 10000; // Prevent resource exhaustion
    const BATCH_SIZE: usize = 100; // Process files in batches to avoid overwhelming the system

    // First, collect all file paths to analyze with limits
    let mut file_count = 0;
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH) // TDD Fix: Limit directory traversal depth
        .into_iter()
        // Prune ignored directories at the directory level: descending into
        // them is what pushed this walk past the MAX_FILES cap on repos with a
        // large gitignored tree (e.g. `.claude/worktrees/`).
        .filter_entry(|entry| {
            !is_gitignored(
                gitignore,
                root_path,
                entry.path(),
                entry.file_type().is_dir(),
            )
        })
        .filter_map(std::result::Result::ok)
        .filter(|entry| !entry.path().is_dir())
        .take(MAX_FILES) // TDD Fix: Limit total files analyzed
        .map(|entry| {
            file_count += 1;
            if file_count % 1000 == 0 {
                eprintln!("📁 Scanning files... ({file_count} so far)");
            }
            entry.path().to_path_buf()
        })
        .collect();

    if file_count > MAX_FILES / 2 {
        eprintln!(
            "⚠️ Large project detected: {file_count} files. Limited to {MAX_FILES} for performance."
        );
    }

    // Process files in controlled batches instead of all at once
    let mut all_results = Vec::new();

    for chunk in paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let toolchain = toolchain.to_string();
                let cache_manager = cache_manager.clone();
                tokio::spawn(async move {
                    // Add timeout for individual file analysis
                    let timeout_duration = tokio::time::Duration::from_secs(5);
                    tokio::time::timeout(timeout_duration, async move {
                        analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
                    })
                    .await
                    .ok()
                    .flatten()
                })
            })
            .collect();

        // Wait for this batch to complete before starting the next
        let batch_results = join_all(batch_tasks).await;
        all_results.extend(
            batch_results
                .into_iter()
                .filter_map(std::result::Result::ok)
                .flatten(),
        );
    }

    all_results
}

async fn analyze_file_by_toolchain(
    path: &Path,
    _toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
) -> Option<FileContext> {
    // FIXED: Analyze files by extension, not by toolchain
    // This enables multi-language project analysis for ALL supported languages
    let ext = path.extension().and_then(|s| s.to_str())?;

    match ext {
        // Rust files
        "rs" => analyze_rust_file_with_cache(path, cache_manager).await.ok(),

        // TypeScript/JavaScript files
        #[cfg(feature = "typescript-ast")]
        "ts" | "tsx" => ast_typescript::analyze_typescript_file(path).await.ok(),
        #[cfg(feature = "typescript-ast")]
        "js" | "jsx" | "mjs" | "cjs" => ast_typescript::analyze_javascript_file(path).await.ok(),

        // Python files
        #[cfg(feature = "python-ast")]
        "py" | "pyi" => ast_python::analyze_python_file(path).await.ok(),

        // Go files
        #[cfg(feature = "go-ast")]
        "go" => {
            use crate::services::languages::go;
            go::analyze_go_file(path).await.ok()
        }

        // NOTE: Languages below need analyze_*_file() implementations
        // See server/src/services/languages/go.rs:analyze_go_file() as reference

        // C files
        #[cfg(feature = "c-ast")]
        "c" | "h" => {
            use crate::services::ast::languages::c;
            c::analyze_c_file(path).await.ok()
        }

        // C++ files
        #[cfg(feature = "cpp-ast")]
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" | "cu" | "cuh" => {
            use crate::services::ast::languages::cpp;
            cpp::analyze_cpp_file(path).await.ok()
        }

        // Java files
        #[cfg(feature = "java-ast")]
        "java" => {
            use crate::services::deep_context;
            // Convert Vec<AstItem> to FileContext
            match deep_context::analyze_java_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "java".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // C# files
        #[cfg(feature = "csharp-ast")]
        "cs" => {
            use crate::services::deep_context;
            // Convert Vec<AstItem> to FileContext
            match deep_context::analyze_csharp_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "csharp".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Kotlin files
        #[cfg(feature = "kotlin-ast")]
        "kt" | "kts" => {
            use crate::services::languages::kotlin;
            match kotlin::analyze_kotlin_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "kotlin".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Swift files
        #[cfg(feature = "swift-ast")]
        "swift" => {
            use crate::services::deep_context;
            // Convert Vec<AstItem> to FileContext
            match deep_context::analyze_swift_file(path).await {
                Ok(items) => Some(FileContext {
                    path: path.display().to_string(),
                    language: "swift".to_string(),
                    items,
                    complexity_metrics: None,
                }),
                Err(_) => None,
            }
        }

        // Lean files
        #[cfg(feature = "lean-ast")]
        "lean" => {
            use crate::services::languages::lean;
            lean::analyze_lean_file(path).await.ok()
        }

        // Unsupported extension (Ruby, Erlang, Elixir, Haskell, OCaml, Shell, WASM pending)
        _ => None,
    }
}

async fn analyze_deno_file(path: &Path) -> Option<FileContext> {
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
        #[cfg(feature = "typescript-ast")]
        Some("ts" | "tsx") => ast_typescript::analyze_typescript_file(path).await.ok(),
        #[cfg(feature = "typescript-ast")]
        Some("js" | "jsx") => ast_typescript::analyze_javascript_file(path).await.ok(),
        _ => None,
    }
}

async fn build_project_summary(
    files: &[FileContext],
    root_path: &Path,
    toolchain: &str,
) -> ProjectSummary {
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

/// Build O(1) context graph for symbol lookups.
///
/// Extracts all functions/structs/etc from files into a trueno-graph CSR keyed by
/// symbol name, giving O(1) `get_item` lookups.
///
/// # What this does not do
///
/// **No edges are extracted, so the graph has no relationships and PageRank
/// produces nothing**: `num_edges()` is 0 and `hot_symbols()` is empty for every
/// project. `update_hotness()` is still called below, but on an edgeless CSR it
/// returns early without scoring anything. Callers get a symbol index, not a call
/// graph, and nothing here ranks symbols by importance.
///
/// The reason is that the input has no call data to extract: [`FileContext`]
/// keeps only [`AstItem`]s, and an `AstItem::Function` records a name, visibility,
/// async-ness and a line number — never its callees. Producing edges here means
/// re-parsing every source file a second time, on a path
/// (`analyze_project_with_cache`, and through it deep-context) whose cost is
/// already dominated by parsing.
///
/// Call edges therefore live where the extra parse is paid for deliberately:
/// [`crate::services::dag_call_edges::add_call_edges`] walks the Rust sources and
/// adds `EdgeType::Calls` edges to a `DependencyGraph`, resolving conservatively
/// so an ambiguous callee yields no edge. Use that for call-graph questions.
///
/// # Arguments
///
/// * `files` - Analyzed file contexts with AST items
///
/// # Returns
///
/// ProjectContextGraph containing every symbol as a node, and no edges
fn build_context_graph(
    files: &[FileContext],
) -> Result<crate::services::context_graph::ProjectContextGraph, TemplateError> {
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

    // No edge phase: see the note on this function. The graph is a symbol index.

    // PageRank over an edgeless CSR scores nothing, so this is a no-op in
    // practice; it is kept so the graph gains hotness the moment edges do exist.
    if let Err(e) = graph.update_hotness() {
        eprintln!("Warning: Failed to compute PageRank: {}", e);
    }

    Ok(graph)
}

