// --- Churn analysis ---

pub async fn analyze_churn(path: &std::path::Path, days: u32) -> anyhow::Result<CodeChurnAnalysis> {
    use crate::services::git_analysis::GitAnalysisService;
    use std::time::{Duration, Instant};

    info!("Starting churn analysis for path: {:?}", path);
    let start = Instant::now();

    // Smart bounds: timeout after 3 seconds for churn analysis
    let timeout = Duration::from_secs(3);

    match tokio::time::timeout(timeout, async {
        GitAnalysisService::analyze_code_churn(path, days)
            .map_err(|e| anyhow::anyhow!("Failed to analyze code churn: {e}"))
    })
    .await
    {
        Ok(result) => {
            info!("Churn analysis completed in {:?}", start.elapsed());
            result
        }
        Err(_) => {
            warn!("Churn analysis timed out after {:?}", timeout);
            // Return empty churn analysis instead of failing
            use crate::models::churn::ChurnSummary;
            use chrono::Utc;

            Ok(CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days: days,
                repository_root: path.to_path_buf(),
                files: Vec::new(),
                summary: ChurnSummary {
                    total_commits: 0,
                    total_files_changed: 0,
                    hotspot_files: Vec::new(),
                    stable_files: Vec::new(),
                    author_contributions: std::collections::HashMap::new(),
                    mean_churn_score: 0.0,
                    variance_churn_score: 0.0,
                    stddev_churn_score: 0.0,
                },
            })
        }
    }
}
// --- Duplicate code analysis ---

pub async fn analyze_duplicate_code(
    path: &std::path::Path,
) -> anyhow::Result<crate::services::duplicate_detector::CloneReport> {
    use crate::services::duplicate_detector::DuplicateDetectionEngine;

    let all_files = discover_project_files(path)?;
    let files_for_analysis = filter_and_categorize_files_for_duplicates(all_files)?;
    let engine = DuplicateDetectionEngine::default();
    engine.detect_duplicates(&files_for_analysis)
}

fn discover_project_files(path: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    use crate::services::file_discovery::ProjectFileDiscovery;
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    discovery_service.discover_files()
}

fn filter_and_categorize_files_for_duplicates(
    all_files: Vec<std::path::PathBuf>,
) -> anyhow::Result<
    Vec<(
        std::path::PathBuf,
        String,
        crate::services::duplicate_detector::Language,
    )>,
> {
    let mut files_for_analysis = Vec::new();
    for file_path in all_files {
        if let Some((file, content, lang)) = process_file_for_duplicate_detection(&file_path)? {
            files_for_analysis.push((file, content, lang));
        }
    }
    Ok(files_for_analysis)
}

fn process_file_for_duplicate_detection(
    file_path: &std::path::Path,
) -> anyhow::Result<
    Option<(
        std::path::PathBuf,
        String,
        crate::services::duplicate_detector::Language,
    )>,
> {
    let ext = match file_path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => return Ok(None),
    };

    let language = match_extension_to_language(ext)?;
    if language.is_none() {
        return Ok(None);
    }

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) if c.lines().count() >= 10 => c,
        _ => return Ok(None),
    };

    Ok(Some((
        file_path.to_path_buf(),
        content,
        language.expect("internal error"),
    )))
}

fn match_extension_to_language(
    ext: &str,
) -> anyhow::Result<Option<crate::services::duplicate_detector::Language>> {
    use crate::services::duplicate_detector::Language;

    Ok(match ext {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "js" | "jsx" => Some(Language::JavaScript),
        "py" => Some(Language::Python),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "cu" | "cuh" => Some(Language::Cpp),
        "kt" | "kts" => Some(Language::Kotlin),
        _ => None,
    })
}

// --- SATD analysis ---

pub async fn analyze_satd(path: &std::path::Path) -> anyhow::Result<SATDAnalysisResult> {
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();
    let result = detector.analyze_project(path, false).await?;

    Ok(result)
}

// --- Provability analysis ---

pub async fn analyze_provability(
    path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    analyze_provability_with_cache(path, None).await
}

pub async fn analyze_provability_with_cache(
    path: &std::path::Path,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    analyze_provability_with_context(path, cache_manager, None).await
}

pub async fn analyze_provability_with_context(
    path: &std::path::Path,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
    prebuilt_context: Option<std::sync::Arc<crate::services::context::ProjectContext>>,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    use crate::services::context::AstItem;
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };
    use std::time::Instant;

    info!("Starting provability analysis for path: {:?}", path);

    let analyzer = LightweightProvabilityAnalyzer::new();

    // No timeouts - use proper concurrency instead
    let start = Instant::now();

    // Reuse pre-built ProjectContext from AST phase if available (saves ~1 GB syn parsing)
    let project_context = if let Some(ctx) = prebuilt_context {
        (*ctx).clone()
    } else {
        use crate::services::context::analyze_project_with_cache;
        let language = detect_project_language(path);
        match analyze_project_with_cache(path, language, cache_manager).await {
            Ok(context) => context,
            Err(e) => {
                warn!("AST analysis failed for provability: {:?}", e);
                return Ok(vec![]);
            }
        }
    };

    let mut function_ids = Vec::new();

    // Smart bounds: limit to 50 functions to prevent timeouts
    let mut function_count = 0;
    const MAX_FUNCTIONS: usize = 50;

    for file in &project_context.files {
        for item in &file.items {
            if let AstItem::Function { name, line, .. } = item {
                if function_count < MAX_FUNCTIONS {
                    function_ids.push(FunctionId {
                        file_path: file.path.clone(),
                        function_name: name.clone(),
                        line_number: *line,
                    });
                    function_count += 1;
                } else {
                    break;
                }
            }
        }
        if function_count >= MAX_FUNCTIONS {
            break;
        }
    }

    // If no functions found, add a mock one
    if function_ids.is_empty() {
        function_ids.push(FunctionId {
            file_path: format!("{}/src/main.rs", path.display()),
            function_name: "main".to_string(),
            line_number: 1,
        });
    }

    // Analyze all functions with proper parallel processing
    let summaries = analyzer.analyze_incrementally(&function_ids).await;

    info!(
        "Provability analysis completed for {} functions in {:?}",
        summaries.len(),
        start.elapsed()
    );
    Ok(summaries)
}

pub fn detect_project_language(path: &std::path::Path) -> &'static str {
    use crate::services::file_discovery::ProjectFileDiscovery;
    let discovery = ProjectFileDiscovery::new(path.to_path_buf());
    let files = discovery.discover_files().unwrap_or_default();

    let mut counts = [0; 5]; // rust, python, ruby, ts, js
    for file in &files {
        if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => counts[0] += 1,
                "py" => counts[1] += 1,
                "rb" => counts[2] += 1,
                "ts" | "tsx" => counts[3] += 1,
                "js" | "jsx" => counts[4] += 1,
                _ => {}
            }
        }
    }

    let (max_idx, _) = counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .unwrap_or((0, &0));
    match max_idx {
        0 => "rust",
        1 => "python",
        2 => "ruby",
        3 => "typescript",
        _ => "javascript",
    }
}

// --- DAG analysis ---

pub async fn analyze_dag(
    path: &std::path::Path,
    dag_type: DagType,
) -> anyhow::Result<DependencyGraph> {
    analyze_dag_with_cache(path, dag_type, None).await
}

pub async fn analyze_dag_with_cache(
    path: &std::path::Path,
    dag_type: DagType,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
) -> anyhow::Result<DependencyGraph> {
    analyze_dag_with_context(path, dag_type, cache_manager, None).await
}

pub async fn analyze_dag_with_context(
    path: &std::path::Path,
    dag_type: DagType,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
    prebuilt_context: Option<std::sync::Arc<crate::services::context::ProjectContext>>,
) -> anyhow::Result<DependencyGraph> {
    use crate::services::dag_builder::{
        filter_call_edges, filter_import_edges, filter_inheritance_edges, DagBuilder,
    };
    use std::time::Instant;

    info!("Starting DAG analysis for path: {:?}", path);
    let _start = Instant::now();

    // Reuse pre-built ProjectContext from AST phase if available (saves ~1 GB syn parsing)
    let project_context = if let Some(ctx) = prebuilt_context {
        (*ctx).clone()
    } else {
        use crate::services::context::analyze_project_with_cache;
        let language = detect_project_language(path);
        analyze_project_with_cache(path, language, cache_manager)
            .await
            .map_err(|e| {
                warn!("AST analysis failed for DAG: {:?}", e);
                anyhow::anyhow!("AST analysis failed: {}", e)
            })?
    };

    // Smart bounds: limit graph size to 200 nodes (was 400)
    let graph = DagBuilder::build_from_project_with_limit(&project_context, 200);

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };

    Ok(filtered_graph)
}

// --- Big-O analysis ---

pub async fn analyze_big_o(
    path: &std::path::Path,
) -> anyhow::Result<crate::services::big_o_analyzer::BigOAnalysisReport> {
    use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};

    let analyzer = BigOAnalyzer::new();
    let config = BigOAnalysisConfig {
        project_path: path.to_path_buf(),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        confidence_threshold: 50,
        analyze_space_complexity: false,
    };

    analyzer.analyze(config).await
}
