//! Example demonstrating quality gate with configurable thresholds
//!
//! This example shows how to use quality gate with custom complexity thresholds.
//! Addresses issue #32 where --max-cyclomatic didn't affect report output.

use pmat::cli::analysis_utilities::handle_quality_gate;
use pmat::cli::enums::{QualityCheckType, QualityGateOutputFormat};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("# Quality Gate with Custom Thresholds Example\n");

    // Create a test project with files of varying complexity
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create test files
    create_test_files(project_path)?;

    println!("## Example 1: Default Thresholds (20/15)\n");

    // Run with default thresholds
    println!("Running quality gate with default thresholds...\n");
    let result = handle_quality_gate(
        project_path.to_path_buf(),
        None,                               // file
        QualityGateOutputFormat::Human,     // format
        false,                              // fail_on_violation
        vec![QualityCheckType::Complexity], // checks
        15.0,                               // max_dead_code
        Some(0.5),                          // min_entropy
        20,                                 // max_complexity_p99 (default)
        false,                              // include_provability
        None,                               // output
        false,                              // _perf
    )
    .await;

    match result {
        Ok(_) => println!("✅ Quality gate passed with default thresholds\n"),
        Err(e) => println!("❌ Quality gate failed: {}\n", e),
    }

    println!("## Example 2: Strict Thresholds (10/8)\n");

    // Run with strict thresholds
    println!("Running quality gate with strict thresholds...\n");
    let result = handle_quality_gate(
        project_path.to_path_buf(),
        None,                               // file
        QualityGateOutputFormat::Human,     // format
        false,                              // fail_on_violation
        vec![QualityCheckType::Complexity], // checks
        15.0,                               // max_dead_code
        Some(0.5),                          // min_entropy
        10,                                 // max_complexity_p99 (strict)
        false,                              // include_provability
        None,                               // output
        false,                              // _perf
    )
    .await;

    match result {
        Ok(_) => println!("✅ Quality gate passed with strict thresholds\n"),
        Err(e) => println!("❌ Quality gate failed: {}\n", e),
    }

    println!("## Example 3: CI/CD Mode with Exit Codes\n");

    // Demonstrate CI/CD mode
    println!("Running quality gate in CI/CD mode (fail-on-violation)...\n");
    let result = handle_quality_gate(
        project_path.to_path_buf(),
        None,                               // file
        QualityGateOutputFormat::Json,      // JSON for parsing
        false,                              // fail_on_violation - would be true in real CI/CD
        vec![QualityCheckType::Complexity], // checks
        15.0,                               // max_dead_code
        Some(0.5),                          // min_entropy
        15,                                 // max_complexity_p99
        false,                              // include_provability
        None,                               // output
        false,                              // _perf
    )
    .await;

    match result {
        Ok(_) => {
            println!("✅ Quality gate passed - would exit with code 0");
            println!("   Perfect for CI/CD pipelines!");
        }
        Err(e) => {
            println!("❌ Quality gate failed - would exit with code 1");
            println!("   CI/CD pipeline would fail the build");
            println!("   Error: {}", e);
        }
    }

    println!("\n## Key Points:");
    println!("- Custom thresholds now properly affect violation detection");
    println!("- Use --fail-on-violation for CI/CD integration");
    println!("- JSON format is ideal for parsing in CI scripts");
    println!("- Exit codes: 0 for success, 1 for failure");

    Ok(())
}

fn create_test_files(project_path: &std::path::Path) -> anyhow::Result<()> {
    use std::fs;

    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;

    // Simple function (complexity: 1)
    fs::write(src_dir.join("simple.rs"), SIMPLE_RS)?;

    // Moderate complexity function (complexity: ~8)
    fs::write(src_dir.join("moderate.rs"), MODERATE_RS)?;

    // High complexity function (complexity: ~15)
    fs::write(src_dir.join("complex.rs"), COMPLEX_RS)?;

    Ok(())
}

const SIMPLE_RS: &str = r#"/// A very simple function
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

const MODERATE_RS: &str = r#"/// Function with moderate complexity
pub fn process_data(items: &[i32]) -> String {
    let mut result = String::new();
    for item in items {
        if *item > 10 {
            result.push_str("high");
        } else if *item > 5 {
            result.push_str("medium");
        } else {
            result.push_str("low");
        }
        match item % 3 {
            0 => result.push_str("-multiple"),
            1 => result.push_str("-plus-one"),
            _ => result.push_str("-other"),
        }
    }
    result
}
"#;

const COMPLEX_RS: &str = r#"/// Function with high complexity
pub fn complex_logic(data: &[u8]) -> Result<String, String> {
    let mut output = String::new();
    for (i, &byte) in data.iter().enumerate() {
        if byte == 0 {
            return Err("Invalid byte".to_string());
        }
        match byte {
            1..=10 => {
                if i % 2 == 0 {
                    output.push('A');
                } else {
                    output.push('B');
                }
            }
            11..=20 => {
                if i % 3 == 0 {
                    output.push('C');
                } else if i % 3 == 1 {
                    output.push('D');
                } else {
                    output.push('E');
                }
            }
            21..=30 => {
                for j in 0..3 {
                    if j == i % 3 {
                        output.push('F');
                    }
                }
            }
            _ => {
                if byte > 100 {
                    output.push('X');
                } else {
                    output.push('Y');
                }
            }
        }
    }
    Ok(output)
}
"#;
