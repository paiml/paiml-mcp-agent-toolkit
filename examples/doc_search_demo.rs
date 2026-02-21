//! Document Search Demo — Search PDFs, SVGs, Images, and Markdown alongside code
//!
//! Demonstrates PMAT's document indexing feature that makes non-code files
//! searchable via FTS5 BM25 ranking. Documents are indexed lazily on first
//! query and cached incrementally via SHA256 checksums.
//!
//! # Run
//! ```bash
//! cargo run --example doc_search_demo
//! ```

use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  PMAT Document Search Demo (--docs / --docs-only)");
    println!("================================================================\n");

    let pmat = find_pmat_binary()?;
    let project_dir = std::env::current_dir()?;

    // Demo 1: Default mode — code + docs
    println!("----------------------------------------------------------------");
    println!("  Demo 1: Default Search (code + documents)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"architecture\" --limit 3\n");

    let output = Command::new(&pmat)
        .args(["query", "architecture", "--limit", "3"])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 2: Docs-only mode
    println!("\n----------------------------------------------------------------");
    println!("  Demo 2: Document-Only Search (--docs-only)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"design specification\" --docs-only --limit 5\n");

    let output = Command::new(&pmat)
        .args(["query", "design specification", "--docs-only", "--limit", "5"])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Demo 3: JSON output
    println!("\n----------------------------------------------------------------");
    println!("  Demo 3: JSON Document Results (for CI/CD)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"error handling\" --docs-only --format json --limit 3\n");

    let output = Command::new(&pmat)
        .args([
            "query",
            "error handling",
            "--docs-only",
            "--format",
            "json",
            "--limit",
            "3",
        ])
        .current_dir(&project_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().take(40) {
        println!("{}", line);
    }
    if stdout.lines().count() > 40 {
        println!("... (output truncated)");
    }

    // Demo 4: No-docs mode
    println!("\n----------------------------------------------------------------");
    println!("  Demo 4: Code-Only Search (--no-docs)");
    println!("----------------------------------------------------------------\n");
    println!("Command: pmat query \"parse\" --no-docs --limit 3\n");

    let output = Command::new(&pmat)
        .args(["query", "parse", "--no-docs", "--limit", "3"])
        .current_dir(&project_dir)
        .output()?;

    print_output(&output);

    // Summary
    println!("\n================================================================");
    println!("  Document Search Features");
    println!("================================================================\n");

    println!("  Supported document types:\n");
    println!("  Type        Extensions              Extraction Method");
    println!("  ─────────   ─────────────────────   ────────────────────────");
    println!("  PDF         .pdf                    Full text (--features doc-indexing)");
    println!("  SVG         .svg                    <text>/<tspan> regex extraction");
    println!("  Image       .png .jpg .jpeg .webp   Filename + path metadata");
    println!("  Markdown    .md .markdown            Heading-based chunking");
    println!("  Plaintext   .txt .rst .adoc          Paragraph-based chunking\n");

    println!("  CLI flags:");
    println!("  (default)     Code + document results (docs on by default)");
    println!("  --docs-only   Search only documents, skip code index");
    println!("  --no-docs     Disable document results, code only\n");

    println!("  Performance:");
    println!("  - Lazy indexing: documents indexed on first query");
    println!("  - Incremental: SHA256 checksums skip unchanged files");
    println!("  - FTS5 BM25: porter stemming + unicode normalization\n");

    println!("Done.");
    Ok(())
}

fn print_output(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        println!("{}", line);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        // Filter out build noise
        if !line.contains("Building index")
            && !line.contains("Loading index")
            && !line.contains("query profile")
            && !line.contains("ANDON")
        {
            if !line.trim().is_empty() {
                eprintln!("{}", line);
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
