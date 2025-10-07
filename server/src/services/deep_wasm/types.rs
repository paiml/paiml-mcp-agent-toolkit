//! Data types for deep WASM analysis

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Source location in the original code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

/// Mapping from source code to WASM binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceToWasmMapping {
    /// Source file path
    pub source_file: PathBuf,

    /// Source line:column
    pub source_location: Location,

    /// WASM function index
    pub wasm_function_idx: u32,

    /// WASM instruction offset within function
    pub wasm_instruction_offset: u32,

    /// DWARF debug entry (if available)
    pub dwarf_die: Option<DwarfDebugEntry>,

    /// Source map entry (if available)
    pub source_map_entry: Option<SourceMapEntry>,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// DWARF debug information entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DwarfDebugEntry {
    pub die_offset: u64,
    pub tag: String,
    pub name: Option<String>,
}

/// Source map entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub generated_line: u32,
    pub generated_column: u32,
    pub original_line: u32,
    pub original_column: u32,
    pub source: String,
    pub name: Option<String>,
}

/// WASM type representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

/// JavaScript type at boundary
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JsType {
    Number,
    BigInt,
    String,
    Boolean,
    Object,
    Undefined,
    Null,
    Function,
}

/// Conversion cost between types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionCost {
    pub complexity: String,
    pub overhead_ns: u64,
    pub lossy: bool,
}

/// Type issue detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub location: Location,
}

/// Issue severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Type flow analysis across layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeFlowAnalysis {
    /// Source type (Rust/Ruchy)
    pub source_type: String,

    /// WASM representation
    pub wasm_types: Vec<WasmType>,

    /// JS type at boundary
    pub js_type: JsType,

    /// Conversion overhead
    pub conversion_cost: ConversionCost,

    /// Potential issues
    pub issues: Vec<TypeIssue>,
}

/// Performance hotspot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHotspot {
    /// Source location
    pub source_location: Location,

    /// WASM function name
    pub wasm_function: String,

    /// Execution time percentage
    pub time_percentage: f64,

    /// Call count
    pub call_count: u64,

    /// Optimization suggestions
    pub suggestions: Vec<OptimizationSuggestion>,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub description: String,
    pub expected_improvement: String,
}

/// Deep WASM analysis request
#[derive(Debug, Clone)]
pub struct DeepWasmAnalysisRequest {
    pub source_path: PathBuf,
    pub wasm_path: Option<PathBuf>,
    pub dwarf_path: Option<PathBuf>,
    pub source_map_path: Option<PathBuf>,
    pub language: SourceLanguage,
    pub analysis_focus: AnalysisFocus,
}

/// Source language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceLanguage {
    Rust,
    Ruchy,
}

/// Analysis focus area
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisFocus {
    Full,
    Source,
    Compilation,
    Runtime,
    Interop,
}

/// Deep WASM analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepWasmReport {
    pub project_name: String,
    pub timestamp: String,
    pub pmat_version: String,
    pub pipeline_overview: PipelineOverview,
    pub source_metrics: SourceMetrics,
    pub wasm_module_analysis: WasmModuleAnalysis,
    pub correlations: Vec<SourceToWasmMapping>,
    pub type_flows: Vec<TypeFlowAnalysis>,
    pub hotspots: Vec<PerformanceHotspot>,
    pub quality_gate_results: QualityGateResults,

    /// Enhanced bytecode analysis (Issue #65)
    pub bytecode_analysis: Option<crate::services::deep_wasm::bytecode_analyzer::ModuleBytecodeAnalysis>,

    /// Disassembled functions (Issue #65)
    pub disassembled_functions: Option<Vec<crate::services::deep_wasm::disassembler::DisassembledFunction>>,

    /// Suspicious patterns detected (Issue #65)
    pub suspicious_patterns: Option<Vec<crate::services::deep_wasm::disassembler::InstructionPattern>>,
}

/// Pipeline overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOverview {
    pub source_language: SourceLanguage,
    pub source_version: String,
    pub target: String,
    pub optimization_level: String,
    pub debug_symbols: Option<String>,
}

/// Source code metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetrics {
    pub lines_of_code: usize,
    pub function_count: usize,
    pub max_complexity: u32,
    pub wasm_boundary_functions: usize,
}

/// WASM module analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleAnalysis {
    pub module_size_bytes: u64,
    pub function_count: u32,
    pub exported_functions: u32,
    pub max_complexity: u32,
    pub has_dwarf: bool,
    pub has_source_map: bool,
}

/// Quality gate evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResults {
    pub passed: bool,
    pub violations: Vec<QualityViolation>,
}

/// Quality gate violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub rule: String,
    pub severity: IssueSeverity,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_creation() {
        let loc = Location { line: 10, column: 5 };
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
    }

    #[test]
    fn test_wasm_type_equality() {
        assert_eq!(WasmType::I32, WasmType::I32);
        assert_ne!(WasmType::I32, WasmType::I64);
    }

    #[test]
    fn test_source_language_equality() {
        assert_eq!(SourceLanguage::Rust, SourceLanguage::Rust);
        assert_ne!(SourceLanguage::Rust, SourceLanguage::Ruchy);
    }

    #[test]
    fn test_analysis_focus_values() {
        let focuses = [AnalysisFocus::Full,
            AnalysisFocus::Source,
            AnalysisFocus::Compilation,
            AnalysisFocus::Runtime,
            AnalysisFocus::Interop];
        assert_eq!(focuses.len(), 5);
    }
}
