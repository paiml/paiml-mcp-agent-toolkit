// Coverage tests: WASM binary fixtures and module-level API tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_api {
    use super::*;

    // =====================================================
    // Helper functions - Valid WASM binary test fixtures
    // =====================================================

    /// Minimal valid WASM module (empty module with proper header)
    fn minimal_wasm_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
        ]
    }

    /// WASM module with a simple function containing arithmetic
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

    /// WASM module with memory section
    fn memory_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Memory section
            0x05, 0x04, // section id 5, size 4
            0x01, // 1 memory
            0x01, 0x02, 0x10, // min 2 pages, max 16 pages
        ]
    }

    /// WASM module with mixed instructions (control flow, memory, arithmetic)
    fn mixed_instructions_wasm() -> Vec<u8> {
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
            // Code section with mixed instructions
            0x0a, 0x11, // section id 10, size 17
            0x01, // 1 function body
            0x0f, // body size 15
            0x00, // 0 locals
            0x02, 0x7f, // block returning i32
            0x41, 0x00, // i32.const 0
            0x28, 0x02, 0x00, // i32.load
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x0c, 0x00, // br 0
            0x0b, // end block
            0x0b, // end function
        ]
    }

    // =====================================================
    // Module-level public function tests
    // =====================================================

    #[tokio::test]
    async fn test_analyze_wasm_module_minimal() {
        let result = analyze_wasm_module(&minimal_wasm_module()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.instruction_mix.total_instructions, 0);
        assert!(analysis.vulnerability_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_simple_function() {
        let result = analyze_wasm_module(&simple_function_wasm()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.instruction_mix.total_instructions > 0);
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_mixed_instructions() {
        let result = analyze_wasm_module(&mixed_instructions_wasm()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.instruction_mix.control_flow > 0);
        assert!(analysis.instruction_mix.memory_ops > 0);
        assert!(analysis.instruction_mix.arithmetic > 0);
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_invalid_binary() {
        let invalid = vec![0x00, 0x01, 0x02, 0x03]; // Not valid WASM
        let result = analyze_wasm_module(&invalid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_empty_input() {
        let result = analyze_wasm_module(&[]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wasm_safety_minimal() {
        let result = verify_wasm_safety(&minimal_wasm_module());
        assert!(result.is_ok());

        let verification = result.unwrap();
        assert!(verification.is_safe());
    }

    #[test]
    fn test_verify_wasm_safety_simple_function() {
        let result = verify_wasm_safety(&simple_function_wasm());
        assert!(result.is_ok());

        let verification = result.unwrap();
        assert!(verification.is_safe());
    }

    #[test]
    fn test_verify_wasm_safety_mixed_instructions() {
        let result = verify_wasm_safety(&mixed_instructions_wasm());
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_wasm_safety_invalid_binary() {
        let invalid = vec![0x00, 0x01, 0x02, 0x03];
        let result = verify_wasm_safety(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wasm_safety_empty_input() {
        let result = verify_wasm_safety(&[]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_profile_wasm_module_minimal() {
        let result = profile_wasm_module(&minimal_wasm_module()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.instruction_mix.total_instructions, 0);
    }

    #[tokio::test]
    async fn test_profile_wasm_module_simple_function() {
        let result = profile_wasm_module(&simple_function_wasm()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.instruction_mix.total_instructions > 0);
    }

    #[tokio::test]
    async fn test_profile_wasm_module_with_memory() {
        let result = profile_wasm_module(&memory_wasm()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.memory_usage.initial_pages, 2);
        assert_eq!(report.memory_usage.max_pages, Some(16));
    }

    #[tokio::test]
    async fn test_profile_wasm_module_mixed_instructions() {
        let result = profile_wasm_module(&mixed_instructions_wasm()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.instruction_mix.control_flow > 0);
        assert!(report.instruction_mix.memory_ops > 0);
        assert!(report.instruction_mix.arithmetic > 0);
    }

    #[tokio::test]
    async fn test_profile_wasm_module_invalid_binary() {
        let invalid = vec![0x00, 0x01, 0x02, 0x03];
        let result = profile_wasm_module(&invalid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_profile_wasm_module_empty_input() {
        let result = profile_wasm_module(&[]).await;
        assert!(result.is_err());
    }
}
