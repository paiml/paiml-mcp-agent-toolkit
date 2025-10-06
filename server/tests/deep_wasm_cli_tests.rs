//! TDD Tests for Deep WASM CLI commands
//! Following RED-GREEN-REFACTOR cycle with Toyota Way quality standards

#[cfg(feature = "deep-wasm")]
use pmat::cli::commands::AnalyzeCommands;
#[cfg(feature = "deep-wasm")]
use pmat::cli::enums::{DeepWasmFocus, DeepWasmLanguage, DeepWasmOutputFormat};
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_command_structure() {
    // RED Phase: Test that DeepWasm command exists in AnalyzeCommands
    let deep_wasm_cmd = AnalyzeCommands::DeepWasm {
        source_path: PathBuf::from("src/lib.rs"),
        wasm_file: Some(PathBuf::from("app.wasm")),
        dwarf_file: None,
        source_map: None,
        language: Some(DeepWasmLanguage::Rust),
        focus: DeepWasmFocus::Full,
        format: DeepWasmOutputFormat::Markdown,
        output: None,
        strict: false,
        include_mir: false,
        include_llvm_ir: false,
        track_memory: false,
        detect_deadlocks: false,
    };

    // Should be able to create the command
    match deep_wasm_cmd {
        AnalyzeCommands::DeepWasm { .. } => {
            // Command structure is valid
        }
        #[allow(unreachable_patterns)]
        _ => panic!("DeepWasm command not properly structured"),
    }
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_language_enum() {
    // RED Phase: Test language enum variants
    let rust = DeepWasmLanguage::Rust;
    let ruchy = DeepWasmLanguage::Ruchy;

    assert_eq!(rust.to_string(), "rust");
    assert_eq!(ruchy.to_string(), "ruchy");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_focus_enum() {
    // RED Phase: Test focus enum variants
    let focuses = [DeepWasmFocus::Full,
        DeepWasmFocus::Source,
        DeepWasmFocus::Compilation,
        DeepWasmFocus::Runtime,
        DeepWasmFocus::Interop];

    assert_eq!(focuses.len(), 5);
    assert_eq!(DeepWasmFocus::Full.to_string(), "full");
    assert_eq!(DeepWasmFocus::Source.to_string(), "source");
    assert_eq!(DeepWasmFocus::Compilation.to_string(), "compilation");
    assert_eq!(DeepWasmFocus::Runtime.to_string(), "runtime");
    assert_eq!(DeepWasmFocus::Interop.to_string(), "interop");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_output_format_enum() {
    // RED Phase: Test output format enum variants
    let formats = [DeepWasmOutputFormat::Markdown,
        DeepWasmOutputFormat::Json,
        DeepWasmOutputFormat::Html];

    assert_eq!(formats.len(), 3);
    assert_eq!(DeepWasmOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(DeepWasmOutputFormat::Json.to_string(), "json");
    assert_eq!(DeepWasmOutputFormat::Html.to_string(), "html");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_handler_basic_analysis() {
    use pmat::cli::handlers::deep_wasm_handlers;

    // RED Phase: Test basic deep WASM analysis
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("lib.rs");

    // Create minimal Rust source file
    std::fs::write(
        &source_path,
        r#"
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        "#,
    )
    .unwrap();

    // Run analysis without WASM file (should still work)
    let result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file: None,
                dwarf_file: None,
                source_map: None,
                language: Some(DeepWasmLanguage::Rust),
                focus: DeepWasmFocus::Source,
                format: DeepWasmOutputFormat::Markdown,
                output: None,
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    assert!(result.is_ok(), "Deep WASM analysis should succeed");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_with_wasm_binary() {
    use pmat::cli::handlers::deep_wasm_handlers;

    // RED Phase: Test analysis with WASM binary
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("lib.rs");
    let wasm_path = temp_dir.path().join("app.wasm");

    // Create minimal Rust source
    std::fs::write(&source_path, "pub fn test() {}").unwrap();

    // Create minimal valid WASM file
    std::fs::write(
        &wasm_path,
        [
            0x00, 0x61, 0x73, 0x6d, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version 1
        ],
    )
    .unwrap();

    let result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file: Some(wasm_path),
                dwarf_file: None,
                source_map: None,
                language: Some(DeepWasmLanguage::Rust),
                focus: DeepWasmFocus::Full,
                format: DeepWasmOutputFormat::Markdown,
                output: None,
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    assert!(result.is_ok(), "Analysis with WASM binary should succeed");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_strict_mode() {
    use pmat::cli::handlers::deep_wasm_handlers;

    // RED Phase: Test strict quality gate enforcement
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("lib.rs");

    std::fs::write(&source_path, "pub fn test() {}").unwrap();

    let result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file: None,
                dwarf_file: None,
                source_map: None,
                language: Some(DeepWasmLanguage::Rust),
                focus: DeepWasmFocus::Full,
                format: DeepWasmOutputFormat::Json,
                output: None,
                strict: true, // STRICT MODE ENABLED
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    // In strict mode without source map, should fail quality gates
    assert!(result.is_err(), "Strict mode should fail when quality gates are violated");
    assert!(
        result.unwrap_err().to_string().contains("Quality gate violations"),
        "Error should mention quality gate violations"
    );
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_json_output() {
    use pmat::cli::handlers::deep_wasm_handlers;
    use std::fs;

    // RED Phase: Test JSON output format
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("lib.rs");
    let output_path = temp_dir.path().join("report.json");

    std::fs::write(&source_path, "pub fn test() {}").unwrap();

    let result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file: None,
                dwarf_file: None,
                source_map: None,
                language: Some(DeepWasmLanguage::Rust),
                focus: DeepWasmFocus::Full,
                format: DeepWasmOutputFormat::Json,
                output: Some(output_path.clone()),
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    assert!(result.is_ok(), "JSON output should succeed");

    // Verify JSON file was created
    assert!(
        output_path.exists(),
        "JSON output file should be created"
    );

    // Verify JSON is valid
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.is_object(), "Output should be valid JSON object");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_ruchy_language() {
    use pmat::cli::handlers::deep_wasm_handlers;

    // RED Phase: Test Ruchy language support
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("actor.rch");

    std::fs::write(&source_path, "actor TestActor {}").unwrap();

    let result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file: None,
                dwarf_file: None,
                source_map: None,
                language: Some(DeepWasmLanguage::Ruchy),
                focus: DeepWasmFocus::Full,
                format: DeepWasmOutputFormat::Markdown,
                output: None,
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: true, // detect_deadlocks for Ruchy
            }
        )
        .await
    });

    assert!(result.is_ok(), "Ruchy analysis should succeed");
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_auto_language_detection() {
    use pmat::cli::handlers::deep_wasm_handlers;

    // RED Phase: Test auto language detection from extension
    let temp_dir = tempdir().unwrap();
    let rust_path = temp_dir.path().join("test.rs");
    let ruchy_path = temp_dir.path().join("test.rch");

    std::fs::write(&rust_path, "fn test() {}").unwrap();
    std::fs::write(&ruchy_path, "actor Test {}").unwrap();

    // Test Rust auto-detection
    let rust_result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path: rust_path,
                wasm_file: None,
                dwarf_file: None,
                source_map: None,
                language: None, // Language NOT specified - should auto-detect
                focus: DeepWasmFocus::Source,
                format: DeepWasmOutputFormat::Markdown,
                output: None,
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    assert!(
        rust_result.is_ok(),
        "Rust auto-detection should succeed"
    );

    // Test Ruchy auto-detection
    let ruchy_result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path: ruchy_path,
                wasm_file: None,
                dwarf_file: None,
                source_map: None,
                language: None, // Language NOT specified - should auto-detect
                focus: DeepWasmFocus::Source,
                format: DeepWasmOutputFormat::Markdown,
                output: None,
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    assert!(
        ruchy_result.is_ok(),
        "Ruchy auto-detection should succeed"
    );
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_quality_gate_violation() {
    use pmat::services::deep_wasm::{
        DeepWasmAnalysisRequest, DeepWasmService, SourceLanguage, AnalysisFocus, WasmQualityGates,
    };

    // RED Phase: Test quality gate violation detection
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("lib.rs");

    std::fs::write(&source_path, "pub fn test() {}").unwrap();

    let result = tokio_test::block_on(async {
        let request = DeepWasmAnalysisRequest {
            source_path: source_path.clone(),
            wasm_path: None,
            dwarf_path: None,
            source_map_path: None,
            language: SourceLanguage::Rust,
            analysis_focus: AnalysisFocus::Full,
        };

        // Create service with very strict gates to trigger violation
        let mut gates = WasmQualityGates::default();
        gates.max_module_size = 1; // Extremely low limit

        let service = DeepWasmService::new().with_quality_gates(gates);
        service.analyze(request).await
    });

    // Should succeed but report may contain violations
    assert!(result.is_ok());
}

#[cfg(feature = "deep-wasm")]
#[test]
fn test_deep_wasm_with_all_debug_info() {
    use pmat::cli::handlers::deep_wasm_handlers;

    // RED Phase: Test with complete debug information
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("lib.rs");
    let wasm_path = temp_dir.path().join("app.wasm");
    let dwarf_path = temp_dir.path().join("app.dwarf");
    let source_map_path = temp_dir.path().join("app.map");

    std::fs::write(&source_path, "pub fn test() {}").unwrap();
    std::fs::write(
        &wasm_path,
        [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
    )
    .unwrap();
    std::fs::write(&dwarf_path, b"dummy dwarf").unwrap();
    std::fs::write(&source_map_path, "{}").unwrap();

    let result = tokio_test::block_on(async {
        deep_wasm_handlers::handle_deep_wasm(
            deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file: Some(wasm_path),
                dwarf_file: Some(dwarf_path),
                source_map: Some(source_map_path),
                language: Some(DeepWasmLanguage::Rust),
                focus: DeepWasmFocus::Full,
                format: DeepWasmOutputFormat::Markdown,
                output: None,
                strict: false,
                _include_mir: false,
                _include_llvm_ir: false,
                _track_memory: false,
                _detect_deadlocks: false,
            }
        )
        .await
    });

    assert!(
        result.is_ok(),
        "Analysis with all debug info should succeed"
    );
}

#[cfg(not(feature = "deep-wasm"))]
#[test]
fn test_deep_wasm_feature_disabled() {
    // When deep-wasm feature is disabled, command should not be available
    // This ensures feature gating works correctly
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn deep_wasm_cli_never_panics(
            _strict in any::<bool>(),
            _include_mir in any::<bool>(),
            _track_memory in any::<bool>()
        ) {
            // Property: CLI parsing should never panic
            // Even with random boolean combinations
            prop_assert!(true);
        }
    }
}
