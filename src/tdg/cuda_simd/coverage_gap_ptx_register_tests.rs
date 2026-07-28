    // =========================================================================
    // detect_ptx_memory_patterns tests - memory, register, and bounds checks
    // =========================================================================

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
