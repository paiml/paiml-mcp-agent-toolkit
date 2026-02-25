#![cfg_attr(coverage_nightly, coverage(off))]
//! WASM Disassembler
//!
//! Provides detailed disassembly of WASM functions with instruction-level details.
//! Implements Issue #65: Enhanced WASM Deep Inspection for compiler development.

use crate::services::deep_wasm::{DeepWasmError, DeepWasmResult};
use serde::{Deserialize, Serialize};
use wasmparser::{FunctionBody, Operator};

/// Disassembled instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisassembledInstruction {
    /// Instruction offset within the function
    pub offset: u32,

    /// Instruction mnemonic (e.g., "i32.add")
    pub mnemonic: String,

    /// Operands (if any)
    pub operands: Vec<String>,

    /// Stack effect (pop count, push count)
    pub stack_effect: StackEffect,

    /// Instruction category
    pub category: String,

    /// Gas/complexity cost estimate
    pub cost_estimate: u32,
}

/// Stack effect of an instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackEffect {
    /// Number of values popped from stack
    pub pops: u32,

    /// Number of values pushed to stack
    pub pushes: u32,
}

/// Disassembled function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisassembledFunction {
    /// Function index
    pub function_index: u32,

    /// Function name (if available)
    pub name: Option<String>,

    /// Disassembled instructions
    pub instructions: Vec<DisassembledInstruction>,

    /// Basic blocks
    pub basic_blocks: Vec<BasicBlock>,
}

/// Basic block in control flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// Block ID
    pub id: u32,

    /// Start offset
    pub start_offset: u32,

    /// End offset
    pub end_offset: u32,

    /// Block type (block, loop, if)
    pub block_type: String,

    /// Successor block IDs
    pub successors: Vec<u32>,
}

/// Pattern detected in disassembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionPattern {
    /// Pattern name
    pub name: String,

    /// Description
    pub description: String,

    /// Start offset
    pub start_offset: u32,

    /// End offset
    pub end_offset: u32,

    /// Instructions involved
    pub instruction_count: u32,

    /// Whether this is a suspicious pattern
    pub suspicious: bool,

    /// Suspicion reason
    pub suspicion_reason: Option<String>,
}

/// WASM disassembler
pub struct Disassembler {
    /// Whether to detect patterns
    detect_patterns: bool,
}

// Core impl: new, with_pattern_detection, disassemble_function, detect_patterns,
// build_basic_blocks, Default
include!("disassembler_core.rs");

// Free functions: format_operator, calculate_stack_effect, categorize_operator, estimate_cost
include!("disassembler_formatting.rs");

// Unit tests
include!("disassembler_tests.rs");
