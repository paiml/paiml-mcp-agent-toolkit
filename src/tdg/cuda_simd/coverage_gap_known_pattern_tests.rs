    // =========================================================================
    // detect_known_patterns tests (18.2% covered -> targeting uncovered branches)
    // =========================================================================

    // --- PAR-041: FlashAttention tile size issues ---

    #[test]
    fn test_known_patterns_flash_attention_tile_kv_less_than_head_dim() {
        let content = "// FlashAttention kernel\nconst tile_kv = 32\nconst head_dim = 64\n";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(has_defect(&analysis, "PAR-041"));
        let defect = analysis.defects.iter().find(|d| d.defect_class.ticket_id == "PAR-041").unwrap();
        assert!(defect.snippet.as_ref().unwrap().contains("tile_kv (32) < head_dim (64)"));
        assert!(defect.suggestion.as_ref().unwrap().contains("64"));
    }

    #[test]
    fn test_known_patterns_flash_attention_tile_kv_equal_head_dim() {
        let content = "// FlashAttention kernel\nconst tile_kv = 64\nconst head_dim = 64\n";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-041"));
    }

    #[test]
    fn test_known_patterns_flash_attention_tile_kv_greater_head_dim() {
        let content = "// FlashAttention kernel\nconst tile_kv = 128\nconst head_dim = 64\n";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-041"));
    }

    #[test]
    fn test_known_patterns_flash_attention_variant_names() {
        // flash_attention (snake_case)
        let content = "void flash_attention() {\nlet tile_kv = 16\nlet head_dim = 128\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(has_defect(&analysis, "PAR-041"));

        // tiled_attention
        let content2 = "void tiled_attention() {\ntile_kv = 8\nhead_dim = 32\n}";
        let analysis2 = analyze_cuda_known_patterns(content2);
        assert!(has_defect(&analysis2, "PAR-041"));
    }

    #[test]
    fn test_known_patterns_flash_attention_missing_tile_kv() {
        let content = "// FlashAttention kernel\nconst head_dim = 64\n";
        let analysis = analyze_cuda_known_patterns(content);
        // tile_kv is None, so (None, Some(_)) does not match => no defect
        assert!(!has_defect(&analysis, "PAR-041"));
    }

    #[test]
    fn test_known_patterns_flash_attention_missing_head_dim() {
        let content = "// FlashAttention kernel\nconst tile_kv = 32\n";
        let analysis = analyze_cuda_known_patterns(content);
        // head_dim is None => no defect
        assert!(!has_defect(&analysis, "PAR-041"));
    }

    #[test]
    fn test_known_patterns_no_flash_attention_keyword() {
        // Without FlashAttention/flash_attention/tiled_attention, PAR-041 is skipped
        let content = "const tile_kv = 16\nconst head_dim = 128\n";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-041"));
    }

    // --- PAR-034: Missing Tensor Core usage ---

    #[test]
    fn test_known_patterns_matmul_without_tensor_core() {
        let content = "void matmul(float* A, float* B, float* C) {\n    // naive impl\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(has_defect(&analysis, "PAR-034"));
        let defect = analysis.defects.iter().find(|d| d.defect_class.ticket_id == "PAR-034").unwrap();
        assert!(defect.snippet.as_ref().unwrap().contains("Matrix multiplication"));
    }

    #[test]
    fn test_known_patterns_gemm_without_tensor_core() {
        let content = "void gemm_kernel() {\n    // generic impl\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(has_defect(&analysis, "PAR-034"));
    }

    #[test]
    fn test_known_patterns_matmul_with_wmma() {
        let content = "void matmul() {\n    wmma::mma_sync(d, a, b, c);\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-034"));
    }

    #[test]
    fn test_known_patterns_matmul_with_mma() {
        let content = "void matmul() {\n    mma.sync.aligned.m16n8k16;\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-034"));
    }

    #[test]
    fn test_known_patterns_matmul_with_tensor_core() {
        let content = "void matmul() {\n    tensor_core_gemm(A, B, C);\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-034"));
    }

    #[test]
    fn test_known_patterns_no_matmul_keyword() {
        // No matmul/gemm keyword => PAR-034 not triggered
        let content = "void dot_product() {\n    float sum = 0;\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(!has_defect(&analysis, "PAR-034"));
    }

    #[test]
    fn test_known_patterns_both_defects() {
        // FlashAttention + matmul without tensor core
        let content = "void flash_attention() {\nlet tile_kv = 16\nlet head_dim = 64\nmatmul(A, B, C);\n}";
        let analysis = analyze_cuda_known_patterns(content);
        assert!(has_defect(&analysis, "PAR-041"));
        assert!(has_defect(&analysis, "PAR-034"));
        assert_eq!(analysis.defects.len(), 2);
    }

    #[test]
    fn test_known_patterns_empty_content() {
        let analysis = analyze_cuda_known_patterns("");
        assert!(analysis.defects.is_empty());
    }

    // --- extract_value helper tests ---

    #[test]
    fn test_extract_value_assignment_with_spaces() {
        let a = analyzer();
        assert_eq!(a.extract_value("tile_kv = 128", "tile_kv"), Some(128));
    }

    #[test]
    fn test_extract_value_assignment_no_spaces() {
        let a = analyzer();
        assert_eq!(a.extract_value("tile_kv=64", "tile_kv"), Some(64));
    }

    #[test]
    fn test_extract_value_const_assignment() {
        let a = analyzer();
        assert_eq!(a.extract_value("const head_dim = 256", "head_dim"), Some(256));
    }

    #[test]
    fn test_extract_value_let_assignment() {
        let a = analyzer();
        assert_eq!(a.extract_value("let tile_kv = 32", "tile_kv"), Some(32));
    }

    #[test]
    fn test_extract_value_not_found() {
        let a = analyzer();
        assert_eq!(a.extract_value("int x = 10", "tile_kv"), None);
    }

    #[test]
    fn test_extract_value_non_numeric() {
        let a = analyzer();
        // After "tile_kv = ", take_while digit finds nothing
        assert_eq!(a.extract_value("tile_kv = abc", "tile_kv"), None);
    }
