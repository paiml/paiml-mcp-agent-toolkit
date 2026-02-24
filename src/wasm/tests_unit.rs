// Unit tests and property tests for WASM types

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_mix_default() {
        let mix = InstructionMix {
            total_instructions: 100,
            control_flow: 20,
            memory_ops: 30,
            arithmetic: 40,
            calls: 10,
        };
        assert_eq!(mix.total_instructions, 100);
        assert_eq!(
            mix.control_flow + mix.memory_ops + mix.arithmetic + mix.calls,
            100
        );
    }

    #[test]
    fn test_hot_function_creation() {
        let hot = HotFunction {
            name: "main".to_string(),
            samples: 1000,
            percentage: 50.0,
        };
        assert_eq!(hot.name, "main");
        assert_eq!(hot.samples, 1000);
        assert!((hot.percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_profile_creation() {
        let profile = MemoryProfile {
            initial_pages: 1,
            max_pages: Some(16),
            growth_events: vec![],
        };
        assert_eq!(profile.initial_pages, 1);
        assert_eq!(profile.max_pages, Some(16));
        assert!(profile.growth_events.is_empty());
    }

    #[test]
    fn test_growth_event_creation() {
        let event = GrowthEvent {
            timestamp: 1000,
            pages_before: 1,
            pages_after: 2,
        };
        assert_eq!(event.timestamp, 1000);
        assert_eq!(event.pages_before, 1);
        assert_eq!(event.pages_after, 2);
    }

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
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: None,
                growth_events: vec![],
            },
        };
        assert_eq!(report.instruction_mix.total_instructions, 100);
        assert!(report.hot_functions.is_empty());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn instruction_mix_invariant(
            control in 0usize..100,
            memory in 0usize..100,
            arithmetic in 0usize..100,
            calls in 0usize..100
        ) {
            let total = control + memory + arithmetic + calls;
            let mix = super::InstructionMix {
                total_instructions: total,
                control_flow: control,
                memory_ops: memory,
                arithmetic,
                calls,
            };
            prop_assert_eq!(
                mix.control_flow + mix.memory_ops + mix.arithmetic + mix.calls,
                mix.total_instructions
            );
        }
    }
}
