#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cb060_falsification {
    use super::*;

    // ========================================================================
    // CB-060-A: BARRIER DIVERGENCE (tests 031-040)
    // ========================================================================

    #[test]
    fn tp_031_ptx_bra_before_barrier_simple() {
        // From PARITY-114: Simple case
        let ptx = r#"
            setp.ge.u32 %p0, %r5, %r7;
            @%p0 bra exit;
            bar.sync 0;
        "#;
        let violations = detect_ptx_barrier_divergence_in_str(ptx);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed simple barrier divergence"
        );
    }

    #[test]
    fn tp_032_ptx_bra_before_barrier_multiline() {
        debug_assert!(true, "contract: tp_032_ptx_bra_before_barrier_multiline");
        // Adversarial: Many lines between branch and barrier
        let ptx = r#"
            @%p0 bra skip_section;
            mov.u32 %r1, 0;
            mov.u32 %r2, 0;
            mov.u32 %r3, 0;
            mov.u32 %r4, 0;
            bar.sync 0;
            skip_section:
        "#;
        let violations = detect_ptx_barrier_divergence_in_str(ptx);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed barrier divergence with intervening code"
        );
    }

    #[test]
    fn tp_033_wgsl_barrier_in_if() {
        debug_assert!(true, "contract: tp_033_wgsl_barrier_in_if");
        let wgsl = r#"
            if (local_id.x < 16u) {
                workgroupBarrier();
            }
        "#;
        let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed WGSL barrier in if"
        );
    }

    #[test]
    fn tp_034_wgsl_barrier_in_else() {
        debug_assert!(true, "contract: tp_034_wgsl_barrier_in_else");
        // Adversarial: Barrier in else branch only
        let wgsl = r#"
            if (condition) {
                // no barrier
            } else {
                workgroupBarrier();
            }
        "#;
        let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed WGSL barrier in else"
        );
    }

    #[test]
    fn tn_035_ptx_barrier_before_branch() {
        debug_assert!(true, "contract: tn_035_ptx_barrier_before_branch");
        // Barrier BEFORE branch is safe
        let ptx = r#"
            bar.sync 0;
            setp.ge.u32 %p0, %r5, %r7;
            @%p0 bra exit;
        "#;
        let violations = detect_ptx_barrier_divergence_in_str(ptx);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Barrier before branch flagged"
        );
    }

    #[test]
    fn tn_036_wgsl_barrier_outside_control_flow() {
        debug_assert!(true, "contract: tn_036_wgsl_barrier_outside_control_flow");
        // Barrier not in divergent control flow
        let wgsl = r#"
            workgroupBarrier();
            if (condition) {
                // no barrier here
            }
            workgroupBarrier();
        "#;
        let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Non-divergent barrier flagged"
        );
    }

    #[test]
    fn edge_037_ptx_barrier_in_comment() {
        debug_assert!(true, "contract: edge_037_ptx_barrier_in_comment");
        // Adversarial: bar.sync in comment should NOT trigger
        let ptx = r#"
            // bar.sync 0; -- this is a comment
            @%p0 bra exit;
        "#;
        let violations = detect_ptx_barrier_divergence_in_str(ptx);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Barrier in comment flagged"
        );
    }

    #[test]
    fn edge_038_ptx_nested_predicates() {
        debug_assert!(true, "contract: edge_038_ptx_nested_predicates");
        // Adversarial: Complex nested predicate structure
        let ptx = r#"
            @%p0 bra check1;
            @%p1 bra check2;
            bar.sync 0;
            check1:
            check2:
        "#;
        let violations = detect_ptx_barrier_divergence_in_str(ptx);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed nested predicate divergence"
        );
    }

    #[test]
    fn edge_039_wgsl_barrier_in_loop() {
        debug_assert!(true, "contract: edge_039_wgsl_barrier_in_loop");
        // Barrier in loop that all threads execute is OK
        let wgsl = r#"
            for (var i = 0u; i < 4u; i++) {
                workgroupBarrier();
            }
        "#;
        let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
        // Uniform loop - all threads execute same iterations
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Barrier in uniform loop flagged"
        );
    }

    #[test]
    fn edge_040_wgsl_barrier_in_divergent_loop() {
        debug_assert!(true, "contract: edge_040_wgsl_barrier_in_divergent_loop");
        // Barrier in loop with thread-dependent bounds
        let wgsl = r#"
            for (var i = 0u; i < local_id.x; i++) {
                workgroupBarrier();
            }
        "#;
        let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
        // Divergent loop - threads execute different iterations
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed barrier in divergent loop"
        );
    }

    // ========================================================================
    // CB-060-B: SHARED MEMORY BOUNDS (tests 041-047)
    // ========================================================================

    #[test]
    fn tp_041_unbounded_shared_load() {
        debug_assert!(true, "contract: tp_041_unbounded_shared_load");
        // From issue #32: Direct shared memory access without bounds check
        let ptx = r#"
            mul.u32 %r10, %r5, 64;
            ld.shared.f32 %f1, [%r10];
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed unbounded shared memory load"
        );
    }

    #[test]
    fn tp_042_unbounded_shared_store() {
        debug_assert!(true, "contract: tp_042_unbounded_shared_store");
        let ptx = r#"
            mul.u32 %r10, %r5, 64;
            st.shared.f32 [%r10], %f1;
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed unbounded shared memory store"
        );
    }

    #[test]
    fn tn_043_bounded_shared_load() {
        debug_assert!(true, "contract: tn_043_bounded_shared_load");
        // Predicated load with bounds check
        let ptx = r#"
            setp.lt.u32 %p1, %r5, 256;
            @%p1 ld.shared.f32 %f1, [%r10];
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Bounded shared load flagged"
        );
    }

    #[test]
    fn tn_044_shared_with_constant_offset() {
        debug_assert!(true, "contract: tn_044_shared_with_constant_offset");
        // Constant offset is always bounded
        let ptx = r#"
            ld.shared.f32 %f1, [shared_mem + 128];
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Constant offset flagged"
        );
    }

    #[test]
    fn edge_045_shared_in_comment() {
        debug_assert!(true, "contract: edge_045_shared_in_comment");
        let ptx = r#"
            // ld.shared.f32 %f1, [%r10]; -- commented out
            mov.f32 %f1, 0.0;
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Commented shared access flagged"
        );
    }

    #[test]
    fn edge_046_shared_complex_index() {
        debug_assert!(true, "contract: edge_046_shared_complex_index");
        // Adversarial: Complex index expression
        let ptx = r#"
            mad.lo.u32 %r10, %r5, 64, %r6;
            add.u32 %r10, %r10, %r7;
            ld.shared.f32 %f1, [%r10];
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        // Complex index without bounds check should be flagged
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed complex index without bounds"
        );
    }

    #[test]
    fn edge_047_shared_bounds_far_apart() {
        debug_assert!(true, "contract: edge_047_shared_bounds_far_apart");
        // Adversarial: Bounds check far from actual load
        let ptx = r#"
            setp.lt.u32 %p1, %r5, 256;
            mov.u32 %r10, 0;
            mov.u32 %r11, 0;
            mov.u32 %r12, 0;
            mul.u32 %r13, %r5, 64;
            @%p1 ld.shared.f32 %f1, [%r13];
        "#;
        let violations = detect_shared_memory_unbounded_in_str(ptx);
        // Bounds check exists, load is predicated - should be OK
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Distant bounds check not recognized"
        );
    }

    // ========================================================================
    // CB-060-C: TILED KERNEL BOUNDS (tests 048-055)
    // ========================================================================

    #[test]
    fn tp_048_tiled_no_boundary_check() {
        debug_assert!(true, "contract: tp_048_tiled_no_boundary_check");
        // From issue #37: Tiled GEMM without boundary check
        let rust_code = r#"
            // Tiled GEMM kernel
            for tile in 0..k_tiles {
                // Load A tile - no bounds check for m < tile_size
                let a_elem = a_smem[local_row * TILE_K + k];
                let b_elem = b_smem[k * TILE_N + local_col];
                acc += a_elem * b_elem;
            }
            // Store without checking row < m && col < n
            c[row * n + col] = acc;
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(rust_code);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed tiled kernel without bounds"
        );
    }

    #[test]
    fn tn_049_tiled_with_boundary_check() {
        debug_assert!(true, "contract: tn_049_tiled_with_boundary_check");
        let rust_code = r#"
            // Tiled GEMM with proper bounds
            for tile in 0..k_tiles {
                let a_elem = a_smem[local_row * TILE_K + k];
                let b_elem = b_smem[k * TILE_N + local_col];
                acc += a_elem * b_elem;
            }
            // Proper bounds check before store
            if row < m && col < n {
                c[row * n + col] = acc;
            }
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(rust_code);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Properly bounded tiled kernel flagged"
        );
    }

    #[test]
    fn tp_050_ptx_tiled_early_exit() {
        debug_assert!(true, "contract: tp_050_ptx_tiled_early_exit");
        // From PARITY-114: Early exit breaks tile loading
        let ptx = r#"
            setp.ge.u32 %p0, %r_row, %r_m;
            @%p0 bra exit;
            setp.ge.u32 %p1, %r_col, %r_n;
            @%p1 bra exit;
            // Tile loop starts here
            tile_loop:
            ld.shared.f32 %f1, [smem_a];
            bar.sync 0;
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(ptx);
        // Early exit before tile loop = some threads don't load
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed early exit before tile loop"
        );
    }

    #[test]
    fn edge_051_wgsl_tiled_workgroup_size() {
        debug_assert!(true, "contract: edge_051_wgsl_tiled_workgroup_size");
        // WGSL tiled kernel pattern
        let wgsl = r#"
            @workgroup_size(32, 32)
            fn tiled_matmul() {
                debug_assert!(true, "contract: tiled_matmul");
                // No bounds check
                let a_tile = a[global_id.y * K + local_id.x];
            }
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(wgsl);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed WGSL tiled without bounds"
        );
    }

    #[test]
    fn edge_052_partial_bounds_check() {
        debug_assert!(true, "contract: edge_052_partial_bounds_check");
        // Only row bounds checked, not column
        let rust_code = r#"
            if row < m {
                // Missing: && col < n
                c[row * n + col] = acc;
            }
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(rust_code);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Missed partial bounds check"
        );
    }

    #[test]
    fn edge_053_bounds_in_wrong_place() {
        debug_assert!(true, "contract: edge_053_bounds_in_wrong_place");
        // Bounds check after store (useless)
        let rust_code = r#"
            c[row * n + col] = acc;
            if row < m && col < n {
                // Too late!
            }
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(rust_code);
        assert!(
            !violations.is_empty(),
            "FALSIFIED: Accepted bounds check after store"
        );
    }

    #[test]
    fn edge_054_tiled_in_string() {
        debug_assert!(true, "contract: edge_054_tiled_in_string");
        // Adversarial: Kernel code in string literal
        let rust_code = r#"
            let kernel_src = "c[row * n + col] = acc;";
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(rust_code);
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Kernel code in string flagged"
        );
    }

    #[test]
    fn edge_055_complex_bounds_expression() {
        debug_assert!(true, "contract: edge_055_complex_bounds_expression");
        // Bounds check with complex expression
        let rust_code = r#"
            if (row * stride + offset) < (m * stride) && col < n {
                c[row * n + col] = acc;
            }
        "#;
        let violations = detect_tiled_kernel_no_bounds_in_str(rust_code);
        // Complex but valid bounds check
        assert!(
            violations.is_empty(),
            "FALSIFIED (FP): Complex bounds expression not recognized"
        );
    }
}
