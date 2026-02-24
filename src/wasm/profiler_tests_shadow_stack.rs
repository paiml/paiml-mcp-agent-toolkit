#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod shadow_stack_tests {
    use super::*;

    // ==================== ShadowStack Tests ====================

    #[test]
    fn test_shadow_stack_from_bytes_empty() {
        let stack = ShadowStack::from_bytes(vec![]);
        assert!(stack.frames.is_empty());
    }

    #[test]
    fn test_shadow_stack_from_bytes_single_frame() {
        // Function index 5 in little-endian
        let bytes = vec![0x05, 0x00, 0x00, 0x00];
        let stack = ShadowStack::from_bytes(bytes);
        assert_eq!(stack.frames.len(), 1);
        assert_eq!(stack.frames[0].function_index, 5);
    }

    #[test]
    fn test_shadow_stack_from_bytes_multiple_frames() {
        let bytes = vec![
            0x01, 0x00, 0x00, 0x00, // func 1
            0x02, 0x00, 0x00, 0x00, // func 2
            0x03, 0x00, 0x00, 0x00, // func 3
        ];
        let stack = ShadowStack::from_bytes(bytes);
        assert_eq!(stack.frames.len(), 3);
        assert_eq!(stack.frames[0].function_index, 1);
        assert_eq!(stack.frames[1].function_index, 2);
        assert_eq!(stack.frames[2].function_index, 3);
    }

    #[test]
    fn test_shadow_stack_from_bytes_zero_index_filtered() {
        // Zero function index should be filtered out
        let bytes = vec![0x00, 0x00, 0x00, 0x00];
        let stack = ShadowStack::from_bytes(bytes);
        assert!(stack.frames.is_empty());
    }

    #[test]
    fn test_shadow_stack_from_bytes_partial_chunk() {
        // Only 3 bytes, not enough for a frame
        let bytes = vec![0x01, 0x02, 0x03];
        let stack = ShadowStack::from_bytes(bytes);
        assert!(stack.frames.is_empty());
    }

    #[test]
    fn test_shadow_stack_sample() {
        let stack = ShadowStack::sample();
        assert_eq!(stack.frames.len(), 2);
        assert_eq!(stack.frames[0].function_index, 1);
        assert_eq!(stack.frames[0].instruction_offset, 10);
        assert_eq!(stack.frames[1].function_index, 5);
        assert_eq!(stack.frames[1].instruction_offset, 42);
    }

    #[test]
    fn test_shadow_stack_depth() {
        let stack = ShadowStack::sample();
        assert_eq!(stack.depth(), 2);
    }

    #[test]
    fn test_shadow_stack_depth_empty() {
        let stack = ShadowStack::from_bytes(vec![]);
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn test_shadow_stack_contains_function_found() {
        let stack = ShadowStack::sample();
        assert!(stack.contains_function(1));
        assert!(stack.contains_function(5));
    }

    #[test]
    fn test_shadow_stack_contains_function_not_found() {
        let stack = ShadowStack::sample();
        assert!(!stack.contains_function(0));
        assert!(!stack.contains_function(999));
    }

    #[test]
    fn test_shadow_stack_clone() {
        let stack = ShadowStack::sample();
        let cloned = stack.clone();
        assert_eq!(stack.frames.len(), cloned.frames.len());
    }

    // ==================== StackFrame Tests ====================

    #[test]
    fn test_stack_frame_clone() {
        let frame = StackFrame {
            function_index: 42,
            instruction_offset: 100,
        };
        let cloned = frame.clone();
        assert_eq!(frame.function_index, cloned.function_index);
        assert_eq!(frame.instruction_offset, cloned.instruction_offset);
    }

    // ==================== ProfileAggregator Tests ====================

    #[test]
    fn test_profile_aggregator_new() {
        let aggregator = ProfileAggregator::new();
        assert!(aggregator.profiles.is_empty());
    }

    #[test]
    fn test_profile_aggregator_default() {
        let aggregator = ProfileAggregator::default();
        assert!(aggregator.profiles.is_empty());
    }

    #[test]
    fn test_profile_aggregator_add_profile() {
        let mut aggregator = ProfileAggregator::new();

        let profile = ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 20,
                memory_ops: 30,
                arithmetic: 40,
                calls: 10,
            },
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: Some(10),
                growth_events: vec![],
            },
        };

        aggregator.add_profile(profile);
        assert_eq!(aggregator.profiles.len(), 1);
    }

    #[test]
    fn test_average_instruction_mix_empty() {
        let aggregator = ProfileAggregator::new();
        let avg = aggregator.average_instruction_mix();

        assert_eq!(avg.total_instructions, 0);
        assert_eq!(avg.control_flow, 0);
        assert_eq!(avg.memory_ops, 0);
        assert_eq!(avg.arithmetic, 0);
        assert_eq!(avg.calls, 0);
    }

    #[test]
    fn test_average_instruction_mix_single() {
        let mut aggregator = ProfileAggregator::new();

        aggregator.add_profile(ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 20,
                memory_ops: 30,
                arithmetic: 40,
                calls: 10,
            },
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: None,
                growth_events: vec![],
            },
        });

        let avg = aggregator.average_instruction_mix();
        assert_eq!(avg.total_instructions, 100);
        assert_eq!(avg.control_flow, 20);
        assert_eq!(avg.memory_ops, 30);
        assert_eq!(avg.arithmetic, 40);
        assert_eq!(avg.calls, 10);
    }

    #[test]
    fn test_average_instruction_mix_multiple() {
        let mut aggregator = ProfileAggregator::new();

        aggregator.add_profile(ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 20,
                memory_ops: 30,
                arithmetic: 40,
                calls: 10,
            },
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: None,
                growth_events: vec![],
            },
        });

        aggregator.add_profile(ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 200,
                control_flow: 40,
                memory_ops: 60,
                arithmetic: 80,
                calls: 20,
            },
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 2,
                max_pages: None,
                growth_events: vec![],
            },
        });

        let avg = aggregator.average_instruction_mix();
        assert_eq!(avg.total_instructions, 150); // (100 + 200) / 2
        assert_eq!(avg.control_flow, 30);
        assert_eq!(avg.memory_ops, 45);
        assert_eq!(avg.arithmetic, 60);
        assert_eq!(avg.calls, 15);
    }

    // ==================== categorize_for_profiling Tests ====================

    #[test]
    fn test_categorize_control_flow() {
        use wasmparser::Operator;

        assert!(matches!(
            categorize_for_profiling(&Operator::Block {
                blockty: wasmparser::BlockType::Empty
            }),
            InstructionCategory::ControlFlow
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::Loop {
                blockty: wasmparser::BlockType::Empty
            }),
            InstructionCategory::ControlFlow
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::If {
                blockty: wasmparser::BlockType::Empty
            }),
            InstructionCategory::ControlFlow
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::Br { relative_depth: 0 }),
            InstructionCategory::ControlFlow
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::Return),
            InstructionCategory::ControlFlow
        ));
    }

    #[test]
    fn test_categorize_memory() {
        use wasmparser::{MemArg, Operator};

        let memarg = MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };
        assert!(matches!(
            categorize_for_profiling(&Operator::I32Load { memarg }),
            InstructionCategory::Memory
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::I32Store { memarg }),
            InstructionCategory::Memory
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::MemoryGrow { mem: 0 }),
            InstructionCategory::Memory
        ));
    }

    #[test]
    fn test_categorize_arithmetic() {
        use wasmparser::Operator;

        assert!(matches!(
            categorize_for_profiling(&Operator::I32Add),
            InstructionCategory::Arithmetic
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::I32Sub),
            InstructionCategory::Arithmetic
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::I64Mul),
            InstructionCategory::Arithmetic
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::F32Div),
            InstructionCategory::Arithmetic
        ));
    }

    #[test]
    fn test_categorize_call() {
        use wasmparser::Operator;

        assert!(matches!(
            categorize_for_profiling(&Operator::Call { function_index: 0 }),
            InstructionCategory::Call
        ));
    }

    #[test]
    fn test_categorize_other() {
        use wasmparser::Operator;

        assert!(matches!(
            categorize_for_profiling(&Operator::Nop),
            InstructionCategory::Other
        ));
        assert!(matches!(
            categorize_for_profiling(&Operator::I32Const { value: 0 }),
            InstructionCategory::Other
        ));
    }
}
