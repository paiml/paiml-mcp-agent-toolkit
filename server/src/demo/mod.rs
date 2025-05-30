pub mod assets;
pub mod runner;
pub mod server;
pub mod templates;

pub use runner::{detect_repository, DemoReport, DemoRunner, DemoStep};
pub use server::{DemoContent, Hotspot, LocalDemoServer};

use anyhow::Result;

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
        // CLI output mode
        let mut runner = DemoRunner::new(server);
        let report = runner.execute(repo).await?;

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
    _server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
    no_browser: bool,
    _port: Option<u16>,
) -> Result<()> {
    use std::time::Instant;

    println!("🎯 PAIML MCP Agent Toolkit Demo");
    println!("📁 Repository: {}", repo_path.display());
    println!("\n🔍 Analyzing codebase...");

    // Run analyses in parallel
    let start = Instant::now();

    // Execute all analyses
    let (context_result, complexity_result, dag_result, churn_result) = tokio::join!(
        analyze_context(&repo_path),
        analyze_complexity(&repo_path),
        analyze_dag(&repo_path),
        analyze_churn(&repo_path)
    );

    let elapsed = start.elapsed().as_millis() as u64;

    // Extract metrics and prepare demo content
    let files_analyzed = context_result.as_ref().map(|c| c.files.len()).unwrap_or(0);
    let avg_complexity = complexity_result
        .as_ref()
        .map(|c| c.summary.avg_cyclomatic as f64)
        .unwrap_or(0.0);
    let tech_debt_hours = complexity_result
        .as_ref()
        .map(|c| c.summary.technical_debt_hours as u32)
        .unwrap_or(0);

    // Get top 5 hotspots
    let hotspots = churn_result
        .as_ref()
        .map(|c| {
            c.files
                .iter()
                .take(5)
                .map(|m| Hotspot {
                    file: m.relative_path.clone(),
                    complexity: 0, // Complexity not available in churn metrics
                    churn_score: (m.churn_score * 100.0) as u32,
                })
                .collect()
        })
        .unwrap_or_default();

    // Generate Mermaid diagram from DAG
    let dag = dag_result.as_ref().map(|d| d.clone()).unwrap_or_default();

    let content = DemoContent::from_analysis_results(
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

async fn analyze_churn(
    repo_path: &std::path::Path,
) -> Result<crate::models::churn::CodeChurnAnalysis> {
    crate::services::git_analysis::GitAnalysisService::analyze_code_churn(repo_path, 30)
        .map_err(|e| anyhow::anyhow!("Error analyzing churn: {}", e))
}

#[derive(Debug, Clone)]
pub struct DemoArgs {
    pub path: Option<std::path::PathBuf>,
    pub format: crate::cli::OutputFormat,
    pub no_browser: bool,
    pub port: Option<u16>,
    pub web: bool,
}
