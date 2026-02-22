//! Extract Candidates Demo — I/O Classification and Module Extraction Suggestions
//!
//! Demonstrates PMAT's `--extract-candidates` feature that scans function source
//! for I/O patterns (println!, File::open, Command::new, etc.), classifies each
//! function as PURE or IO, groups by name prefix / call graph clusters, and
//! suggests module extractions for large files.
//!
//! # Run
//! ```bash
//! cargo run --example extract_candidates_demo
//! ```

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  PMAT Extract-Candidates Demo (Issue #235)");
    println!("  I/O Classification + Module Extraction Suggestions");
    println!("================================================================\n");

    let pmat = find_pmat_binary()?;
    let project_dir = std::env::current_dir()?;

    // Demo 1: Basic extract-candidates
    println!("----------------------------------------------------------------");
    println!("  Demo 1: Extract Candidates (top 5 groups)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --extract-candidates --limit 5 --exclude-tests\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "--extract-candidates",
            "--limit",
            "5",
            "--exclude-tests",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 2: JSON output for scripting/CI
    println!("\n----------------------------------------------------------------");
    println!("  Demo 2: JSON Output (for CI/CD and scripting)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --extract-candidates --format json --limit 2 --exclude-tests\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "--extract-candidates",
            "--format",
            "json",
            "--limit",
            "2",
            "--exclude-tests",
        ])
        .current_dir(&project_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(50) {
        println!("{}", line);
    }
    if stdout.lines().count() > 50 {
        println!("... (output truncated)");
    }

    // Demo 3: Markdown output
    println!("\n----------------------------------------------------------------");
    println!("  Demo 3: Markdown Output (for docs and reports)");
    println!("----------------------------------------------------------------\n");
    println!(
        "Command: pmat query --extract-candidates --format markdown --limit 2 --exclude-tests\n"
    );

    let output = Command::new(&pmat)
        .args([
            "query",
            "--extract-candidates",
            "--format",
            "markdown",
            "--limit",
            "2",
            "--exclude-tests",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 4: Path-scoped with custom max-module-lines
    println!("\n----------------------------------------------------------------");
    println!("  Demo 4: Path-Scoped + max-module-lines=300");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query --extract-candidates --path src/cli --max-module-lines 300 --limit 3 --exclude-tests\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "--extract-candidates",
            "--path",
            "src/cli",
            "--max-module-lines",
            "300",
            "--limit",
            "3",
            "--exclude-tests",
        ])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Summary
    println!("\n================================================================");
    println!("  How extract-candidates works");
    println!("================================================================\n");

    println!("  1. SCAN:  Load all function source code from the index");
    println!("  2. CLASSIFY:  Detect I/O patterns in each function:");
    println!("     - PRINT (println!, print!)      - FS (File::open, std::fs::)");
    println!("     - PROCESS (Command::new)        - DB (rusqlite::, sqlx::)");
    println!("     - NET (reqwest::, tokio::net::)  - WRITE (write!, writeln!)");
    println!("  3. GROUP:  Cluster functions by name prefix and call graph");
    println!("  4. SUGGEST:  Recommend module extractions with purity %\n");

    println!("  Use cases:");
    println!("  - Refactoring large files (identify extractable pure-logic modules)");
    println!("  - Architecture review (find I/O-heavy vs pure-logic boundaries)");
    println!("  - Code quality (pure functions are easier to test and reason about)\n");

    println!("  Output formats:");
    println!("  --format text       Colored output with [PURE]/[IO] badges (default)");
    println!("  --format json       Structured JSON for CI/CD");
    println!("  --format markdown   Markdown tables for reports\n");

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
        for line in stderr.lines() {
            // Skip index loading noise
            if !line.contains("Loading index")
                && !line.contains("Building index")
                && !line.contains("Checking for incremental")
            {
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

    let release_path = std::path::Path::new("target/release/pmat");
    if release_path.exists() {
        return Ok(release_path.to_string_lossy().to_string());
    }

    let debug_path = std::path::Path::new("target/debug/pmat");
    if debug_path.exists() {
        return Ok(debug_path.to_string_lossy().to_string());
    }

    Err("pmat binary not found. Run 'cargo install --path .' first.".into())
}
