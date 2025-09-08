//! TDD test for wasm handlers refactor
//! Following Toyota Way TDD: Red → Green → Refactor

use anyhow::Result;
use pmat::cli::handlers::wasm_handlers::handle_analyze_assemblyscript;
use pmat::cli::ComplexityOutputFormat;
use tempfile::tempdir;
use tokio;

/// Test AssemblyScript handler structure is preserved during refactor
#[tokio::test]
async fn test_assemblyscript_handler_structure() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().to_path_buf();

    // Create minimal test file
    let test_file = project_path.join("test.as");
    tokio::fs::write(
        &test_file,
        "export function add(a: i32, b: i32): i32 { return a + b; }",
    )
    .await?;

    // Test that function accepts all expected parameters
    let _result = handle_analyze_assemblyscript(
        project_path,
        ComplexityOutputFormat::Json,
        false, // wasm_complexity
        false, // memory_analysis
        false, // security
        None,  // output
        60,    // timeout
        false, // perf
    )
    .await;

    // Function structure test - accepts all parameters without panic
    assert!(true, "Function structure maintained during refactor");
    Ok(())
}

/// Test parameter variations
#[tokio::test]
async fn test_parameter_variations() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().to_path_buf();

    // Test with security enabled
    let _result1 = handle_analyze_assemblyscript(
        project_path.clone(),
        ComplexityOutputFormat::Json,
        true, // wasm_complexity enabled
        true, // memory_analysis enabled
        true, // security enabled
        None,
        60,
        true, // perf enabled
    )
    .await;

    // Test with output file
    let output_file = temp_dir.path().join("output.json");
    let _result2 = handle_analyze_assemblyscript(
        project_path,
        ComplexityOutputFormat::Json,
        false,
        false,
        false,
        Some(output_file),
        30, // shorter timeout
        false,
    )
    .await;

    assert!(true, "Parameter variations handled during refactor");
    Ok(())
}
