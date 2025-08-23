//! Example: Analyze a GitHub repository
//! 
//! This example demonstrates how to analyze a GitHub repository using pmat.
//! It clones the repository, performs analysis, and outputs results.
//!
//! Usage:
//! ```bash
//! cargo run --example analyze_github_repo
//! cargo run --example analyze_github_repo -- --url https://github.com/rust-lang/rust-clippy
//! ```

use anyhow::Result;
use pmat::demo::runner::{resolve_repository_async, DemoRunner};
use pmat::stateless_server::StatelessTemplateServer;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let url = if args.len() > 2 && args[1] == "--url" {
        args[2].clone()
    } else {
        // Default to a small, popular repository for testing
        "https://github.com/serde-rs/json".to_string()
    };

    info!("Analyzing GitHub repository: {}", url);

    // Resolve and clone the repository
    let repo_path = resolve_repository_async(None, Some(url.clone()), None).await?;
    info!("Repository cloned to: {:?}", repo_path);

    // Create server and runner
    let server = Arc::new(StatelessTemplateServer::new()?);
    let mut runner = DemoRunner::new(server);

    // Run the demo analysis
    info!("Starting analysis...");
    let report = runner.execute(repo_path).await?;
    let result = &report.analysis;

    // Output results
    println!("\n=== Analysis Results ===");
    println!("Repository: {}", url);
    
    println!("\n📊 Complexity Metrics:");
    println!("  Files analyzed: {}", result.files_analyzed);
    println!("  Total functions: {}", result.functions_analyzed);
    println!("  Average complexity: {:.2}", result.avg_complexity);
    println!("  Hotspot functions: {}", result.hotspot_functions);
    println!("  Quality score: {:.2}", result.quality_score);

    if let Some(lang_stats) = &result.language_stats {
        println!("\n🌳 Language Summary:");
        for (lang, stats) in lang_stats {
            println!("  {}: {} files", lang, stats.get("file_count").unwrap_or(&serde_json::Value::Null));
        }
    }

    if let Some(qa_status) = &result.qa_verification {
        println!("\n✅ Quality Verification:");
        println!("  Status: {}", qa_status);
        println!("  Technical Debt: {} hours", result.tech_debt_hours);
    }

    info!("Analysis complete!");
    
    // Print execution time
    println!("\n⏱️  Execution time: {:.2}s", report.execution_time_ms as f64 / 1000.0);
    
    Ok(())
}