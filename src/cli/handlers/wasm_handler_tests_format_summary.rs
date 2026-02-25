// ==================== format_results Tests ====================

#[test]
fn test_format_results_summary() {
    let analysis = mock_analysis_result();
    let result = format_results(
        WasmOutputFormat::Summary,
        &analysis,
        None,
        None,
        None,
        None,
        false,
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("WASM Analysis Summary"));
    assert!(output.contains("Functions: 10"));
    assert!(output.contains("Instructions: 500"));
}

#[test]
fn test_format_results_json() {
    let analysis = mock_analysis_result();
    let result = format_results(
        WasmOutputFormat::Json,
        &analysis,
        None,
        None,
        None,
        None,
        false,
    );

    assert!(result.is_ok());
    let output = result.unwrap();

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.get("analysis").is_some());
}

#[test]
fn test_format_results_detailed_no_verbose() {
    let analysis = mock_analysis_result();
    let result = format_results(
        WasmOutputFormat::Detailed,
        &analysis,
        None,
        None,
        None,
        None,
        false,
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("WASM Analysis Summary"));
    // Should NOT contain detailed section without verbose
    assert!(!output.contains("Detailed Analysis"));
}

#[test]
fn test_format_results_detailed_with_verbose() {
    let analysis = mock_analysis_result();
    let profiling = mock_profiling_report();
    let result = format_results(
        WasmOutputFormat::Detailed,
        &analysis,
        None,
        None,
        Some(&profiling),
        None,
        true,
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Detailed Analysis"));
}

#[test]
fn test_format_results_sarif() {
    let analysis = mock_analysis_result();
    let security = mock_security_results_no_critical();
    let result = format_results(
        WasmOutputFormat::Sarif,
        &analysis,
        None,
        Some(&security),
        None,
        None,
        false,
    );

    assert!(result.is_ok());
    let output = result.unwrap();

    // Verify it's valid SARIF JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.get("$schema").is_some());
    assert!(parsed.get("version").is_some());
    assert!(parsed.get("runs").is_some());
}

#[test]
fn test_format_results_sarif_empty_security() {
    let analysis = mock_analysis_result();
    let result = format_results(
        WasmOutputFormat::Sarif,
        &analysis,
        None,
        None,
        None,
        None,
        false,
    );

    assert!(result.is_ok());
    let output = result.unwrap();

    // Should still produce valid SARIF with empty results
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.get("runs").is_some());
}

// ==================== format_summary Tests ====================

#[test]
fn test_format_summary_basic() {
    let analysis = mock_analysis_result();
    let result = format_summary(&analysis, None, None, None, None);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("WASM Analysis Summary"));
    assert!(output.contains("Functions: 10"));
    assert!(output.contains("Instructions: 500"));
    assert!(output.contains("Binary Size: 2048 bytes"));
    assert!(output.contains("Memory Pages: 2"));
    assert!(output.contains("Max Complexity: 15"));
}

#[test]
fn test_format_summary_with_verification_safe() {
    let analysis = mock_analysis_result();
    let verification = mock_verification_safe();
    let result = format_summary(&analysis, Some(&verification), None, None, None);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Verification: "));
    assert!(output.contains("SAFE"));
}

#[test]
fn test_format_summary_with_verification_unsafe() {
    let analysis = mock_analysis_result();
    let verification = mock_verification_unsafe();
    let result = format_summary(&analysis, Some(&verification), None, None, None);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("UNSAFE"));
}

#[test]
fn test_format_summary_with_security() {
    let analysis = mock_analysis_result();
    let security = mock_security_results_with_critical();
    let result = format_summary(&analysis, None, Some(&security), None, None);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Security Vulnerabilities:"));
    assert!(output.contains("Critical: 1"));
    assert!(output.contains("High: 1"));
}

#[test]
fn test_format_summary_with_profiling() {
    let analysis = mock_analysis_result();
    let profiling = mock_profiling_report();
    let result = format_summary(&analysis, None, None, Some(&profiling), None);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Performance Profile:"));
    assert!(output.contains("Control Flow:"));
    assert!(output.contains("Memory Ops:"));
}

#[test]
fn test_format_summary_with_baseline_passing() {
    let analysis = mock_analysis_result();
    let baseline = mock_quality_assessment_passing();
    let result = format_summary(&analysis, None, None, None, Some(&baseline));

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Quality Assessment:"));
    assert!(output.contains("Health Score:"));
    assert!(output.contains("PASSING"));
}

#[test]
fn test_format_summary_with_baseline_failing() {
    let analysis = mock_analysis_result();
    let baseline = mock_quality_assessment_failing();
    let result = format_summary(&analysis, None, None, None, Some(&baseline));

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("FAILING"));
}

// ==================== append_* Helper Function Tests ====================

#[test]
fn test_append_summary_header() {
    let mut output = String::new();
    append_summary_header(&mut output);

    assert!(output.contains("WASM Analysis Summary"));
    assert!(output.contains("===================="));
}

#[test]
fn test_append_basic_metrics() {
    let mut output = String::new();
    let analysis = mock_analysis_result();
    append_basic_metrics(&mut output, &analysis);

    assert!(output.contains("Functions: 10"));
    assert!(output.contains("Instructions: 500"));
    assert!(output.contains("Binary Size: 2048 bytes"));
    assert!(output.contains("Memory Pages: 2"));
    assert!(output.contains("Max Complexity: 15"));
}

#[test]
fn test_append_verification_status_none() {
    let mut output = String::new();
    append_verification_status(&mut output, None);
    assert!(output.is_empty());
}

#[test]
fn test_append_verification_status_safe() {
    let mut output = String::new();
    let verification = mock_verification_safe();
    append_verification_status(&mut output, Some(&verification));

    assert!(output.contains("Verification:"));
    assert!(output.contains("SAFE"));
}

#[test]
fn test_append_security_summary_none() {
    let mut output = String::new();
    append_security_summary(&mut output, None);
    assert!(output.is_empty());
}

#[test]
fn test_append_security_summary_with_all_severities() {
    let mut output = String::new();
    let mut security = mock_security_results_with_critical();
    security.push(VulnerabilityMatch {
        pattern: "medium".to_string(),
        location: 0..10,
        severity: Severity::Medium,
        operator_index: 1,
    });
    security.push(VulnerabilityMatch {
        pattern: "low".to_string(),
        location: 10..20,
        severity: Severity::Low,
        operator_index: 2,
    });
    append_security_summary(&mut output, Some(&security));

    assert!(output.contains("Critical: 1"));
    assert!(output.contains("High: 1"));
    assert!(output.contains("Medium: 1"));
    assert!(output.contains("Low: 1"));
}

#[test]
fn test_append_profiling_summary_none() {
    let mut output = String::new();
    append_profiling_summary(&mut output, None);
    assert!(output.is_empty());
}

#[test]
fn test_append_profiling_summary_with_data() {
    let mut output = String::new();
    let profiling = mock_profiling_report();
    append_profiling_summary(&mut output, Some(&profiling));

    assert!(output.contains("Performance Profile:"));
    assert!(output.contains("Control Flow:"));
    assert!(output.contains("Memory Ops:"));
    assert!(output.contains("Arithmetic:"));
    assert!(output.contains("Function Calls:"));
}

#[test]
fn test_append_baseline_summary_none() {
    let mut output = String::new();
    append_baseline_summary(&mut output, None);
    assert!(output.is_empty());
}

#[test]
fn test_append_baseline_summary_passing() {
    let mut output = String::new();
    let baseline = mock_quality_assessment_passing();
    append_baseline_summary(&mut output, Some(&baseline));

    assert!(output.contains("Quality Assessment:"));
    assert!(output.contains("Health Score: 95.0%"));
    assert!(output.contains("PASSING"));
}

// ==================== count_by_severity Tests ====================

#[test]
fn test_count_by_severity_empty() {
    let vulns: Vec<VulnerabilityMatch> = vec![];
    assert_eq!(count_by_severity(&vulns, Severity::Critical), 0);
    assert_eq!(count_by_severity(&vulns, Severity::High), 0);
    assert_eq!(count_by_severity(&vulns, Severity::Medium), 0);
    assert_eq!(count_by_severity(&vulns, Severity::Low), 0);
}

#[test]
fn test_count_by_severity_mixed() {
    let vulns = vec![
        VulnerabilityMatch {
            pattern: "a".to_string(),
            location: 0..10,
            severity: Severity::Critical,
            operator_index: 1,
        },
        VulnerabilityMatch {
            pattern: "b".to_string(),
            location: 10..20,
            severity: Severity::Critical,
            operator_index: 2,
        },
        VulnerabilityMatch {
            pattern: "c".to_string(),
            location: 20..30,
            severity: Severity::High,
            operator_index: 3,
        },
        VulnerabilityMatch {
            pattern: "d".to_string(),
            location: 30..40,
            severity: Severity::Medium,
            operator_index: 4,
        },
    ];

    assert_eq!(count_by_severity(&vulns, Severity::Critical), 2);
    assert_eq!(count_by_severity(&vulns, Severity::High), 1);
    assert_eq!(count_by_severity(&vulns, Severity::Medium), 1);
    assert_eq!(count_by_severity(&vulns, Severity::Low), 0);
}

// ==================== calculate_percentage Tests ====================

#[test]
fn test_calculate_percentage_zero_total() {
    assert_eq!(calculate_percentage(10, 0), 0);
}

#[test]
fn test_calculate_percentage_zero_part() {
    assert_eq!(calculate_percentage(0, 100), 0);
}

#[test]
fn test_calculate_percentage_half() {
    assert_eq!(calculate_percentage(50, 100), 50);
}

#[test]
fn test_calculate_percentage_full() {
    assert_eq!(calculate_percentage(100, 100), 100);
}

#[test]
fn test_calculate_percentage_integer_division() {
    // 33 * 100 / 100 = 33 (not 33.333...)
    assert_eq!(calculate_percentage(33, 100), 33);
}

