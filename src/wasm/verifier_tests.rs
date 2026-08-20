#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // Minimal valid WASM module (empty module with proper header)
    fn minimal_wasm_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
        ]
    }

    // WASM module with a simple function that does i32.const + i32.add
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

    // WASM module with memory and i32.load
    fn memory_load_wasm() -> Vec<u8> {
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
            // Memory section
            0x05, 0x03, // section id 5, size 3
            0x01, // 1 memory
            0x00, 0x01, // min 1 page, no max
            // Code section
            0x0a, 0x09, // section id 10, size 9
            0x01, // 1 function body
            0x07, // body size 7
            0x00, // 0 locals
            0x41, 0x00, // i32.const 0 (address)
            0x28, 0x02, 0x00, // i32.load align=2 offset=0
            0x0b, // end
        ]
    }

    // WASM module with i32.store
    fn memory_store_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x04, // section id 1, size 4
            0x01, // 1 type
            0x60, 0x00, 0x00, // func type: () -> ()
            // Function section
            0x03, 0x02, // section id 3, size 2
            0x01, 0x00, // 1 function, type 0
            // Memory section
            0x05, 0x03, // section id 5, size 3
            0x01, // 1 memory
            0x00, 0x01, // min 1 page, no max
            // Code section
            0x0a, 0x0b, // section id 10, size 11
            0x01, // 1 function body
            0x09, // body size 9
            0x00, // 0 locals
            0x41, 0x00, // i32.const 0 (address)
            0x41, 0x2a, // i32.const 42 (value)
            0x36, 0x02, 0x00, // i32.store align=2 offset=0
            0x0b, // end
        ]
    }

    // ==================== IncrementalVerifier Tests ====================

    #[test]
    fn test_incremental_verifier_new() {
        let verifier = IncrementalVerifier::new();
        assert!(verifier.is_ok());
    }

    #[test]
    fn test_verify_minimal_module() {
        let verifier = IncrementalVerifier::new().unwrap();
        let result = verifier.verify_module(&minimal_wasm_module());
        assert!(result.is_ok());
        assert!(result.unwrap().is_safe());
    }

    #[test]
    fn test_verify_simple_function() {
        let verifier = IncrementalVerifier::new().unwrap();
        let result = verifier.verify_module(&simple_function_wasm());
        assert!(result.is_ok());
        assert!(result.unwrap().is_safe());
    }

    #[test]
    fn test_verify_memory_load() {
        let verifier = IncrementalVerifier::new().unwrap();
        let result = verifier.verify_module(&memory_load_wasm());
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_memory_store() {
        let verifier = IncrementalVerifier::new().unwrap();
        let result = verifier.verify_module(&memory_store_wasm());
        assert!(result.is_ok());
    }

    /// verifier_core.rs:61-66 — `if stack_types.pop() != Some(ValType::I32)` TypeError
    /// arm of the I32Store match. Existing `test_verify_memory_store` covers the
    /// happy path (stack is [i32, i32] before store). To hit this arm the stack
    /// must be [NOT-i32, i32] before i32.store — check_i32_load_store pops the
    /// top i32 (address), leaving a non-i32 for the subsequent pop. We craft a
    /// module whose body is: i64.const 0; i32.const 0; i32.store offset=0 end.
    #[test]
    fn test_verify_i32_store_non_i32_value_hits_type_error() {
        let bytes: Vec<u8> = vec![
            // magic + version
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: () -> ()
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            // Function section: 1 func, type 0
            0x03, 0x02, 0x01, 0x00,
            // Memory section: 1 memory, flags=0, min=1
            0x05, 0x03, 0x01, 0x00, 0x01,
            // Code section
            0x0a, 0x0c, // section size 12
            0x01, // 1 body
            0x0a, // body size 10
            0x00, // 0 locals
            0x42, 0x00, // i64.const 0        — leaves i64 under the address
            0x41, 0x00, // i32.const 0        — address
            0x36, 0x02, 0x00, // i32.store align=2 offset=0
            0x1a, // drop                    — consume the leftover i64 if we got here
            0x0b, // end
        ];

        let verifier = IncrementalVerifier::new().unwrap();
        let result = verifier.verify_module(&bytes).expect("parse must succeed");
        assert!(
            matches!(result, VerificationResult::TypeError { .. }),
            "stack [i64, i32] before i32.store must hit the TypeError arm of \
             verify_function — got {:?}",
            result
        );
    }

    #[test]
    fn test_verify_invalid_wasm() {
        let verifier = IncrementalVerifier::new().unwrap();
        let invalid_wasm = vec![0x00, 0x01, 0x02, 0x03]; // Not a valid WASM
        let result = verifier.verify_module(&invalid_wasm);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_empty_input() {
        let verifier = IncrementalVerifier::new().unwrap();
        let result = verifier.verify_module(&[]);
        // Empty input should fail parsing
        assert!(result.is_err());
    }

    // ==================== VerificationResult Tests ====================

    #[test]
    fn test_verification_result_safe_is_safe() {
        let result = VerificationResult::Safe;
        assert!(result.is_safe());
    }

    #[test]
    fn test_verification_result_out_of_bounds_not_safe() {
        let result = VerificationResult::OutOfBounds {
            offset: 100,
            size: 50,
        };
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_type_error_not_safe() {
        let result = VerificationResult::TypeError {
            expected: "i32".to_string(),
            found: Some("i64".to_string()),
        };
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_type_error_none_found() {
        let result = VerificationResult::TypeError {
            expected: "i32".to_string(),
            found: None,
        };
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_stack_underflow_not_safe() {
        let result = VerificationResult::StackUnderflow;
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_stack_overflow_not_safe() {
        let result = VerificationResult::StackOverflow;
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_invalid_indirect_call_not_safe() {
        let result = VerificationResult::InvalidIndirectCall;
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_integer_overflow_not_safe() {
        let result = VerificationResult::IntegerOverflow;
        assert!(!result.is_safe());
    }

    #[test]
    fn test_verification_result_serialization() {
        let result = VerificationResult::Safe;
        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: VerificationResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_verification_result_out_of_bounds_serialization() {
        let result = VerificationResult::OutOfBounds {
            offset: 1000,
            size: 65536,
        };
        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: VerificationResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(result, deserialized);
    }

    // ==================== DifferentialTester Tests ====================

    #[test]
    fn test_differential_tester_new() {
        let tester = DifferentialTester::new();
        // Just verify it can be created
        assert!(std::mem::size_of_val(&tester) > 0);
    }

    #[test]
    fn test_differential_tester_default() {
        let tester = DifferentialTester::default();
        assert!(std::mem::size_of_val(&tester) > 0);
    }

    #[test]
    fn test_generate_test_cases_zero() {
        let mut tester = DifferentialTester::new();
        let cases = tester.generate_test_cases(&minimal_wasm_module(), 0);
        assert!(cases.is_empty());
    }

    #[test]
    fn test_generate_test_cases_multiple() {
        let mut tester = DifferentialTester::new();
        let cases = tester.generate_test_cases(&minimal_wasm_module(), 5);
        assert_eq!(cases.len(), 5);
    }

    #[test]
    fn test_generate_test_cases_values() {
        let mut tester = DifferentialTester::new();
        let cases = tester.generate_test_cases(&minimal_wasm_module(), 3);

        assert_eq!(cases[0].inputs, vec![0, 0]);
        assert_eq!(cases[1].inputs, vec![1, 2]);
        assert_eq!(cases[2].inputs, vec![2, 4]);

        for case in &cases {
            assert!(case.expected_output.is_none());
        }
    }

    #[test]
    fn test_differential_test_consistent() {
        let tester = DifferentialTester::new();
        let result = tester.differential_test(&minimal_wasm_module());
        assert_eq!(result, DifferentialResult::Consistent);
    }

    // ==================== TestCase Tests ====================

    #[test]
    fn test_test_case_equality() {
        let case1 = TestCase {
            inputs: vec![1, 2, 3],
            expected_output: Some(vec![6]),
        };
        let case2 = TestCase {
            inputs: vec![1, 2, 3],
            expected_output: Some(vec![6]),
        };
        assert_eq!(case1, case2);
    }

    #[test]
    fn test_test_case_inequality() {
        let case1 = TestCase {
            inputs: vec![1, 2, 3],
            expected_output: Some(vec![6]),
        };
        let case2 = TestCase {
            inputs: vec![1, 2, 4],
            expected_output: Some(vec![6]),
        };
        assert_ne!(case1, case2);
    }

    #[test]
    fn test_test_case_clone() {
        let case = TestCase {
            inputs: vec![1, 2, 3],
            expected_output: Some(vec![6]),
        };
        let cloned = case.clone();
        assert_eq!(case, cloned);
    }

    // ==================== DifferentialResult Tests ====================

    #[test]
    fn test_differential_result_consistent() {
        let result = DifferentialResult::Consistent;
        assert_eq!(result, DifferentialResult::Consistent);
    }

    #[test]
    fn test_differential_result_divergence() {
        let result = DifferentialResult::Divergence {
            test_case: TestCase {
                inputs: vec![1],
                expected_output: None,
            },
            runtime1_result: vec![1],
            runtime2_result: vec![2],
        };
        assert!(matches!(result, DifferentialResult::Divergence { .. }));
    }

    #[test]
    fn test_differential_result_clone() {
        let result = DifferentialResult::Consistent;
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // ==================== InvariantChecker Tests ====================

    #[test]
    fn test_invariant_checker_new() {
        let checker = InvariantChecker::new();
        // Verify default invariants are created
        assert_eq!(checker.invariants.len(), 4);
    }

    #[test]
    fn test_invariant_checker_check_all_returns_empty_vec() {
        let checker = InvariantChecker::new();
        let violations = checker.check_all(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
        // Current impl is a placeholder that always returns an empty Vec.
        assert!(violations.is_empty());
    }
}
