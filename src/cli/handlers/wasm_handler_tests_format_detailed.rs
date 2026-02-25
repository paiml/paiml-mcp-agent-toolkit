// ==================== format_json Tests ====================

#[test]
fn test_format_json_minimal() {
    let analysis = mock_analysis_result();
    let result = format_json(&analysis, None, None, None, None);

    assert!(result.is_ok());
    let output = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed["analysis"]["function_count"].as_i64().unwrap() == 10);
    assert!(parsed["verification"].is_null());
    assert!(parsed["security"].is_null());
    assert!(parsed["profiling"].is_null());
    assert!(parsed["baseline"].is_null());
}

#[test]
fn test_format_json_full() {
    let analysis = mock_analysis_result();
    let verification = mock_verification_safe();
    let security = mock_security_results_no_critical();
    let profiling = mock_profiling_report();
    let baseline = mock_quality_assessment_passing();

    let result = format_json(
        &analysis,
        Some(&verification),
        Some(&security),
        Some(&profiling),
        Some(&baseline),
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed["analysis"].is_object());
    assert!(parsed["verification"].is_string()); // "Safe" serializes as string
    assert!(parsed["security"].is_array());
    assert!(parsed["profiling"].is_object());
    assert!(parsed["baseline"].is_object());
}

// ==================== format_detailed Tests ====================

#[test]
fn test_format_detailed_not_verbose() {
    let analysis = mock_analysis_result();
    let result = format_detailed(&analysis, None, None, None, None, false);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("WASM Analysis Summary"));
    assert!(!output.contains("Detailed Analysis"));
}

#[test]
fn test_format_detailed_verbose_with_profiling() {
    let analysis = mock_analysis_result();
    let profiling = mock_profiling_report();
    let result = format_detailed(&analysis, None, None, Some(&profiling), None, true);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Detailed Analysis"));
    assert!(output.contains("Instruction Breakdown:"));
    assert!(output.contains("Hot Functions:"));
}

#[test]
fn test_format_detailed_verbose_with_security() {
    let analysis = mock_analysis_result();
    let security = mock_security_results_no_critical();
    let result = format_detailed(&analysis, None, Some(&security), None, None, true);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Vulnerability Details:"));
}

// ==================== append_detailed_* Tests ====================

#[test]
fn test_append_detailed_information() {
    let mut output = String::new();
    let profiling = mock_profiling_report();
    let security = mock_security_results_no_critical();
    append_detailed_information(&mut output, Some(&profiling), Some(&security));

    assert!(output.contains("Detailed Analysis"));
    assert!(output.contains("================="));
}

#[test]
fn test_append_detailed_profiling_none() {
    let mut output = String::new();
    append_detailed_profiling(&mut output, None);
    assert!(output.is_empty());
}

#[test]
fn test_append_detailed_profiling_with_data() {
    let mut output = String::new();
    let profiling = mock_profiling_report();
    append_detailed_profiling(&mut output, Some(&profiling));

    assert!(output.contains("Instruction Breakdown:"));
    assert!(output.contains("Total: 100"));
    assert!(output.contains("Control Flow: 20"));
    assert!(output.contains("Memory Operations: 30"));
    assert!(output.contains("Arithmetic: 40"));
    assert!(output.contains("Calls: 10"));
}

#[test]
fn test_append_hot_functions_empty() {
    let mut output = String::new();
    let profiling = ProfilingReport {
        instruction_mix: InstructionMix {
            total_instructions: 100,
            control_flow: 20,
            memory_ops: 30,
            arithmetic: 40,
            calls: 10,
        },
        hot_functions: vec![], // Empty
        memory_usage: MemoryProfile {
            initial_pages: 1,
            max_pages: None,
            growth_events: vec![],
        },
    };
    append_hot_functions(&mut output, &profiling);

    assert!(!output.contains("Hot Functions:"));
}

#[test]
fn test_append_hot_functions_with_data() {
    let mut output = String::new();
    let profiling = mock_profiling_report();
    append_hot_functions(&mut output, &profiling);

    assert!(output.contains("Hot Functions:"));
    assert!(output.contains("func_0"));
    assert!(output.contains("50.0%"));
    assert!(output.contains("500 samples"));
}

#[test]
fn test_append_detailed_vulnerabilities_none() {
    let mut output = String::new();
    append_detailed_vulnerabilities(&mut output, None);
    assert!(output.is_empty());
}

#[test]
fn test_append_detailed_vulnerabilities_empty() {
    let mut output = String::new();
    let security: Vec<VulnerabilityMatch> = vec![];
    append_detailed_vulnerabilities(&mut output, Some(&security));
    assert!(!output.contains("Vulnerability Details:"));
}

#[test]
fn test_append_detailed_vulnerabilities_with_data() {
    let mut output = String::new();
    let security = mock_security_results_no_critical();
    append_detailed_vulnerabilities(&mut output, Some(&security));

    assert!(output.contains("Vulnerability Details:"));
    assert!(output.contains("potential-integer-overflow"));
    assert!(output.contains("offset 42"));
}
