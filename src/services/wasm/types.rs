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
    pub memory_sections: u32,
    pub table_sections: u32,
    pub import_count: u32,
    pub export_count: u32,
    pub function_count: u32,
    pub global_count: u32,
    /// Linear memory size in pages (64KB each)
    pub linear_memory_pages: u32,
    /// Number of indirect calls (performance impact)
    pub indirect_calls: u32,
    pub memory_operations: MemoryOpStats,
    pub instruction_histogram: HashMap<WasmOpcode, u32>,
    pub custom_sections: u32,
    pub element_segments: u32,
    pub data_segments: u32,
}

/// Memory operation statistics for performance analysis
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryOpStats {
    pub loads: u32,
    pub stores: u32,
    pub grows: u32,
    pub atomic_ops: u32,
    pub simd_ops: u32,
    pub bulk_ops: u32,
}

/// WebAssembly complexity metrics with gas estimation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WasmComplexity {
    pub cyclomatic: u32,
    /// Memory pressure score (0-100)
    pub memory_pressure: f32,
    pub indirect_call_overhead: f32,
    /// Estimated gas cost for blockchain deployment
    pub estimated_gas: f64,
    pub cognitive: u32,
    pub hot_path_score: f32,
    pub max_loop_depth: u32,
}

/// Memory analysis results for optimization
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysis {
    pub peak_usage_bytes: u64,
    pub allocation_patterns: Vec<AllocationPattern>,
    /// Memory leak risk score (0-100)
    pub leak_risk_score: f32,
    pub optimization_hints: Vec<MemoryOptimizationHint>,
    pub max_stack_depth: u32,
    pub alignment_issues: Vec<AlignmentIssue>,
}

/// Memory allocation pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    /// Pattern name (e.g., "`linear_growth`", "`exponential_growth`")
    pub pattern_type: String,
    /// `Location` in source
    pub location: SourceLocation,
    pub severity: Severity,
    pub description: String,
}

/// Memory optimization hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimizationHint {
    pub hint_type: OptimizationType,
    pub expected_improvement: f32,
    pub difficulty: Difficulty,
    pub suggestion: String,
}

/// Memory alignment issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentIssue {
    pub offset: u32,
    pub required_alignment: u32,
    pub actual_alignment: u32,
    pub performance_impact: f32,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
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

// Trait implementations (Display, From)
include!("types_impls.rs");

// Tests
include!("types_tests.rs");
