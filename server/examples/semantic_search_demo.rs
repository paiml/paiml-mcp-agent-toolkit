//! Semantic Search Demo - Pure Rust Implementation
//!
//! Demonstrates PMAT's semantic search capabilities using:
//! - trueno-rag: Hybrid retrieval with RRF fusion
//! - trueno-graph: PageRank-based code importance
//! - aprender: TF-IDF, LDA topic modeling, clustering
//!
//! Zero API keys required - works completely offline.
//!
//! # Run
//! ```bash
//! cargo run --example semantic_search_demo
//! ```

use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     PMAT Semantic Search Demo (Pure Rust - Zero API Keys)     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let pmat = find_pmat_binary()?;
    let test_dir = std::env::current_dir()?;

    // Demo 1: Topic Extraction with LDA
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Demo 1: Topic Extraction (LDA - Latent Dirichlet Allocation)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Command: pmat analyze topics --num-topics 5\n");

    let output = Command::new(&pmat)
        .args(["analyze", "topics", "--num-topics", "5"])
        .current_dir(&test_dir)
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        eprintln!("Topics extraction failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Demo 2: K-means Clustering
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Demo 2: Code Clustering (K-means)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Command: pmat analyze cluster --method kmeans --k 5\n");

    let output = Command::new(&pmat)
        .args(["analyze", "cluster", "--method", "kmeans", "--k", "5"])
        .current_dir(&test_dir)
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Demo 3: DBSCAN Clustering (density-based)
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Demo 3: Density-Based Clustering (DBSCAN)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Command: pmat analyze cluster --method dbscan\n");

    let output = Command::new(&pmat)
        .args(["analyze", "cluster", "--method", "dbscan"])
        .current_dir(&test_dir)
        .output()?;

    // Only show first 30 lines for DBSCAN (can produce many clusters)
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(30) {
        println!("{}", line);
    }
    if stdout.lines().count() > 30 {
        println!("... (output truncated)");
    }

    // Demo 4: Hierarchical Clustering
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Demo 4: Hierarchical Clustering (Agglomerative)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Command: pmat analyze cluster --method hierarchical --k 5\n");

    let output = Command::new(&pmat)
        .args(["analyze", "cluster", "--method", "hierarchical", "--k", "5"])
        .current_dir(&test_dir)
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Demo 5: JSON Output for CI/CD
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Demo 5: JSON Output (for CI/CD Integration)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Command: pmat analyze topics --num-topics 3 --format json\n");

    let output = Command::new(&pmat)
        .args(["analyze", "topics", "--num-topics", "3", "--format", "json"])
        .current_dir(&test_dir)
        .output()?;

    // Pretty print first 50 lines of JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(50) {
        println!("{}", line);
    }

    // Summary
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Summary: Pure Rust Semantic Search Stack");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Component        │ Algorithm                │ Citation     │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│ aprender         │ TF-IDF Vectorization     │ Manning 2008 │");
    println!("│ aprender         │ LDA Topic Modeling       │ Blei 2003    │");
    println!("│ aprender         │ K-means Clustering       │ MacQueen 1967│");
    println!("│ aprender         │ DBSCAN Clustering        │ Ester 1996   │");
    println!("│ trueno-rag       │ BM25 Sparse Retrieval    │ Robertson 09 │");
    println!("│ trueno-rag       │ RRF Fusion               │ Cormack 2009 │");
    println!("│ trueno-graph     │ PageRank Importance      │ Page 1999    │");
    println!("│ trueno-graph     │ Louvain Clustering       │ Blondel 2008 │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n✅ All demos completed - Zero API keys used!");
    println!("📖 See: docs/specifications/semantic-search-feature.md\n");

    Ok(())
}

fn find_pmat_binary() -> Result<String, Box<dyn std::error::Error>> {
    // Try release binary first
    let release_path = Path::new("target/release/pmat");
    if release_path.exists() {
        return Ok(release_path.to_string_lossy().to_string());
    }

    // Try debug binary
    let debug_path = Path::new("target/debug/pmat");
    if debug_path.exists() {
        return Ok(debug_path.to_string_lossy().to_string());
    }

    // Try system PATH
    if Command::new("pmat").arg("--version").output().is_ok() {
        return Ok("pmat".to_string());
    }

    Err("pmat binary not found. Run 'cargo build --release' first.".into())
}
