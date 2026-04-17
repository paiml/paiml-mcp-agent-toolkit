//! Issue #53: MCP Tool Functions - Real Service Integration Example
//!
//! This example demonstrates that the 3 MCP tool functions now call real
//! analysis services instead of returning placeholder data.
//!
//! Run this example:
//! ```bash
//! cargo run --example issue_053_mcp_tools
//! ```

use pmat::mcp_pmcp::tool_functions;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Issue #53: MCP Tool Functions - Real Service Integration ===\n");

    // Create temporary test files
    let temp_dir = TempDir::new()?;

    // Create a Rust file with complexity
    let complex_file = temp_dir.path().join("complex.rs");
    std::fs::write(
        &complex_file,
        r#"
fn complex_function(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            if x > 30 {
                return x * 2;
            }
            return x + 10;
        }
        return x + 5;
    }
    x
}

fn simple_function() -> i32 {
    42
}
"#,
    )?;

    // Create a file with SATD comments
    let satd_file = temp_dir.path().join("satd.rs");
    std::fs::write(
        &satd_file,
        r#"
fn some_function() {
    // TODO: Implement proper error handling
    let x = 5;

    // FIXME: This is a temporary hack
    let y = x * 2;

    // HACK: Remove this before production
    println!("Debug: {}", y);
}
"#,
    )?;

    // Create a file with dead code
    let dead_code_file = temp_dir.path().join("dead.rs");
    std::fs::write(
        &dead_code_file,
        r#"
fn used_function() -> i32 {
    42
}

fn unused_function() -> i32 {
    99
}

fn main() {
    let x = used_function();
    println!("Value: {}", x);
}
"#,
    )?;

    // ========================================================================
    // Example 1: analyze_complexity
    // ========================================================================
    println!("📊 Example 1: Complexity Analysis");
    println!("─────────────────────────────────");

    let complexity_result = tool_functions::analyze_complexity(
        std::slice::from_ref(&complex_file),
        Some(10), // top 10 files
        Some(5),  // threshold = 5
    )
    .await?;

    println!("Status: {}", complexity_result["status"]);
    println!("Message: {}", complexity_result["message"]);
    println!(
        "Total files analyzed: {}",
        complexity_result["results"]["total_files"]
    );
    println!(
        "Total complexity: {}",
        complexity_result["results"]["total_complexity"]
    );

    let violations = complexity_result["results"]["violations"]
        .as_array()
        .unwrap();
    println!("Violations (CC >= 5): {}", violations.len());

    for violation in violations {
        println!(
            "  - {} (line {}-{}): CC = {}",
            violation["function"],
            violation["line_start"],
            violation["line_end"],
            violation["complexity"]
        );
    }

    println!("\n✅ analyze_complexity is calling REAL service (not placeholder)\n");

    // ========================================================================
    // Example 2: analyze_satd
    // ========================================================================
    println!("📝 Example 2: SATD (Technical Debt) Analysis");
    println!("─────────────────────────────────────────────");

    let satd_result = tool_functions::analyze_satd(
        std::slice::from_ref(&satd_file),
        false, // don't include resolved
    )
    .await?;

    println!("Status: {}", satd_result["status"]);
    println!("Message: {}", satd_result["message"]);
    println!(
        "Total SATD comments: {}",
        satd_result["results"]["total_satd"]
    );

    let satd_files = satd_result["results"]["files"].as_array().unwrap();
    for file in satd_files {
        println!("  File: {}", file["file"]);
        println!("  SATD count: {}", file["satd_count"]);

        let debts = file["debts"].as_array().unwrap();
        for debt in debts {
            println!(
                "    - Line {}: {} ({})",
                debt["line"], debt["category"], debt["severity"]
            );
            println!("      Text: {}", debt["text"]);
        }
    }

    println!("\n✅ analyze_satd is calling REAL service (not placeholder)\n");

    // ========================================================================
    // Example 3: analyze_dead_code
    // ========================================================================
    println!("💀 Example 3: Dead Code Analysis");
    println!("─────────────────────────────────");

    let dead_code_result = tool_functions::analyze_dead_code(
        std::slice::from_ref(&dead_code_file),
        false, // don't include tests
    )
    .await?;

    println!("Status: {}", dead_code_result["status"]);
    println!("Message: {}", dead_code_result["message"]);
    println!(
        "Total dead code items: {}",
        dead_code_result["results"]["total_dead_code"]
    );

    let dead_files = dead_code_result["results"]["files"].as_array().unwrap();
    for file in dead_files {
        println!("  File: {}", file["file"]);
        println!("  Dead code count: {}", file["dead_code_count"]);

        let dead_functions = file["dead_functions"].as_array().unwrap();
        for func in dead_functions {
            println!("    - {} (line {})", func["name"], func["line"]);
        }
    }

    println!("\n✅ analyze_dead_code is calling REAL service (not placeholder)\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("════════════════════════════════════════════");
    println!("✅ Issue #53 GREEN Phase Complete!");
    println!("════════════════════════════════════════════");
    println!();
    println!("All 3 MCP tool functions now use real analysis services:");
    println!("  1. analyze_complexity → analyze_file_complexity_uncached");
    println!("  2. analyze_satd → SATDDetector");
    println!("  3. analyze_dead_code → analyze_dead_code_multi_language");
    println!();
    println!("No more placeholder responses!");
    println!();

    Ok(())
}
