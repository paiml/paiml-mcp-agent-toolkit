pub mod assets;
pub mod runner;
pub mod server;
pub mod templates;

pub use runner::{detect_repository, DemoReport, DemoRunner, DemoStep};
pub use server::{DemoContent, Hotspot, LocalDemoServer};

use anyhow::Result;
use tracing::{debug, info};

pub async fn run_demo(
    args: DemoArgs,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
) -> Result<()> {
    use crate::cli::{ExecutionMode, OutputFormat};

    let repo = detect_repository(args.path)?;

    if args.web {
        // Web server mode
        run_web_demo(repo, server, args.no_browser, args.port).await
    } else {
        // CLI output mode with enhanced demo capabilities
        let mut runner = DemoRunner::new(server);

        // Use the new execute_with_diagram method that supports URL and graph reduction
        let report = if args.url.is_some() {
            runner
                .execute_with_diagram(&repo, args.url.as_deref())
                .await?
        } else {
            runner.execute_with_diagram(&repo, None).await?
        };

        let output = match args.format {
            OutputFormat::Table | OutputFormat::Yaml => report.render(ExecutionMode::Cli),
            OutputFormat::Json => report.render(ExecutionMode::Mcp),
        };

        println!("{}", output);
        Ok(())
    }
}

async fn run_web_demo(
    repo_path: std::path::PathBuf,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
    no_browser: bool,
    _port: Option<u16>,
) -> Result<()> {
    use std::time::Instant;

    let version = env!("CARGO_PKG_VERSION");
    println!("🎯 PAIML MCP Agent Toolkit Demo v{}", version);
    println!("📁 Repository: {}", repo_path.display());
    println!("\n🔍 Analyzing codebase...");
    info!("Starting codebase analysis");

    // Use DemoRunner to get full analysis including system diagram
    let start = Instant::now();
    debug!("Starting demo runner analysis");

    let mut demo_runner = DemoRunner::new(server);
    let demo_report = demo_runner.execute_with_diagram(&repo_path, None).await?;

    let elapsed = start.elapsed().as_millis() as u64;
    info!(elapsed_ms = elapsed, "Analysis completed");

    // Extract metrics from demo report
    let complexity_result = analyze_complexity(&repo_path).await.ok();
    let dag_result = analyze_dag(&repo_path).await.ok();

    let files_analyzed = complexity_result
        .as_ref()
        .map(|c| c.files.len())
        .unwrap_or(50);
    let avg_complexity = complexity_result
        .as_ref()
        .map(|c| c.summary.avg_cyclomatic as f64)
        .unwrap_or(0.0);
    let tech_debt_hours = complexity_result
        .as_ref()
        .map(|c| c.summary.technical_debt_hours as u32)
        .unwrap_or(0);

    // Get actual complexity hotspots instead of churn
    let hotspots = complexity_result
        .as_ref()
        .map(|c| {
            let mut all_functions: Vec<_> = c
                .files
                .iter()
                .flat_map(|file| {
                    file.functions.iter().map(move |func| Hotspot {
                        file: format!("{}::{}", file.path, func.name),
                        complexity: func.metrics.cyclomatic as u32,
                        churn_score: func.metrics.cognitive as u32, // Use cognitive as churn score for display
                    })
                })
                .collect();

            // Sort by complexity and take top 10
            all_functions.sort_by(|a, b| b.complexity.cmp(&a.complexity));
            all_functions.truncate(10);
            all_functions
        })
        .unwrap_or_default();

    // Generate Mermaid diagram from DAG
    let dag = dag_result.clone().unwrap_or_default();

    let mut content = DemoContent::from_analysis_results(
        &dag,
        files_analyzed,
        avg_complexity,
        tech_debt_hours,
        hotspots,
        100, // Placeholder timing values
        150,
        200,
        250,
    );

    // IMPORTANT: Add the system diagram from demo_report
    content.system_diagram = demo_report.system_diagram;

    // Start web server
    let (_demo_server, port) = LocalDemoServer::spawn(content).await?;
    let url = format!("http://127.0.0.1:{}", port);

    println!("\n📊 Demo server running at: {}", url);
    println!("   Analysis completed in {} ms", elapsed);

    // Open browser unless disabled
    #[cfg(not(feature = "no-demo"))]
    if !no_browser {
        if let Err(e) = webbrowser::open(&url) {
            println!(
                "   Please open {} in your browser (auto-open failed: {})",
                url, e
            );
        }
    }

    #[cfg(feature = "no-demo")]
    let _ = no_browser; // Avoid unused variable warning when demo is disabled

    println!("\nPress Ctrl+C to stop the demo server");

    // Keep server running
    tokio::signal::ctrl_c().await?;
    println!("\n👋 Shutting down demo server...");

    Ok(())
}

// Helper functions for web demo analyses
#[allow(dead_code)]
async fn analyze_context(
    repo_path: &std::path::Path,
) -> Result<crate::services::context::ProjectContext> {
    crate::services::context::analyze_project(repo_path, "rust")
        .await
        .map_err(|e| anyhow::anyhow!("Error analyzing project: {}", e))
}

async fn analyze_complexity(
    repo_path: &std::path::Path,
) -> Result<crate::services::complexity::ComplexityReport> {
    use crate::services::ast_rust::analyze_rust_file_with_complexity;
    use crate::services::complexity::aggregate_results;
    use walkdir::WalkDir;

    let mut file_metrics = Vec::new();

    for entry in WalkDir::new(repo_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(metrics) = analyze_rust_file_with_complexity(path).await {
                file_metrics.push(metrics);
            }
        }
    }

    Ok(aggregate_results(file_metrics))
}

async fn analyze_dag(repo_path: &std::path::Path) -> Result<crate::models::dag::DependencyGraph> {
    use crate::services::dag_builder::DagBuilder;

    let context = crate::services::context::analyze_project(repo_path, "rust")
        .await
        .map_err(|e| anyhow::anyhow!("Error analyzing project: {}", e))?;
    let graph = DagBuilder::build_from_project(&context);

    Ok(graph)
}

#[allow(dead_code)]
async fn analyze_churn(
    repo_path: &std::path::Path,
) -> Result<crate::models::churn::CodeChurnAnalysis> {
    crate::services::git_analysis::GitAnalysisService::analyze_code_churn(repo_path, 30)
        .map_err(|e| anyhow::anyhow!("Error analyzing churn: {}", e))
}

#[allow(dead_code)]
async fn analyze_system_architecture(
    repo_path: &std::path::Path,
) -> Result<crate::services::canonical_query::QueryResult> {
    use crate::services::canonical_query::{
        AnalysisContext, CallGraph, CanonicalQuery, SystemArchitectureQuery,
    };
    use std::collections::HashMap;

    // Build analysis context
    let _context_result = analyze_context(repo_path).await?;
    let dag_result = analyze_dag(repo_path).await?;
    let complexity_result = analyze_complexity(repo_path).await?;
    let churn_result = analyze_churn(repo_path).await.ok(); // Optional

    // Convert complexity report to map
    let mut complexity_map = HashMap::new();
    for file in &complexity_result.files {
        for function in &file.functions {
            complexity_map.insert(function.name.clone(), function.metrics);
        }
    }

    let context = AnalysisContext {
        project_path: repo_path.to_path_buf(),
        ast_dag: dag_result,
        call_graph: CallGraph::default(), // TODO: Build actual call graph
        complexity_map,
        churn_analysis: churn_result,
    };

    let query = SystemArchitectureQuery;
    query
        .execute(&context)
        .map_err(|e| anyhow::anyhow!("Error analyzing architecture: {}", e))
}

#[allow(dead_code)]
async fn analyze_defect_probability(
    repo_path: &std::path::Path,
) -> Result<crate::services::defect_probability::ProjectDefectAnalysis> {
    use crate::services::defect_probability::{
        DefectProbabilityCalculator, FileMetrics, ProjectDefectAnalysis,
    };
    use walkdir::WalkDir;

    let calculator = DefectProbabilityCalculator::new();
    let mut file_metrics = Vec::new();

    // Get complexity and churn data
    let complexity_result = analyze_complexity(repo_path).await?;
    let churn_result = analyze_churn(repo_path).await.ok();

    // Build churn map for quick lookup
    let churn_map: std::collections::HashMap<String, f32> = churn_result
        .as_ref()
        .map(|churn| {
            churn
                .files
                .iter()
                .map(|f| (f.relative_path.clone(), f.churn_score))
                .collect()
        })
        .unwrap_or_default();

    // Analyze each Rust file
    for entry in WalkDir::new(repo_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let relative_path = path
                .strip_prefix(repo_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Find complexity data for this file
            if let Some(file_complexity) = complexity_result
                .files
                .iter()
                .find(|f| f.path == relative_path)
            {
                let churn_score = churn_map.get(&relative_path).copied().unwrap_or(0.0);

                // Aggregate complexity from all functions in file
                let total_complexity: f32 = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.cyclomatic as f32)
                    .sum();
                let avg_complexity = if !file_complexity.functions.is_empty() {
                    total_complexity / file_complexity.functions.len() as f32
                } else {
                    1.0
                };

                let max_cyclomatic = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.cyclomatic)
                    .max()
                    .unwrap_or(1);

                let max_cognitive = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.cognitive)
                    .max()
                    .unwrap_or(1);

                let total_loc: usize = file_complexity
                    .functions
                    .iter()
                    .map(|f| f.metrics.lines as usize)
                    .sum();

                let metrics = FileMetrics {
                    file_path: relative_path,
                    churn_score,
                    complexity: avg_complexity,
                    duplicate_ratio: 0.0, // TODO: Implement duplication detection
                    afferent_coupling: 0.0, // TODO: Implement coupling analysis
                    efferent_coupling: 0.0,
                    lines_of_code: total_loc,
                    cyclomatic_complexity: max_cyclomatic as u32,
                    cognitive_complexity: max_cognitive as u32,
                };

                file_metrics.push(metrics);
            }
        }
    }

    let scores = calculator.calculate_batch(&file_metrics);
    Ok(ProjectDefectAnalysis::from_scores(scores))
}

#[derive(Debug, Clone)]
pub struct DemoArgs {
    pub path: Option<std::path::PathBuf>,
    pub url: Option<String>,
    pub format: crate::cli::OutputFormat,
    pub no_browser: bool,
    pub port: Option<u16>,
    pub web: bool,
    pub target_nodes: usize,
    pub centrality_threshold: f64,
    pub merge_threshold: usize,
}
