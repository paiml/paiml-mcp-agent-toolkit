#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod stack_analyzer_tests {
    use super::*;

    #[test]
    fn test_stack_analyzer_new() {
        let analyzer = StackAnalyzer::new();
        assert!(analyzer.type_stack.is_empty());
    }

    #[test]
    fn test_stack_analyzer_i32_const() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::I32Const { value: 42 };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::I32);
    }

    #[test]
    fn test_stack_analyzer_i64_const() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::I64Const { value: 42 };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::I64);
    }

    #[test]
    fn test_stack_analyzer_f32_const() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::F32Const {
            value: wasmparser::Ieee32::from(0.0f32),
        };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::F32);
    }

    #[test]
    fn test_stack_analyzer_f64_const() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::F64Const {
            value: wasmparser::Ieee64::from(0.0f64),
        };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::F64);
    }

    #[test]
    fn test_stack_analyzer_i32_add_valid() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I32, ValType::I32];
        let op = Operator::I32Add;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::I32);
    }

    #[test]
    fn test_stack_analyzer_i32_add_underflow() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I32]; // Only one operand
        let op = Operator::I32Add;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_analyzer_i64_add_valid() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I64, ValType::I64];
        let op = Operator::I64Add;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::I64);
    }

    #[test]
    fn test_stack_analyzer_i32_eqz_valid() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I32];
        let op = Operator::I32Eqz;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], ValType::I32);
    }

    #[test]
    fn test_stack_analyzer_i32_eqz_type_error() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I64]; // Wrong type
        let op = Operator::I32Eqz;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_analyzer_i32_eq_valid() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I32, ValType::I32];
        let op = Operator::I32Eq;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_stack_analyzer_local_get() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::LocalGet { local_index: 0 };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_stack_analyzer_local_set_valid() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I32];
        let op = Operator::LocalSet { local_index: 0 };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_analyzer_local_set_underflow() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::LocalSet { local_index: 0 };
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_analyzer_drop_valid() {
        let analyzer = StackAnalyzer::new();
        let mut stack = vec![ValType::I32];
        let op = Operator::Drop;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_analyzer_drop_underflow() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::Drop;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_analyzer_nop() {
        let analyzer = StackAnalyzer::new();
        let mut stack = Vec::new();
        let op = Operator::Nop;
        let result = analyzer.update_stack(&mut stack, &op);
        assert!(result.is_ok());
        assert!(stack.is_empty());
    }

    /// OutOfBounds branch: offset + access_size > memory_size.
    #[test]
    fn test_check_i32_load_store_out_of_bounds() {
        let mut stack = vec![ValType::I32];
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 100,
            memory: 0,
        };
        // memory_size=10, access_size=4 → offset (100) > 10 - 4
        let result = check_i32_load_store(&mut stack, &memarg, 10, 4);
        assert!(matches!(
            result,
            Some(VerificationResult::OutOfBounds { .. })
        ));
    }

    /// TypeError branch: top of stack is not i32.
    #[test]
    fn test_check_i32_load_store_type_error() {
        let mut stack = vec![ValType::I64];
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };
        let result = check_i32_load_store(&mut stack, &memarg, 1024, 4);
        assert!(matches!(result, Some(VerificationResult::TypeError { .. })));
    }

    /// Success branch: i32 address on stack, offset within bounds → None.
    #[test]
    fn test_check_i32_load_store_success() {
        let mut stack = vec![ValType::I32];
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };
        let result = check_i32_load_store(&mut stack, &memarg, 1024, 4);
        assert!(result.is_none());
        // Address was popped off the stack.
        assert!(stack.is_empty());
    }

    /// TypeError branch: empty stack → pop returns None, not i32.
    #[test]
    fn test_check_i32_load_store_empty_stack() {
        let mut stack: Vec<ValType> = Vec::new();
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };
        let result = check_i32_load_store(&mut stack, &memarg, 1024, 4);
        assert!(matches!(result, Some(VerificationResult::TypeError { .. })));
    }
}
