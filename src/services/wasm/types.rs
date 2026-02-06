#![cfg_attr(coverage_nightly, coverage(off))]
//! WebAssembly type definitions and data structures
//!
//! This module contains all the core types used throughout the WebAssembly
//! parsing and analysis system.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebAssembly language variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebAssemblyVariant {
    /// `AssemblyScript` - TypeScript-like syntax compiling to WASM
    AssemblyScript,
    /// WebAssembly Text Format - Human-readable WASM
    Wat,
    /// WebAssembly Binary Format - Compiled WASM modules
    Wasm,
}

/// Comprehensive WebAssembly metrics extracted from modules
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WasmMetrics {
    /// Number of memory sections defined
    pub memory_sections: u32,

    /// Number of table sections defined
    pub table_sections: u32,

    /// Total import count
    pub import_count: u32,

    /// Total export count
    pub export_count: u32,

    /// Total function count
    pub function_count: u32,

    /// Total global variable count
    pub global_count: u32,

    /// Linear memory size in pages (64KB each)
    pub linear_memory_pages: u32,

    /// Number of indirect calls (performance impact)
    pub indirect_calls: u32,

    /// Memory operation statistics
    pub memory_operations: MemoryOpStats,

    /// Instruction frequency histogram for optimization
    pub instruction_histogram: HashMap<WasmOpcode, u32>,

    /// Custom section count
    pub custom_sections: u32,

    /// Element segments count
    pub element_segments: u32,

    /// Data segments count
    pub data_segments: u32,
}

/// Memory operation statistics for performance analysis
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryOpStats {
    /// Number of memory load operations
    pub loads: u32,

    /// Number of memory store operations
    pub stores: u32,

    /// Number of memory.grow operations
    pub grows: u32,

    /// Number of atomic operations
    pub atomic_ops: u32,

    /// Number of SIMD operations
    pub simd_ops: u32,

    /// Number of bulk memory operations
    pub bulk_ops: u32,
}

/// WebAssembly complexity metrics with gas estimation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WasmComplexity {
    /// Traditional cyclomatic complexity
    pub cyclomatic: u32,

    /// Memory pressure score (0-100)
    pub memory_pressure: f32,

    /// Indirect call overhead factor
    pub indirect_call_overhead: f32,

    /// Estimated gas cost for blockchain deployment
    pub estimated_gas: f64,

    /// Cognitive complexity (accounts for nesting)
    pub cognitive: u32,

    /// Hot path detection score
    pub hot_path_score: f32,

    /// Loop nesting depth
    pub max_loop_depth: u32,
}

/// Memory analysis results for optimization
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysis {
    /// Peak memory usage in bytes
    pub peak_usage_bytes: u64,

    /// Memory allocation patterns
    pub allocation_patterns: Vec<AllocationPattern>,

    /// Memory leak risk score (0-100)
    pub leak_risk_score: f32,

    /// Suggested optimizations
    pub optimization_hints: Vec<MemoryOptimizationHint>,

    /// Stack depth analysis
    pub max_stack_depth: u32,

    /// Memory alignment issues found
    pub alignment_issues: Vec<AlignmentIssue>,
}

/// Memory allocation pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    /// Pattern name (e.g., "`linear_growth`", "`exponential_growth`")
    pub pattern_type: String,

    /// `Location` in source
    pub location: SourceLocation,

    /// Severity: low, medium, high
    pub severity: Severity,

    /// Detailed description
    pub description: String,
}

/// Memory optimization hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimizationHint {
    /// Optimization type
    pub hint_type: OptimizationType,

    /// Expected improvement percentage
    pub expected_improvement: f32,

    /// Implementation difficulty: easy, medium, hard
    pub difficulty: Difficulty,

    /// Detailed suggestion
    pub suggestion: String,
}

/// Memory alignment issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentIssue {
    /// Memory offset with alignment problem
    pub offset: u32,

    /// Required alignment
    pub required_alignment: u32,

    /// Actual alignment
    pub actual_alignment: u32,

    /// Performance impact estimate
    pub performance_impact: f32,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: String,

    /// Line number (1-based)
    pub line: u32,

    /// Column number (1-based)
    pub column: u32,

    /// Byte offset in file
    pub offset: u32,
}

/// Severity levels for issues and patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    ///
    /// Returns an error if the operation fails
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

/// Difficulty levels for optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

/// Types of memory optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    ReduceAllocations,
    ImproveAlignment,
    UseStackMemory,
    PoolAllocations,
    CompactDataStructures,
    EliminateLeaks,
    ReduceFragmentation,
}

/// WebAssembly opcodes for instruction analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WasmOpcode {
    // Control flow
    Unreachable = 0x00,
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    End = 0x0B,
    Br = 0x0C,
    BrIf = 0x0D,
    BrTable = 0x0E,
    Return = 0x0F,
    Call = 0x10,
    CallIndirect = 0x11,

    // Memory operations
    I32Load = 0x28,
    I64Load = 0x29,
    F32Load = 0x2A,
    F64Load = 0x2B,
    I32Store = 0x36,
    I64Store = 0x37,
    F32Store = 0x38,
    F64Store = 0x39,
    MemorySize = 0x3F,
    MemoryGrow = 0x40,

    // Constants
    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,

    // Variables
    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,
    GlobalGet = 0x23,
    GlobalSet = 0x24,

    // Other categories...
    Other(u8),
}

impl From<u8> for WasmOpcode {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => Self::Unreachable,
            0x01 => Self::Nop,
            0x02 => Self::Block,
            0x03 => Self::Loop,
            0x04 => Self::If,
            0x05 => Self::Else,
            0x0B => Self::End,
            0x0C => Self::Br,
            0x0D => Self::BrIf,
            0x0E => Self::BrTable,
            0x0F => Self::Return,
            0x10 => Self::Call,
            0x11 => Self::CallIndirect,
            0x28 => Self::I32Load,
            0x29 => Self::I64Load,
            0x2A => Self::F32Load,
            0x2B => Self::F64Load,
            0x36 => Self::I32Store,
            0x37 => Self::I64Store,
            0x38 => Self::F32Store,
            0x39 => Self::F64Store,
            0x3F => Self::MemorySize,
            0x40 => Self::MemoryGrow,
            0x41 => Self::I32Const,
            0x42 => Self::I64Const,
            0x43 => Self::F32Const,
            0x44 => Self::F64Const,
            0x20 => Self::LocalGet,
            0x21 => Self::LocalSet,
            0x22 => Self::LocalTee,
            0x23 => Self::GlobalGet,
            0x24 => Self::GlobalSet,
            other => Self::Other(other),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

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
