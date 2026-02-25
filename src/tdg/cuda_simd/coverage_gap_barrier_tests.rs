    // =========================================================================
    // detect_barrier_issues tests (32.3% covered -> targeting uncovered branches)
    // =========================================================================

    // --- detect_barrier_issues: basic barrier detection ---

    #[test]
    fn test_barrier_syncthreads_no_early_return() {
        let content = "void kernel() {\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        assert_eq!(analysis.barrier_safety.safe_barriers, 1);
        assert!(analysis.barrier_safety.unsafe_barriers.is_empty());
        assert!((analysis.barrier_safety.safety_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_barrier_syncwarp_no_early_return() {
        let content = "void kernel() {\n    __syncwarp();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        assert_eq!(analysis.barrier_safety.safe_barriers, 1);
        assert!(analysis.barrier_safety.unsafe_barriers.is_empty());
    }

    #[test]
    fn test_barrier_bar_sync_no_early_return() {
        let content = "void kernel() {\n    bar.sync;\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        assert_eq!(analysis.barrier_safety.safe_barriers, 1);
    }

    #[test]
    fn test_barrier_syncthreads_with_early_return_in_scope() {
        // return; at brace_depth == 0 before __syncthreads => unsafe
        let content = "void kernel() {\n    if (tid > N) return;\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers.len(), 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers[0].barrier_type, "__syncthreads");
        assert!(analysis.barrier_safety.unsafe_barriers[0].issue.contains("PARITY-114"));
    }

    #[test]
    fn test_barrier_syncwarp_with_early_return_in_scope() {
        let content = "void kernel() {\n    return;\n    __syncwarp();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers.len(), 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers[0].barrier_type, "__syncwarp");
    }

    #[test]
    fn test_barrier_bar_sync_with_early_return_in_scope() {
        let content = "void kernel() {\n    return;\n    bar.sync;\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers.len(), 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers[0].barrier_type, "bar.sync");
    }

    #[test]
    fn test_barrier_early_return_in_nested_scope_not_detected() {
        // return is inside a nested block (brace_depth != 0),
        // so the reverse scan should NOT flag it.
        let content = "void kernel() {\n    {\n        return;\n    }\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
        // The return is in inner scope - reverse scan will see } then {, depth goes to -1 and breaks
        assert_eq!(analysis.barrier_safety.safe_barriers, 1);
    }

    #[test]
    fn test_barrier_early_exit_before_barrier() {
        let content = "void kernel() {\n    exit;\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        // "exit" triggers the before_barrier check but the reverse scan looks for "return;" or "return "
        // exit alone does not match return pattern, so it goes to safe path
        assert_eq!(analysis.barrier_safety.total_barriers, 1);
    }

    #[test]
    fn test_barrier_multiple_barriers_mixed_safety() {
        let content = "void kernel() {\n    __syncthreads();\n    return;\n    __syncwarp();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 2);
        // First barrier: no return before it => safe
        // Second barrier: return before it at depth 0 => unsafe
        assert_eq!(analysis.barrier_safety.safe_barriers, 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers.len(), 1);
        assert_eq!(analysis.barrier_safety.unsafe_barriers[0].barrier_type, "__syncwarp");
    }

    #[test]
    fn test_barrier_safety_score_partial() {
        // 2 barriers, 1 safe, 1 unsafe => safety_score = 0.5
        let content = "void kernel() {\n    __syncthreads();\n    return;\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 2);
        assert!((analysis.barrier_safety.safety_score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_barrier_no_barriers_zero_score() {
        let content = "void kernel() {\n    int x = 1;\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 0);
        assert_eq!(analysis.barrier_safety.safe_barriers, 0);
        // safety_score stays default (0.0) when no barriers
        assert!((analysis.barrier_safety.safety_score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_barrier_defect_added_for_unsafe_barrier() {
        let content = "void kernel() {\n    return;\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert!(has_defect(&analysis, "PARITY-114"));
    }

    #[test]
    fn test_barrier_no_defect_for_safe_barrier() {
        let content = "void kernel() {\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert!(!has_defect(&analysis, "PARITY-114"));
    }

    #[test]
    fn test_barrier_return_with_value_before_barrier() {
        // "return 0" has "return " which should also trigger
        let content = "void kernel() {\n    return 0;\n    __syncthreads();\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.unsafe_barriers.len(), 1);
    }

    #[test]
    fn test_barrier_many_safe_barriers() {
        let content = "void kernel() {\n    __syncthreads();\n    __syncwarp();\n    bar.sync;\n}";
        let analysis = analyze_cuda_barriers(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 3);
        assert_eq!(analysis.barrier_safety.safe_barriers, 3);
        assert!(analysis.barrier_safety.unsafe_barriers.is_empty());
        assert!((analysis.barrier_safety.safety_score - 1.0).abs() < f64::EPSILON);
    }
