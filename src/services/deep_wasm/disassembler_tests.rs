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
}
