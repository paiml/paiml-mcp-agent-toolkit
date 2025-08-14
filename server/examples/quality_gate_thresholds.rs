//! Example demonstrating quality gate with configurable thresholds
//!
//! This example shows how to use quality gate with custom complexity thresholds.
//! Addresses issue #32 where --max-cyclomatic didn't affect report output.

use pmat::cli::stubs::handle_quality_gate;
use pmat::cli::{QualityCheckType, QualityGateOutputFormat};
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
        0.5,                                // min_entropy
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
        0.5,                                // min_entropy
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
        0.5,                                // min_entropy
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
    use std::io::Write;

    // Create src directory
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;

    // Simple function (complexity: 1)
    let mut file = fs::File::create(src_dir.join("simple.rs"))?;
    writeln!(file, "/// A very simple function")?;
    writeln!(file, "pub fn add(a: i32, b: i32) -> i32 {{")?;
    writeln!(file, "    a + b")?;
    writeln!(file, "}}")?;

    // Moderate complexity function (complexity: ~8)
    let mut file = fs::File::create(src_dir.join("moderate.rs"))?;
    writeln!(file, "/// Function with moderate complexity")?;
    writeln!(file, "pub fn process_data(items: &[i32]) -> String {{")?;
    writeln!(file, "    let mut result = String::new();")?;
    writeln!(file, "    for item in items {{")?;
    writeln!(file, "        if *item > 10 {{")?;
    writeln!(file, "            result.push_str(\"high\");")?;
    writeln!(file, "        }} else if *item > 5 {{")?;
    writeln!(file, "            result.push_str(\"medium\");")?;
    writeln!(file, "        }} else {{")?;
    writeln!(file, "            result.push_str(\"low\");")?;
    writeln!(file, "        }}")?;
    writeln!(file, "        match item % 3 {{")?;
    writeln!(file, "            0 => result.push_str(\"-multiple\"),")?;
    writeln!(file, "            1 => result.push_str(\"-plus-one\"),")?;
    writeln!(file, "            _ => result.push_str(\"-other\"),")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "    result")?;
    writeln!(file, "}}")?;

    // High complexity function (complexity: ~15)
    let mut file = fs::File::create(src_dir.join("complex.rs"))?;
    writeln!(file, "/// Function with high complexity")?;
    writeln!(
        file,
        "pub fn complex_logic(data: &[u8]) -> Result<String, String> {{"
    )?;
    writeln!(file, "    let mut output = String::new();")?;
    writeln!(file, "    for (i, &byte) in data.iter().enumerate() {{")?;
    writeln!(file, "        if byte == 0 {{")?;
    writeln!(
        file,
        "            return Err(\"Invalid byte\".to_string());"
    )?;
    writeln!(file, "        }}")?;
    writeln!(file, "        match byte {{")?;
    writeln!(file, "            1..=10 => {{")?;
    writeln!(file, "                if i % 2 == 0 {{")?;
    writeln!(file, "                    output.push('A');")?;
    writeln!(file, "                }} else {{")?;
    writeln!(file, "                    output.push('B');")?;
    writeln!(file, "                }}")?;
    writeln!(file, "            }}")?;
    writeln!(file, "            11..=20 => {{")?;
    writeln!(file, "                if i % 3 == 0 {{")?;
    writeln!(file, "                    output.push('C');")?;
    writeln!(file, "                }} else if i % 3 == 1 {{")?;
    writeln!(file, "                    output.push('D');")?;
    writeln!(file, "                }} else {{")?;
    writeln!(file, "                    output.push('E');")?;
    writeln!(file, "                }}")?;
    writeln!(file, "            }}")?;
    writeln!(file, "            21..=30 => {{")?;
    writeln!(file, "                for j in 0..3 {{")?;
    writeln!(file, "                    if j == i % 3 {{")?;
    writeln!(file, "                        output.push('F');")?;
    writeln!(file, "                    }}")?;
    writeln!(file, "                }}")?;
    writeln!(file, "            }}")?;
    writeln!(file, "            _ => {{")?;
    writeln!(file, "                if byte > 100 {{")?;
    writeln!(file, "                    output.push('X');")?;
    writeln!(file, "                }} else {{")?;
    writeln!(file, "                    output.push('Y');")?;
    writeln!(file, "                }}")?;
    writeln!(file, "            }}")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "    Ok(output)")?;
    writeln!(file, "}}")?;

    Ok(())
}
