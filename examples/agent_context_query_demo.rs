//! Agent Context Query Demo - RAG-Powered Semantic Code Search
//!
//! Demonstrates PMAT's agent context system that indexes functions with
//! quality metadata (TDG, complexity, SATD, Big-O) and enables semantic
//! search via `pmat query`.
//!
//! Unlike grep, `pmat query` understands intent and returns quality-ranked
//! results with full signatures, documentation, and metrics.
//!
//! # Run
//! ```bash
//! cargo run --example agent_context_query_demo
//! ```

use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  PMAT Agent Context Demo (RAG-Powered Semantic Code Search)");
    println!("================================================================\n");

    let pmat = find_pmat_binary()?;
    let project_dir = std::env::current_dir()?;

    // Demo 1: Basic semantic search
    println!("----------------------------------------------------------------");
    println!("  Demo 1: Semantic Search - Find by Intent");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"error handling\" --limit 5\n");

    let output = Command::new(&pmat)
        .args(["query", "error handling", "--limit", "5"])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 2: Quality-filtered search
    println!("\n----------------------------------------------------------------");
    println!("  Demo 2: Quality-Filtered Search (TDG grade + complexity)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"complexity analysis\" --min-grade A --max-complexity 10 --limit 5\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "complexity analysis",
            "--min-grade",
            "A",
            "--max-complexity",
            "10",
            "--limit",
            "5",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 3: JSON output for scripting/CI
    println!("\n----------------------------------------------------------------");
    println!("  Demo 3: JSON Output (for CI/CD and scripting)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"MCP tool\" --format json --limit 3\n");

    let output = Command::new(&pmat)
        .args(["query", "MCP tool", "--format", "json", "--limit", "3"])
        .current_dir(&project_dir)
        .output()?;

    // Pretty-print first 40 lines of JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(40) {
        println!("{}", line);
    }
    if stdout.lines().count() > 40 {
        println!("... (output truncated)");
    }

    // Demo 4: Markdown output for documentation
    println!("\n----------------------------------------------------------------");
    println!("  Demo 4: Markdown Output (for docs and reports)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"TDG scoring\" --format markdown --limit 3\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "TDG scoring",
            "--format",
            "markdown",
            "--limit",
            "3",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 5: Language-filtered search
    println!("\n----------------------------------------------------------------");
    println!("  Demo 5: Language-Filtered Search");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"validation\" --language rust --limit 5\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "validation",
            "--language",
            "rust",
            "--limit",
            "5",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Summary: grep vs pmat query comparison
    println!("\n================================================================");
    println!("  Why pmat query > grep for agents");
    println!("================================================================\n");

    println!("  grep -r \"error\" src/ | head -5");
    println!("  -> 500+ irrelevant matches, no context, no quality info\n");

    println!("  pmat query \"error handling\" --min-grade B --limit 5");
    println!("  -> 5 quality-ranked results with:");
    println!("     - Full function signatures");
    println!("     - TDG grades and complexity scores");
    println!("     - Big-O estimates");
    println!("     - Documentation strings");
    println!("     - Relevance scores\n");

    println!("  Agent context is also available via MCP tools:");
    println!("  - pmat_query_code: Semantic search by intent");
    println!("  - pmat_get_function: Get full function with metrics");
    println!("  - pmat_find_similar: Find similar functions");
    println!("  - pmat_index_stats: Index health and statistics\n");

    println!("Done.");
    Ok(())
}

fn print_output(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Skip index-building lines, show results
    for line in stdout.lines() {
        if !line.starts_with("Building index") && !line.starts_with("Loading index") {
            println!("{}", line);
        }
    }
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprintln!("stderr: {}", stderr);
        }
    }
}

fn find_pmat_binary() -> Result<String, Box<dyn std::error::Error>> {
    // Prefer installed pmat (has latest features) over local builds
    if let Ok(output) = Command::new("pmat").arg("--version").output() {
        if output.status.success() {
            return Ok("pmat".to_string());
        }
    }

    // Fall back to local builds
    let release_path = Path::new("target/release/pmat");
    if release_path.exists() {
        return Ok(release_path.to_string_lossy().to_string());
    }

    let debug_path = Path::new("target/debug/pmat");
    if debug_path.exists() {
        return Ok(debug_path.to_string_lossy().to_string());
    }

    Err("pmat binary not found. Run 'cargo install --path .' first.".into())
}
