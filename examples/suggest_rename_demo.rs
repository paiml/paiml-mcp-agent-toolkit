//! Suggest-Rename Demo — Semantic File Rename Suggestions
//!
//! Demonstrates PMAT's `--suggest-rename` feature that analyzes `_part_` files
//! (created by file-splitting refactors) and suggests meaningful names based on
//! cascading signal analysis: DominantType, ExistingSuffix, OriginalBase,
//! FunctionTheme, CommonPrefix, and DocCommentConsensus.
//!
//! # Run
//! ```bash
//! cargo run --example suggest_rename_demo
//! ```

use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  PMAT Suggest-Rename Demo (Semantic File Rename Suggestions)");
    println!("================================================================\n");

    let pmat = find_pmat_binary()?;
    let project_dir = std::env::current_dir()?;

    // Demo 1: Basic suggest-rename
    println!("----------------------------------------------------------------");
    println!("  Demo 1: Basic Suggest-Rename (find _part_ files to rename)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --suggest-rename --limit 10\n");

    let output = Command::new(&pmat)
        .args(["query", "--suggest-rename", "--limit", "10"])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 2: JSON output for scripting/CI
    println!("\n----------------------------------------------------------------");
    println!("  Demo 2: JSON Output (for CI/CD and scripting)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --suggest-rename --format json --limit 5\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "--suggest-rename",
            "--format",
            "json",
            "--limit",
            "5",
        ])
        .current_dir(&project_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(60) {
        println!("{}", line);
    }
    if stdout.lines().count() > 60 {
        println!("... (output truncated)");
    }

    // Demo 3: Markdown output for documentation
    println!("\n----------------------------------------------------------------");
    println!("  Demo 3: Markdown Output (for docs and reports)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --suggest-rename --format markdown --limit 5\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "--suggest-rename",
            "--format",
            "markdown",
            "--limit",
            "5",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 4: Path-filtered suggestions
    println!("\n----------------------------------------------------------------");
    println!("  Demo 4: Path-Filtered Suggestions (specific directory)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --suggest-rename --path src/services/ --limit 5\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "--suggest-rename",
            "--path",
            "src/services/",
            "--limit",
            "5",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Summary
    println!("\n================================================================");
    println!("  How suggest-rename works");
    println!("================================================================\n");

    println!("  Signal types (cascading priority):\n");
    println!("  Signal                  Confidence  Description");
    println!("  ─────────────────────   ──────────  ─────────────────────────────────");
    println!("  DominantType            0.95        >80% of definitions share a type");
    println!("  ExistingSuffix          0.88        _part_ file already has a suffix");
    println!("  OriginalBase            0.82        Pre-split filename still valid");
    println!("  FunctionTheme           0.85        Functions share a common theme");
    println!("  CommonPrefix            0.80        Shared name prefix across defs");
    println!("  DocCommentConsensus     0.70        Doc comments agree on topic\n");

    println!("  Apply workflow:");
    println!("  1. Review:  pmat query --suggest-rename");
    println!("  2. Apply:   pmat query --suggest-rename --apply");
    println!("     - Renames files with confidence >= 0.70");
    println!("     - Updates parent include!() declarations");
    println!("     - Uses git mv for tracked files\n");

    println!("  Output formats:");
    println!("  --format text       Human-readable (default)");
    println!("  --format json       Structured JSON for CI/CD");
    println!("  --format markdown   Markdown table for reports\n");

    println!("Done.");
    Ok(())
}

fn print_output(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
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
