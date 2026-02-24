// Coverage tests: ProfilingReport and InstructionMix struct tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_report {
    use super::*;

    // =====================================================
    // ProfilingReport tests
    // =====================================================

    #[test]
    fn test_profiling_report_creation() {
        let report = ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 20,
                memory_ops: 30,
                arithmetic: 40,
                calls: 10,
            },
            hot_functions: vec![HotFunction {
                name: "main".to_string(),
                samples: 500,
                percentage: 50.0,
            }],
            memory_usage: MemoryProfile {
                initial_pages: 2,
                max_pages: Some(16),
                growth_events: vec![],
            },
        };

        assert_eq!(report.instruction_mix.total_instructions, 100);
        assert_eq!(report.hot_functions.len(), 1);
        assert_eq!(report.memory_usage.initial_pages, 2);
    }

    #[test]
    fn test_profiling_report_clone() {
        let report = ProfilingReport {
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
        };

        let cloned = report.clone();
        assert_eq!(
            report.instruction_mix.total_instructions,
            cloned.instruction_mix.total_instructions
        );
        assert_eq!(
            report.memory_usage.initial_pages,
            cloned.memory_usage.initial_pages
        );
    }

    #[test]
    fn test_profiling_report_serialization() {
        let report = ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 50,
                control_flow: 10,
                memory_ops: 15,
                arithmetic: 20,
                calls: 5,
            },
            hot_functions: vec![HotFunction {
                name: "func_0".to_string(),
                samples: 100,
                percentage: 75.5,
            }],
            memory_usage: MemoryProfile {
                initial_pages: 4,
                max_pages: Some(64),
                growth_events: vec![GrowthEvent {
                    timestamp: 1000,
                    pages_before: 4,
                    pages_after: 8,
                }],
            },
        };

        let serialized = serde_json::to_string(&report).unwrap();
        let deserialized: ProfilingReport = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            report.instruction_mix.total_instructions,
            deserialized.instruction_mix.total_instructions
        );
        assert_eq!(report.hot_functions.len(), deserialized.hot_functions.len());
        assert_eq!(
            report.memory_usage.growth_events.len(),
            deserialized.memory_usage.growth_events.len()
        );
    }

    #[test]
    fn test_profiling_report_debug_format() {
        let report = ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 10,
                control_flow: 2,
                memory_ops: 3,
                arithmetic: 4,
                calls: 1,
            },
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: None,
                growth_events: vec![],
            },
        };

        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("ProfilingReport"));
        assert!(debug_str.contains("instruction_mix"));
    }

    // =====================================================
    // InstructionMix tests
    // =====================================================

    #[test]
    fn test_instruction_mix_creation() {
        let mix = InstructionMix {
            total_instructions: 100,
            control_flow: 20,
            memory_ops: 30,
            arithmetic: 40,
            calls: 10,
        };

        assert_eq!(mix.total_instructions, 100);
        assert_eq!(mix.control_flow, 20);
        assert_eq!(mix.memory_ops, 30);
        assert_eq!(mix.arithmetic, 40);
        assert_eq!(mix.calls, 10);
    }

    #[test]
    fn test_instruction_mix_clone() {
        let mix = InstructionMix {
            total_instructions: 50,
            control_flow: 10,
            memory_ops: 15,
            arithmetic: 20,
            calls: 5,
        };

        let cloned = mix.clone();
        assert_eq!(mix.total_instructions, cloned.total_instructions);
        assert_eq!(mix.control_flow, cloned.control_flow);
        assert_eq!(mix.memory_ops, cloned.memory_ops);
        assert_eq!(mix.arithmetic, cloned.arithmetic);
        assert_eq!(mix.calls, cloned.calls);
    }

    #[test]
    fn test_instruction_mix_serialization() {
        let mix = InstructionMix {
            total_instructions: 200,
            control_flow: 40,
            memory_ops: 60,
            arithmetic: 80,
            calls: 20,
        };

        let serialized = serde_json::to_string(&mix).unwrap();
        let deserialized: InstructionMix = serde_json::from_str(&serialized).unwrap();

        assert_eq!(mix.total_instructions, deserialized.total_instructions);
        assert_eq!(mix.control_flow, deserialized.control_flow);
        assert_eq!(mix.memory_ops, deserialized.memory_ops);
        assert_eq!(mix.arithmetic, deserialized.arithmetic);
        assert_eq!(mix.calls, deserialized.calls);
    }

    #[test]
    fn test_instruction_mix_zero_values() {
        let mix = InstructionMix {
            total_instructions: 0,
            control_flow: 0,
            memory_ops: 0,
            arithmetic: 0,
            calls: 0,
        };

        assert_eq!(mix.total_instructions, 0);
        assert_eq!(
            mix.control_flow + mix.memory_ops + mix.arithmetic + mix.calls,
            0
        );
    }

    #[test]
    fn test_instruction_mix_large_values() {
        let mix = InstructionMix {
            total_instructions: usize::MAX,
            control_flow: 1000000,
            memory_ops: 2000000,
            arithmetic: 3000000,
            calls: 4000000,
        };

        assert_eq!(mix.total_instructions, usize::MAX);
    }
}
