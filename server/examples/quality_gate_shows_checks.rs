//! Example demonstrating that quality gate now shows which checks are being run
//!
//! This example addresses issue #30 where quality-gate didn't show checks.

use pmat::cli::{QualityCheckType, QualityGateOutputFormat};
use pmat::cli::stubs::handle_quality_gate;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("# Quality Gate Shows Checks Example\n");
    println!("This example demonstrates the fix for issue #30.\n");
    
    // Create a test project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();
    
    // Create some test files
    create_test_files(project_path)?;
    
    println!("## Example 1: Default Checks (All)\n");
    println!("When no checks are specified, quality gate runs all checks:\n");
    
    handle_quality_gate(
        project_path.to_path_buf(),
        None,
        QualityGateOutputFormat::Human,
        false,
        vec![], // Empty = run all checks
        15.0,
        0.5,
        20,
        false,
        None,
        false,
    ).await?;
    
    println!("\n## Example 2: Specific Checks\n");
    println!("When specific checks are requested:\n");
    
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
        false,
    ).await?;
    
    println!("\n## Example 3: Single Check\n");
    println!("Running only complexity check:\n");
    
    handle_quality_gate(
        project_path.to_path_buf(),
        None,
        QualityGateOutputFormat::Human,
        false,
        vec![QualityCheckType::Complexity],
        15.0,
        0.5,
        20,
        false,
        None,
        false,
    ).await?;
    
    println!("\n## Key Points:");
    println!("✅ Quality gate now displays '📋 Checks to run:' before running");
    println!("✅ Shows exactly which checks will be performed");
    println!("✅ Empty checks vector correctly shows all checks");
    println!("✅ Progress is shown for each check as it runs");
    
    Ok(())
}

fn create_test_files(project_path: &std::path::Path) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;
    
    // Create src directory
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create a file with various issues to detect
    let mut file = fs::File::create(src_dir.join("main.rs"))?;
    writeln!(file, "// TODO: Refactor this later")?;
    writeln!(file, "fn main() {{")?;
    writeln!(file, "    let password = \"hardcoded123\";")?;
    writeln!(file, "    println!(\"Hello\");")?;
    writeln!(file, "}}")?;
    writeln!(file)?;
    writeln!(file, "#[allow(dead_code)]")?;
    writeln!(file, "fn unused_function() {{")?;
    writeln!(file, "    // This function is never called")?;
    writeln!(file, "}}")?;
    
    // Create a README
    let mut readme = fs::File::create(project_path.join("README.md"))?;
    writeln!(readme, "# Test Project")?;
    writeln!(readme)?;
    writeln!(readme, "A simple test project for quality gate.")?;
    
    Ok(())
}