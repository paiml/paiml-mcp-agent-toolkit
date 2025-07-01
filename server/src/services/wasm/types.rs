//! WebAssembly type definitions and data structures
//!
//! This module contains all the core types used throughout the WebAssembly
//! parsing and analysis system.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebAssembly language variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebAssemblyVariant {
    /// AssemblyScript - TypeScript-like syntax compiling to WASM
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
    /// Pattern name (e.g., "linear_growth", "exponential_growth")
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
