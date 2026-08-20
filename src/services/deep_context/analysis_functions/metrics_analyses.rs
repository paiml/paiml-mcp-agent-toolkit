// --- Churn analysis ---

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    let files = discovery_service.discover_files()?;
    // Skip test files — they add noise to duplicate/clone detection
    Ok(files
        .into_iter()
        .filter(|f| !crate::services::deep_context::is_test_file(f))
        .collect())
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_satd(path: &std::path::Path) -> anyhow::Result<SATDAnalysisResult> {
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();
    let result = detector.analyze_project(path, false).await?;

    Ok(result)
}

// --- Provability analysis ---

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_provability(
    path: &std::path::Path,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    analyze_provability_with_cache(path, None).await
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_provability_with_cache(
    path: &std::path::Path,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
) -> anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>> {
    analyze_provability_with_context(path, cache_manager, None).await
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    // Use owned context only when we need to call analyze_project_with_cache;
    // otherwise borrow from Arc to avoid cloning the entire ProjectContext.
    let owned_context;
    let project_context: &crate::services::context::ProjectContext =
        if let Some(ref ctx) = prebuilt_context {
            ctx.as_ref()
        } else {
            use crate::services::context::analyze_project_with_cache;
            let language = detect_project_language(path);
            match analyze_project_with_cache(path, language, cache_manager).await {
                Ok(context) => {
                    owned_context = context;
                    &owned_context
                }
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect project language.
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag(
    path: &std::path::Path,
    dag_type: DagType,
) -> anyhow::Result<DependencyGraph> {
    analyze_dag_with_cache(path, dag_type, None).await
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag_with_cache(
    path: &std::path::Path,
    dag_type: DagType,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
) -> anyhow::Result<DependencyGraph> {
    analyze_dag_with_context(path, dag_type, cache_manager, None).await
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag_with_context(
    path: &std::path::Path,
    dag_type: DagType,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
    prebuilt_context: Option<std::sync::Arc<crate::services::context::ProjectContext>>,
) -> anyhow::Result<DependencyGraph> {
    Ok(
        analyze_dag_detailed(path, dag_type, cache_manager, prebuilt_context)
            .await?
            .0,
    )
}

/// The edge types a [`DagType`] selects; `None` means "keep everything".
#[must_use]
pub fn dag_type_edge_types(dag_type: DagType) -> Option<&'static [crate::models::dag::EdgeType]> {
    use crate::models::dag::EdgeType;
    match dag_type {
        DagType::CallGraph => Some(&[EdgeType::Calls]),
        DagType::ImportGraph => Some(&[EdgeType::Imports]),
        DagType::Inheritance => Some(&[EdgeType::Inherits, EdgeType::Implements]),
        DagType::FullDependency => None,
    }
}

/// As [`analyze_dag_with_context`], but also reports what the complete graph
/// contained — so a caller that gets an empty graph can say WHY instead of
/// presenting absence as a successful measurement (#1020).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_dag_detailed(
    path: &std::path::Path,
    dag_type: DagType,
    cache_manager: Option<std::sync::Arc<crate::services::cache::SessionCacheManager>>,
    prebuilt_context: Option<std::sync::Arc<crate::services::context::ProjectContext>>,
) -> anyhow::Result<(
    DependencyGraph,
    crate::services::dag_pipeline::DagBuildStats,
)> {
    use std::time::Instant;

    info!("Starting DAG analysis for path: {:?}", path);
    let _start = Instant::now();

    // Reuse pre-built ProjectContext from AST phase if available (saves ~1 GB syn parsing)
    // Borrow from Arc to avoid cloning the entire ProjectContext.
    let owned_context;
    let project_context: &crate::services::context::ProjectContext =
        if let Some(ref ctx) = prebuilt_context {
            ctx.as_ref()
        } else {
            use crate::services::context::analyze_project_with_cache;
            let language = detect_project_language(path);
            owned_context = analyze_project_with_cache(path, language, cache_manager)
                .await
                .map_err(|e| {
                    warn!("AST analysis failed for DAG: {:?}", e);
                    anyhow::anyhow!("AST analysis failed: {}", e)
                })?;
            &owned_context
        };

    // #653 was only ever fixed on the CLI path, and #1020 showed the CLI's fix
    // never worked either: both surfaces budgeted the graph down to 400 edges
    // BEFORE extracting call edges, which deletes every function node on any
    // real tree. `build_typed_dag` owns the one correct order — complete graph,
    // enrich, select, budget last — so neither surface can drift from it again.
    let edge_types = dag_type_edge_types(dag_type);
    let (filtered_graph, stats) =
        crate::services::dag_pipeline::build_typed_dag(project_context, path, edge_types).await;

    Ok((filtered_graph, stats))
}

// --- Big-O analysis ---

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dag_service_tests {
    use super::*;

    /// The MCP `analyze_dag` tool goes through `analyze_dag_with_context`, not
    /// through the CLI handler, and only the CLI handler added call edges — so
    /// the tool reported "0 nodes, 0 edges" for a tree the CLI drew 24 edges
    /// over. A call graph with no edges over real Rust sources is a wrong
    /// answer, not an empty project.
    #[tokio::test]
    async fn call_graph_from_the_service_path_has_call_edges() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("main.rs"),
            "mod helper;\nfn main() { helper_one(); }\nfn helper_one() { helper_two(); }\nfn helper_two() {}\n",
        )
        .expect("write main.rs");

        let graph = analyze_dag(dir.path(), DagType::CallGraph)
            .await
            .expect("call-graph analysis must succeed");

        assert!(
            !graph.edges.is_empty(),
            "call graph over Rust sources must contain Calls edges, got {} nodes / {} edges",
            graph.nodes.len(),
            graph.edges.len()
        );
        assert!(
            !graph.nodes.is_empty(),
            "edges without nodes would be a filtered-away graph again"
        );
    }

    /// Write `files` modules that import each other and call round in a ring.
    ///
    /// Each module carries four `use crate::mN;` items, so the tree crosses the
    /// 400-edge Mermaid budget — the condition under which the bug appeared.
    fn write_ring_project(dir: &std::path::Path, files: usize) {
        for i in 0..files {
            let mut source = String::new();
            for target in 0..4 {
                source.push_str(&format!("use crate::m{target};\n"));
            }
            source.push_str(&format!("pub fn f{i}() {{ f{}(); }}\n", (i + 1) % files));
            std::fs::write(dir.join(format!("m{i}.rs")), source).expect("write module");
        }
    }

    /// #1020: the call graph was destroyed by SIZE, not by content.
    ///
    /// `DagBuilder::build_from_project` truncates to the 400-edge Mermaid budget
    /// and then keeps only the nodes those edges touch, which on any tree with
    /// more than 400 import edges is zero function nodes — so the call-edge pass
    /// that ran afterwards had nothing to walk. `analyze_dag {dag_type:
    /// "call-graph"}` over `src/services` answered "0 nodes, 0 edges" while
    /// `full-dependency` over the identical path answered 369/400, and a 10-file
    /// fixture (under the budget) answered 28/24 — which is why every existing
    /// test passed.
    #[tokio::test]
    async fn call_graph_survives_a_tree_bigger_than_the_edge_budget() {
        use crate::services::dag_builder::{DagBuilder, EDGE_BUDGET};

        let dir = tempfile::tempdir().expect("tempdir");
        write_ring_project(dir.path(), 140);

        // Non-vacuity guard, measured straight off the fixture so it cannot be
        // fooled by whatever the pipeline does: a tree under the budget never
        // reproduced the bug, so a shrinking fixture must fail here loudly
        // rather than let the assertion below pass for the wrong reason.
        let project = crate::services::context::analyze_project(dir.path(), "rust")
            .await
            .expect("fixture must parse");
        let complete = DagBuilder::build_from_project_unbudgeted(&project);
        assert!(
            complete.edges.len() > EDGE_BUDGET,
            "fixture must exceed the {EDGE_BUDGET}-edge budget to reproduce #1020, got {}",
            complete.edges.len()
        );

        let (graph, stats) = analyze_dag_detailed(dir.path(), DagType::CallGraph, None, None)
            .await
            .expect("call-graph analysis must succeed");

        assert!(
            !graph.edges.is_empty() && !graph.nodes.is_empty(),
            "call graph collapsed to {} nodes / {} edges on a {}-file tree ({} call edges were resolved)",
            graph.nodes.len(),
            graph.edges.len(),
            stats.files_analyzed,
            stats.call_edges
        );
        assert!(
            graph
                .edges
                .iter()
                .all(|e| e.edge_type == crate::models::dag::EdgeType::Calls),
            "a call graph must contain only Calls edges"
        );
    }

    /// An empty graph must be explainable, not silently reported as a completed
    /// measurement: absence rendered as success is the defect this release is
    /// named for.
    #[tokio::test]
    async fn an_empty_call_graph_says_why_it_is_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("notes.txt"), "no code here\n").expect("write");

        let (graph, stats) = analyze_dag_detailed(dir.path(), DagType::CallGraph, None, None)
            .await
            .expect("analysis must succeed");

        assert!(graph.nodes.is_empty());
        let reason = stats
            .explain_empty(&graph, dag_type_edge_types(DagType::CallGraph))
            .expect("an empty graph must carry a reason");
        assert!(
            reason.contains("no source file") || reason.contains("no function declarations"),
            "unhelpful reason: {reason}"
        );
    }

    /// #1020: `top_nodes[].complexity` was 1 for every node, including a
    /// function `analyze_complexity` scored 7 in the same process. The number on
    /// the node must be the number the complexity analyzer reports for that
    /// function — not a placeholder, and not a different metric wearing the name.
    #[tokio::test]
    async fn node_complexity_agrees_with_the_complexity_analyzer() {
        use crate::services::complexity::analyze_file_complexity_uncached;

        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("branchy.rs");
        std::fs::write(
            &file,
            r"
pub fn caller() {
    branchy(1, 2);
}

pub fn branchy(a: i32, b: i32) -> i32 {
    if a > 0 && b > 0 {
        return 1;
    }
    if a < 0 || b < 0 {
        return 2;
    }
    for i in 0..a {
        if i == b {
            return 3;
        }
    }
    match a {
        0 => 4,
        _ => 5,
    }
}
",
        )
        .expect("write");

        let expected = analyze_file_complexity_uncached(&file, None)
            .await
            .expect("the complexity analyzer must read the fixture")
            .functions
            .iter()
            .find(|f| f.name == "branchy")
            .expect("the complexity analyzer must see branchy")
            .metrics
            .cyclomatic;
        assert!(
            expected > 1,
            "fixture must be branchy enough to tell a measurement from the old constant"
        );

        let graph = analyze_dag(dir.path(), DagType::CallGraph)
            .await
            .expect("call-graph analysis must succeed");
        let node = graph
            .nodes
            .values()
            .find(|n| n.id.ends_with("::branchy"))
            .expect("branchy must be in the call graph");

        assert_eq!(
            node.complexity,
            u32::from(expected),
            "dag reports complexity {} for branchy while analyze_complexity reports {expected}",
            node.complexity
        );
        assert_eq!(
            node.metadata
                .get(crate::services::dag_complexity::COMPLEXITY_SOURCE_KEY)
                .map(String::as_str),
            Some(crate::services::dag_complexity::SOURCE_CYCLOMATIC)
        );
    }
}
