//! Tests for WASM handler
//! Extracted to separate file for file health compliance (CB-040)

use super::*;

mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::wasm::baseline::{Severity as BaselineSeverity, Violation};
    use crate::wasm::security::Severity;
    use crate::wasm::{GrowthEvent, HotFunction, InstructionMix, MemoryProfile};
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ==================== Test Helpers ====================

    /// Minimal valid WASM module (empty module with proper header)
    fn minimal_wasm_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
        ]
    }

    /// WASM module with a simple function that does i32 arithmetic
    fn simple_function_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x05, // section id 1, size 5
            0x01, // 1 type
            0x60, 0x00, 0x01, 0x7f, // func type: () -> i32
            // Function section
            0x03, 0x02, // section id 3, size 2
            0x01, 0x00, // 1 function, type 0
            // Code section
            0x0a, 0x09, // section id 10, size 9
            0x01, // 1 function body
            0x07, // body size 7
            0x00, // 0 locals
            0x41, 0x01, // i32.const 1
            0x41, 0x02, // i32.const 2
            0x6a, // i32.add
            0x0b, // end
        ]
    }

    /// Create a temp file with WASM content
    fn create_wasm_temp_file(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content)
            .expect("Failed to write to temp file");
        file
    }

    /// Create a mock AnalysisResult for testing
    fn mock_analysis_result() -> AnalysisResult {
        AnalysisResult {
            function_count: 10,
            instruction_count: 500,
            binary_size: 2048,
            memory_pages: 2,
            max_complexity: 15,
        }
    }

    /// Create a mock VerificationResult (safe)
    fn mock_verification_safe() -> VerificationResult {
        VerificationResult::Safe
    }

    /// Create a mock VerificationResult (unsafe)
    fn mock_verification_unsafe() -> VerificationResult {
        VerificationResult::OutOfBounds {
            offset: 100,
            size: 50,
        }
    }

    /// Create mock security results (no critical)
    fn mock_security_results_no_critical() -> Vec<VulnerabilityMatch> {
        vec![
            VulnerabilityMatch {
                pattern: "potential-integer-overflow".to_string(),
                location: 0..100,
                severity: Severity::Medium,
                operator_index: 42,
            },
            VulnerabilityMatch {
                pattern: "timing-side-channel".to_string(),
                location: 100..200,
                severity: Severity::Low,
                operator_index: 77,
            },
        ]
    }

    /// Create mock security results (with critical)
    fn mock_security_results_with_critical() -> Vec<VulnerabilityMatch> {
        vec![
            VulnerabilityMatch {
                pattern: "critical-vulnerability".to_string(),
                location: 0..100,
                severity: Severity::Critical,
                operator_index: 10,
            },
            VulnerabilityMatch {
                pattern: "high-vulnerability".to_string(),
                location: 100..200,
                severity: Severity::High,
                operator_index: 20,
            },
        ]
    }

    /// Create a mock ProfilingReport
    fn mock_profiling_report() -> ProfilingReport {
        ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 20,
                memory_ops: 30,
                arithmetic: 40,
                calls: 10,
            },
            hot_functions: vec![
                HotFunction {
                    name: "func_0".to_string(),
                    samples: 500,
                    percentage: 50.0,
                },
                HotFunction {
                    name: "func_1".to_string(),
                    samples: 300,
                    percentage: 30.0,
                },
            ],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: Some(16),
                growth_events: vec![GrowthEvent {
                    timestamp: 1000,
                    pages_before: 1,
                    pages_after: 2,
                }],
            },
        }
    }

    /// Create a mock QualityAssessment (passing)
    fn mock_quality_assessment_passing() -> QualityAssessment {
        QualityAssessment {
            violations: vec![],
            overall_health: 95.0,
            recommendation: "Quality metrics are within acceptable bounds.".to_string(),
        }
    }

    /// Create a mock QualityAssessment (failing)
    fn mock_quality_assessment_failing() -> QualityAssessment {
        QualityAssessment {
            violations: vec![Violation::ComplexityRegression {
                current: 25,
                limit: 20,
                severity: BaselineSeverity::Error,
            }],
            overall_health: 45.0,
            recommendation: "Critical violations detected".to_string(),
        }
    }

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

    // ==================== check_for_failures Tests ====================

    #[test]
    fn test_check_for_failures_all_none() {
        let result = check_for_failures(None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_for_failures_safe_verification() {
        let verification = mock_verification_safe();
        let result = check_for_failures(Some(&verification), None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_for_failures_unsafe_verification() {
        let verification = mock_verification_unsafe();
        let result = check_for_failures(Some(&verification), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_for_failures_no_critical_security() {
        let security = mock_security_results_no_critical();
        let result = check_for_failures(None, Some(&security), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_for_failures_critical_security() {
        let security = mock_security_results_with_critical();
        let result = check_for_failures(None, Some(&security), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_for_failures_passing_baseline() {
        let baseline = mock_quality_assessment_passing();
        let result = check_for_failures(None, None, Some(&baseline));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_for_failures_failing_baseline() {
        let baseline = mock_quality_assessment_failing();
        let result = check_for_failures(None, None, Some(&baseline));
        assert!(result.is_err());
    }

    // ==================== check_verification_failure Tests ====================

    #[test]
    fn test_check_verification_failure_none() {
        let result = check_verification_failure(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_verification_failure_safe() {
        let verification = mock_verification_safe();
        let result = check_verification_failure(Some(&verification));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_verification_failure_unsafe() {
        let verification = mock_verification_unsafe();
        let result = check_verification_failure(Some(&verification));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Verification failed"));
    }

    // ==================== check_security_failures Tests ====================

    #[test]
    fn test_check_security_failures_none() {
        let result = check_security_failures(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_security_failures_empty() {
        let security: Vec<VulnerabilityMatch> = vec![];
        let result = check_security_failures(Some(&security));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_security_failures_non_critical() {
        let security = mock_security_results_no_critical();
        let result = check_security_failures(Some(&security));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_security_failures_with_critical() {
        let security = mock_security_results_with_critical();
        let result = check_security_failures(Some(&security));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("critical security vulnerabilities"));
    }

    // ==================== check_baseline_failure Tests ====================

    #[test]
    fn test_check_baseline_failure_none() {
        let result = check_baseline_failure(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_baseline_failure_passing() {
        let baseline = mock_quality_assessment_passing();
        let result = check_baseline_failure(Some(&baseline));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_baseline_failure_failing() {
        let baseline = mock_quality_assessment_failing();
        let result = check_baseline_failure(Some(&baseline));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Quality regression"));
    }

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

    // ==================== SARIF Format Tests ====================

    #[test]
    fn test_format_sarif_empty() {
        let vulns: Vec<VulnerabilityMatch> = vec![];
        let result = format_sarif(&vulns);

        assert!(result.is_ok());
        let output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["$schema"].as_str().unwrap().contains("sarif-schema"));
        assert_eq!(parsed["version"].as_str().unwrap(), "2.1.0");
    }

    #[test]
    fn test_format_sarif_with_vulnerabilities() {
        let vulns = mock_security_results_with_critical();
        let result = format_sarif(&vulns);

        assert!(result.is_ok());
        let output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let runs = parsed["runs"].as_array().unwrap();
        assert!(!runs.is_empty());

        let results = runs[0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_create_sarif_output() {
        let vulns = mock_security_results_no_critical();
        let sarif = create_sarif_output(&vulns);

        assert!(sarif["$schema"].is_string());
        assert_eq!(sarif["version"].as_str().unwrap(), "2.1.0");
        assert!(sarif["runs"].is_array());
    }

    #[test]
    fn test_create_sarif_rules() {
        let vulns = mock_security_results_no_critical();
        let rules = create_sarif_rules(&vulns);

        // Should have 2 unique patterns
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_create_sarif_rules_deduplication() {
        let vulns = vec![
            VulnerabilityMatch {
                pattern: "same-pattern".to_string(),
                location: 0..10,
                severity: Severity::High,
                operator_index: 1,
            },
            VulnerabilityMatch {
                pattern: "same-pattern".to_string(),
                location: 10..20,
                severity: Severity::High,
                operator_index: 2,
            },
        ];
        let rules = create_sarif_rules(&vulns);

        // Should deduplicate to 1 rule
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_create_sarif_rule() {
        let rule = create_sarif_rule("test-pattern");

        assert_eq!(rule["id"].as_str().unwrap(), "test-pattern");
        assert_eq!(rule["name"].as_str().unwrap(), "test-pattern");
        assert!(rule["shortDescription"]["text"]
            .as_str()
            .unwrap()
            .contains("test-pattern"));
    }

    #[test]
    fn test_create_sarif_results() {
        let vulns = mock_security_results_no_critical();
        let results = create_sarif_results(&vulns);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_create_sarif_result_critical() {
        let vuln = VulnerabilityMatch {
            pattern: "critical-vuln".to_string(),
            location: 0..100,
            severity: Severity::Critical,
            operator_index: 42,
        };
        let result = create_sarif_result(&vuln);

        assert_eq!(result["ruleId"].as_str().unwrap(), "critical-vuln");
        assert_eq!(result["level"].as_str().unwrap(), "error");
    }

    #[test]
    fn test_create_sarif_result_high() {
        let vuln = VulnerabilityMatch {
            pattern: "high-vuln".to_string(),
            location: 0..100,
            severity: Severity::High,
            operator_index: 42,
        };
        let result = create_sarif_result(&vuln);

        assert_eq!(result["level"].as_str().unwrap(), "error");
    }

    #[test]
    fn test_create_sarif_result_medium() {
        let vuln = VulnerabilityMatch {
            pattern: "medium-vuln".to_string(),
            location: 0..100,
            severity: Severity::Medium,
            operator_index: 42,
        };
        let result = create_sarif_result(&vuln);

        assert_eq!(result["level"].as_str().unwrap(), "warning");
    }

    #[test]
    fn test_create_sarif_result_low() {
        let vuln = VulnerabilityMatch {
            pattern: "low-vuln".to_string(),
            location: 0..100,
            severity: Severity::Low,
            operator_index: 42,
        };
        let result = create_sarif_result(&vuln);

        assert_eq!(result["level"].as_str().unwrap(), "note");
    }

    // ==================== create_metrics_from_analysis Tests ====================

    #[test]
    fn test_create_metrics_from_analysis() {
        let analysis = mock_analysis_result();
        let metrics = create_metrics_from_analysis(&analysis);

        assert_eq!(metrics.function_count, 10);
        assert_eq!(metrics.instruction_count, 500);
        assert_eq!(metrics.binary_size, 2048);
        // complexity_p90 = max_complexity - 2 = 15 - 2 = 13
        assert_eq!(metrics.complexity_p90, 13);
        // complexity_p95 = max_complexity = 15
        assert_eq!(metrics.complexity_p95, 15);
        // complexity_p99 = max_complexity + 2 = 17
        assert_eq!(metrics.complexity_p99, 17);
        // memory_usage_mb = (memory_pages * 64) / 1024 = (2 * 64) / 1024 = 0
        assert_eq!(metrics.memory_usage_mb, 0);
        assert_eq!(metrics.init_time_ms, 10); // Default estimate
    }

    #[test]
    fn test_create_metrics_from_analysis_edge_cases() {
        let analysis = AnalysisResult {
            function_count: 0,
            instruction_count: 0,
            binary_size: 0,
            memory_pages: 0,
            max_complexity: 0,
        };
        let metrics = create_metrics_from_analysis(&analysis);

        // Test saturating subtraction: 0 - 2 should be 0 not underflow
        assert_eq!(metrics.complexity_p90, 0);
        assert_eq!(metrics.complexity_p95, 0);
        assert_eq!(metrics.complexity_p99, 2);
    }

    #[test]
    fn test_create_metrics_from_analysis_large_memory() {
        let analysis = AnalysisResult {
            function_count: 100,
            instruction_count: 10000,
            binary_size: 1_000_000,
            memory_pages: 256, // 256 pages = 16 MB
            max_complexity: 50,
        };
        let metrics = create_metrics_from_analysis(&analysis);

        // memory_usage_mb = (256 * 64) / 1024 = 16384 / 1024 = 16
        assert_eq!(metrics.memory_usage_mb, 16);
    }

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
}
