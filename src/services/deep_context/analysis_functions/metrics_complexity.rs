pub(super) fn detect_language(path: &std::path::Path) -> String {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            // Core languages with full support
            "rs" => "rust".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "js" | "jsx" | "mjs" | "cjs" => "javascript".to_string(),
            "py" | "pyi" => "python".to_string(),
            "go" => "go".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp".to_string(),

            // JVM languages
            "java" => "java".to_string(),
            "kt" | "kts" => "kotlin".to_string(),

            // .NET languages
            "cs" => "csharp".to_string(),

            // Scripting languages
            "sh" | "bash" => "bash".to_string(),
            "rb" => "ruby".to_string(),
            "lua" => "lua".to_string(),

            // Functional languages
            "ex" | "exs" => "elixir".to_string(),
            "erl" | "hrl" => "erlang".to_string(),
            "hs" | "lhs" => "haskell".to_string(),
            "ml" | "mli" => "ocaml".to_string(),

            // Apple ecosystem
            "swift" => "swift".to_string(),

            // WebAssembly
            "wat" | "wasm" => "wasm".to_string(),

            // Proof assistants
            "lean" => "lean".to_string(),

            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}

// --- Complexity analysis ---

pub async fn analyze_complexity(path: &std::path::Path) -> anyhow::Result<ComplexityReport> {
    use crate::services::complexity::aggregate_results;

    info!("Starting complexity analysis for path: {:?}", path);

    // Extract Method: Discover source files
    let source_files = discover_source_files_for_complexity(path)?;
    info!(
        "Discovered {} source files for complexity analysis",
        source_files.len()
    );

    // Extract Method: Analyze all files
    let file_metrics = analyze_files_complexity(source_files).await;
    info!(
        "Complexity analysis completed. Analyzed {} files",
        file_metrics.len()
    );

    // Aggregate results into final report
    Ok(aggregate_results(file_metrics))
}

fn discover_source_files_for_complexity(
    path: &std::path::Path,
) -> anyhow::Result<Vec<std::path::PathBuf>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(5_000), // Reasonable limit for complexity analysis
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);
    discovery.discover_files()
}

async fn analyze_files_complexity(
    source_files: Vec<std::path::PathBuf>,
) -> Vec<crate::services::complexity::FileComplexityMetrics> {
    // Parallelize complexity analysis using futures for better performance
    use futures::stream::{self, StreamExt};

    stream::iter(source_files)
        .map(|file_path| async move { analyze_single_file_complexity(&file_path).await })
        .buffer_unordered(num_cpus::get()) // Process multiple files concurrently
        .filter_map(|opt| async move { opt })
        .collect::<Vec<_>>()
        .await
}

#[allow(clippy::manual_map)] // Complex async + feature-gate pattern, not easily simplified
async fn analyze_single_file_complexity(
    file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    let ext = file_path.extension()?.to_str()?;

    match ext {
        "rs" => {
            // OPTIMIZATION: Check unified cache first (from analyze_rust_language)
            // This avoids the second parse that analyze_rust_file_with_complexity would do
            let cached = RUST_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned());

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path if not in cache (shouldn't happen in normal flow)
                use crate::services::ast_rust::analyze_rust_file_with_complexity;
                analyze_rust_file_with_complexity(file_path).await.ok()
            }
        }
        "ts" | "js" | "jsx" | "tsx" => {
            // OPTIMIZATION: Check TypeScript unified cache first (from analyze_typescript_language)
            // This avoids the second parse that analyze_typescript_file_with_complexity would do
            let cached =
                TYPESCRIPT_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned());

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path if not in cache (shouldn't happen in normal flow)
                #[cfg(feature = "typescript-ast")]
                {
                    use crate::services::ast_typescript::analyze_typescript_file_with_complexity;
                    analyze_typescript_file_with_complexity(file_path)
                        .await
                        .ok()
                }
                #[cfg(not(feature = "typescript-ast"))]
                None
            }
        }
        "py" => {
            // OPTIMIZATION: Check Python unified cache first (from analyze_python_language)
            // This avoids the second parse that analyze_python_file_with_complexity would do
            let cached = PYTHON_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned());

            if let Some(metrics) = cached {
                Some(metrics)
            } else {
                // Fallback to old path if not in cache (shouldn't happen in normal flow)
                #[cfg(feature = "python-ast")]
                {
                    use crate::services::ast_python::analyze_python_file_with_complexity;
                    analyze_python_file_with_complexity(file_path, None)
                        .await
                        .ok()
                }
                #[cfg(not(feature = "python-ast"))]
                None
            }
        }
        "go" => {
            // OPTIMIZATION: Check Go unified cache first (from analyze_go_language)
            // This avoids the second parse - TICKET-3004
            GO_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned())
        }
        "wat" | "wasm" => {
            // OPTIMIZATION: Check WASM unified cache first (from analyze_wasm_language)
            // This avoids the second parse - TICKET-3005
            WASM_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned())
        }
        "sh" | "bash" => {
            // OPTIMIZATION: Check Bash unified cache first (from analyze_bash_language)
            // This avoids the second parse - TICKET-3006
            BASH_UNIFIED_CACHE.with(|cache| cache.borrow().get(file_path).cloned())
        }
        "lua" => analyze_lua_complexity_metrics(file_path).await,
        _ => None,
    }
}

/// Lua complexity metrics: tree-sitter for totals, regex for function names
#[cfg(feature = "lua-ast")]
#[allow(clippy::cast_possible_truncation)]
async fn analyze_lua_complexity_metrics(
    file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    use crate::ast::languages::lua::LuaStrategy;
    use crate::ast::languages::LanguageStrategy;
    use crate::services::complexity::{
        ComplexityMetrics as CMetrics, FileComplexityMetrics as FCMetrics,
    };

    let content = tokio::fs::read_to_string(file_path).await.ok()?;
    let strategy = LuaStrategy::new();
    let ast = strategy.parse_file(file_path, &content).await.ok()?;
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);

    let mut func_complexities = extract_lua_function_complexities(&content);
    let func_count = func_complexities.len().max(1) as u32;
    for fc in &mut func_complexities {
        fc.metrics.cyclomatic = (cyclomatic / func_count) as u16;
        fc.metrics.cognitive = (cognitive / func_count) as u16;
    }

    Some(FCMetrics {
        path: file_path.to_string_lossy().to_string(),
        total_complexity: CMetrics {
            cyclomatic: cyclomatic as u16,
            cognitive: cognitive as u16,
            nesting_max: 0,
            lines: content.lines().count() as u16,
            halstead: None,
        },
        functions: func_complexities,
        classes: vec![],
    })
}

#[cfg(not(feature = "lua-ast"))]
async fn analyze_lua_complexity_metrics(
    _file_path: &std::path::Path,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    None
}

/// Extract Lua function names and line numbers using regex patterns
#[allow(clippy::cast_possible_truncation)]
fn extract_lua_function_complexities(
    content: &str,
) -> Vec<crate::services::complexity::FunctionComplexity> {
    use crate::services::complexity::{ComplexityMetrics as CMetrics, FunctionComplexity as FComp};

    let patterns = [
        r"(?m)^\s*function\s+(\w+(?:[.:]\w+)*)\s*\(",
        r"(?m)^\s*local\s+function\s+(\w+)\s*\(",
    ];
    let mut funcs = Vec::new();
    for pat in &patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            for cap in re.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let line = content.get(..m.start()).unwrap_or_default().lines().count() as u32;
                    funcs.push(FComp {
                        name: m.as_str().to_string(),
                        line_start: line,
                        line_end: 0,
                        metrics: CMetrics {
                            cyclomatic: 0,
                            cognitive: 0,
                            nesting_max: 0,
                            lines: 0,
                            halstead: None,
                        },
                    });
                }
            }
        }
    }
    funcs.sort_by_key(|f| f.line_start);
    funcs
}
