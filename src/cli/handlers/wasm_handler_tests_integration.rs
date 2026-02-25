// ==================== Integration Tests ====================

#[tokio::test]
async fn test_handle_analyze_wasm_minimal() {
    let file = create_wasm_temp_file(&minimal_wasm_module());
    let result = handle_analyze_wasm(
        file.path().to_path_buf(),
        WasmOutputFormat::Summary,
        false,
        false,
        false,
        None,
        None,
        false,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_wasm_with_all_options() {
    let file = create_wasm_temp_file(&simple_function_wasm());
    let output_file = NamedTempFile::new().expect("Failed to create temp file");

    let result = handle_analyze_wasm(
        file.path().to_path_buf(),
        WasmOutputFormat::Json,
        true, // verify
        true, // security
        true, // profile
        None, // baseline
        Some(output_file.path().to_path_buf()),
        true, // verbose
    )
    .await;

    assert!(result.is_ok());

    // Verify output was written
    let contents = std::fs::read_to_string(output_file.path()).expect("Failed to read output");
    assert!(!contents.is_empty());
}

#[tokio::test]
async fn test_handle_analyze_wasm_file_not_found() {
    let result = handle_analyze_wasm(
        PathBuf::from("/nonexistent/file.wasm"),
        WasmOutputFormat::Summary,
        false,
        false,
        false,
        None,
        None,
        false,
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_analyze_wasm_with_baseline() {
    let main_file = create_wasm_temp_file(&minimal_wasm_module());
    let baseline_file = create_wasm_temp_file(&minimal_wasm_module());

    let result = handle_analyze_wasm(
        main_file.path().to_path_buf(),
        WasmOutputFormat::Summary,
        false,
        false,
        false,
        Some(baseline_file.path().to_path_buf()),
        None,
        false,
    )
    .await;

    assert!(result.is_ok());
}
