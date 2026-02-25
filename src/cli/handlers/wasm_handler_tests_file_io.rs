// ==================== load_wasm_file Tests ====================

#[test]
fn test_load_wasm_file_valid() {
    let file = create_wasm_temp_file(&minimal_wasm_module());
    let result = load_wasm_file(&file.path().to_path_buf());

    assert!(result.is_ok());
    let binary = result.unwrap();
    assert_eq!(binary.len(), 8); // Magic + version
}

#[test]
fn test_load_wasm_file_with_content() {
    let wasm = simple_function_wasm();
    let file = create_wasm_temp_file(&wasm);
    let result = load_wasm_file(&file.path().to_path_buf());

    assert!(result.is_ok());
    let binary = result.unwrap();
    assert_eq!(binary.len(), wasm.len());
}

#[test]
fn test_load_wasm_file_not_found() {
    let path = PathBuf::from("/nonexistent/path/to/file.wasm");
    let result = load_wasm_file(&path);

    assert!(result.is_err());
}

// ==================== run_basic_analysis Tests ====================

#[test]
fn test_run_basic_analysis_minimal() {
    let wasm = minimal_wasm_module();
    let result = run_basic_analysis(&wasm);

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.function_count, 0);
    assert_eq!(analysis.instruction_count, 0);
}

#[test]
fn test_run_basic_analysis_with_function() {
    let wasm = simple_function_wasm();
    let result = run_basic_analysis(&wasm);

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert!(analysis.instruction_count > 0);
}

#[test]
fn test_run_basic_analysis_invalid_wasm() {
    let invalid = vec![0x00, 0x01, 0x02, 0x03];
    let result = run_basic_analysis(&invalid);

    assert!(result.is_err());
}

// ==================== run_verification_if_requested Tests ====================

#[tokio::test]
async fn test_run_verification_not_requested() {
    let wasm = minimal_wasm_module();
    let result = run_verification_if_requested(false, &wasm).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_run_verification_requested_valid() {
    let wasm = minimal_wasm_module();
    let result = run_verification_if_requested(true, &wasm).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(verification.is_some());
    assert!(verification.unwrap().is_safe());
}

#[tokio::test]
async fn test_run_verification_requested_with_function() {
    let wasm = simple_function_wasm();
    let result = run_verification_if_requested(true, &wasm).await;

    assert!(result.is_ok());
    let verification = result.unwrap();
    assert!(verification.is_some());
}

// ==================== run_security_scan_if_requested Tests ====================

#[test]
fn test_run_security_scan_not_requested() {
    let wasm = minimal_wasm_module();
    let result = run_security_scan_if_requested(false, &wasm);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_run_security_scan_requested_minimal() {
    let wasm = minimal_wasm_module();
    let result = run_security_scan_if_requested(true, &wasm);

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_some());
}

#[test]
fn test_run_security_scan_requested_with_function() {
    let wasm = simple_function_wasm();
    let result = run_security_scan_if_requested(true, &wasm);

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_some());
}

// ==================== run_profiling_if_requested Tests ====================

#[tokio::test]
async fn test_run_profiling_not_requested() {
    let wasm = minimal_wasm_module();
    let result = run_profiling_if_requested(false, &wasm).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_run_profiling_requested_minimal() {
    let wasm = minimal_wasm_module();
    let result = run_profiling_if_requested(true, &wasm).await;

    assert!(result.is_ok());
    let profiling = result.unwrap();
    assert!(profiling.is_some());
}

#[tokio::test]
async fn test_run_profiling_requested_with_function() {
    let wasm = simple_function_wasm();
    let result = run_profiling_if_requested(true, &wasm).await;

    assert!(result.is_ok());
    let profiling = result.unwrap();
    assert!(profiling.is_some());
    let report = profiling.unwrap();
    assert!(report.instruction_mix.total_instructions > 0);
}

// ==================== run_baseline_comparison_if_requested Tests ====================

#[tokio::test]
async fn test_run_baseline_comparison_not_requested() {
    let wasm = minimal_wasm_module();
    let analysis = mock_analysis_result();
    let result = run_baseline_comparison_if_requested(None, &wasm, &analysis).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_run_baseline_comparison_requested() {
    let wasm = minimal_wasm_module();
    let baseline_file = create_wasm_temp_file(&wasm);
    let analysis = mock_analysis_result();

    let result = run_baseline_comparison_if_requested(
        Some(baseline_file.path().to_path_buf()),
        &wasm,
        &analysis,
    )
    .await;

    assert!(result.is_ok());
    let comparison = result.unwrap();
    assert!(comparison.is_some());
}

#[tokio::test]
async fn test_run_baseline_comparison_nonexistent_file() {
    let wasm = minimal_wasm_module();
    let analysis = mock_analysis_result();
    let nonexistent = PathBuf::from("/nonexistent/baseline.wasm");

    let result =
        run_baseline_comparison_if_requested(Some(nonexistent), &wasm, &analysis).await;

    assert!(result.is_err());
}

// ==================== load_and_analyze_baseline Tests ====================

#[test]
fn test_load_and_analyze_baseline_valid() {
    let wasm = minimal_wasm_module();
    let file = create_wasm_temp_file(&wasm);
    let result = load_and_analyze_baseline(&file.path().to_path_buf());

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.function_count == 0); // Minimal module has no functions
}

#[test]
fn test_load_and_analyze_baseline_with_function() {
    let wasm = simple_function_wasm();
    let file = create_wasm_temp_file(&wasm);
    let result = load_and_analyze_baseline(&file.path().to_path_buf());

    assert!(result.is_ok());
}

#[test]
fn test_load_and_analyze_baseline_not_found() {
    let path = PathBuf::from("/nonexistent/baseline.wasm");
    let result = load_and_analyze_baseline(&path);

    assert!(result.is_err());
}

// ==================== write_output Tests ====================

#[test]
fn test_write_output_to_stdout() {
    let output = "Test output".to_string();
    let result = write_output(output, None);

    assert!(result.is_ok());
}

#[test]
fn test_write_output_to_file() {
    let file = NamedTempFile::new().expect("Failed to create temp file");
    let output = "Test output to file".to_string();
    let result = write_output(output.clone(), Some(file.path().to_path_buf()));

    assert!(result.is_ok());

    // Verify file contents
    let contents = std::fs::read_to_string(file.path()).expect("Failed to read file");
    assert_eq!(contents, output);
}

#[test]
fn test_write_output_to_invalid_path() {
    let output = "Test output".to_string();
    let invalid_path = PathBuf::from("/nonexistent/directory/output.txt");
    let result = write_output(output, Some(invalid_path));

    assert!(result.is_err());
}
