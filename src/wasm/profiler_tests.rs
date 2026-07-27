#[cfg_attr(coverage_nightly, coverage(off))]

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

    // WASM module with a function containing various instructions
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
            0x00, 0x02, // min 2 pages, no max
            // Code section with control flow, memory, arithmetic
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

    // WASM module with memory section
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

    // WASM module with function calls
    fn function_call_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x04, // section id 1, size 4
            0x01, // 1 type
            0x60, 0x00, 0x00, // func type: () -> ()
            // Function section
            0x03, 0x03, // section id 3, size 3
            0x02, 0x00, 0x00, // 2 functions, both type 0
            // Code section
            0x0a, 0x09, // section id 10, size 9
            0x02, // 2 function bodies
            // First function: calls second
            0x04, // body size 4
            0x00, // 0 locals
            0x10, 0x01, // call 1
            0x0b, // end
            // Second function: empty
            0x02, // body size 2
            0x00, // 0 locals
            0x0b, // end
        ]
    }

    // ==================== AsyncProfiler Tests ====================

    #[test]
    fn test_async_profiler_new() {
        let profiler = AsyncProfiler::new();
        assert_eq!(profiler.sample_interval, Duration::from_millis(10));
    }

    #[test]
    fn test_async_profiler_default() {
        let profiler = AsyncProfiler::default();
        assert_eq!(profiler.sample_interval, Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_profile_minimal_module() {
        let profiler = AsyncProfiler::new();
        let result = profiler.profile_module(&minimal_wasm_module()).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.instruction_mix.total_instructions, 0);
    }

    #[tokio::test]
    async fn test_profile_mixed_instructions() {
        let profiler = AsyncProfiler::new();
        let result = profiler.profile_module(&mixed_instructions_wasm()).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.instruction_mix.total_instructions > 0);
        assert!(report.instruction_mix.control_flow > 0);
        assert!(report.instruction_mix.memory_ops > 0);
        assert!(report.instruction_mix.arithmetic > 0);
    }

    #[tokio::test]
    async fn test_profile_memory_section() {
        let profiler = AsyncProfiler::new();
        let result = profiler.profile_module(&memory_wasm()).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        // Memory section should be detected
        assert_eq!(report.memory_usage.initial_pages, 2);
        assert_eq!(report.memory_usage.max_pages, Some(16));
    }

    #[tokio::test]
    async fn test_profile_function_calls() {
        let profiler = AsyncProfiler::new();
        let result = profiler.profile_module(&function_call_wasm()).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.instruction_mix.calls > 0);
    }

    #[tokio::test]
    async fn test_profile_invalid_wasm() {
        let profiler = AsyncProfiler::new();
        let result = profiler.profile_module(&[0x00, 0x01, 0x02]).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_instruction_mix() {
        let profiler = AsyncProfiler::new();
        let result = profiler.analyze_instruction_mix(&mixed_instructions_wasm());

        assert!(result.is_ok());
        let mix = result.unwrap();
        assert!(mix.total_instructions > 0);
    }

    #[test]
    fn test_identify_hot_functions() {
        let profiler = AsyncProfiler::new();
        let result = profiler.identify_hot_functions(&mixed_instructions_wasm());

        assert!(result.is_ok());
        let hot_functions = result.unwrap();
        // Single function should be 100% of code
        assert!(!hot_functions.is_empty());
        // Function should be marked as hot (>5% of code)
        assert!(hot_functions.iter().all(|f| f.percentage > 5.0));
    }

    #[test]
    fn test_analyze_memory_usage_with_memory() {
        let profiler = AsyncProfiler::new();
        let result = profiler.analyze_memory_usage(&memory_wasm());

        assert!(result.is_ok());
        let memory = result.unwrap();
        assert_eq!(memory.initial_pages, 2);
        assert_eq!(memory.max_pages, Some(16));
    }

    #[test]
    fn test_analyze_memory_usage_no_memory() {
        let profiler = AsyncProfiler::new();
        let result = profiler.analyze_memory_usage(&minimal_wasm_module());

        assert!(result.is_ok());
        let memory = result.unwrap();
        // Default values when no memory section
        assert_eq!(memory.initial_pages, 1);
        assert_eq!(memory.max_pages, Some(256));
    }
}
