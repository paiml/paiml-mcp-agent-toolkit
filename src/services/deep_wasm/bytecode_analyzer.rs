#![cfg_attr(coverage_nightly, coverage(off))]
//! WASM Bytecode Analyzer
//!
//! Provides detailed function-level and instruction-level analysis of WASM bytecode.
//! Implements Issue #65: Enhanced WASM Deep Inspection for compiler development.
//!
//! Key Features:
//! - Function signatures and metadata extraction
//! - Complexity metrics per function
//! - Instruction counts and breakdowns
//! - Stack depth analysis
//! - Control flow pattern detection

use crate::services::deep_wasm::{DeepWasmError, DeepWasmResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasmparser::{FuncType, FunctionBody, Operator, Parser, Payload, TypeRef, ValType};

/// Detailed function analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionAnalysis {
    pub function_index: u32,
    pub name: Option<String>,
    pub signature: FunctionSignature,
    pub complexity: ComplexityMetrics,
    pub instruction_stats: InstructionStats,
    pub stack_depth: StackDepthAnalysis,
    pub control_flow_patterns: Vec<ControlFlowPattern>,
    pub is_exported: bool,
    pub export_name: Option<String>,
}

/// Function signature details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub params: Vec<String>,
    pub results: Vec<String>,
    pub type_index: u32,
}

/// Complexity metrics for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub instruction_count: u32,
    pub basic_block_count: u32,
    pub branch_count: u32,
    pub loop_count: u32,
    pub call_count: u32,
    pub nesting_depth: u32,
}

/// Instruction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionStats {
    pub total: u32,
    pub by_type: HashMap<String, u32>,
    pub by_category: InstructionCategoryBreakdown,
}

/// Instruction breakdown by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionCategoryBreakdown {
    pub control_flow: u32,
    pub memory_ops: u32,
    pub numeric_ops: u32,
    pub variable_ops: u32,
    pub table_ops: u32,
    pub reference_ops: u32,
    pub parametric_ops: u32,
}

/// Stack depth analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackDepthAnalysis {
    pub max_depth: u32,
    pub avg_depth: f64,
    pub entry_depth: u32,
    pub exit_depth: u32,
}

/// Control flow pattern detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlowPattern {
    pub pattern_type: String,
    pub description: String,
    pub offset: u32,
    pub suspicious: bool,
    pub suspicion_reason: Option<String>,
}

/// Module-level bytecode analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleBytecodeAnalysis {
    pub functions: Vec<FunctionAnalysis>,
    pub module_stats: ModuleStats,
    pub imports: Vec<ImportAnalysis>,
    pub exports: Vec<ExportAnalysis>,
    pub validation_errors: Vec<ValidationError>,
}

/// Module-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStats {
    pub total_functions: u32,
    pub total_instructions: u32,
    pub avg_complexity: f64,
    pub max_complexity: u32,
    pub import_count: u32,
    pub export_count: u32,
}

/// Import analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAnalysis {
    pub module: String,
    pub field: String,
    pub kind: String,
    pub signature: Option<FunctionSignature>,
}

/// Export analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAnalysis {
    pub name: String,
    pub kind: String,
    pub index: u32,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub offset: Option<usize>,
}

/// Collected WASM sections from first parsing pass
struct WasmSections<'a> {
    type_section: Vec<FuncType>,
    function_section: Vec<u32>,
    code_section: Vec<FunctionBody<'a>>,
    export_section: Vec<(String, u32, String)>,
    import_section: Vec<ImportAnalysis>,
    name_map: HashMap<u32, String>,
    validation_errors: Vec<ValidationError>,
}

/// Bytecode analyzer
pub struct BytecodeAnalyzer {
    /// Whether to perform deep analysis (slower but more detailed)
    deep_analysis: bool,
}

impl BytecodeAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            deep_analysis: true,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_deep_analysis(deep_analysis: bool) -> Self {
        Self { deep_analysis }
    }
}

impl Default for BytecodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

include!("bytecode_analyzer_instruction_ops.rs");
include!("bytecode_analyzer_parsing.rs");
include!("bytecode_analyzer_body_analysis.rs");
include!("bytecode_analyzer_tests.rs");
