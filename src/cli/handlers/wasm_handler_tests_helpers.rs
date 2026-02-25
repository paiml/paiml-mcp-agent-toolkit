// ==================== Test Helpers ====================
// Included into coverage_tests mod in wasm_handler_tests.rs
// NO use statements allowed here - they are in the parent module.

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
