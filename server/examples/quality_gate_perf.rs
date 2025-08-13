//! Example demonstrating quality gate performance metrics
//!
//! This example addresses issue #31 where --perf flag didn't show performance metrics.

use pmat::cli::stubs::handle_quality_gate;
use pmat::cli::{QualityCheckType, QualityGateOutputFormat};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("# Quality Gate Performance Metrics Example\n");
    println!("This example demonstrates the fix for issue #31.\n");

    // Create a test project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create test files with varying complexity
    create_test_project(project_path)?;

    println!("## Example 1: Without Performance Metrics\n");

    handle_quality_gate(
        project_path.to_path_buf(),
        None,
        QualityGateOutputFormat::Human,
        false,
        vec![
            QualityCheckType::Complexity,
            QualityCheckType::Security,
            QualityCheckType::Satd,
        ],
        15.0,
        0.5,
        20,
        false,
        None,
        false, // perf = false
    )
    .await?;

    println!("\n## Example 2: With Performance Metrics (--perf)\n");

    handle_quality_gate(
        project_path.to_path_buf(),
        None,
        QualityGateOutputFormat::Human,
        false,
        vec![
            QualityCheckType::Complexity,
            QualityCheckType::Security,
            QualityCheckType::Satd,
        ],
        15.0,
        0.5,
        20,
        false,
        None,
        true, // perf = true (--perf flag)
    )
    .await?;

    println!("\n## Example 3: All Checks with Performance Metrics\n");

    handle_quality_gate(
        project_path.to_path_buf(),
        None,
        QualityGateOutputFormat::Human,
        false,
        vec![], // Empty = all checks
        15.0,
        0.5,
        20,
        false,
        None,
        true, // perf = true
    )
    .await?;

    println!("\n## Key Points:");
    println!("✅ --perf flag now shows timing for each check");
    println!("✅ Total execution time is displayed");
    println!("✅ Individual check timings help identify slow checks");
    println!("✅ Average time per check provides quick performance overview");
    println!("✅ Works for both specific checks and all checks");

    Ok(())
}

fn create_test_project(project_path: &std::path::Path) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;

    // Create src directory
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create multiple files to make timing more visible
    for i in 0..5 {
        let mut file = fs::File::create(src_dir.join(format!("module{}.rs", i)))?;

        // Add some content with varying issues
        if i == 0 {
            writeln!(file, "// TODO: Refactor this module")?;
            writeln!(file, "// FIXME: Security issue here")?;
        }

        writeln!(file, "#[allow(dead_code)]")?;
        writeln!(
            file,
            "fn process_{i}(data: &str) -> Result<String, String> {{"
        )?;

        if i == 1 {
            writeln!(file, "    let api_key = \"hardcoded-secret-key\";")?;
        }

        // Add some complexity
        for j in 0..3 {
            writeln!(file, "    if data.len() > {j} {{")?;
            writeln!(file, "        if data.contains(\"{j}\") {{")?;
            writeln!(file, "            return Ok(format!(\"Found {j}\"));")?;
            writeln!(file, "        }}")?;
            writeln!(file, "    }}")?;
        }

        writeln!(file, "    Ok(data.to_string())")?;
        writeln!(file, "}}")?;

        // Add more functions to increase analysis time
        for j in 0..10 {
            writeln!(file)?;
            writeln!(file, "fn helper_{i}_{j}(x: i32) -> i32 {{")?;
            writeln!(file, "    x * {j} + {i}")?;
            writeln!(file, "}}")?;
        }
    }

    // Create a README
    let mut readme = fs::File::create(project_path.join("README.md"))?;
    writeln!(readme, "# Performance Test Project")?;
    writeln!(readme)?;
    writeln!(
        readme,
        "This project has multiple files to demonstrate performance metrics."
    )?;

    Ok(())
}
