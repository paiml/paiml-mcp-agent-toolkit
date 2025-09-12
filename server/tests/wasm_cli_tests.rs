//! TDD Tests for WASM CLI commands
//! Following RED-GREEN-REFACTOR cycle

use pmat::cli::commands::AnalyzeCommands;
use pmat::cli::enums::WasmOutputFormat;
use pmat::cli::handlers::wasm_handler;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_wasm_analyze_command_structure() {
    // RED Phase: Test that WASM command exists in AnalyzeCommands
    let wasm_cmd = AnalyzeCommands::Wasm {
        wasm_file: PathBuf::from("test.wasm"),
        format: WasmOutputFormat::Summary,
        verify: false,
        security: false,
        profile: false,
        baseline: None,
        output: None,
        verbose: false,
    };

    // Should be able to create the command
    match wasm_cmd {
        AnalyzeCommands::Wasm { .. } => {
            // Command structure is valid
        }
        _ => panic!("WASM command not properly structured"),
    }
}

#[test]
fn test_wasm_handler_basic_analysis() {
    // RED Phase: Test basic WASM analysis
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("test.wasm");

    // Create minimal valid WASM file
    std::fs::write(
        &wasm_path,
        &[
            0x00, 0x61, 0x73, 0x6d, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version 1
        ],
    )
    .unwrap();

    // Run analysis
    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            wasm_path,
            WasmOutputFormat::Summary,
            false, // verify
            false, // security
            false, // profile
            None,  // baseline
            None,  // output
            false, // verbose
        )
        .await
    });

    assert!(result.is_ok(), "WASM analysis should succeed");
}

#[test]
fn test_wasm_security_scanning() {
    // RED Phase: Test security vulnerability scanning
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("vulnerable.wasm");

    // Create WASM with potential vulnerability pattern
    // This would contain actual bytecode with vulnerable patterns
    std::fs::write(&wasm_path, create_vulnerable_wasm()).unwrap();

    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            wasm_path,
            WasmOutputFormat::Json,
            false, // verify
            true,  // security - ENABLE
            false, // profile
            None,  // baseline
            None,  // output
            false, // verbose
        )
        .await
    });

    assert!(result.is_ok(), "Security scanning should complete");
    // In real implementation, we'd parse JSON and check for vulnerabilities
}

#[test]
fn test_wasm_verification() {
    // RED Phase: Test formal verification
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("verify.wasm");

    std::fs::write(&wasm_path, create_safe_wasm()).unwrap();

    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            wasm_path,
            WasmOutputFormat::Summary,
            true,  // verify - ENABLE
            false, // security
            false, // profile
            None,  // baseline
            None,  // output
            false, // verbose
        )
        .await
    });

    assert!(result.is_ok(), "Verification should complete");
}

#[test]
fn test_wasm_profiling() {
    // RED Phase: Test performance profiling
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("profile.wasm");

    std::fs::write(&wasm_path, create_complex_wasm()).unwrap();

    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            wasm_path,
            WasmOutputFormat::Detailed,
            false, // verify
            false, // security
            true,  // profile - ENABLE
            None,  // baseline
            None,  // output
            false, // verbose
        )
        .await
    });

    assert!(result.is_ok(), "Profiling should complete");
}

#[test]
fn test_wasm_baseline_comparison() {
    // RED Phase: Test quality baseline comparison
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("current.wasm");
    let baseline_path = temp_dir.path().join("baseline.wasm");

    std::fs::write(&wasm_path, create_complex_wasm()).unwrap();
    std::fs::write(&baseline_path, create_safe_wasm()).unwrap();

    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            wasm_path,
            WasmOutputFormat::Summary,
            false,               // verify
            false,               // security
            false,               // profile
            Some(baseline_path), // baseline - PROVIDED
            None,                // output
            false,               // verbose
        )
        .await
    });

    assert!(result.is_ok(), "Baseline comparison should complete");
}

#[test]
fn test_wasm_output_formats() {
    // RED Phase: Test different output formats
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("test.wasm");

    std::fs::write(&wasm_path, create_safe_wasm()).unwrap();

    for format in [
        WasmOutputFormat::Summary,
        WasmOutputFormat::Json,
        WasmOutputFormat::Detailed,
        WasmOutputFormat::Sarif,
    ] {
        let format_clone = format.clone();
        let result = tokio_test::block_on(async {
            wasm_handler::handle_analyze_wasm(
                wasm_path.clone(),
                format_clone,
                false,
                false,
                false,
                None,
                None,
                false,
            )
            .await
        });

        assert!(result.is_ok(), "Format {:?} should work", format);
    }
}

#[test]
fn test_wasm_file_not_found() {
    // RED Phase: Test error handling for missing file
    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            PathBuf::from("nonexistent.wasm"),
            WasmOutputFormat::Summary,
            false,
            false,
            false,
            None,
            None,
            false,
        )
        .await
    });

    assert!(result.is_err(), "Should error on missing file");
}

#[test]
fn test_wasm_invalid_binary() {
    // RED Phase: Test error handling for invalid WASM
    let temp_dir = tempdir().unwrap();
    let wasm_path = temp_dir.path().join("invalid.wasm");

    // Write invalid data
    std::fs::write(&wasm_path, b"not a wasm file").unwrap();

    let result = tokio_test::block_on(async {
        wasm_handler::handle_analyze_wasm(
            wasm_path,
            WasmOutputFormat::Summary,
            false,
            false,
            false,
            None,
            None,
            false,
        )
        .await
    });

    assert!(result.is_err(), "Should error on invalid WASM");
}

// Helper functions to create test WASM binaries

fn create_vulnerable_wasm() -> Vec<u8> {
    // Minimal WASM with unchecked memory growth pattern
    vec![
        0x00, 0x61, 0x73, 0x6d, // Magic
        0x01, 0x00, 0x00, 0x00, // Version
        // Memory section with growth
        0x05, 0x03, 0x01, 0x00, 0x01, // Memory: 1 page initial
        // Code section with memory.grow without bounds check
        0x0a, 0x09, 0x01, 0x07, 0x00, 0x41, 0x01, // i32.const 1
        0x40, 0x00, // memory.grow
        0x1a, // drop
        0x0b, // end
    ]
}

fn create_safe_wasm() -> Vec<u8> {
    // Minimal valid WASM
    vec![
        0x00, 0x61, 0x73, 0x6d, // Magic
        0x01, 0x00, 0x00, 0x00, // Version
        // Type section
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Function section
        0x03, 0x02, 0x01, 0x00, // Code section
        0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
    ]
}

fn create_complex_wasm() -> Vec<u8> {
    // WASM with multiple functions for profiling
    vec![
        0x00, 0x61, 0x73, 0x6d, // Magic
        0x01, 0x00, 0x00, 0x00, // Version
        // Type section
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // Function section
        0x03, 0x03, 0x02, 0x00, 0x00, // Export section
        0x07, 0x08, 0x01, 0x04, 0x61, 0x64, 0x64, 0x00, 0x00,
        // Code section with arithmetic
        0x0a, 0x09, 0x02, 0x04, 0x00, 0x20, 0x00, 0x0b, // Function 0
        0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, // Function 1: add
    ]
}
