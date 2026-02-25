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
