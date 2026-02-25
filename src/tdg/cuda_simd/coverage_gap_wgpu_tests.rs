    // =========================================================================
    // detect_wgpu_memory_patterns tests
    // =========================================================================

    #[test]
    fn test_wgpu_empty_content() {
        let analysis = analyze_wgpu("");
        assert!(analysis.defects.is_empty());
        assert_eq!(analysis.coalescing.total_operations, 0);
    }

    #[test]
    fn test_wgpu_missing_workgroup_size_compute_shader() {
        let content = "@compute\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_MISSING_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_no_missing_workgroup_non_compute() {
        let content = "fn vertex_main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_MISSING_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_small_workgroup_16() {
        let content = "@compute @workgroup_size(16)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_small_workgroup_2d() {
        let content = "@compute @workgroup_size(4, 4)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_small_workgroup_3d() {
        let content = "@compute @workgroup_size(2, 2, 2)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_no_small_workgroup_64() {
        let content = "@compute @workgroup_size(64)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_large_workgroup_2048() {
        let content = "@compute @workgroup_size(2048)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_LARGE_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_large_workgroup_3d() {
        let content = "@compute @workgroup_size(32, 32, 2)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_LARGE_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_no_large_workgroup_1024() {
        let content = "@compute @workgroup_size(1024)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_LARGE_WORKGROUP"));
    }

    #[test]
    fn test_wgpu_non_warp_aligned_100() {
        let content = "@compute @workgroup_size(100)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_NON_WARP_ALIGNED"));
    }

    #[test]
    fn test_wgpu_non_warp_aligned_48() {
        let content = "@compute @workgroup_size(48)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_NON_WARP_ALIGNED"));
    }

    #[test]
    fn test_wgpu_warp_aligned_256() {
        let content = "@compute @workgroup_size(256)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_NON_WARP_ALIGNED"));
    }

    #[test]
    fn test_wgpu_no_bounds_check_with_global_invocation() {
        let content = "@compute @workgroup_size(256)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\ndata[gid.x] = 1.0;\n}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_wgpu_has_bounds_check() {
        let content = "@compute @workgroup_size(256)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\nif (gid.x < params.size) {\ndata[gid.x] = 1.0;\n}\n}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_wgpu_bounds_check_with_select_and_len() {
        let content = "@compute @workgroup_size(256)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\nlet v = select(0.0, data[gid.x], gid.x < params.len);\n}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_wgpu_bounds_check_with_count() {
        let content = "@compute @workgroup_size(256)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\nif (gid.x >= params.count) { return; }\n}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_wgpu_excessive_barriers() {
        let mut content = String::from("@compute @workgroup_size(256)\nfn main() {\n");
        for _ in 0..6 {
            content.push_str("workgroupBarrier();\n");
        }
        content.push_str("}\n");
        let analysis = analyze_wgpu(&content);
        assert!(has_defect(&analysis, "WGPU_EXCESSIVE_BARRIERS"));
    }

    #[test]
    fn test_wgpu_no_excessive_barriers_under_threshold() {
        let content = "@compute @workgroup_size(256)\nfn main() {\nworkgroupBarrier();\nworkgroupBarrier();\n}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_EXCESSIVE_BARRIERS"));
    }

    #[test]
    fn test_wgpu_storage_barrier_counted() {
        let mut content = String::from("@compute @workgroup_size(256)\nfn main() {\n");
        for _ in 0..6 {
            content.push_str("storageBarrier();\n");
        }
        content.push_str("}\n");
        let analysis = analyze_wgpu(&content);
        assert!(has_defect(&analysis, "WGPU_EXCESSIVE_BARRIERS"));
        assert_eq!(analysis.barrier_safety.total_barriers, 6);
        assert_eq!(analysis.barrier_safety.safe_barriers, 6);
    }

    #[test]
    fn test_wgpu_storage_buffer_access_coalescing() {
        let content = "@group(0) @binding(0) var<storage, read_write> data: array<f32>;\n@compute @workgroup_size(256)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(analysis.coalescing.total_operations >= 1);
    }

    #[test]
    fn test_wgpu_array_indexing_coalescing() {
        let content = "@compute @workgroup_size(256)\nfn main() {\ndata[gid.x] = 1.0;\nresult[gid.x] = data[gid.x];\n}";
        let analysis = analyze_wgpu(content);
        assert!(analysis.coalescing.total_operations >= 2);
    }

    #[test]
    fn test_wgpu_coalescing_efficiency() {
        let content = "@compute @workgroup_size(256)\nfn main() {\ndata[gid.x] = 1.0;\n}";
        let analysis = analyze_wgpu(content);
        if analysis.coalescing.total_operations > 0 {
            assert!(analysis.coalescing.efficiency > 0.0);
        }
    }

    #[test]
    fn test_wgpu_barrier_safety_score() {
        let content = "@compute @workgroup_size(256)\nfn main() {\nworkgroupBarrier();\nstorageBarrier();\n}";
        let analysis = analyze_wgpu(content);
        assert_eq!(analysis.barrier_safety.total_barriers, 2);
        assert_eq!(analysis.barrier_safety.safe_barriers, 2);
        assert!((analysis.barrier_safety.safety_score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_wgpu_workgroup_size_1d_only() {
        let content = "@compute @workgroup_size(128)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
        assert!(!has_defect(&analysis, "WGPU_LARGE_WORKGROUP"));
        assert!(!has_defect(&analysis, "WGPU_NON_WARP_ALIGNED"));
    }

    #[test]
    fn test_wgpu_workgroup_size_2d() {
        let content = "@compute @workgroup_size(16, 16)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
        assert!(!has_defect(&analysis, "WGPU_NON_WARP_ALIGNED"));
    }

    #[test]
    fn test_wgpu_workgroup_size_3d_optimal() {
        let content = "@compute @workgroup_size(8, 8, 4)\nfn main() {}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_SMALL_WORKGROUP"));
        assert!(!has_defect(&analysis, "WGPU_NON_WARP_ALIGNED"));
    }

    #[test]
    fn test_wgpu_multiple_defects() {
        let content = "@compute\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\ndata[gid.x] = 1.0;\n}";
        let analysis = analyze_wgpu(content);
        assert!(has_defect(&analysis, "WGPU_MISSING_WORKGROUP"));
        assert!(has_defect(&analysis, "WGPU_NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_wgpu_no_global_invocation_no_bounds_check_needed() {
        let content = "@compute @workgroup_size(256)\nfn main() {\nlet x = 1 + 2;\n}";
        let analysis = analyze_wgpu(content);
        assert!(!has_defect(&analysis, "WGPU_NO_BOUNDS_CHECK"));
    }
