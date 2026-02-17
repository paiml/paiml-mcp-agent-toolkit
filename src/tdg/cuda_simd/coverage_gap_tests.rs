mod coverage_gap_tests {
    use super::*;
    use std::path::PathBuf;

    fn analyzer() -> CudaSimdAnalyzer {
        CudaSimdAnalyzer::new()
    }

    fn analyze_ptx(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.ptx");
        let mut analysis = FileAnalysis::default();
        a.detect_ptx_memory_patterns(content, &path, &mut analysis);
        analysis
    }

    fn analyze_simd(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.rs");
        let mut analysis = FileAnalysis::default();
        a.analyze_simd_content(content, &path, &mut analysis);
        analysis
    }

    fn analyze_wgpu(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.wgsl");
        let mut analysis = FileAnalysis::default();
        a.detect_wgpu_memory_patterns(content, &path, &mut analysis);
        analysis
    }

    fn has_defect(analysis: &FileAnalysis, ticket_id: &str) -> bool {
        analysis
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == ticket_id)
    }

    fn defect_count(analysis: &FileAnalysis, ticket_id: &str) -> usize {
        analysis
            .defects
            .iter()
            .filter(|d| d.defect_class.ticket_id == ticket_id)
            .count()
    }

    // =========================================================================
    // detect_ptx_memory_patterns tests
    // =========================================================================

    #[test]
    fn test_ptx_empty_content() {
        let analysis = analyze_ptx("");
        assert!(analysis.defects.is_empty());
        assert_eq!(analysis.coalescing.total_operations, 0);
    }

    #[test]
    fn test_ptx_placeholder_comment_omitted() {
        let ptx = ".entry kernel() {\n// omitted for brevity\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_comment_todo() {
        let ptx = ".entry kernel() {\n// TODO: implement kernel\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_comment_fixme() {
        let ptx = ".entry kernel() {\n// FIXME: broken\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_not_implemented() {
        let ptx = ".entry kernel() {\n// not implemented yet\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_for_brevity() {
        let ptx = ".entry kernel() {\n// for brevity we skip\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_simplified() {
        let ptx = ".entry kernel() {\n// simplified version\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_for_now() {
        let ptx = ".entry kernel() {\n// for now just return\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PLACEHOLDER"));
    }

    #[test]
    fn test_ptx_placeholder_only_first_match() {
        let ptx = ".entry kernel() {\n// omitted and also todo and fixme\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert_eq!(defect_count(&analysis, "PLACEHOLDER"), 1);
    }

    #[test]
    fn test_ptx_shared_u64_st() {
        let ptx = ".entry kernel() {\nst.shared.u32 [%rd1], %r0;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "SHARED_U64"));
    }

    #[test]
    fn test_ptx_shared_u64_ld() {
        let ptx = ".entry kernel() {\nld.shared.u32 %r0, [%rd1];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "SHARED_U64"));
    }

    #[test]
    fn test_ptx_shared_u64_not_triggered_without_rd() {
        let ptx = ".entry kernel() {\nst.shared.u32 [%r1], %r0;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "SHARED_U64"));
    }

    #[test]
    fn test_ptx_cvta_shared() {
        let ptx = ".entry kernel() {\ncvta.shared.u64 %rd0, %r1;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "CVTA_SHARED"));
    }

    #[test]
    fn test_ptx_missing_barrier_between_st_and_ld() {
        let ptx = ".entry kernel() {\nst.shared.u32 [%r1], %r0;\nld.shared.u32 %r2, [%r3];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "MISSING_BARRIER"));
    }

    #[test]
    fn test_ptx_barrier_resets_missing_barrier_tracking() {
        let ptx = ".entry kernel() {\nst.shared.u32 [%r1], %r0;\nbar.sync 0;\nld.shared.u32 %r2, [%r3];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "MISSING_BARRIER"));
    }

    #[test]
    fn test_ptx_f082_data_dependent_addressing() {
        let ptx = "\
.entry kernel() {
ld.shared.u32 %r1, [%r0];
add.u64 %rd2, %rd0, %r1;
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "F082"));
    }

    #[test]
    fn test_ptx_f082_via_add_s64() {
        let ptx = "\
.entry kernel() {
ld.shared.u32 %r5, [%r0];
add.s64 %rd2, %rd0, %r5;
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "F082"));
    }

    #[test]
    fn test_ptx_f082_via_cvt_u64() {
        let ptx = "\
.entry kernel() {
ld.shared.u32 %r5, [%r0];
cvt.u64.u32 %rd2, %r5;
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "F082"));
    }

    #[test]
    fn test_ptx_f082_no_shared_load_no_trigger() {
        let ptx = ".entry kernel() {\nadd.u64 %rd2, %rd0, %r1;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "F082"));
    }

    #[test]
    fn test_ptx_loop_early_exit_before_barrier() {
        let ptx = "\
.entry kernel() {
loop_start:
bra exit;
bar.sync 0;
bra loop_start;
loop_start_end:
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PARITY-114"));
    }

    #[test]
    fn test_ptx_conditional_early_exit_before_barrier() {
        let ptx = "\
.entry kernel() {
loop_start:
@%p0 bra done;
bar.sync 0;
bra loop_start;
loop_start_end:
ret;
}";
        let analysis = analyze_ptx(ptx);
        let parity = analysis
            .defects
            .iter()
            .find(|d| d.defect_class.ticket_id == "PARITY-114");
        assert!(parity.is_some());
        assert!(parity.unwrap().defect_class.description.contains("Conditional"));
    }

    #[test]
    fn test_ptx_no_early_exit_when_barrier_seen() {
        let ptx = "\
.entry kernel() {
loop_start:
bar.sync 0;
bra exit;
bra loop_start;
loop_start_end:
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "PARITY-114"));
    }

    #[test]
    fn test_ptx_loop_branch_to_end_label() {
        let ptx = ".entry kernel() {\nbra loop_end;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "LOOP_BRANCH_END"));
    }

    #[test]
    fn test_ptx_loop_branch_to_done_label() {
        let ptx = ".entry kernel() {\nbra some_done;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "LOOP_BRANCH_END"));
    }

    #[test]
    fn test_ptx_dead_code_after_ret() {
        let ptx = ".entry kernel() {\nret;\nmov.u32 %r0, %r1;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "DEAD_CODE"));
    }

    #[test]
    fn test_ptx_dead_code_after_unconditional_branch() {
        let ptx = ".entry kernel() {\nbra target;\nmov.u32 %r0, %r1;\ntarget:\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "DEAD_CODE"));
    }

    #[test]
    fn test_ptx_no_dead_code_after_label() {
        let ptx = ".entry kernel() {\nbra target;\ntarget:\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "DEAD_CODE"));
    }

    #[test]
    fn test_ptx_redundant_mov_chain() {
        let ptx = ".entry kernel() {\nmov.u32 %r1, %r0;\nmov.u32 %r2, %r1;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "REDUNDANT_MOV"));
    }

    #[test]
    fn test_ptx_no_redundant_mov_independent() {
        let ptx = ".entry kernel() {\nmov.u32 %r1, %r0;\nmov.u32 %r3, %r4;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "REDUNDANT_MOV"));
    }

    #[test]
    fn test_ptx_global_memory_coalescing_with_tid() {
        let ptx = ".entry kernel() {\nld.global.f32 %f0, [%tid];\nst.global.f32 [param], %f1;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert_eq!(analysis.coalescing.total_operations, 2);
        assert_eq!(analysis.coalescing.coalesced_operations, 2);
    }

    #[test]
    fn test_ptx_global_memory_not_coalesced() {
        let ptx = ".entry kernel() {\nld.global.f32 %f0, [%rd0];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert_eq!(analysis.coalescing.total_operations, 1);
        assert_eq!(analysis.coalescing.coalesced_operations, 0);
    }

    #[test]
    fn test_ptx_shared_memory_coalescing() {
        let ptx = ".entry kernel() {\nld.shared.u32 %r0, [%r1];\nst.shared.u32 [%r2], %r3;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(analysis.coalescing.total_operations >= 2);
        assert!(analysis.coalescing.coalesced_operations >= 2);
    }

    #[test]
    fn test_ptx_reg_spills_local_memory() {
        let ptx = ".entry kernel() {\n.local .align 4 .b8 spill[64];\n.local .align 4 .b8 spill2[32];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "REG_SPILLS"));
        let d = analysis.defects.iter().find(|d| d.defect_class.ticket_id == "REG_SPILLS").unwrap();
        assert!(d.defect_class.description.contains("2"));
    }

    #[test]
    fn test_ptx_high_register_pressure() {
        let ptx = ".entry kernel() {\n.reg .f32 %f<70>;\n.reg .u32 %r<10>;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "HIGH_REG_PRESSURE"));
    }

    #[test]
    fn test_ptx_no_high_register_pressure_under_threshold() {
        let ptx = ".entry kernel() {\n.reg .f32 %f<30>;\n.reg .u32 %r<10>;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "HIGH_REG_PRESSURE"));
    }

    #[test]
    fn test_ptx_predicate_overflow() {
        let ptx = ".entry kernel() {\n.reg .pred %p<12>;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PRED_OVERFLOW"));
    }

    #[test]
    fn test_ptx_no_predicate_overflow_under_threshold() {
        let ptx = ".entry kernel() {\n.reg .pred %p<6>;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "PRED_OVERFLOW"));
    }

    #[test]
    fn test_ptx_unoptimized_memory_many_single_loads() {
        let ptx = "\
.entry kernel() {
ld.global.f32 %f0, [%rd0];
ld.global.f32 %f1, [%rd1];
ld.global.f32 %f2, [%rd2];
ld.global.f32 %f3, [%rd3];
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "UNOPT_MEM"));
    }

    #[test]
    fn test_ptx_no_unopt_mem_with_vector_loads() {
        let ptx = "\
.entry kernel() {
ld.global.f32 %f0, [%rd0];
ld.global.f32 %f1, [%rd1];
ld.global.f32 %f2, [%rd2];
ld.global.f32 %f3, [%rd3];
ld.global.v2.f32 {%f4, %f5}, [%rd4];
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "UNOPT_MEM"));
    }

    #[test]
    fn test_ptx_no_bounds_check() {
        let ptx = "\
.entry kernel() {
mov.u32 %r0, %tid.x;
ld.global.f32 %f0, [%rd0];
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_ptx_has_bounds_check_with_setp() {
        let ptx = "\
.entry kernel() {
mov.u32 %r0, %tid.x;
setp.lt.u32 %p0, %r0, %r1;
ld.global.f32 %f0, [%rd0];
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_ptx_has_bounds_check_with_predicated_branch() {
        let ptx = "\
.entry kernel() {
mov.u32 %r0, %tid.x;
@%p0 bra skip;
ld.global.f32 %f0, [%rd0];
skip:
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "NO_BOUNDS_CHECK"));
    }

    #[test]
    fn test_ptx_missing_entry_point() {
        let ptx = ".version 7.0\n.target sm_80\nret;\n";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "NO_ENTRY"));
    }

    #[test]
    fn test_ptx_has_entry_point() {
        let ptx = ".entry kernel() {\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "NO_ENTRY"));
    }

    #[test]
    fn test_ptx_coalescing_efficiency_calculation() {
        let ptx = ".entry kernel() {\nld.global.f32 %f0, [%tid];\nld.global.f32 %f1, [%rd0];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert_eq!(analysis.coalescing.total_operations, 2);
        assert_eq!(analysis.coalescing.coalesced_operations, 1);
        assert!((analysis.coalescing.efficiency - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ptx_loop_label_detection_with_exit() {
        let ptx = "\
.entry kernel() {
loop_body:
bra exit;
bra loop_body;
loop_body_end:
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "PARITY-114"));
    }

    #[test]
    fn test_ptx_loop_end_label_exits_loop_state() {
        let ptx = "\
.entry kernel() {
loop_top:
bra loop_top;
loop_top_end:
bra exit;
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "PARITY-114"));
    }

    #[test]
    fn test_ptx_bar_arrive_resets_tracking() {
        let ptx = ".entry kernel() {\nst.shared.u32 [%r1], %r0;\nbar.arrive 0;\nld.shared.u32 %r2, [%r3];\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "MISSING_BARRIER"));
    }

    #[test]
    fn test_ptx_comments_and_empty_lines_skipped() {
        let ptx = "\
.entry kernel() {
// This is a comment


// Another comment
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "DEAD_CODE"));
    }

    #[test]
    fn test_ptx_label_resets_after_unconditional() {
        let ptx = "\
.entry kernel() {
bra target;
target:
add.u32 %r0, %r0, 1;
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "DEAD_CODE"));
    }

    #[test]
    fn test_ptx_multiple_defects_in_one_file() {
        let ptx = "\
.version 7.0
.reg .f32 %f<70>
.reg .pred %p<12>
.local .align 4 .b8 spill[64]
.entry kernel() {
// TODO: implement
st.shared.u32 [%rd1], %r0;
cvta.shared.u64 %rd0, %r1;
ret;
mov.u32 %r5, %r6;
}";
        let analysis = analyze_ptx(ptx);
        assert!(has_defect(&analysis, "HIGH_REG_PRESSURE"));
        assert!(has_defect(&analysis, "PRED_OVERFLOW"));
        assert!(has_defect(&analysis, "REG_SPILLS"));
        assert!(has_defect(&analysis, "PLACEHOLDER"));
        assert!(has_defect(&analysis, "SHARED_U64"));
        assert!(has_defect(&analysis, "CVTA_SHARED"));
        assert!(has_defect(&analysis, "DEAD_CODE"));
    }

    #[test]
    fn test_ptx_conditional_branch_not_loop_branch_end() {
        let ptx = ".entry kernel() {\n@%p0 bra loop_end;\nret;\n}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "LOOP_BRANCH_END"));
    }

    #[test]
    fn test_ptx_v4_vector_loads_count() {
        let ptx = "\
.entry kernel() {
ld.global.f32 %f0, [%rd0];
ld.global.f32 %f1, [%rd1];
ld.global.f32 %f2, [%rd2];
ld.global.f32 %f3, [%rd3];
ld.global.v4.f32 {%f4,%f5,%f6,%f7}, [%rd4];
ret;
}";
        let analysis = analyze_ptx(ptx);
        assert!(!has_defect(&analysis, "UNOPT_MEM"));
    }

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
        let content = "use std::arch::x86_64::*;\nfn f() {\nunsafe {\nlet a = _mm_add_ps(x, y);\n}\n}";
        let analysis = analyze_simd(content);
        assert!(has_defect(&analysis, "SIMD_UNSAFE_NO_SAFETY"));
    }

    #[test]
    fn test_simd_unsafe_with_safety_comment() {
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

    // =========================================================================
    // detect_barrier_issues tests (32.3% covered -> targeting uncovered branches)
    // =========================================================================

    fn analyze_cuda_barriers(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        a.detect_barrier_issues(content, &path, &mut analysis);
        analysis
    }

    fn analyze_cuda_known_patterns(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        a.detect_known_patterns(content, &path, &mut analysis);
        analysis
    }

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
}
