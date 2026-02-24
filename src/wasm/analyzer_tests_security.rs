// Security and operator categorization tests
// Included from analyzer.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_security {
    use super::*;

    // Minimal valid WASM module (empty module with proper header)
    fn minimal_wasm_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
        ]
    }

    // ==================== SecurityAuditor Tests ====================

    #[test]
    fn test_security_auditor_new() {
        let auditor = SecurityAuditor::new();
        assert_eq!(auditor.checks.len(), 5);
    }

    #[test]
    fn test_security_auditor_default() {
        let auditor = SecurityAuditor::default();
        assert_eq!(auditor.checks.len(), 5);
    }

    #[test]
    fn test_security_auditor_audit() {
        let auditor = SecurityAuditor::new();
        let result = auditor.audit(&minimal_wasm_module());

        assert!(result.is_ok());
        let report = result.unwrap();
        // All default checks should pass on minimal module
        assert!(report.is_safe);
        assert!(!report.passed_checks.is_empty());
    }

    // ==================== SecurityReport Tests ====================

    #[test]
    fn test_security_report_new() {
        let report = SecurityReport::new();
        assert!(report.passed_checks.is_empty());
        assert!(report.failed_checks.is_empty());
        assert!(report.warnings.is_empty());
        assert!(report.is_safe);
    }

    #[test]
    fn test_security_report_default() {
        let report = SecurityReport::default();
        assert!(report.is_safe);
    }

    #[test]
    fn test_security_report_add_check_passed() {
        let mut report = SecurityReport::new();
        report.add_check_result("test-check", true);

        assert_eq!(report.passed_checks.len(), 1);
        assert!(report.passed_checks.contains(&"test-check".to_string()));
        assert!(report.is_safe);
    }

    #[test]
    fn test_security_report_add_check_failed() {
        let mut report = SecurityReport::new();
        report.add_check_result("test-check", false);

        assert_eq!(report.failed_checks.len(), 1);
        assert!(report.failed_checks.contains(&"test-check".to_string()));
        assert!(!report.is_safe);
    }

    #[test]
    fn test_security_report_mixed_results() {
        let mut report = SecurityReport::new();
        report.add_check_result("check-1", true);
        report.add_check_result("check-2", false);
        report.add_check_result("check-3", true);

        assert_eq!(report.passed_checks.len(), 2);
        assert_eq!(report.failed_checks.len(), 1);
        assert!(!report.is_safe);
    }

    #[test]
    fn test_security_report_serialization() {
        let mut report = SecurityReport::new();
        report.add_check_result("memory-bounds", true);
        report.add_check_result("integer-overflow", false);

        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: SecurityReport = serde_json::from_str(&serialized).unwrap();

        assert_eq!(report.passed_checks, deserialized.passed_checks);
        assert_eq!(report.failed_checks, deserialized.failed_checks);
        assert_eq!(report.is_safe, deserialized.is_safe);
    }

    // ==================== SecurityCheck Tests ====================

    #[test]
    fn test_security_check_names() {
        let checks = vec![
            SecurityCheck::NoFilesystemAccess,
            SecurityCheck::NoNetworkAccess,
            SecurityCheck::MemoryBoundsChecked,
            SecurityCheck::NoUnvalidatedIndirectCalls,
            SecurityCheck::NoIntegerOverflow,
        ];

        let names: Vec<_> = checks.iter().map(|c| c.name()).collect();

        assert!(names.contains(&"no-filesystem-access"));
        assert!(names.contains(&"no-network-access"));
        assert!(names.contains(&"memory-bounds-checked"));
        assert!(names.contains(&"no-unvalidated-indirect-calls"));
        assert!(names.contains(&"no-integer-overflow"));
    }

    #[test]
    fn test_security_check_verify_all_pass() {
        let checks = vec![
            SecurityCheck::NoFilesystemAccess,
            SecurityCheck::NoNetworkAccess,
            SecurityCheck::MemoryBoundsChecked,
            SecurityCheck::NoUnvalidatedIndirectCalls,
            SecurityCheck::NoIntegerOverflow,
        ];

        let binary = minimal_wasm_module();

        for check in checks {
            assert!(check.verify(&binary));
        }
    }

    // ==================== categorize_operator Tests ====================

    #[test]
    fn test_categorize_operator_control() {
        use wasmparser::Operator;

        assert_eq!(
            categorize_operator(&Operator::Block {
                blockty: wasmparser::BlockType::Empty
            }),
            "control"
        );
        assert_eq!(
            categorize_operator(&Operator::Loop {
                blockty: wasmparser::BlockType::Empty
            }),
            "control"
        );
        assert_eq!(
            categorize_operator(&Operator::If {
                blockty: wasmparser::BlockType::Empty
            }),
            "control"
        );
        assert_eq!(categorize_operator(&Operator::Else), "control");
        assert_eq!(categorize_operator(&Operator::End), "control");
        assert_eq!(
            categorize_operator(&Operator::Br { relative_depth: 0 }),
            "control"
        );
        assert_eq!(categorize_operator(&Operator::Return), "control");
    }

    #[test]
    fn test_categorize_operator_memory() {
        use wasmparser::{MemArg, Operator};

        let memarg = MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };

        assert_eq!(categorize_operator(&Operator::I32Load { memarg }), "memory");
        assert_eq!(
            categorize_operator(&Operator::I32Store { memarg }),
            "memory"
        );
        assert_eq!(
            categorize_operator(&Operator::MemoryGrow { mem: 0 }),
            "memory"
        );
    }

    #[test]
    fn test_categorize_operator_call() {
        use wasmparser::Operator;

        assert_eq!(
            categorize_operator(&Operator::Call { function_index: 0 }),
            "call"
        );
    }

    #[test]
    fn test_categorize_operator_arithmetic() {
        use wasmparser::Operator;

        assert_eq!(categorize_operator(&Operator::I32Add), "arithmetic");
        assert_eq!(categorize_operator(&Operator::I32Sub), "arithmetic");
        assert_eq!(categorize_operator(&Operator::I32Mul), "arithmetic");
        assert_eq!(categorize_operator(&Operator::F64Div), "arithmetic");
    }

    #[test]
    fn test_categorize_operator_other() {
        use wasmparser::Operator;

        assert_eq!(categorize_operator(&Operator::Nop), "other");
        assert_eq!(
            categorize_operator(&Operator::I32Const { value: 0 }),
            "other"
        );
        assert_eq!(categorize_operator(&Operator::Drop), "other");
    }
}
