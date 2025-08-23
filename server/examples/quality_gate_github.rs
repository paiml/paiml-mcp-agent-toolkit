//! Example: Run quality gates on GitHub repositories
//! 
//! This example demonstrates running quality gate checks on GitHub repositories
//! to ensure they meet specified quality standards.
//!
//! Usage:
//! ```bash
//! cargo run --example quality_gate_github
//! cargo run --example quality_gate_github -- https://github.com/owner/repo
//! ```

use anyhow::Result;
use pmat::demo::runner::{resolve_repository_async, DemoRunner};
use pmat::stateless_server::StatelessTemplateServer;
use std::sync::Arc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;


#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Get repository URL from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let url = if args.len() > 1 {
        args[1].clone()
    } else {
        "https://github.com/rust-lang/rust-clippy".to_string()
    };

    println!("🔍 Running quality gates on: {}\n", url);

    // Clone and analyze repository
    info!("Cloning repository...");
    let repo_path = resolve_repository_async(None, Some(url.clone()), None).await?;
    
    info!("Running analysis...");
    let server = Arc::new(StatelessTemplateServer::new()?);
    let mut runner = DemoRunner::new(server);
    let report = runner.execute(repo_path).await?;
    let result = &report.analysis;

    // Check if we have QA verification results
    if let Some(qa_status) = &result.qa_verification {
        println!("📋 Quality Gate Results:");
        println!("{:-<60}", "");
        
        // Overall status
        let overall_icon = if qa_status == "PASSED" {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        println!("Overall Status: {}\n", overall_icon);
        
        println!("📊 Analysis Summary:");
        println!("  Files Analyzed: {}", result.files_analyzed);
        println!("  Functions Analyzed: {}", result.functions_analyzed);
        println!("  Average Complexity: {:.1}", result.avg_complexity);
        println!("  Hotspot Functions: {}", result.hotspot_functions);
        println!("  Quality Score: {:.1}%", result.quality_score * 100.0);
        println!("  Technical Debt: {} hours", result.tech_debt_hours);
        
        if qa_status != "PASSED" {
            println!("\n💡 Recommendations:");
            println!("  • Review functions with high complexity");
            println!("  • Consider refactoring hotspot functions");
            println!("  • Add comprehensive test coverage");
            
            warn!("Repository needs improvement");
            std::process::exit(1);
        } else {
            println!("\n✅ Repository passes quality gates!");
            std::process::exit(0);
        }
    } else {
        println!("❌ No quality verification results available");
        println!("   This may indicate an analysis failure");
        std::process::exit(3);
    }
}