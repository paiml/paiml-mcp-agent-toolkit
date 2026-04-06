#![cfg_attr(coverage_nightly, coverage(off))]
//! Web/HTTP and TUI demo functions.

use anyhow::Result;
use tracing::{debug, info};

use super::processing::extract_analysis_from_demo_report;
use super::runner::DemoRunner;
use super::server::{DemoContent, Hotspot, LocalDemoServer};

/// Open URL in default browser using platform-specific command
/// Replaces webbrowser crate to reduce transitive dependencies
#[allow(dead_code)] // Used only when "demo" feature is enabled
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }
    Ok(())
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn run_web_demo(
    repo_path: std::path::PathBuf,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
    no_browser: bool,
    _port: Option<u16>,
) -> Result<()> {
    use std::time::Instant;

    let version = env!("CARGO_PKG_VERSION");
    println!("🎯 PAIML MCP Agent Toolkit Demo v{version}");
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
        .map_or(demo_report.steps.len() * 10, |c| c.files.len()); // Better fallback based on actual analysis
    let avg_complexity = complexity_result
        .as_ref()
        .map_or(2.5, |c| f64::from(c.summary.median_cyclomatic)); // More realistic fallback
    let tech_debt_hours = complexity_result
        .as_ref()
        .map_or((files_analyzed / 10) as u32, |c| {
            c.summary.technical_debt_hours as u32
        }); // Estimate based on file count

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
                        complexity: u32::from(func.metrics.cyclomatic),
                        churn_score: u32::from(func.metrics.cognitive), // Use cognitive as churn score for display
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

    // Start web server with actual analysis results
    let (_demo_server, port) = LocalDemoServer::spawn_with_results(
        content,
        complexity_result,
        None, // churn_result not extracted from demo report yet
        dag_result,
    )
    .await?;
    let url = format!("http://127.0.0.1:{port}");

    println!("\n📊 Demo server running at: {url}");
    println!("   Analysis completed in {elapsed} ms");

    // Open browser unless disabled
    #[cfg(feature = "demo")]
    if !no_browser {
        if let Err(e) = open_browser(&url) {
            println!("   Please open {url} in your browser (auto-open failed: {e})");
        }
    }

    #[cfg(not(feature = "demo"))]
    let _ = no_browser; // Avoid unused variable warning when demo is disabled

    println!("\nPress Ctrl+C to stop the demo server");

    // Keep server running
    tokio::signal::ctrl_c().await?;
    println!("\n👋 Shutting down demo server...");

    Ok(())
}

/// TUI demo runner function
#[cfg(feature = "tui")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn run_tui_demo(repo_path: std::path::PathBuf) -> Result<()> {
    use super::adapters::tui::TuiDemoAdapter;

    println!("📺 Starting TUI Demo for: {}", repo_path.display());

    let mut adapter = TuiDemoAdapter::new()
        .map_err(|e| anyhow::anyhow!("Failed to create TUI adapter: {}", e))?;

    // Initialize terminal
    adapter
        .initialize()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize TUI: {}", e))?;

    // Start analysis
    let analyze_request = crate::demo::adapters::tui::TuiRequest {
        action: "analyze".to_string(),
        params: {
            let mut params = std::collections::HashMap::new();
            params.insert(
                "path".to_string(),
                serde_json::Value::String(repo_path.to_string_lossy().into_owned()),
            );
            params
        },
    };

    let _response = adapter
        .handle_request(analyze_request)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start analysis: {}", e))?;

    // Run the main event loop
    adapter
        .run_event_loop()
        .await
        .map_err(|e| anyhow::anyhow!("TUI event loop failed: {}", e))?;

    Ok(())
}
