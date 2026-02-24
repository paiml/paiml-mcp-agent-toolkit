// Coverage tests: HotFunction, MemoryProfile, GrowthEvent, re-exports, and edge cases

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_types {
    use super::*;

    // =====================================================
    // HotFunction tests
    // =====================================================

    #[test]
    fn test_hot_function_creation() {
        let hot = HotFunction {
            name: "main".to_string(),
            samples: 1000,
            percentage: 50.5,
        };

        assert_eq!(hot.name, "main");
        assert_eq!(hot.samples, 1000);
        assert!((hot.percentage - 50.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hot_function_clone() {
        let hot = HotFunction {
            name: "compute".to_string(),
            samples: 500,
            percentage: 25.0,
        };

        let cloned = hot.clone();
        assert_eq!(hot.name, cloned.name);
        assert_eq!(hot.samples, cloned.samples);
        assert_eq!(hot.percentage, cloned.percentage);
    }

    #[test]
    fn test_hot_function_serialization() {
        let hot = HotFunction {
            name: "process_data".to_string(),
            samples: 750,
            percentage: 37.5,
        };

        let serialized = serde_json::to_string(&hot).unwrap();
        let deserialized: HotFunction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(hot.name, deserialized.name);
        assert_eq!(hot.samples, deserialized.samples);
        assert_eq!(hot.percentage, deserialized.percentage);
    }

    #[test]
    fn test_hot_function_empty_name() {
        let hot = HotFunction {
            name: String::new(),
            samples: 0,
            percentage: 0.0,
        };

        assert!(hot.name.is_empty());
        assert_eq!(hot.samples, 0);
    }

    #[test]
    fn test_hot_function_100_percent() {
        let hot = HotFunction {
            name: "only_function".to_string(),
            samples: 10000,
            percentage: 100.0,
        };

        assert_eq!(hot.percentage, 100.0);
    }

    // =====================================================
    // MemoryProfile tests
    // =====================================================

    #[test]
    fn test_memory_profile_creation() {
        let profile = MemoryProfile {
            initial_pages: 4,
            max_pages: Some(64),
            growth_events: vec![],
        };

        assert_eq!(profile.initial_pages, 4);
        assert_eq!(profile.max_pages, Some(64));
        assert!(profile.growth_events.is_empty());
    }

    #[test]
    fn test_memory_profile_no_max() {
        let profile = MemoryProfile {
            initial_pages: 1,
            max_pages: None,
            growth_events: vec![],
        };

        assert_eq!(profile.initial_pages, 1);
        assert!(profile.max_pages.is_none());
    }

    #[test]
    fn test_memory_profile_clone() {
        let profile = MemoryProfile {
            initial_pages: 2,
            max_pages: Some(32),
            growth_events: vec![GrowthEvent {
                timestamp: 100,
                pages_before: 2,
                pages_after: 4,
            }],
        };

        let cloned = profile.clone();
        assert_eq!(profile.initial_pages, cloned.initial_pages);
        assert_eq!(profile.max_pages, cloned.max_pages);
        assert_eq!(profile.growth_events.len(), cloned.growth_events.len());
    }

    #[test]
    fn test_memory_profile_serialization() {
        let profile = MemoryProfile {
            initial_pages: 8,
            max_pages: Some(128),
            growth_events: vec![
                GrowthEvent {
                    timestamp: 500,
                    pages_before: 8,
                    pages_after: 16,
                },
                GrowthEvent {
                    timestamp: 1000,
                    pages_before: 16,
                    pages_after: 32,
                },
            ],
        };

        let serialized = serde_json::to_string(&profile).unwrap();
        let deserialized: MemoryProfile = serde_json::from_str(&serialized).unwrap();

        assert_eq!(profile.initial_pages, deserialized.initial_pages);
        assert_eq!(profile.max_pages, deserialized.max_pages);
        assert_eq!(
            profile.growth_events.len(),
            deserialized.growth_events.len()
        );
    }

    #[test]
    fn test_memory_profile_with_multiple_growth_events() {
        let profile = MemoryProfile {
            initial_pages: 1,
            max_pages: Some(256),
            growth_events: vec![
                GrowthEvent {
                    timestamp: 100,
                    pages_before: 1,
                    pages_after: 2,
                },
                GrowthEvent {
                    timestamp: 200,
                    pages_before: 2,
                    pages_after: 4,
                },
                GrowthEvent {
                    timestamp: 300,
                    pages_before: 4,
                    pages_after: 8,
                },
                GrowthEvent {
                    timestamp: 400,
                    pages_before: 8,
                    pages_after: 16,
                },
            ],
        };

        assert_eq!(profile.growth_events.len(), 4);
        assert_eq!(profile.growth_events[0].pages_before, 1);
        assert_eq!(profile.growth_events[3].pages_after, 16);
    }

    // =====================================================
    // GrowthEvent tests
    // =====================================================

    #[test]
    fn test_growth_event_creation() {
        let event = GrowthEvent {
            timestamp: 12345,
            pages_before: 4,
            pages_after: 8,
        };

        assert_eq!(event.timestamp, 12345);
        assert_eq!(event.pages_before, 4);
        assert_eq!(event.pages_after, 8);
    }

    #[test]
    fn test_growth_event_clone() {
        let event = GrowthEvent {
            timestamp: 5000,
            pages_before: 16,
            pages_after: 32,
        };

        let cloned = event.clone();
        assert_eq!(event.timestamp, cloned.timestamp);
        assert_eq!(event.pages_before, cloned.pages_before);
        assert_eq!(event.pages_after, cloned.pages_after);
    }

    #[test]
    fn test_growth_event_serialization() {
        let event = GrowthEvent {
            timestamp: 99999,
            pages_before: 64,
            pages_after: 128,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: GrowthEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event.timestamp, deserialized.timestamp);
        assert_eq!(event.pages_before, deserialized.pages_before);
        assert_eq!(event.pages_after, deserialized.pages_after);
    }

    #[test]
    fn test_growth_event_zero_timestamp() {
        let event = GrowthEvent {
            timestamp: 0,
            pages_before: 1,
            pages_after: 2,
        };

        assert_eq!(event.timestamp, 0);
    }

    #[test]
    fn test_growth_event_large_timestamp() {
        let event = GrowthEvent {
            timestamp: u64::MAX,
            pages_before: 1,
            pages_after: 2,
        };

        assert_eq!(event.timestamp, u64::MAX);
    }

    #[test]
    fn test_growth_event_same_pages() {
        let event = GrowthEvent {
            timestamp: 100,
            pages_before: 8,
            pages_after: 8,
        };

        assert_eq!(event.pages_before, event.pages_after);
    }

    // =====================================================
    // Re-export verification tests
    // =====================================================

    #[test]
    fn test_wasm_analyzer_reexport() {
        let analyzer = WasmAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_incremental_verifier_reexport() {
        let verifier = IncrementalVerifier::new();
        assert!(verifier.is_ok());
    }

    #[test]
    fn test_async_profiler_reexport() {
        let profiler = AsyncProfiler::new();
        assert!(std::mem::size_of_val(&profiler) > 0);
    }

    #[test]
    fn test_shadow_stack_reexport() {
        let stack = ShadowStack::sample();
        assert!(stack.depth() > 0);
    }

    #[test]
    fn test_quality_baseline_reexport() {
        use super::baseline::Metrics;
        let release_metrics = Metrics::default();
        let stable_metrics = Metrics::default();
        let baseline = QualityBaseline::new(release_metrics, stable_metrics);
        assert!(std::mem::size_of_val(&baseline) > 0);
    }

    #[test]
    fn test_pattern_detector_reexport() {
        let detector = PatternDetector::new();
        assert!(std::mem::size_of_val(&detector) > 0);
    }

    // =====================================================
    // Edge case tests
    // =====================================================

    #[test]
    fn test_instruction_mix_components_exceed_total() {
        // This is technically invalid data but should not panic
        let mix = InstructionMix {
            total_instructions: 10,
            control_flow: 100,
            memory_ops: 100,
            arithmetic: 100,
            calls: 100,
        };

        // Should still be serializable
        let serialized = serde_json::to_string(&mix).unwrap();
        let deserialized: InstructionMix = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mix.total_instructions, deserialized.total_instructions);
    }

    #[test]
    fn test_profiling_report_empty_hot_functions() {
        let report = ProfilingReport {
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 25,
                memory_ops: 25,
                arithmetic: 25,
                calls: 25,
            },
            hot_functions: vec![],
            memory_usage: MemoryProfile {
                initial_pages: 1,
                max_pages: None,
                growth_events: vec![],
            },
        };

        assert!(report.hot_functions.is_empty());
    }

    #[test]
    fn test_hot_function_special_characters_in_name() {
        let hot = HotFunction {
            name: "module::namespace::func<T>".to_string(),
            samples: 100,
            percentage: 10.0,
        };

        let serialized = serde_json::to_string(&hot).unwrap();
        let deserialized: HotFunction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(hot.name, deserialized.name);
    }

    #[test]
    fn test_hot_function_unicode_name() {
        let hot = HotFunction {
            name: "函数_处理".to_string(),
            samples: 50,
            percentage: 5.0,
        };

        let serialized = serde_json::to_string(&hot).unwrap();
        let deserialized: HotFunction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(hot.name, deserialized.name);
    }

    #[test]
    fn test_memory_profile_zero_pages() {
        let profile = MemoryProfile {
            initial_pages: 0,
            max_pages: Some(0),
            growth_events: vec![],
        };

        assert_eq!(profile.initial_pages, 0);
        assert_eq!(profile.max_pages, Some(0));
    }

    #[test]
    fn test_memory_profile_max_less_than_initial() {
        // This is technically invalid but should not panic
        let profile = MemoryProfile {
            initial_pages: 100,
            max_pages: Some(10),
            growth_events: vec![],
        };

        let serialized = serde_json::to_string(&profile).unwrap();
        let deserialized: MemoryProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(profile.initial_pages, deserialized.initial_pages);
    }

    #[test]
    fn test_growth_event_pages_decrease() {
        // Memory shrinking (edge case)
        let event = GrowthEvent {
            timestamp: 100,
            pages_before: 16,
            pages_after: 8,
        };

        assert!(event.pages_before > event.pages_after);
    }
}
