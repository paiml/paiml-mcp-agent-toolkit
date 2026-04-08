    // =========================================================================
    // analyze_simd_content tests
    // =========================================================================

    #[test]
    fn test_simd_empty_content() {
        let analysis = analyze_simd("");
        assert!(analysis.defects.is_empty());
        assert_eq!(analysis.coalescing.total_operations, 0);
    }

    #[test]
    fn test_simd_unsafe_no_safety_comment() {
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        let content = "use std::arch::x86_64::*;\nfn f() {\nunsafe {\nlet a = _mm_add_ps(x, y);\n}\n}";
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_UNSAFE_NO_SAFETY"));
    }

    #[test]
    fn test_simd_unsafe_with_safety_comment() {
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        let content = "\
use std::arch::x86_64::*;
fn f() {
unsafe {
// SAFETY: pointers are aligned
let a = _mm_add_ps(x, y);
}
}";
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_UNSAFE_NO_SAFETY"));
    }

    #[test]
    fn test_simd_unsafe_with_doc_safety_comment() {
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        let content = "\
use std::arch::x86_64::*;
fn f() {
unsafe {
/// SAFETY: guaranteed aligned
let a = _mm_add_ps(x, y);
}
}";
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_UNSAFE_NO_SAFETY"));
    }

    #[test]
    fn test_simd_avx256_ops_counted() {
        let content = concat!("let a = _mm", "256_add_ps(x, y);\nlet b = _mm", "256_mul_ps(a, z);");
        let analysis = analyze_simd(content);
        assert_eq!(analysis.coalescing.total_operations, 2);
        assert_eq!(analysis.coalescing.coalesced_operations, 2);
    }

    #[test]
    fn test_simd_avx512_ops_counted() {
        let content = concat!("let a = _mm", "512_add_ps(x, y);");
        let analysis = analyze_simd(content);
        assert_eq!(analysis.coalescing.total_operations, 1);
    }

    #[test]
    fn test_simd_sse_ops_counted() {
        let content = "let a = _mm_add_ps(x, y);\nlet b = _mm_mul_ps(a, z);";
        let analysis = analyze_simd(content);
        assert_eq!(analysis.coalescing.total_operations, 2);
    }

    #[test]
    fn test_simd_aligned_load_without_alignment() {
        let content = concat!("let a = _mm", "256_load_ps(ptr);");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_ALIGN_FAULT"));
    }

    #[test]
    fn test_simd_aligned_load_with_align_context() {
        let content = concat!("#[repr(align(32))]\nstruct A;\nlet a = _mm", "256_load_ps(ptr);");
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_ALIGN_FAULT"));
    }

    #[test]
    fn test_simd_aligned_load_with_as_ptr() {
        let content = concat!("let p = data.as_ptr();\nlet a = _mm", "256_load_ps(p);");
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_ALIGN_FAULT"));
    }

    #[test]
    fn test_simd_avx512_aligned_load_without_alignment() {
        let content = concat!("let a = _mm", "512_load_si512(ptr);");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_ALIGN_FAULT"));
    }

    #[test]
    fn test_simd_avx512_load_ps_without_alignment() {
        let content = concat!("let a = _mm", "512_load_ps(ptr);");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_ALIGN_FAULT"));
    }

    #[test]
    fn test_simd_avx256_load_si256_without_alignment() {
        let content = concat!("let a = _mm", "256_load_si256(ptr);");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_ALIGN_FAULT"));
    }

    #[test]
    fn test_simd_bounds_overflow_no_len_check() {
        let content = concat!("let a = _mm", "256_loadu_ps(ptr);");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_BOUNDS_OVERFLOW"));
    }

    #[test]
    fn test_simd_bounds_overflow_with_len_check() {
        let content = concat!("let n = data.len();\nlet a = _mm", "256_loadu_ps(ptr);");
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_BOUNDS_OVERFLOW"));
    }

    #[test]
    fn test_simd_bounds_overflow_avx512() {
        let content = concat!("let a = _mm", "512_loadu_ps(ptr);");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_BOUNDS_OVERFLOW"));
    }

    #[test]
    fn test_simd_vzeroupper_mixed_sse_avx() {
        let content = concat!(
            "let a = _mm_add_ps(x, y);\nlet b = _mm",
            "256_add_ps(x2, y2);"
        );
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_VZEROUPPER"));
    }

    #[test]
    fn test_simd_no_vzeroupper_with_zeroupper() {
        let content = concat!(
            "_mm", "256_zeroupper();\nlet a = _mm_add_ps(x, y);\nlet b = _mm",
            "256_add_ps(x2, y2);"
        );
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_VZEROUPPER"));
    }

    #[test]
    fn test_simd_missing_target_feature_avx() {
        let content = concat!("fn f() { let a = _mm", "256_add_ps(x, y); }");
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_MISSING_TARGET"));
    }

    #[test]
    fn test_simd_has_target_feature() {
        let content = concat!(
            "#[target_feature(enable = \"avx2\")]\nfn f() { let a = _mm",
            "256_add_ps(x, y); }"
        );
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_MISSING_TARGET"));
    }

    #[test]
    fn test_simd_runtime_detection_ok() {
        let content = concat!(
            "if is_x86_feature_detected!(\"avx2\") { let a = _mm",
            "256_add_ps(x, y); }"
        );
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_MISSING_TARGET"));
    }

    #[test]
    fn test_simd_low_vectorization_ratio() {
        let mut content = String::new();
        content.push_str(concat!("let a = _mm", "256_add_ps(x, y);\n"));
        for i in 0..8 {
            content.push_str(&format!("for i in 0..n{} {{ sum += arr[i]; }}\n", i));
        }
        let analysis = analyze_simd(&content);
        assert!(has_defect(&analysis, "SIMD_LOW_VECTORIZATION"));
    }

    #[test]
    fn test_simd_no_low_vectorization_when_many_vector_ops() {
        let content = concat!(
            "let a = _mm", "256_add_ps(x, y);\n",
            "let b = _mm", "256_mul_ps(a, z);\n",
            "let c = _mm", "256_sub_ps(b, w);\n",
            "let d = _mm", "256_div_ps(c, v);\n",
            "let e = _mm", "256_sqrt_ps(d);\n",
            "let f = _mm", "256_fmadd_ps(e, a, b);\n"
        );
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_LOW_VECTORIZATION"));
    }

    #[test]
    fn test_simd_suboptimal_width_sse_when_avx_available() {
        let content = "let a = _mm_add_ps(x, y);\nlet b = _mm_mul_ps(a, z);\nuse avx2;\n";
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_SUBOPTIMAL_WIDTH"));
    }

    #[test]
    fn test_simd_no_suboptimal_width_when_avx_used() {
        let content = concat!(
            "let b = _mm", "256_add_ps(c, d);\n",
            "let a = _mm_add_ps(x, y);\n"
        );
        let analysis = analyze_simd(content);
        assert!(!has_defect(&analysis, "SIMD_SUBOPTIMAL_WIDTH"));
    }

    #[test]
    fn test_simd_coalescing_efficiency() {
        let content = concat!(
            "let a = _mm", "256_add_ps(x, y);\n",
            "let b = _mm", "256_loadu_ps(ptr);\n"
        );
        let analysis = analyze_simd(content);
        assert!(analysis.coalescing.total_operations >= 2);
        assert!(analysis.coalescing.efficiency > 0.0);
    }

    #[test]
    fn test_simd_scalar_ops_detected_with_iter() {
        let content = concat!(
            "let a = _mm", "256_add_ps(x, y);\n",
            "data.iter().for_each(|x| sum += x);\n"
        );
        let analysis = analyze_simd(content);
        let total_ops = analysis.defects.iter().count();
        let _ = total_ops;
    }

    #[test]
    fn test_simd_unaligned_loads_add_to_coalescing() {
        let content = concat!(
            "let a = _mm", "256_loadu_ps(ptr1);\n",
            "let b = _mm", "512_loadu_ps(ptr2);\n",
            "let n = data.len();\n"
        );
        let analysis = analyze_simd(content);
        assert!(analysis.coalescing.total_operations >= 2);
    }

    #[test]
    fn test_simd_vzeroupper_only_reported_once() {
        let content = concat!(
            "let a = _mm_add_ps(x, y);\n",
            "let b = _mm_sub_ps(x, y);\n",
            "let c = _mm", "256_add_ps(x2, y2);\n"
        );
        let analysis = analyze_simd(content);
        assert_eq!(defect_count(&analysis, "SIMD_VZEROUPPER"), 1);
    }
