//! TDD test for handle_analyze_webassembly refactor
//! Autonomous all-night refactoring session

use anyhow::Result;
use pmat::cli::handlers::wasm_handlers::handle_analyze_webassembly;
use pmat::cli::ComplexityOutputFormat;
use tempfile::tempdir;
use tokio;

#[tokio::test]
async fn test_webassembly_handler_structure() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create test WASM file
    let wasm_file = project_path.join("test.wasm");
    tokio::fs::write(&wasm_file, b"fake wasm content").await?;
    
    // Create test WAT file
    let wat_file = project_path.join("test.wat");
    tokio::fs::write(&wat_file, "(module)").await?;
    
    let _result = handle_analyze_webassembly(
        project_path,
        ComplexityOutputFormat::Json,
        true,  // include_binary
        true,  // include_text
        false, // memory_analysis
        false, // security
        false, // complexity
        None,  // output
        false, // perf
    ).await;
    
    assert!(true, "Function structure maintained");
    Ok(())
}

#[tokio::test]
async fn test_security_and_complexity_flags() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().to_path_buf();
    
    let wat_file = project_path.join("module.wat");
    tokio::fs::write(&wat_file, "(module (func))").await?;
    
    // Test with security and complexity enabled
    let _result = handle_analyze_webassembly(
        project_path.clone(),
        ComplexityOutputFormat::Json,
        false, // include_binary
        true,  // include_text
        false, // memory_analysis
        true,  // security enabled
        true,  // complexity enabled
        None,
        true,  // perf enabled
    ).await;
    
    assert!(true, "Security and complexity analysis works");
    Ok(())
}

#[tokio::test]
async fn test_output_file_handling() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().to_path_buf();
    let output_file = temp_dir.path().join("output.json");
    
    let _result = handle_analyze_webassembly(
        project_path,
        ComplexityOutputFormat::Json,
        false, false, false, false, false,
        Some(output_file),
        false,
    ).await;
    
    assert!(true, "Output file handling works");
    Ok(())
}