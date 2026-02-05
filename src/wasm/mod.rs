//! WebAssembly analysis and quality assurance module
//!
//! Implements the WebAssembly Integration Specification v3 for
//! pragmatic zero-defect validation through incremental verification.

pub mod analyzer;
pub mod baseline;
pub mod hardware;
pub mod profiler;
pub mod security;
pub mod verifier;

pub use analyzer::{Analysis, WasmAnalyzer};
pub use baseline::{QualityAssessment, QualityBaseline, Violation};
pub use hardware::{CacheClass, CoreClass, HardwareClass};
pub use profiler::{AsyncProfiler, ShadowStack};
pub use security::{PatternDetector, VulnerabilityMatch, VulnerabilityPattern};
pub use verifier::{IncrementalVerifier, VerificationResult};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Main entry point for WASM analysis
pub async fn analyze_wasm_module(binary: &[u8]) -> Result<Analysis> {
    let analyzer = WasmAnalyzer::new()?;
    analyzer.analyze_streaming(binary)
}

/// Verify WASM module safety properties
pub fn verify_wasm_safety(binary: &[u8]) -> Result<VerificationResult> {
    let verifier = IncrementalVerifier::new()?;
    verifier.verify_module(binary)
}

/// Profile WASM module performance
pub async fn profile_wasm_module(binary: &[u8]) -> Result<ProfilingReport> {
    let profiler = AsyncProfiler::new();
    profiler.profile_module(binary).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    pub instruction_mix: InstructionMix,
    pub hot_functions: Vec<HotFunction>,
    pub memory_usage: MemoryProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionMix {
    pub total_instructions: usize,
    pub control_flow: usize,
    pub memory_ops: usize,
    pub arithmetic: usize,
    pub calls: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotFunction {
    pub name: String,
    pub samples: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub initial_pages: u32,
    pub max_pages: Option<u32>,
    pub growth_events: Vec<GrowthEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthEvent {
    pub timestamp: u64,
    pub pages_before: u32,
    pub pages_after: u32,
}
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // =====================================================
    // Helper functions - Valid WASM binary test fixtures
    // =====================================================

    /// Minimal valid WASM module (empty module with proper header)
    fn minimal_wasm_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
        ]
    }

    /// WASM module with a simple function containing arithmetic
    fn simple_function_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x05, // section id 1, size 5
            0x01, // 1 type
            0x60, 0x00, 0x01, 0x7f, // func type: () -> i32
            // Function section
            0x03, 0x02, // section id 3, size 2
            0x01, 0x00, // 1 function, type 0
            // Code section
            0x0a, 0x09, // section id 10, size 9
            0x01, // 1 function body
            0x07, // body size 7
            0x00, // 0 locals
            0x41, 0x01, // i32.const 1
            0x41, 0x02, // i32.const 2
            0x6a, // i32.add
            0x0b, // end
        ]
    }

    /// WASM module with memory section
    fn memory_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Memory section
            0x05, 0x04, // section id 5, size 4
            0x01, // 1 memory
            0x01, 0x02, 0x10, // min 2 pages, max 16 pages
        ]
    }

    /// WASM module with mixed instructions (control flow, memory, arithmetic)
    fn mixed_instructions_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x05, // section id 1, size 5
            0x01, // 1 type
            0x60, 0x00, 0x01, 0x7f, // func type: () -> i32
            // Function section
            0x03, 0x02, // section id 3, size 2
            0x01, 0x00, // 1 function, type 0
            // Memory section
            0x05, 0x03, // section id 5, size 3
            0x01, // 1 memory
            0x00, 0x01, // min 1 page, no max
            // Code section with mixed instructions
            0x0a, 0x11, // section id 10, size 17
            0x01, // 1 function body
            0x0f, // body size 15
            0x00, // 0 locals
            0x02, 0x7f, // block returning i32
            0x41, 0x00, // i32.const 0
            0x28, 0x02, 0x00, // i32.load
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x0c, 0x00, // br 0
            0x0b, // end block
            0x0b, // end function
        ]
    }

    // =====================================================
    // Module-level public function tests
    // =====================================================

    #[tokio::test]
    async fn test_analyze_wasm_module_minimal() {
        let result = analyze_wasm_module(&minimal_wasm_module()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.instruction_mix.total_instructions, 0);
        assert!(analysis.vulnerability_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_simple_function() {
        let result = analyze_wasm_module(&simple_function_wasm()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.instruction_mix.total_instructions > 0);
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_mixed_instructions() {
        let result = analyze_wasm_module(&mixed_instructions_wasm()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.instruction_mix.control_flow > 0);
        assert!(analysis.instruction_mix.memory_ops > 0);
        assert!(analysis.instruction_mix.arithmetic > 0);
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_invalid_binary() {
        let invalid = vec![0x00, 0x01, 0x02, 0x03]; // Not valid WASM
        let result = analyze_wasm_module(&invalid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_wasm_module_empty_input() {
        let result = analyze_wasm_module(&[]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wasm_safety_minimal() {
        let result = verify_wasm_safety(&minimal_wasm_module());
        assert!(result.is_ok());

        let verification = result.unwrap();
        assert!(verification.is_safe());
    }

    #[test]
    fn test_verify_wasm_safety_simple_function() {
        let result = verify_wasm_safety(&simple_function_wasm());
        assert!(result.is_ok());

        let verification = result.unwrap();
        assert!(verification.is_safe());
    }

    #[test]
    fn test_verify_wasm_safety_mixed_instructions() {
        let result = verify_wasm_safety(&mixed_instructions_wasm());
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_wasm_safety_invalid_binary() {
        let invalid = vec![0x00, 0x01, 0x02, 0x03];
        let result = verify_wasm_safety(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wasm_safety_empty_input() {
        let result = verify_wasm_safety(&[]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_profile_wasm_module_minimal() {
        let result = profile_wasm_module(&minimal_wasm_module()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.instruction_mix.total_instructions, 0);
    }

    #[tokio::test]
    async fn test_profile_wasm_module_simple_function() {
        let result = profile_wasm_module(&simple_function_wasm()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.instruction_mix.total_instructions > 0);
    }

    #[tokio::test]
    async fn test_profile_wasm_module_with_memory() {
        let result = profile_wasm_module(&memory_wasm()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.memory_usage.initial_pages, 2);
        assert_eq!(report.memory_usage.max_pages, Some(16));
    }

    #[tokio::test]
    async fn test_profile_wasm_module_mixed_instructions() {
        let result = profile_wasm_module(&mixed_instructions_wasm()).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.instruction_mix.control_flow > 0);
        assert!(report.instruction_mix.memory_ops > 0);
        assert!(report.instruction_mix.arithmetic > 0);
    }

    #[tokio::test]
    async fn test_profile_wasm_module_invalid_binary() {
        let invalid = vec![0x00, 0x01, 0x02, 0x03];
        let result = profile_wasm_module(&invalid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_profile_wasm_module_empty_input() {
        let result = profile_wasm_module(&[]).await;
        assert!(result.is_err());
    }

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
