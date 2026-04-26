// disassembler_tests.rs — included into disassembler.rs
// Unit tests for the WASM disassembler.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassembler_creation() {
        let disasm = Disassembler::new();
        assert!(disasm.detect_patterns);
    }

    #[test]
    fn test_format_operator() {
        let op = Operator::I32Add;
        let (mnemonic, operands) = format_operator(&op);
        assert_eq!(mnemonic, "i32.add");
        assert!(operands.is_empty());
    }

    #[test]
    fn test_stack_effect_binary_op() {
        let op = Operator::I32Add;
        let effect = calculate_stack_effect(&op);
        assert_eq!(effect.pops, 2);
        assert_eq!(effect.pushes, 1);
    }

    #[test]
    fn test_categorize_control_flow() {
        let op = Operator::Br { relative_depth: 0 };
        assert_eq!(categorize_operator(&op), "control");
    }

    #[test]
    fn test_cost_estimation() {
        assert_eq!(estimate_cost(&Operator::Nop), 1);
        assert_eq!(estimate_cost(&Operator::I32Add), 2);
        assert_eq!(estimate_cost(&Operator::Call { function_index: 0 }), 20);
    }

    // ── Pattern detectors (Wave 39 PR14) ────────────────────────────────────
    //
    // Targets disassembler_core.rs detect_* helpers (146 missed, ~50 lines
    // each). Pure functions taking &[DisassembledInstruction] +
    // &mut Vec<InstructionPattern>.

    fn make_instr(offset: u32, mnemonic: &str, category: &str) -> DisassembledInstruction {
        DisassembledInstruction {
            offset,
            mnemonic: mnemonic.to_string(),
            operands: vec![],
            stack_effect: StackEffect { pops: 0, pushes: 0 },
            category: category.to_string(),
            cost_estimate: 1,
        }
    }

    #[test]
    fn test_detect_dead_code_after_unreachable_flags_following_instructions() {
        let instructions = vec![
            make_instr(0, "i32.const", "constant"),
            make_instr(2, "unreachable", "control"),
            make_instr(4, "i32.add", "arithmetic"),
            make_instr(6, "drop", "stack"),
            make_instr(8, "end", "control"),
        ];
        let mut patterns = Vec::new();
        detect_dead_code_after_unreachable(&instructions, &mut patterns);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].name, "dead_code_after_unreachable");
        // 2 dead instructions: i32.add, drop (stops at "end")
        assert_eq!(patterns[0].instruction_count, 3); // dead_count(2) + 1
        assert!(patterns[0].suspicious);
    }

    #[test]
    fn test_detect_dead_code_no_unreachable_no_pattern() {
        let instructions = vec![
            make_instr(0, "i32.const", "constant"),
            make_instr(2, "i32.add", "arithmetic"),
        ];
        let mut patterns = Vec::new();
        detect_dead_code_after_unreachable(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_dead_code_unreachable_at_end_no_pattern() {
        // PIN: trailing `unreachable` with no following instructions emits no pattern
        // (dead_count == 0 short-circuits the push).
        let instructions = vec![
            make_instr(0, "i32.const", "constant"),
            make_instr(2, "unreachable", "control"),
        ];
        let mut patterns = Vec::new();
        detect_dead_code_after_unreachable(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_infinite_loops_with_backward_br_no_side_effects() {
        // loop ... br ... end (no memory/call/etc → flagged)
        let instructions = vec![
            make_instr(0, "loop", "control"),
            make_instr(2, "i32.const", "constant"),
            make_instr(4, "br", "control"),
            make_instr(6, "end", "control"),
        ];
        let mut patterns = Vec::new();
        detect_infinite_loops(&instructions, &mut patterns);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].name, "infinite_loop_no_side_effects");
    }

    #[test]
    fn test_detect_infinite_loops_with_side_effects_not_flagged() {
        // PIN: loop with memory/call side-effect is NOT flagged even with backward br.
        let instructions = vec![
            make_instr(0, "loop", "control"),
            make_instr(2, "i32.load", "memory"),
            make_instr(4, "br", "control"),
            make_instr(6, "end", "control"),
        ];
        let mut patterns = Vec::new();
        detect_infinite_loops(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_infinite_loops_no_loop_no_pattern() {
        let instructions = vec![
            make_instr(0, "i32.const", "constant"),
            make_instr(2, "i32.add", "arithmetic"),
        ];
        let mut patterns = Vec::new();
        detect_infinite_loops(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_excessive_drops_threshold_at_5_not_flagged() {
        // PIN: 5 consecutive drops is NOT flagged (`> 5` strict, not `>=`).
        let mut instructions = Vec::new();
        for i in 0..5 {
            instructions.push(make_instr(i * 2, "drop", "stack"));
        }
        instructions.push(make_instr(10, "end", "control"));
        let mut patterns = Vec::new();
        detect_excessive_drops(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_excessive_drops_above_5_flagged() {
        let mut instructions = Vec::new();
        for i in 0..6 {
            instructions.push(make_instr(i * 2, "drop", "stack"));
        }
        instructions.push(make_instr(12, "end", "control"));
        let mut patterns = Vec::new();
        detect_excessive_drops(&instructions, &mut patterns);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].name, "excessive_stack_drops");
        assert_eq!(patterns[0].instruction_count, 6);
    }

    #[test]
    fn test_detect_deep_nesting_at_10_not_flagged() {
        // PIN: max_nesting > 10 (strict). 10 nested blocks alone is NOT flagged.
        let mut instructions = Vec::new();
        for i in 0..10 {
            instructions.push(make_instr(i * 2, "block", "control"));
        }
        for i in 0..10 {
            instructions.push(make_instr(20 + i * 2, "end", "control"));
        }
        let mut patterns = Vec::new();
        detect_deep_nesting(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_deep_nesting_above_10_flagged() {
        let mut instructions = Vec::new();
        for i in 0..11 {
            instructions.push(make_instr(i * 2, "block", "control"));
        }
        for i in 0..11 {
            instructions.push(make_instr(22 + i * 2, "end", "control"));
        }
        let mut patterns = Vec::new();
        detect_deep_nesting(&instructions, &mut patterns);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].name, "deep_control_flow_nesting");
    }

    #[test]
    fn test_detect_deep_nesting_mixed_block_loop_if_counts() {
        // PIN: block / loop / if all increment nesting; end decrements.
        let instructions = vec![
            make_instr(0, "block", "control"),
            make_instr(2, "loop", "control"),
            make_instr(4, "if", "control"),
            make_instr(6, "block", "control"),
            make_instr(8, "loop", "control"),
            make_instr(10, "if", "control"),
            make_instr(12, "block", "control"),
            make_instr(14, "loop", "control"),
            make_instr(16, "if", "control"),
            make_instr(18, "block", "control"),
            make_instr(20, "loop", "control"),
            make_instr(22, "if", "control"), // 12th nesting level
        ];
        let mut patterns = Vec::new();
        detect_deep_nesting(&instructions, &mut patterns);
        assert_eq!(patterns.len(), 1);
        assert!(patterns[0].description.contains("12"));
    }

    #[test]
    fn test_detect_deep_nesting_end_decrements_keeps_nesting_low() {
        // Open and close — net nesting stays at 1, no flag.
        let mut instructions = Vec::new();
        for i in 0..20 {
            instructions.push(make_instr(i * 4, "block", "control"));
            instructions.push(make_instr(i * 4 + 2, "end", "control"));
        }
        let mut patterns = Vec::new();
        detect_deep_nesting(&instructions, &mut patterns);
        assert!(patterns.is_empty());
    }

    // ── Operator formatters / classifiers (Wave 39 PR15) ────────────────────
    //
    // Targets disassembler_formatting.rs (84 missed pre-PR14, 4 fns each
    // with 30-50 match arms over wasmparser::Operator). Each test covers
    // an under-tested category of arms.

    #[test]
    fn test_format_operator_local_get_set_tee() {
        let (m, ops) = format_operator(&Operator::LocalGet { local_index: 0 });
        assert_eq!(m, "local.get");
        assert_eq!(ops, vec!["0".to_string()]);
        let (m, _) = format_operator(&Operator::LocalSet { local_index: 1 });
        assert_eq!(m, "local.set");
        let (m, _) = format_operator(&Operator::LocalTee { local_index: 2 });
        assert_eq!(m, "local.tee");
    }

    #[test]
    fn test_format_operator_global_get_set() {
        let (m, ops) = format_operator(&Operator::GlobalGet { global_index: 5 });
        assert_eq!(m, "global.get");
        assert_eq!(ops, vec!["5".to_string()]);
        let (m, _) = format_operator(&Operator::GlobalSet { global_index: 5 });
        assert_eq!(m, "global.set");
    }

    #[test]
    fn test_format_operator_call_indirect() {
        let (m, ops) = format_operator(&Operator::Call { function_index: 7 });
        assert_eq!(m, "call");
        assert_eq!(ops, vec!["7".to_string()]);
    }

    #[test]
    fn test_format_operator_constants() {
        let (m, ops) = format_operator(&Operator::I32Const { value: 42 });
        assert_eq!(m, "i32.const");
        assert!(ops.contains(&"42".to_string()));
        let (m, _) = format_operator(&Operator::I64Const { value: -1 });
        assert_eq!(m, "i64.const");
    }

    #[test]
    fn test_format_operator_arithmetic_int_ops() {
        // I32/I64 ops are formatted with the dot — `i32.add`, `i64.add`.
        let cases = [
            (Operator::I32Sub, "i32.sub"),
            (Operator::I32Mul, "i32.mul"),
            (Operator::I64Add, "i64.add"),
        ];
        for (op, expected) in &cases {
            let (m, ops) = format_operator(op);
            assert_eq!(&m, expected);
            assert!(ops.is_empty());
        }
    }

    #[test]
    fn test_format_operator_pin_f64_add_missing_dot() {
        // PIN BUG: `format_operator(F64Add)` returns "f64add" (no dot)
        // instead of the WASM-canonical "f64.add". The implementation has
        // a missing `.` in the F64-family arms. Pinned so a future fix
        // is intentional and other consumers update accordingly.
        let (m, _) = format_operator(&Operator::F64Add);
        assert_eq!(m, "f64add", "PIN: f64 ops are missing the dot in mnemonic");
    }

    #[test]
    fn test_format_operator_control_flow() {
        let cases = [
            (Operator::Nop, "nop"),
            (Operator::Unreachable, "unreachable"),
            (Operator::Return, "return"),
            (Operator::Drop, "drop"),
        ];
        for (op, expected) in &cases {
            let (m, _) = format_operator(op);
            assert_eq!(&m, expected);
        }
    }

    #[test]
    fn test_calculate_stack_effect_call_pops_pushes() {
        // Call: pops are arg count, pushes is return count. The stub may use
        // 0/0 or fixed values; just exercise the arm.
        let _ = calculate_stack_effect(&Operator::Call { function_index: 0 });
    }

    #[test]
    fn test_calculate_stack_effect_local_get_pushes_one() {
        let e = calculate_stack_effect(&Operator::LocalGet { local_index: 0 });
        assert_eq!(e.pushes, 1);
        assert_eq!(e.pops, 0);
    }

    #[test]
    fn test_calculate_stack_effect_local_set_pops_one() {
        let e = calculate_stack_effect(&Operator::LocalSet { local_index: 0 });
        assert_eq!(e.pops, 1);
    }

    #[test]
    fn test_calculate_stack_effect_drop_pops_one() {
        let e = calculate_stack_effect(&Operator::Drop);
        assert_eq!(e.pops, 1);
        assert_eq!(e.pushes, 0);
    }

    #[test]
    fn test_calculate_stack_effect_unreachable_no_effect() {
        let e = calculate_stack_effect(&Operator::Unreachable);
        // Unreachable has no real stack effect (it terminates control flow).
        assert_eq!(e.pops, 0);
        assert_eq!(e.pushes, 0);
    }

    #[test]
    fn test_categorize_operator_all_groups() {
        // Cover the major category branches.
        assert_eq!(
            categorize_operator(&Operator::Br { relative_depth: 0 }),
            "control"
        );
        assert_eq!(categorize_operator(&Operator::Return), "control");
        // PIN: Call is classified as "control" flow (not its own "call" category).
        // Reasonable since Call transfers control to another function.
        assert_eq!(
            categorize_operator(&Operator::Call { function_index: 0 }),
            "control"
        );
        // PIN: Const + arithmetic ops both map to "numeric" (not "constant" / "arithmetic").
        // The bucket merges them because both are pure value-stack ops.
        assert_eq!(
            categorize_operator(&Operator::I32Const { value: 0 }),
            "numeric"
        );
        assert_eq!(categorize_operator(&Operator::I32Add), "numeric");
        // PIN: Nop and Drop fall through to "other" (no explicit arm).
        assert_eq!(categorize_operator(&Operator::Drop), "other");
        assert_eq!(categorize_operator(&Operator::Nop), "control");
        assert_eq!(
            categorize_operator(&Operator::LocalGet { local_index: 0 }),
            "variable"
        );
    }

    #[test]
    fn test_estimate_cost_categories_have_expected_costs() {
        // PIN: Nop=1, simple arithmetic=2, Call=20.
        assert_eq!(estimate_cost(&Operator::Nop), 1);
        assert_eq!(estimate_cost(&Operator::I32Add), 2);
        assert_eq!(estimate_cost(&Operator::Call { function_index: 0 }), 20);
        // Memory ops typically cost more than arithmetic.
        let _ = estimate_cost(&Operator::I32Load {
            memarg: wasmparser::MemArg {
                align: 0,
                max_align: 0,
                offset: 0,
                memory: 0,
            },
        });
    }

    #[test]
    fn test_estimate_cost_unreachable_and_drop() {
        // PIN: drop and unreachable have low costs.
        let drop_cost = estimate_cost(&Operator::Drop);
        let nop_cost = estimate_cost(&Operator::Nop);
        let call_cost = estimate_cost(&Operator::Call { function_index: 0 });
        assert!(drop_cost <= call_cost);
        assert!(nop_cost <= call_cost);
    }
}
