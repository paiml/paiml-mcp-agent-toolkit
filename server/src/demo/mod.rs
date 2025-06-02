pub mod assets;
pub mod cli_renderer;
pub mod engine;
pub mod runner;
pub mod server;
pub mod templates;

pub use engine::{DemoEngine, DemoAnalysis, DemoSource, ProgressEvent};
pub use runner::{detect_repository, DemoReport, DemoRunner, DemoStep};
pub use server::{DemoContent, Hotspot, LocalDemoServer};

use anyhow::Result;
use tracing::{debug, info};

pub async fn run_demo(
    args: DemoArgs,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
) -> Result<()> {
    // Determine source: local path, remote URL, or current directory
    let source = if let Some(url) = args.url.as_ref() {
        DemoSource::Remote(url.clone())
    } else {
        let repo = detect_repository(args.path)?;
        DemoSource::Local(repo)
    };

    if args.web {
        // Web server mode with unified engine
        run_unified_web_demo(source, server, args.no_browser, args.port).await
    } else {
        // CLI output mode with unified engine
        run_unified_cli_demo(source, args.format).await
    }
}

// New unified CLI demo mode
async fn run_unified_cli_demo(source: DemoSource, format: crate::cli::OutputFormat) -> Result<()> {
    use crate::demo::cli_renderer::render_demo_cli;
    
    let engine = DemoEngine::new();
    let analysis = engine.analyze(source).await?;
    
    match format {
        crate::cli::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        crate::cli::OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&analysis)?);
        }
        crate::cli::OutputFormat::Table => {
            println!("{}", render_demo_cli(&analysis)?);
        }
    }
    
    Ok(())
}

// New unified web demo mode  
async fn run_unified_web_demo(
    source: DemoSource,
    _server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
    no_browser: bool,
    _port: Option<u16>,
) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!("🎯 PAIML MCP Agent Toolkit Demo v{}", version);
    
    // Display source information
    match &source {
        DemoSource::Local(path) => {
            println!("📁 Repository: {}", path.display());
        }
        DemoSource::Remote(url) => {
            println!("🌐 Cloning: {}", url);
        }
        DemoSource::Cached(key) => {
            println!("📦 Cached: {}", key);
        }
    }
    
    println!("\n🔍 Analyzing codebase...");
    info!("Starting unified demo analysis");
    
    let engine = DemoEngine::new();
    let analysis = engine.analyze(source).await?;
    
    // Convert to legacy DemoContent format for web server compatibility
    let content = convert_analysis_to_demo_content(&analysis);
    
    // Start web server
    let (_demo_server, server_port) = LocalDemoServer::spawn(content).await?;
    let url = format!("http://127.0.0.1:{}", server_port);
    
    println!("\n📊 Demo server running at: {}", url);
    println!("   Analysis completed in {:.2}s", analysis.timings.total_duration.as_secs_f64());
    
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

// Convert DemoAnalysis to legacy DemoContent for web compatibility
fn convert_analysis_to_demo_content(analysis: &DemoAnalysis) -> DemoContent {
    
    // Convert hotspots to legacy format
    let hotspots: Vec<Hotspot> = analysis.metrics.hotspots
        .iter()
        .map(|h| Hotspot {
            file: h.file.clone(),
            complexity: h.complexity,
            churn_score: h.churn,
        })
        .collect();
    
    // Use actual DAG or create default
    let dag = analysis.metrics.dag.clone().unwrap_or_default();
    
    let mut content = DemoContent::from_analysis_results(
        &dag,
        analysis.repository.file_count,
        analysis.metrics.complexity.as_ref()
            .map(|c| c.summary.median_cyclomatic as f64)
            .unwrap_or(1.0),
        analysis.metrics.complexity.as_ref()
            .map(|c| c.summary.technical_debt_hours as u32)
            .unwrap_or(0),
        hotspots,
        analysis.timings.ast_analysis.as_millis() as u64,
        analysis.timings.complexity_analysis.as_millis() as u64,
        analysis.timings.dag_analysis.as_millis() as u64,
        analysis.timings.churn_analysis.as_millis() as u64,
    );
    
    // Add system diagram from analysis
    content.system_diagram = Some(analysis.visualization.mermaid.clone());
    
    content
}

// Extract actual analysis results and timings from demo report
#[allow(dead_code)]
fn extract_analysis_from_demo_report(
    demo_report: &crate::demo::DemoReport,
) -> (
    Option<crate::services::complexity::ComplexityReport>,
    Option<crate::models::dag::DependencyGraph>,
    (u64, u64, u64, u64), // timings: (ast, complexity, dag, churn)
) {
    let mut complexity_result = None;
    let mut dag_result = None;
    let mut timings = (0u64, 0u64, 0u64, 0u64);

    for step in &demo_report.steps {
        match step.capability {
            "AST Context Analysis" => timings.0 = step.elapsed_ms,
            "Code Complexity Analysis" => {
                timings.1 = step.elapsed_ms;
                // Try to extract complexity data from step response
                if let Some(result) = &step.response.result {
                    if let Ok(complexity_data) =
                        serde_json::from_value::<serde_json::Value>(result.clone())
                    {
                        // Parse complexity summary if available
                        if let Some(summary) = complexity_data.get("summary") {
                            complexity_result = parse_complexity_summary(summary);
                        }
                    }
                }
            }
            "DAG Visualization" => {
                timings.2 = step.elapsed_ms;
                // Try to extract DAG data from step response
                if let Some(result) = &step.response.result {
                    if let Ok(dag_data) =
                        serde_json::from_value::<serde_json::Value>(result.clone())
                    {
                        dag_result = parse_dag_data(&dag_data);
                    }
                }
            }
            "Code Churn Analysis" => timings.3 = step.elapsed_ms,
            _ => {}
        }
    }

    (complexity_result, dag_result, timings)
}

#[allow(dead_code)]
fn parse_complexity_summary(
    summary: &serde_json::Value,
) -> Option<crate::services::complexity::ComplexityReport> {
    // Create a minimal complexity report from summary data
    let total_files = summary.get("total_files")?.as_u64().unwrap_or(0) as usize;
    let total_functions = summary.get("total_functions")?.as_u64().unwrap_or(0) as usize;
    let avg_cyclomatic = summary.get("avg_cyclomatic")?.as_f64().unwrap_or(0.0);
    let tech_debt_hours = summary.get("technical_debt_hours")?.as_f64().unwrap_or(0.0);

    Some(crate::services::complexity::ComplexityReport {
        summary: crate::services::complexity::ComplexitySummary {
            total_files,
            total_functions,
            median_cyclomatic: avg_cyclomatic as f32,
            median_cognitive: (avg_cyclomatic * 1.2) as f32, // Estimate
            max_cyclomatic: (avg_cyclomatic * 3.0) as u16,
            max_cognitive: (avg_cyclomatic * 3.5) as u16,
            p90_cyclomatic: (avg_cyclomatic * 2.0) as u16,
            p90_cognitive: (avg_cyclomatic * 2.4) as u16,
            technical_debt_hours: tech_debt_hours as f32,
        },
        violations: vec![],
        hotspots: vec![],
        files: vec![], // Would need more complex parsing to populate
    })
}

#[allow(dead_code)]
fn parse_dag_data(dag_data: &serde_json::Value) -> Option<crate::models::dag::DependencyGraph> {
    // Try to extract basic graph structure
    if let Some(stats) = dag_data.get("stats") {
        let node_count = stats.get("nodes")?.as_u64().unwrap_or(0) as usize;
        let edge_count = stats.get("edges")?.as_u64().unwrap_or(0) as usize;

        // Create a minimal graph structure
        if node_count > 0 || edge_count > 0 {
            return Some(crate::models::dag::DependencyGraph {
                nodes: (0..node_count)
                    .map(|i| {
                        let node_id = format!("node_{}", i);
                        (
                            node_id.clone(),
                            crate::models::dag::NodeInfo {
                                id: node_id,
                                label: format!("Module {}", i),
                                node_type: crate::models::dag::NodeType::Module,
                                file_path: format!("module_{}.rs", i),
                                line_number: 1,
                                complexity: 1,
                            },
                        )
                    })
                    .collect(),
                edges: (0..edge_count)
                    .map(|i| crate::models::dag::Edge {
                        from: format!("node_{}", i % node_count),
                        to: format!("node_{}", (i + 1) % node_count),
                        edge_type: crate::models::dag::EdgeType::Imports,
                        weight: 1,
                    })
                    .collect(),
            });
        }
    }
    None
}

#[allow(dead_code)]
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

    // Extract metrics directly from demo report steps instead of re-analyzing
    let (complexity_result, dag_result, actual_timings) =
        extract_analysis_from_demo_report(&demo_report);

    let files_analyzed = complexity_result
        .as_ref()
        .map(|c| c.files.len())
        .unwrap_or(demo_report.steps.len() * 10); // Better fallback based on actual analysis
    let avg_complexity = complexity_result
        .as_ref()
        .map(|c| c.summary.median_cyclomatic as f64)
        .unwrap_or(2.5); // More realistic fallback
    let tech_debt_hours = complexity_result
        .as_ref()
        .map(|c| c.summary.technical_debt_hours as u32)
        .unwrap_or((files_analyzed / 10) as u32); // Estimate based on file count

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
        actual_timings.0, // Use actual demo execution timings
        actual_timings.1,
        actual_timings.2,
        actual_timings.3,
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
