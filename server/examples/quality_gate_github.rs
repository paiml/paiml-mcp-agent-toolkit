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
use pmat::services::quality_gates::{QAVerification, VerificationStatus};
use pmat::stateless_server::StatelessTemplateServer;
use std::sync::Arc;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

fn print_gate_result(name: &str, status: VerificationStatus, details: &str) {
    let icon = match status {
        VerificationStatus::Pass => "✅",
        VerificationStatus::Fail => "❌",
        VerificationStatus::Partial => "⚠️",
    };
    println!("  {} {} - {}", icon, name, details);
}

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
    if let Some(qa) = &result.qa_verification {
        println!("📋 Quality Gate Results:");
        println!("{:-<60}", "");
        
        // Overall status
        let overall_icon = match qa.overall {
            VerificationStatus::Pass => "✅ PASS",
            VerificationStatus::Fail => "❌ FAIL",
            VerificationStatus::Partial => "⚠️  PARTIAL",
        };
        println!("Overall Status: {}\n", overall_icon);
        
        // Dead code check
        print_gate_result(
            "Dead Code",
            qa.dead_code.status.clone(),
            &format!("{:.1}% (expected: {:.1}%-{:.1}%)",
                    qa.dead_code.actual * 100.0,
                    qa.dead_code.expected_range[0] * 100.0,
                    qa.dead_code.expected_range[1] * 100.0)
        );
        if let Some(notes) = &qa.dead_code.notes {
            println!("    Note: {}", notes);
        }
        
        // Complexity check
        print_gate_result(
            "Complexity",
            qa.complexity.status.clone(),
            &format!("P99: {} (entropy: {:.2}, CV: {:.2})",
                    qa.complexity.p99,
                    qa.complexity.entropy,
                    qa.complexity.cv)
        );
        if let Some(notes) = &qa.complexity.notes {
            println!("    Note: {}", notes);
        }
        
        // Provability check
        print_gate_result(
            "Provability",
            qa.provability.status.clone(),
            &format!("Coverage: {:.1}%, Invariants: {}",
                    qa.provability.pure_reducer_coverage * 100.0,
                    qa.provability.state_invariants_tested)
        );
        if let Some(notes) = &qa.provability.notes {
            println!("    Note: {}", notes);
        }
        
        println!("{:-<60}", "");
        
        // Recommendations based on failures
        if qa.overall != VerificationStatus::Pass {
            println!("\n💡 Recommendations:");
            
            if qa.dead_code.status == VerificationStatus::Fail {
                println!("  • Run dead code elimination: pmat refactor dead-code");
                println!("  • Review and remove unused functions and modules");
            }
            
            if qa.complexity.status == VerificationStatus::Fail {
                println!("  • Refactor complex functions: pmat refactor auto --complexity");
                println!("  • Target functions with cyclomatic complexity > 20");
            }
            
            if qa.provability.status == VerificationStatus::Fail {
                println!("  • Add property-based tests for core logic");
                println!("  • Ensure state invariants are tested");
            }
        }
        
        // Exit with appropriate code
        match qa.overall {
            VerificationStatus::Pass => {
                println!("\n✅ Repository passes all quality gates!");
                std::process::exit(0);
            }
            VerificationStatus::Partial => {
                warn!("Repository partially passes quality gates");
                std::process::exit(1);
            }
            VerificationStatus::Fail => {
                warn!("Repository fails quality gates");
                std::process::exit(2);
            }
        }
    } else {
        println!("❌ No quality verification results available");
        println!("   This may indicate an analysis failure");
        std::process::exit(3);
    }
}