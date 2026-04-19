// Complexity analysis handlers (extracted from extended_tools.rs for CB-040)

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeComplexityArgs {
    project_path: Option<String>,
    toolchain: Option<String>,
    format: Option<String>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Option<Vec<String>>,
    top_files: Option<usize>,
}

fn parse_complexity_args(arguments: serde_json::Value) -> Result<AnalyzeComplexityArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_complexity arguments: {e}"))
}

struct ComplexityAnalysisContext {
    project_path: PathBuf,
    toolchain: String,
    _thresholds: crate::services::complexity::ComplexityThresholds,
}

/// R22-1 / D101: project_path is required. `resolve_project_path_complexity`
/// rejects null/missing/empty values.
/// R22-2 / D102: resolve glob patterns in project_path before toolchain
/// detection so that `src/**` or `crates/*/src/**` produce a concrete
/// directory instead of the "authoritative zeros" pre-R21-4 behavior.
fn prepare_complexity_analysis(
    args: &AnalyzeComplexityArgs,
) -> Result<ComplexityAnalysisContext, String> {
    let project_path = resolve_project_path_complexity(args.project_path.clone())?;
    let toolchain = detect_toolchain(&args.toolchain, &project_path);
    let thresholds = build_complexity_thresholds(args);

    Ok(ComplexityAnalysisContext {
        project_path,
        toolchain,
        _thresholds: thresholds,
    })
}

async fn perform_complexity_analysis(
    context: &ComplexityAnalysisContext,
    args: &AnalyzeComplexityArgs,
) -> (crate::services::complexity::ComplexityReport, usize) {
    use crate::services::complexity::aggregate_results;

    let (file_metrics, file_count) =
        analyze_project_files(&context.project_path, &context.toolchain, args).await;

    let report = aggregate_results(file_metrics);
    (report, file_count)
}

fn generate_complexity_content(
    report: &crate::services::complexity::ComplexityReport,
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    args: &AnalyzeComplexityArgs,
) -> String {
    if let Some(top_files_count) = args.top_files {
        if top_files_count > 0 {
            generate_ranked_content(file_metrics, top_files_count, args)
        } else {
            format_complexity_output(report, args)
        }
    } else {
        format_complexity_output(report, args)
    }
}

fn generate_ranked_content(
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    top_files_count: usize,
    args: &AnalyzeComplexityArgs,
) -> String {
    use crate::services::ranking::{rank_files_by_complexity, ComplexityRanker};

    let ranker = ComplexityRanker::default();
    let rankings = rank_files_by_complexity(file_metrics, top_files_count, &ranker);
    format_complexity_rankings(&rankings, args)
}

fn build_complexity_response(
    request_id: serde_json::Value,
    content_text: String,
    report: &crate::services::complexity::ComplexityReport,
    toolchain: &str,
    file_count: usize,
    args: &AnalyzeComplexityArgs,
) -> McpResponse {
    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "report": report,
        "toolchain": toolchain,
        "files_analyzed": file_count,
        "format": args.format.as_deref().unwrap_or("summary"),
        "top_files": args.top_files,
    });

    McpResponse::success(request_id, result)
}

async fn handle_analyze_complexity(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_complexity_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    // R22-1 / D101 + R22-2 / D102: validate project_path (reject null/
    // missing/empty) then glob-expand; fail loud with `-32602` if empty glob.
    let context = match prepare_complexity_analysis(&args) {
        Ok(ctx) => ctx,
        Err(msg) => return McpResponse::error(request_id, -32602, msg),
    };

    info!(
        "Analyzing complexity for {:?} using {} toolchain",
        context.project_path, context.toolchain
    );

    let (file_metrics, file_count) =
        analyze_project_files(&context.project_path, &context.toolchain, &args).await;

    let report = crate::services::complexity::aggregate_results(file_metrics.clone());
    let content_text = generate_complexity_content(&report, &file_metrics, &args);

    build_complexity_response(
        request_id,
        content_text,
        &report,
        &context.toolchain,
        file_count,
        &args,
    )
}

/// R22-1 / D101 + R22-2 / D102: Validate and glob-expand `project_path`.
///
/// D101: reject null/missing/empty values to avoid silently analyzing the
/// server's cwd.
/// D102: expand shell-style globs via the shared `services::path_glob`
/// helper, failing loud on empty expansion.
fn resolve_project_path_complexity(project_path_arg: Option<String>) -> Result<PathBuf, String> {
    let _validated = require_project_path(project_path_arg.clone())?;
    let raw = project_path_arg
        .as_deref()
        .expect("require_project_path returned Ok for None");
    match resolve_project_path_with_globs(raw) {
        ResolvedProjectPath::Concrete(p) => Ok(p),
        e @ ResolvedProjectPath::EmptyGlob(_) => Err(e.into_error_message()),
    }
}


fn detect_toolchain(toolchain_arg: &Option<String>, project_path: &Path) -> String {
    if let Some(t) = toolchain_arg {
        t.clone()
    } else if project_path.join("Cargo.toml").exists() {
        "rust".to_string()
    } else if project_path.join("package.json").exists() || project_path.join("deno.json").exists()
    {
        "deno".to_string()
    } else if project_path.join("pyproject.toml").exists()
        || project_path.join("requirements.txt").exists()
    {
        "python-uv".to_string()
    } else {
        "rust".to_string() // default
    }
}

fn build_complexity_thresholds(
    args: &AnalyzeComplexityArgs,
) -> crate::services::complexity::ComplexityThresholds {
    use crate::services::complexity::ComplexityThresholds;

    let mut thresholds = ComplexityThresholds::default();
    if let Some(max) = args.max_cyclomatic {
        thresholds.cyclomatic_error = max;
        thresholds.cyclomatic_warn = (max * 3 / 4).max(1);
    }
    if let Some(max) = args.max_cognitive {
        thresholds.cognitive_error = max;
        thresholds.cognitive_warn = (max * 3 / 4).max(1);
    }
    thresholds
}

async fn analyze_project_files(
    project_path: &Path,
    toolchain: &str,
    args: &AnalyzeComplexityArgs,
) -> (
    Vec<crate::services::complexity::FileComplexityMetrics>,
    usize,
) {
    use crate::services::file_discovery::ProjectFileDiscovery;

    let mut file_metrics = Vec::with_capacity(256);
    let mut file_count = 0;

    // Use ProjectFileDiscovery which properly respects .gitignore files
    let discovery = ProjectFileDiscovery::new(project_path.to_path_buf());
    let discovered_files = match discovery.discover_files() {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to discover files: {}", e);
            return (file_metrics, file_count);
        }
    };

    for path in discovered_files {
        if path.is_dir() || !should_analyze_file(&path, toolchain) {
            continue;
        }

        if !matches_include_filters(&path, &args.include) {
            continue;
        }

        file_count += 1;

        if let Some(metrics) = analyze_file_complexity(&path, toolchain).await {
            file_metrics.push(metrics);
        }
    }

    (file_metrics, file_count)
}

fn should_analyze_file(path: &Path, toolchain: &str) -> bool {
    match toolchain {
        "rust" => path.extension().and_then(|s| s.to_str()) == Some("rs"),
        "deno" => matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("ts" | "tsx" | "js" | "jsx")
        ),
        "python-uv" => path.extension().and_then(|s| s.to_str()) == Some("py"),
        _ => false,
    }
}

fn matches_include_filters(path: &Path, include_patterns: &Option<Vec<String>>) -> bool {
    let Some(ref patterns) = include_patterns else {
        return true;
    };

    if patterns.is_empty() {
        return true;
    }

    let path_str = path.to_string_lossy();
    patterns
        .iter()
        .any(|pattern| matches_pattern(&path_str, pattern))
}

fn matches_pattern(path_str: &str, pattern: &str) -> bool {
    if pattern.contains("**") {
        // Match any path containing the pattern after **
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            path_str.contains(parts[1].trim_start_matches('/'))
        } else {
            false
        }
    } else if pattern.starts_with("*.") {
        // Match by extension
        path_str.ends_with(&pattern[1..])
    } else {
        // Direct substring match
        path_str.contains(pattern)
    }
}

async fn analyze_file_complexity(
    path: &Path,
    toolchain: &str,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    match toolchain {
        "rust" => {
            use crate::services::ast_rust;
            ast_rust::analyze_rust_file_with_complexity(path).await.ok()
        }
        "deno" => {
            #[cfg(feature = "typescript-ast")]
            {
                use crate::services::ast_typescript;
                ast_typescript::analyze_typescript_file_with_complexity(path)
                    .await
                    .ok()
            }
            #[cfg(not(feature = "typescript-ast"))]
            None
        }
        "python-uv" => {
            #[cfg(feature = "python-ast")]
            {
                use crate::services::ast_python;
                ast_python::analyze_python_file_with_complexity(path, None)
                    .await
                    .ok()
            }
            #[cfg(not(feature = "python-ast"))]
            None
        }
        _ => None,
    }
}

fn format_complexity_output(
    report: &crate::services::complexity::ComplexityReport,
    args: &AnalyzeComplexityArgs,
) -> String {
    use crate::services::complexity::{
        format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    let format = args.format.as_deref().unwrap_or("summary");
    match format {
        "full" => format_complexity_report(report),
        "json" => serde_json::to_string_pretty(report).unwrap_or_default(),
        "sarif" => match format_as_sarif(report) {
            Ok(sarif) => sarif,
            Err(_) => "Error generating SARIF format".to_string(),
        },
        _ => format_complexity_summary(report), // default to summary
    }
}

fn format_complexity_rankings(
    rankings: &[(String, crate::services::ranking::CompositeComplexityScore)],
    args: &AnalyzeComplexityArgs,
) -> String {
    use crate::services::ranking::{ComplexityRanker, FileRanker};

    let format = args.format.as_deref().unwrap_or("summary");
    if format == "json" {
        let ranker = ComplexityRanker::default();
        let rankings_json = serde_json::json!({
            "analysis_type": ranker.ranking_type(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "top_files": {
                "requested": rankings.len(),
                "returned": rankings.len()
            },
            "rankings": rankings.iter().enumerate().map(|(i, (file, score))| {
                serde_json::json!({
                    "rank": i + 1,
                    "file": file,
                    "metrics": {
                        "functions": score.function_count,
                        "max_cyclomatic": score.cyclomatic_max,
                        "avg_cognitive": score.cognitive_avg,
                        "halstead_effort": score.halstead_effort,
                        "total_score": score.total_score
                    }
                })
            }).collect::<Vec<_>>()
        });
        serde_json::to_string_pretty(&rankings_json).unwrap_or_default()
    } else {
        // Table format (default)
        let mut output = String::with_capacity(1024);
        output.push_str(&format!("## Top {} Complexity Files\n\n", rankings.len()));
        output.push_str("| Rank | File                               | Functions | Max Cyclomatic | Avg Cognitive | Halstead | Score |\n");
        output.push_str("|------|------------------------------------|-----------|--------------  |---------------|----------|-------|\n");

        for (i, (file, score)) in rankings.iter().enumerate() {
            output.push_str(&format!(
                "| {:>4} | {:<50} | {:>9} | {:>14} | {:>13.1} | {:>11.1} | {:>11.1} |\n",
                i + 1,
                file,
                score.function_count,
                score.cyclomatic_max,
                score.cognitive_avg,
                score.halstead_effort,
                score.total_score
            ));
        }
        output.push('\n');
        output
    }
}
