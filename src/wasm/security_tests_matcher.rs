// Tests for OperatorMatcher, Severity, VulnerabilityMatch, and default patterns.
// Included from security.rs -- shares parent module scope (no `use` imports here).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_matcher {
    use super::*;

    // ==================== OperatorMatcher Tests ====================

    #[test]
    fn test_operator_matcher_i32_add() {
        assert!(OperatorMatcher::I32Add.matches(&Operator::I32Add));
        assert!(!OperatorMatcher::I32Add.matches(&Operator::I32Sub));
    }

    #[test]
    fn test_operator_matcher_i32_sub() {
        assert!(OperatorMatcher::I32Sub.matches(&Operator::I32Sub));
        assert!(!OperatorMatcher::I32Sub.matches(&Operator::I32Add));
    }

    #[test]
    fn test_operator_matcher_i32_mul() {
        assert!(OperatorMatcher::I32Mul.matches(&Operator::I32Mul));
    }

    #[test]
    fn test_operator_matcher_i32_div_s() {
        assert!(OperatorMatcher::I32DivS.matches(&Operator::I32DivS));
    }

    #[test]
    fn test_operator_matcher_i32_div_u() {
        assert!(OperatorMatcher::I32DivU.matches(&Operator::I32DivU));
    }

    #[test]
    fn test_operator_matcher_i32_rem_u() {
        assert!(OperatorMatcher::I32RemU.matches(&Operator::I32RemU));
    }

    #[test]
    fn test_operator_matcher_i32_and() {
        assert!(OperatorMatcher::I32And.matches(&Operator::I32And));
    }

    #[test]
    fn test_operator_matcher_i32_or() {
        assert!(OperatorMatcher::I32Or.matches(&Operator::I32Or));
    }

    #[test]
    fn test_operator_matcher_i32_xor() {
        assert!(OperatorMatcher::I32Xor.matches(&Operator::I32Xor));
    }

    #[test]
    fn test_operator_matcher_comparisons() {
        assert!(OperatorMatcher::I32Eqz.matches(&Operator::I32Eqz));
        assert!(OperatorMatcher::I32Eq.matches(&Operator::I32Eq));
        assert!(OperatorMatcher::I32Ne.matches(&Operator::I32Ne));
        assert!(OperatorMatcher::I32LtS.matches(&Operator::I32LtS));
        assert!(OperatorMatcher::I32LtU.matches(&Operator::I32LtU));
        assert!(OperatorMatcher::I32GtS.matches(&Operator::I32GtS));
        assert!(OperatorMatcher::I32GtU.matches(&Operator::I32GtU));
    }

    #[test]
    fn test_operator_matcher_memory_ops() {
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };
        assert!(OperatorMatcher::I32Load.matches(&Operator::I32Load { memarg }));
        assert!(OperatorMatcher::I32Store.matches(&Operator::I32Store { memarg }));
        assert!(OperatorMatcher::I64Load.matches(&Operator::I64Load { memarg }));
        assert!(OperatorMatcher::I64Store.matches(&Operator::I64Store { memarg }));
    }

    #[test]
    fn test_operator_matcher_control_flow() {
        assert!(OperatorMatcher::BrIf.matches(&Operator::BrIf { relative_depth: 0 }));
        assert!(OperatorMatcher::Br.matches(&Operator::Br { relative_depth: 0 }));
        assert!(OperatorMatcher::Call.matches(&Operator::Call { function_index: 0 }));
    }

    #[test]
    fn test_operator_matcher_call_indirect() {
        let op = Operator::CallIndirect {
            type_index: 0,
            table_index: 0,
        };
        assert!(OperatorMatcher::CallIndirect.matches(&op));
        assert!(!OperatorMatcher::Call.matches(&op));
    }

    #[test]
    fn test_operator_matcher_memory_growth() {
        assert!(OperatorMatcher::MemoryGrow.matches(&Operator::MemoryGrow { mem: 0 }));
        assert!(OperatorMatcher::MemorySize.matches(&Operator::MemorySize { mem: 0 }));
    }

    #[test]
    fn test_operator_matcher_any() {
        assert!(OperatorMatcher::Any.matches(&Operator::I32Add));
        assert!(OperatorMatcher::Any.matches(&Operator::Nop));
        assert!(OperatorMatcher::Any.matches(&Operator::End));
    }

    #[test]
    fn test_operator_matcher_no_match() {
        assert!(!OperatorMatcher::I32Add.matches(&Operator::I64Add));
        assert!(!OperatorMatcher::BrIf.matches(&Operator::Br { relative_depth: 0 }));
    }

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Low, Severity::Low);
        assert_eq!(Severity::Medium, Severity::Medium);
        assert_eq!(Severity::High, Severity::High);
        assert_eq!(Severity::Critical, Severity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Low, Severity::High);
        assert_ne!(Severity::Medium, Severity::Critical);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::High;
        let serialized = serde_json::to_string(&severity).unwrap();
        let deserialized: Severity = serde_json::from_str(&serialized).unwrap();
        assert_eq!(severity, deserialized);
    }

    // ==================== VulnerabilityMatch Tests ====================

    #[test]
    fn test_vulnerability_match_risk_score_low() {
        let vuln = VulnerabilityMatch {
            pattern: "test".to_string(),
            location: 0..10,
            severity: Severity::Low,
            operator_index: 0,
        };
        assert_eq!(vuln.risk_score(), 25);
    }

    #[test]
    fn test_vulnerability_match_risk_score_medium() {
        let vuln = VulnerabilityMatch {
            pattern: "test".to_string(),
            location: 0..10,
            severity: Severity::Medium,
            operator_index: 0,
        };
        assert_eq!(vuln.risk_score(), 50);
    }

    #[test]
    fn test_vulnerability_match_risk_score_high() {
        let vuln = VulnerabilityMatch {
            pattern: "test".to_string(),
            location: 0..10,
            severity: Severity::High,
            operator_index: 0,
        };
        assert_eq!(vuln.risk_score(), 75);
    }

    #[test]
    fn test_vulnerability_match_risk_score_critical() {
        let vuln = VulnerabilityMatch {
            pattern: "test".to_string(),
            location: 0..10,
            severity: Severity::Critical,
            operator_index: 0,
        };
        assert_eq!(vuln.risk_score(), 100);
    }

    #[test]
    fn test_vulnerability_match_serialization() {
        let vuln = VulnerabilityMatch {
            pattern: "test-pattern".to_string(),
            location: 100..200,
            severity: Severity::High,
            operator_index: 42,
        };

        let serialized = serde_json::to_string(&vuln).unwrap();
        let deserialized: VulnerabilityMatch = serde_json::from_str(&serialized).unwrap();

        assert_eq!(vuln.pattern, deserialized.pattern);
        assert_eq!(vuln.location, deserialized.location);
        assert_eq!(vuln.severity, deserialized.severity);
        assert_eq!(vuln.operator_index, deserialized.operator_index);
    }

    #[test]
    fn test_vulnerability_match_clone() {
        let vuln = VulnerabilityMatch {
            pattern: "clone-test".to_string(),
            location: 50..100,
            severity: Severity::Medium,
            operator_index: 10,
        };

        let cloned = vuln.clone();
        assert_eq!(vuln.pattern, cloned.pattern);
        assert_eq!(vuln.risk_score(), cloned.risk_score());
    }

    // ==================== Default Patterns Tests ====================

    #[test]
    fn test_default_patterns_contain_expected() {
        let detector = PatternDetector::new();
        let pattern_names: Vec<_> = detector.patterns.iter().map(|p| p.name).collect();

        assert!(pattern_names.contains(&"potential-integer-overflow"));
        assert!(pattern_names.contains(&"timing-side-channel"));
        assert!(pattern_names.contains(&"unvalidated-indirect-call"));
        assert!(pattern_names.contains(&"unchecked-memory-growth"));
        assert!(pattern_names.contains(&"potential-buffer-overflow"));
    }

    #[test]
    fn test_default_patterns_severity_levels() {
        let detector = PatternDetector::new();

        for pattern in &detector.patterns {
            match pattern.name {
                "potential-integer-overflow" => assert_eq!(pattern.severity, Severity::Medium),
                "timing-side-channel" => assert_eq!(pattern.severity, Severity::Low),
                "unvalidated-indirect-call" => assert_eq!(pattern.severity, Severity::High),
                "unchecked-memory-growth" => assert_eq!(pattern.severity, Severity::Medium),
                "potential-buffer-overflow" => assert_eq!(pattern.severity, Severity::High),
                _ => {}
            }
        }
    }
}
