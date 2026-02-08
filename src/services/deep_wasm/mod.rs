#![cfg_attr(coverage_nightly, coverage(off))]
//! Deep WASM Inspection Module
//!
//! This module provides comprehensive deep inspection of the Rust → WebAssembly → JavaScript → HTML pipeline.
//! It implements multi-layer bidirectional tracing that reconstructs the complete compilation and execution pipeline.
//!
//! ## Key Features
//! - Source-to-WASM correlation with DWARF debug info
//! - Type flow analysis across language boundaries
//! - Performance hotspot attribution to source code
//! - Quality gates specific to WASM artifacts
//! - Support for Rust and Ruchy languages
//!
//! ## Architecture
//! ```text
//! ┌─────────────────────────────────────────┐
//! │     DeepWasmService                      │
//! ├─────────────────────────────────────────┤
//! │  - WasmInspector                         │
//! │  - DwarfParser                           │
//! │  - SourceMapHandler                      │
//! │  - CorrelationEngine                     │
//! └─────────────────────────────────────────┘
//! ```

pub mod bytecode_analyzer;
pub mod correlation_engine;
pub mod disassembler;
pub mod dwarf_parser;
pub mod error;
pub mod quality_gates;
pub mod report_generator;
pub mod service;
pub mod source_map_handler;
pub mod types;
pub mod wasm_inspector;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dwarf_parser_phase2_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod correlation_engine_phase2_tests;

pub use bytecode_analyzer::{
    BytecodeAnalyzer, ComplexityMetrics, ControlFlowPattern, ExportAnalysis, FunctionAnalysis,
    FunctionSignature, ImportAnalysis, InstructionCategoryBreakdown, InstructionStats,
    ModuleBytecodeAnalysis, ModuleStats, StackDepthAnalysis, ValidationError,
};
pub use correlation_engine::CorrelationEngine;
pub use disassembler::{
    BasicBlock, DisassembledFunction, DisassembledInstruction, Disassembler, InstructionPattern,
    StackEffect,
};
pub use dwarf_parser::DwarfParser;
pub use error::{DeepWasmError, DeepWasmResult};
pub use quality_gates::WasmQualityGates;
pub use report_generator::ReportGenerator;
pub use service::DeepWasmService;
pub use source_map_handler::SourceMapHandler;
pub use wasm_inspector::WasmInspector;

// Re-export commonly used types
pub use types::{
    AnalysisFocus, DeepWasmAnalysisRequest, DeepWasmReport, DwarfDebugEntry, IssueSeverity,
    Location, PipelineOverview, QualityGateResults, QualityViolation, SourceLanguage,
    SourceMapEntry, SourceMetrics, SourceToWasmMapping, WasmModuleAnalysis,
};
