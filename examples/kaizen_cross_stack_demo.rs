//! Cross-Stack Kaizen Example
//!
//! Demonstrates `pmat kaizen --cross-stack`, which scans all batuta stack crates
//! in a single invocation, applies fixes per-crate, and produces a unified report.
//!
//! Run with: `cargo run --example kaizen_cross_stack_demo`
//!
//! # Features Demonstrated
//!
//! 1. Cross-stack discovery via `discover_workspace_crates()`
//! 2. Per-crate scanning (clippy, fmt, comply, defects, github issues)
//! 3. Per-crate auto-fix with cargo check verification
//! 4. Per-crate commit and push with branch detection
//! 5. Grouped report output (text, markdown, JSON)
//!
//! # CLI Usage
//!
//! ```bash
//! # Dry-run scan across entire stack
//! pmat kaizen --cross-stack --dry-run
//!
//! # Cross-stack with auto-fix, no issues filed
//! pmat kaizen --cross-stack --no-issues
//!
//! # Cross-stack with push per-crate
//! pmat kaizen --cross-stack --push
//!
//! # JSON output for CI/CD
//! pmat kaizen --cross-stack --dry-run -f json
//!
//! # Markdown report to file
//! pmat kaizen --cross-stack --dry-run -f markdown -o kaizen-report.md
//!
//! # Skip specific scanners
//! pmat kaizen --cross-stack --skip-github --skip-defects --dry-run
//!
//! # Single-project kaizen (unchanged behavior)
//! pmat kaizen --dry-run
//! ```
//!
//! # Cross-Stack Discovery
//!
//! Crate discovery uses a 5-priority chain:
//! 1. Explicit `--crates` paths (not yet exposed in kaizen)
//! 2. `Cargo.toml` workspace members
//! 3. `batuta oracle --local` (batuta stack auto-discovery)
//! 4. `.pmat/workspace.toml` siblings
//! 5. Single-crate fallback
//!
//! # Report Grouping
//!
//! In cross-stack mode, findings are grouped by crate name:
//!
//! ```text
//! === Kaizen Cross-Stack Report ===
//!
//! --- pmat (12 findings, 8 fixed) ---
//!   [FIXED] [MED ] clippy::needless_return src/lib.rs
//!   [TODO ] [LOW ] rustfmt::unformatted src/main.rs
//!
//! --- trueno (3 findings, 1 fixed) ---
//!   [FIXED] [LOW ] rustfmt::unformatted src/lib.rs
//!   [AGENT] [HIGH] defect::RUST-UNWRAP-001 src/tensor.rs
//! ```

use std::process::Command;

fn main() {
    println!("=== pmat kaizen --cross-stack Demo ===\n");

    // Single-project dry-run
    println!("1. Single-project kaizen (default):");
    let output = Command::new("pmat")
        .args(["kaizen", "--dry-run", "--skip-github", "--skip-defects"])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            for line in stderr.lines().take(3) {
                println!("  {line}");
            }
            for line in stdout.lines().take(5) {
                println!("  {line}");
            }
        }
        Err(e) => println!("  pmat not available: {e}"),
    }

    println!();

    // Cross-stack dry-run
    println!("2. Cross-stack kaizen (--cross-stack):");
    let output = Command::new("pmat")
        .args([
            "kaizen",
            "--cross-stack",
            "--dry-run",
            "--skip-github",
            "--skip-defects",
        ])
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stderr.lines().take(5) {
                println!("  {line}");
            }
            for line in stdout.lines().take(10) {
                println!("  {line}");
            }
        }
        Err(e) => println!("  pmat not available: {e}"),
    }

    println!();

    // JSON output
    println!("3. Cross-stack JSON output:");
    let output = Command::new("pmat")
        .args([
            "kaizen",
            "--cross-stack",
            "--dry-run",
            "--skip-github",
            "--skip-defects",
            "--skip-clippy",
            "--skip-comply",
            "-f",
            "json",
        ])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Parse and show summary
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let findings = json["findings"].as_array().map(|a| a.len()).unwrap_or(0);
                let crates = json["crates_scanned"]
                    .as_array()
                    .map(|a| a.len())
                    .unwrap_or(0);
                println!("  Crates: {crates}, Findings: {findings}");
            }
        }
        Err(e) => println!("  pmat not available: {e}"),
    }

    println!("\nDone.");
}
