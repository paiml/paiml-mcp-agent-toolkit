pub async fn analyze_project_with_persistent_cache(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Result<ProjectContext, TemplateError> {
    let gitignore = build_gitignore(root_path)?;
    let files =
        scan_and_analyze_files_persistent(root_path, toolchain, cache_manager, &gitignore).await;
    let summary = build_project_summary(&files, root_path, toolchain).await;

    Ok(ProjectContext {
        project_type: toolchain.to_string(),
        files,
        summary,
        graph: None,
    })
}

async fn scan_and_analyze_files_persistent(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    // FIXED: Add same depth and file limits as non-persistent version
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    const MAX_FILES: usize = 10000; // Prevent resource exhaustion

    let mut files = Vec::new();
    let mut file_count = 0;

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH) // TDD Fix: Limit directory traversal depth
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();

        // Skip if gitignored
        if gitignore.matched(path, path.is_dir()).is_ignore() {
            continue;
        }

        // TDD Fix: Limit total files analyzed
        file_count += 1;
        if file_count > MAX_FILES {
            eprintln!("⚠️ Reached file limit of {MAX_FILES}. Stopping analysis.");
            break;
        }

        if file_count % 1000 == 0 {
            eprintln!("📁 Scanning files... ({file_count} so far)");
        }

        // Add timeout for individual file analysis
        let timeout_duration = tokio::time::Duration::from_secs(5);
        let result = tokio::time::timeout(timeout_duration, async {
            analyze_file_by_toolchain_persistent(path, toolchain, cache_manager.clone()).await
        })
        .await;

        if let Ok(Some(file_context)) = result {
            files.push(file_context);
        }
    }

    files
}

async fn analyze_file_by_toolchain_persistent(
    path: &Path,
    _toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
) -> Option<FileContext> {
    // FIXED: Analyze files by extension, not by toolchain
    // This enables multi-language project analysis for ALL 20+ supported languages
    let ext = path.extension().and_then(|s| s.to_str())?;

    match ext {
        // Rust files
        "rs" => analyze_rust_file_with_persistent_cache(path, cache_manager)
            .await
            .ok(),

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
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => {
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
