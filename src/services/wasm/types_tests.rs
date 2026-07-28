#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // WebAssemblyVariant tests
    #[test]
    fn test_webassembly_variant_assemblyscript() {
        let variant = WebAssemblyVariant::AssemblyScript;
        assert_eq!(variant, WebAssemblyVariant::AssemblyScript);
        let cloned = variant;
        assert_eq!(cloned, WebAssemblyVariant::AssemblyScript);
    }

    #[test]
    fn test_webassembly_variant_wat() {
        let variant = WebAssemblyVariant::Wat;
        assert_eq!(variant, WebAssemblyVariant::Wat);
    }

    #[test]
    fn test_webassembly_variant_wasm() {
        let variant = WebAssemblyVariant::Wasm;
        assert_eq!(variant, WebAssemblyVariant::Wasm);
    }

    #[test]
    fn test_webassembly_variant_serialization() {
        let variant = WebAssemblyVariant::AssemblyScript;
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: WebAssemblyVariant = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }

    // WasmMetrics tests
    #[test]
    fn test_wasm_metrics_default() {
        let metrics = WasmMetrics::default();
        assert_eq!(metrics.memory_sections, 0);
        assert_eq!(metrics.function_count, 0);
        assert!(metrics.instruction_histogram.is_empty());
    }

    #[test]
    fn test_wasm_metrics_clone() {
        let metrics = WasmMetrics {
            memory_sections: 1,
            table_sections: 2,
            import_count: 3,
            export_count: 4,
            function_count: 5,
            global_count: 6,
            linear_memory_pages: 7,
            indirect_calls: 8,
            memory_operations: MemoryOpStats::default(),
            instruction_histogram: HashMap::from([(WasmOpcode::Nop, 10)]),
            custom_sections: 9,
            element_segments: 10,
            data_segments: 11,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.memory_sections, 1);
        assert_eq!(cloned.function_count, 5);
    }

    // MemoryOpStats tests
    #[test]
    fn test_memory_op_stats_default() {
        let stats = MemoryOpStats::default();
        assert_eq!(stats.loads, 0);
        assert_eq!(stats.stores, 0);
        assert_eq!(stats.atomic_ops, 0);
    }

    #[test]
    fn test_memory_op_stats_custom() {
        let stats = MemoryOpStats {
            loads: 100,
            stores: 50,
            grows: 5,
            atomic_ops: 10,
            simd_ops: 20,
            bulk_ops: 3,
        };
        assert_eq!(stats.loads, 100);
        assert_eq!(stats.simd_ops, 20);
    }

    // WasmComplexity tests
    #[test]
    fn test_wasm_complexity_default() {
        let complexity = WasmComplexity::default();
        assert_eq!(complexity.cyclomatic, 0);
        assert_eq!(complexity.cognitive, 0);
    }

    #[test]
    fn test_wasm_complexity_custom() {
        let complexity = WasmComplexity {
            cyclomatic: 10,
            memory_pressure: 50.5,
            indirect_call_overhead: 2.5,
            estimated_gas: 1000.0,
            cognitive: 15,
            hot_path_score: 0.8,
            max_loop_depth: 3,
        };
        assert_eq!(complexity.cyclomatic, 10);
        assert!((complexity.memory_pressure - 50.5).abs() < f32::EPSILON);
    }

    // MemoryAnalysis tests
    #[test]
    fn test_memory_analysis_default() {
        let analysis = MemoryAnalysis::default();
        assert_eq!(analysis.peak_usage_bytes, 0);
        assert!(analysis.allocation_patterns.is_empty());
    }

    // Severity tests
    #[test]
    fn test_severity_display_low() {
        let severity = Severity::Low;
        assert_eq!(format!("{}", severity), "Low");
    }

    #[test]
    fn test_severity_display_medium() {
        let severity = Severity::Medium;
        assert_eq!(format!("{}", severity), "Medium");
    }

    #[test]
    fn test_severity_display_high() {
        let severity = Severity::High;
        assert_eq!(format!("{}", severity), "High");
    }

    #[test]
    fn test_severity_display_critical() {
        let severity = Severity::Critical;
        assert_eq!(format!("{}", severity), "Critical");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    // Difficulty tests
    #[test]
    fn test_difficulty_variants() {
        assert_eq!(Difficulty::Easy, Difficulty::Easy);
        assert_eq!(Difficulty::Medium, Difficulty::Medium);
        assert_eq!(Difficulty::Hard, Difficulty::Hard);
        assert_ne!(Difficulty::Easy, Difficulty::Hard);
    }

    // OptimizationType tests
    #[test]
    fn test_optimization_type_variants() {
        let types = [
            OptimizationType::ReduceAllocations,
            OptimizationType::ImproveAlignment,
            OptimizationType::UseStackMemory,
            OptimizationType::PoolAllocations,
            OptimizationType::CompactDataStructures,
            OptimizationType::EliminateLeaks,
            OptimizationType::ReduceFragmentation,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let deserialized: OptimizationType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, t);
        }
    }

    // WasmOpcode tests
    #[test]
    fn test_wasm_opcode_from_u8_control_flow() {
        assert_eq!(WasmOpcode::from(0x00), WasmOpcode::Unreachable);
        assert_eq!(WasmOpcode::from(0x01), WasmOpcode::Nop);
        assert_eq!(WasmOpcode::from(0x02), WasmOpcode::Block);
        assert_eq!(WasmOpcode::from(0x03), WasmOpcode::Loop);
        assert_eq!(WasmOpcode::from(0x04), WasmOpcode::If);
        assert_eq!(WasmOpcode::from(0x05), WasmOpcode::Else);
        assert_eq!(WasmOpcode::from(0x0B), WasmOpcode::End);
        assert_eq!(WasmOpcode::from(0x0C), WasmOpcode::Br);
        assert_eq!(WasmOpcode::from(0x0D), WasmOpcode::BrIf);
        assert_eq!(WasmOpcode::from(0x0E), WasmOpcode::BrTable);
        assert_eq!(WasmOpcode::from(0x0F), WasmOpcode::Return);
        assert_eq!(WasmOpcode::from(0x10), WasmOpcode::Call);
        assert_eq!(WasmOpcode::from(0x11), WasmOpcode::CallIndirect);
    }

    #[test]
    fn test_wasm_opcode_from_u8_memory() {
        assert_eq!(WasmOpcode::from(0x28), WasmOpcode::I32Load);
        assert_eq!(WasmOpcode::from(0x29), WasmOpcode::I64Load);
        assert_eq!(WasmOpcode::from(0x2A), WasmOpcode::F32Load);
        assert_eq!(WasmOpcode::from(0x2B), WasmOpcode::F64Load);
        assert_eq!(WasmOpcode::from(0x36), WasmOpcode::I32Store);
        assert_eq!(WasmOpcode::from(0x37), WasmOpcode::I64Store);
        assert_eq!(WasmOpcode::from(0x38), WasmOpcode::F32Store);
        assert_eq!(WasmOpcode::from(0x39), WasmOpcode::F64Store);
        assert_eq!(WasmOpcode::from(0x3F), WasmOpcode::MemorySize);
        assert_eq!(WasmOpcode::from(0x40), WasmOpcode::MemoryGrow);
    }

    #[test]
    fn test_wasm_opcode_from_u8_constants() {
        assert_eq!(WasmOpcode::from(0x41), WasmOpcode::I32Const);
        assert_eq!(WasmOpcode::from(0x42), WasmOpcode::I64Const);
        assert_eq!(WasmOpcode::from(0x43), WasmOpcode::F32Const);
        assert_eq!(WasmOpcode::from(0x44), WasmOpcode::F64Const);
    }

    #[test]
    fn test_wasm_opcode_from_u8_variables() {
        assert_eq!(WasmOpcode::from(0x20), WasmOpcode::LocalGet);
        assert_eq!(WasmOpcode::from(0x21), WasmOpcode::LocalSet);
        assert_eq!(WasmOpcode::from(0x22), WasmOpcode::LocalTee);
        assert_eq!(WasmOpcode::from(0x23), WasmOpcode::GlobalGet);
        assert_eq!(WasmOpcode::from(0x24), WasmOpcode::GlobalSet);
    }

    #[test]
    fn test_wasm_opcode_from_u8_other() {
        assert_eq!(WasmOpcode::from(0xFF), WasmOpcode::Other(0xFF));
        assert_eq!(WasmOpcode::from(0x99), WasmOpcode::Other(0x99));
    }

    #[test]
    fn test_wasm_opcode_hash() {
        let mut map: HashMap<WasmOpcode, u32> = HashMap::new();
        map.insert(WasmOpcode::Nop, 5);
        map.insert(WasmOpcode::Call, 10);
        assert_eq!(map.get(&WasmOpcode::Nop), Some(&5));
        assert_eq!(map.get(&WasmOpcode::Call), Some(&10));
    }

    // SourceLocation tests
    #[test]
    fn test_source_location() {
        let loc = SourceLocation {
            file: "test.wat".to_string(),
            line: 10,
            column: 5,
            offset: 100,
        };
        assert_eq!(loc.file, "test.wat");
        assert_eq!(loc.line, 10);
    }

    // AllocationPattern tests
    #[test]
    fn test_allocation_pattern() {
        let pattern = AllocationPattern {
            pattern_type: "linear_growth".to_string(),
            location: SourceLocation {
                file: "main.wat".to_string(),
                line: 1,
                column: 1,
                offset: 0,
            },
            severity: Severity::Medium,
            description: "Linear memory growth detected".to_string(),
        };
        assert_eq!(pattern.pattern_type, "linear_growth");
        assert_eq!(pattern.severity, Severity::Medium);
    }

    // MemoryOptimizationHint tests
    #[test]
    fn test_memory_optimization_hint() {
        let hint = MemoryOptimizationHint {
            hint_type: OptimizationType::ReduceAllocations,
            expected_improvement: 25.0,
            difficulty: Difficulty::Easy,
            suggestion: "Pool allocations for frequently created objects".to_string(),
        };
        assert_eq!(hint.hint_type, OptimizationType::ReduceAllocations);
        assert!((hint.expected_improvement - 25.0).abs() < f32::EPSILON);
    }

    // AlignmentIssue tests
    #[test]
    fn test_alignment_issue() {
        let issue = AlignmentIssue {
            offset: 1023,
            required_alignment: 8,
            actual_alignment: 1,
            performance_impact: 15.5,
        };
        assert_eq!(issue.offset, 1023);
        assert_eq!(issue.required_alignment, 8);
    }
}
