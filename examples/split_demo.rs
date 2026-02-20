//! File Split Demo — Semantic file splitting via Louvain community detection
//!
//! Demonstrates PMAT's `pmat split` command that analyzes function call graphs
//! within a file, detects communities using Louvain, and suggests semantic splits
//! with meaningful cluster names.
//!
//! # Run
//! ```bash
//! cargo run --example split_demo
//! ```

use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  PMAT Split Demo (Semantic File Splitting)");
    println!("================================================================\n");

    let pmat = find_pmat_binary()?;
    let project_dir = std::env::current_dir()?;

    // Demo 1: Analyze a file for splitting (dry-run, default)
    println!("----------------------------------------------------------------");
    println!("  Demo 1: Analyze a file for splitting (dry-run)");
    println!("----------------------------------------------------------------\n");

    let target_file = "src/services/file_health.rs";
    println!("Command: pmat split {}\n", target_file);

    let output = Command::new(&pmat)
        .args(["split", target_file])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 2: JSON output for CI/CD
    println!("\n----------------------------------------------------------------");
    println!("  Demo 2: JSON Output (for CI/CD and scripting)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat split {} --format json\n", target_file);

    let output = Command::new(&pmat)
        .args(["split", target_file, "--format", "json"])
        .current_dir(&project_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(40) {
        println!("{}", line);
    }
    if stdout.lines().count() > 40 {
        println!("... (output truncated)");
    }

    // Demo 3: Higher resolution (more clusters)
    println!("\n----------------------------------------------------------------");
    println!("  Demo 3: Higher Resolution (more granular clusters)");
    println!("----------------------------------------------------------------\n");
    println!(
        "Command: pmat split {} --resolution 1.5 --min-cluster-lines 30\n",
        target_file
    );

    let output = Command::new(&pmat)
        .args([
            "split",
            target_file,
            "--resolution",
            "1.5",
            "--min-cluster-lines",
            "30",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Summary
    println!("\n================================================================");
    println!("  How pmat split works");
    println!("================================================================\n");

    println!("  1. Loads the function index for the target file");
    println!("  2. Builds an intra-file call graph (functions that call each other)");
    println!("  3. Runs Louvain community detection to find cohesive clusters");
    println!("  4. Names each cluster using signal cascade:");
    println!("     - DominantType:       Struct/enum/trait that dominates the cluster");
    println!("     - FunctionTheme:      Common keyword across function names");
    println!("     - CommonPrefix:       Shared name prefix (min 4 chars)");
    println!("     - DocCommentConsensus: Keywords from doc comments");
    println!("     - ContextWord:        Most frequent word in function names");
    println!("  5. Reports impact (which files import this module)\n");

    println!("  Flags:");
    println!("  --execute              Create split files with include!() pattern");
    println!("  --format json          Machine-readable output");
    println!("  --resolution N         Louvain resolution (higher = more clusters)");
    println!("  --min-cluster-lines N  Minimum lines per cluster (default: 50)\n");

    println!("Done.");
    Ok(())
}

fn print_output(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        println!("{}", line);
    }
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            for line in stderr.lines() {
                eprintln!("  {}", line);
            }
        }
    }
}

fn find_pmat_binary() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(output) = Command::new("pmat").arg("--version").output() {
        if output.status.success() {
            return Ok("pmat".to_string());
        }
    }

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
