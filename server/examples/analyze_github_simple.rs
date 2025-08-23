//! Example: Simple GitHub repository analysis
//! 
//! This example demonstrates basic analysis of a GitHub repository.
//!
//! Usage:
//! ```bash
//! cargo run --example analyze_github_simple
//! cargo run --example analyze_github_simple -- https://github.com/serde-rs/json
//! ```

use anyhow::Result;
use pmat::demo::runner::resolve_repository_async;
use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
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
    let url = if args.len() > 1 {
        args[1].clone()
    } else {
        // Default to a small, popular repository for testing
        "https://github.com/serde-rs/json".to_string()
    };

    println!("🔍 Analyzing GitHub repository: {}", url);

    // Resolve and clone the repository
    info!("Cloning repository...");
    let repo_path = resolve_repository_async(None, Some(url.clone()), None).await?;
    info!("Repository cloned to: {:?}", repo_path);

    // Create analyzer and run analysis
    info!("Starting deep context analysis...");
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let result = analyzer.analyze_project(&repo_path).await?;

    // Output results
    println!("\n=== Analysis Results ===");
    println!("Repository: {}", url);
    
    // Project metadata
    println!("\n📁 Project Metadata:");
    println!("  Project root: {:?}", result.metadata.project_root);
    println!("  Analysis duration: {:.2}s", result.metadata.analysis_duration.as_secs_f64());
    
    // Quality scorecard
    println!("\n📊 Quality Scorecard:");
    println!("  Overall health: {:.0}%", result.quality_scorecard.overall_health);
    println!("  Complexity score: {:.0}%", result.quality_scorecard.complexity_score);
    println!("  Maintainability: {:.0}%", result.quality_scorecard.maintainability_index);
    println!("  Technical debt: {:.0} hours", result.quality_scorecard.technical_debt_hours);

    // Complexity report from analyses
    if let Some(complexity) = &result.analyses.complexity_report {
        println!("\n🔬 Complexity Analysis:");
        println!("  Files analyzed: {}", complexity.files.len());
        let total_functions: usize = complexity.files.iter()
            .map(|f| f.functions.len())
            .sum();
        println!("  Total functions: {}", total_functions);
        if !complexity.hotspots.is_empty() {
            println!("  Complexity hotspots: {}", complexity.hotspots.len());
        }
    }

    // Quality verification
    if let Some(qa) = &result.qa_verification {
        println!("\n✅ Quality Verification:");
        println!("  Overall: {:?}", qa.overall);
        println!("  Dead code: {:.1}%", qa.dead_code.actual * 100.0);
        
        if qa.dead_code.actual == 0.0 {
            if let Some(notes) = &qa.dead_code.notes {
                println!("    Note: {}", notes);
            }
        }
        
        println!("  Complexity P99: {}", qa.complexity.p99);
    }

    // SATD Analysis
    if let Some(satd) = &result.analyses.satd_results {
        println!("\n📝 Technical Debt:");
        println!("  Total SATD items: {}", satd.summary.total_items);
        println!("  Files with debt: {}", satd.files_with_debt);
    }
    
    // Defect summary
    if result.defect_summary.total_defects > 0 {
        println!("\n⚠️  Defects Found:");
        println!("  Total defects: {}", result.defect_summary.total_defects);
        println!("  Defect density: {:.2}", result.defect_summary.defect_density);
    }

    info!("Analysis complete!");
    Ok(())
}