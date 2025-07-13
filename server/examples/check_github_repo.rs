//! Example demonstrating GitHub repository size checking
//! 
//! This example shows how to check the size of GitHub repositories
//! before cloning them, which is useful for CI/CD systems with
//! limited disk space.

use anyhow::Result;
use pmat::services::git_clone::{GitCloner, ParsedGitHubUrl};

#[tokio::main]
async fn main() -> Result<()> {
    println!("📦 GitHub Repository Size Checker\n");
    
    // Get repository URL from command line or use defaults
    let args: Vec<String> = std::env::args().collect();
    
    let repos = if args.len() > 1 {
        // Parse command line arguments as owner/repo pairs
        args[1..]
            .iter()
            .filter_map(|arg| {
                let parts: Vec<&str> = arg.split('/').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    eprintln!("⚠️  Invalid format '{}', expected 'owner/repo'", arg);
                    None
                }
            })
            .collect()
    } else {
        // Default repositories to check
        vec![
            ("github".to_string(), "gitignore".to_string()),
            ("rust-lang".to_string(), "mdBook".to_string()),
            ("BurntSushi".to_string(), "ripgrep".to_string()),
        ]
    };
    
    if repos.is_empty() {
        println!("Usage: {} [owner/repo] [owner/repo] ...", args[0]);
        println!("Example: {} github/gitignore rust-lang/rust", args[0]);
        return Ok(());
    }
    
    let git_clone = GitCloner::new();
    let mut total_size = 0u64;
    
    println!("Checking {} repositories...\n", repos.len());
    
    for (owner, repo) in repos {
        let parsed_url = ParsedGitHubUrl {
            owner: owner.clone(),
            repo: repo.clone(),
            branch: None,
            subdir: None,
        };
        
        print!("📊 {}/{}... ", owner, repo);
        std::io::Write::flush(&mut std::io::stdout())?;
        
        match git_clone.check_repo_size(&parsed_url).await {
            Ok(size_kb) => {
                total_size += size_kb;
                let size_mb = size_kb as f64 / 1024.0;
                
                let size_str = if size_mb < 1.0 {
                    format!("{} KB", size_kb)
                } else if size_mb < 1024.0 {
                    format!("{:.1} MB", size_mb)
                } else {
                    format!("{:.1} GB", size_mb / 1024.0)
                };
                
                let emoji = match size_kb {
                    s if s < 1024 => "🟢",      // < 1 MB
                    s if s < 10240 => "🟡",     // < 10 MB
                    s if s < 102400 => "🟠",    // < 100 MB
                    _ => "🔴",                   // > 100 MB
                };
                
                println!("{} {}", emoji, size_str);
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                if e.to_string().contains("404") {
                    println!("   → Repository not found or private");
                } else if e.to_string().contains("403") {
                    println!("   → API rate limit exceeded (set GITHUB_TOKEN environment variable)");
                }
            }
        }
    }
    
    println!("\n📈 Summary:");
    println!("   Total size: {:.1} MB", total_size as f64 / 1024.0);
    
    // Provide recommendations based on total size
    if total_size > 1024 * 1024 {
        println!("   ⚠️  Warning: Total size exceeds 1 GB");
        println!("   💡 Consider cloning with --depth=1 for large repositories");
    } else if total_size > 100 * 1024 {
        println!("   💡 Tip: These repos are moderately sized, ensure adequate disk space");
    } else {
        println!("   ✅ All repositories are reasonably sized for cloning");
    }
    
    // Check if GitHub token is set
    if std::env::var("GITHUB_TOKEN").is_err() {
        println!("\n💡 Tip: Set GITHUB_TOKEN environment variable to increase API rate limits");
    }
    
    Ok(())
}