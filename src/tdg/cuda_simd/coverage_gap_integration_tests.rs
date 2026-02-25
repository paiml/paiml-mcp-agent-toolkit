    // =========================================================================
    // Integration: analyze_file routes to correct analyzer
    // =========================================================================

    #[test]
    fn test_analyze_file_ptx_routes_to_ptx_patterns() {
        let a = analyzer();
        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("kernel.ptx");
        std::fs::write(&ptx_file, ".entry kernel() {\nst.shared.u32 [%rd1], %r0;\nret;\n}").unwrap();
        let result = a.analyze(&ptx_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert!(result.defects.iter().any(|d| d.defect_class.ticket_id == "SHARED_U64"));
    }

    #[test]
    fn test_analyze_file_wgsl_routes_to_wgpu_patterns() {
        let a = analyzer();
        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("shader.wgsl");
        std::fs::write(&wgsl_file, "@compute @workgroup_size(16)\nfn main() {}").unwrap();
        let result = a.analyze(&wgsl_file).unwrap();
        assert_eq!(result.wgpu_files, 1);
        assert!(result.defects.iter().any(|d| d.defect_class.ticket_id == "WGPU_SMALL_WORKGROUP"));
    }

    #[test]
    fn test_analyze_file_rs_simd_routes_to_simd_patterns() {
        let a = analyzer();
        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("simd.rs");
        let content = concat!(
            "use std::arch::x86_64::*;\nfn f() { let a = _mm",
            "256_add_ps(x, y); }"
        );
        std::fs::write(&rs_file, content).unwrap();
        let result = a.analyze(&rs_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_detect_memory_patterns_non_ptx_global_mem() {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        let content = "float val = global_mem[tid];";
        a.detect_memory_patterns(content, &path, &mut analysis);
        assert!(analysis.coalescing.total_operations >= 1);
    }

    #[test]
    fn test_detect_memory_patterns_non_ptx_strided() {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        let content = "float val = data[threadIdx.x * stride];";
        a.detect_memory_patterns(content, &path, &mut analysis);
        assert_eq!(analysis.coalescing.problematic_accesses.len(), 1);
    }

    #[test]
    fn test_detect_memory_patterns_non_ptx_coalesced() {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        let content = "float val = data[threadIdx.x];";
        a.detect_memory_patterns(content, &path, &mut analysis);
        assert_eq!(analysis.coalescing.total_operations, 1);
        assert_eq!(analysis.coalescing.coalesced_operations, 1);
    }

    #[test]
    fn test_detect_memory_patterns_non_ptx_shared_bank_conflict_mitigation() {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        let content = "__shared__ float smem[threadIdx.x % 32];";
        a.detect_memory_patterns(content, &path, &mut analysis);
        assert!(analysis.coalescing.coalesced_operations >= 1);
    }
