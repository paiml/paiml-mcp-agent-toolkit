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
use wasmparser::{
    FuncType, FunctionBody, Operator, Parser, Payload, TypeRef, ValType,
};

/// Detailed function analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionAnalysis {
    /// Function index in the module
    pub function_index: u32,

    /// Function name (if available from name section)
    pub name: Option<String>,

    /// Function signature
    pub signature: FunctionSignature,

    /// Complexity metrics
    pub complexity: ComplexityMetrics,

    /// Instruction statistics
    pub instruction_stats: InstructionStats,

    /// Stack depth analysis
    pub stack_depth: StackDepthAnalysis,

    /// Control flow patterns detected
    pub control_flow_patterns: Vec<ControlFlowPattern>,

    /// Whether this function is exported
    pub is_exported: bool,

    /// Export name (if exported)
    pub export_name: Option<String>,
}

/// Function signature details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Parameter types
    pub params: Vec<String>,

    /// Return types
    pub results: Vec<String>,

    /// Type index in the type section
    pub type_index: u32,
}

/// Complexity metrics for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity (control flow edges)
    pub cyclomatic_complexity: u32,

    /// Total instruction count
    pub instruction_count: u32,

    /// Number of basic blocks
    pub basic_block_count: u32,

    /// Number of branches (if/br/br_if/br_table)
    pub branch_count: u32,

    /// Number of loops
    pub loop_count: u32,

    /// Number of calls (direct and indirect)
    pub call_count: u32,

    /// Nesting depth (deepest level of block nesting)
    pub nesting_depth: u32,
}

/// Instruction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionStats {
    /// Total instruction count
    pub total: u32,

    /// Breakdown by instruction type
    pub by_type: HashMap<String, u32>,

    /// Breakdown by category
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
    /// Maximum stack depth reached
    pub max_depth: u32,

    /// Average stack depth
    pub avg_depth: f64,

    /// Stack depth at function entry
    pub entry_depth: u32,

    /// Stack depth at function exit
    pub exit_depth: u32,
}

/// Control flow pattern detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlowPattern {
    /// Pattern type
    pub pattern_type: String,

    /// Description
    pub description: String,

    /// Location (instruction offset)
    pub offset: u32,

    /// Whether this pattern might be suspicious
    pub suspicious: bool,

    /// Reason for suspicion (if suspicious)
    pub suspicion_reason: Option<String>,
}

/// Module-level bytecode analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleBytecodeAnalysis {
    /// Per-function analyses
    pub functions: Vec<FunctionAnalysis>,

    /// Module-wide statistics
    pub module_stats: ModuleStats,

    /// Import/export analysis
    pub imports: Vec<ImportAnalysis>,
    pub exports: Vec<ExportAnalysis>,

    /// Validation errors (if any)
    pub validation_errors: Vec<ValidationError>,
}

/// Module-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStats {
    /// Total number of functions
    pub total_functions: u32,

    /// Total instructions across all functions
    pub total_instructions: u32,

    /// Average function complexity
    pub avg_complexity: f64,

    /// Maximum function complexity
    pub max_complexity: u32,

    /// Total imports
    pub import_count: u32,

    /// Total exports
    pub export_count: u32,
}

/// Import analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAnalysis {
    /// Module name
    pub module: String,

    /// Field name
    pub field: String,

    /// Import kind (function, memory, table, global)
    pub kind: String,

    /// Type signature (for functions)
    pub signature: Option<FunctionSignature>,
}

/// Export analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAnalysis {
    /// Export name
    pub name: String,

    /// Export kind (function, memory, table, global)
    pub kind: String,

    /// Index in the respective index space
    pub index: u32,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Offset in the binary (if available)
    pub offset: Option<usize>,
}

/// Bytecode analyzer
pub struct BytecodeAnalyzer {
    /// Whether to perform deep analysis (slower but more detailed)
    deep_analysis: bool,
}

impl BytecodeAnalyzer {
    /// Create a new bytecode analyzer
    pub fn new() -> Self {
        Self {
            deep_analysis: true,
        }
    }

    /// Create analyzer with custom settings
    pub fn with_deep_analysis(deep_analysis: bool) -> Self {
        Self { deep_analysis }
    }

    /// Analyze WASM bytecode
    pub fn analyze(&self, bytes: &[u8]) -> DeepWasmResult<ModuleBytecodeAnalysis> {
        let parser = Parser::new(0);

        let mut type_section: Vec<FuncType> = Vec::new();
        let mut function_section: Vec<u32> = Vec::new();
        let mut code_section: Vec<FunctionBody> = Vec::new();
        let mut export_section: Vec<(String, u32, String)> = Vec::new();
        let mut import_section: Vec<ImportAnalysis> = Vec::new();
        let mut name_map: HashMap<u32, String> = HashMap::new();
        let mut validation_errors: Vec<ValidationError> = Vec::new();

        // First pass: collect all sections
        for payload in parser.parse_all(bytes) {
            match payload {
                Ok(Payload::TypeSection(reader)) => {
                    for recgroup in reader {
                        match recgroup {
                            Ok(recgroup) => {
                                for func_type in recgroup.into_types() {
                                    type_section.push(func_type.unwrap_func().clone());
                                }
                            }
                            Err(e) => {
                                validation_errors.push(ValidationError {
                                    message: format!("Type section error: {}", e),
                                    offset: None,
                                });
                            }
                        }
                    }
                }
                Ok(Payload::FunctionSection(reader)) => {
                    for func_type_idx in reader {
                        match func_type_idx {
                            Ok(idx) => function_section.push(idx),
                            Err(e) => {
                                validation_errors.push(ValidationError {
                                    message: format!("Function section error: {}", e),
                                    offset: None,
                                });
                            }
                        }
                    }
                }
                Ok(Payload::CodeSectionEntry(body)) => {
                    code_section.push(body);
                }
                Ok(Payload::ExportSection(reader)) => {
                    for export in reader {
                        match export {
                            Ok(export) => {
                                let kind = match export.kind {
                                    wasmparser::ExternalKind::Func => "function",
                                    wasmparser::ExternalKind::Memory => "memory",
                                    wasmparser::ExternalKind::Table => "table",
                                    wasmparser::ExternalKind::Global => "global",
                                    wasmparser::ExternalKind::Tag => "tag",
                                };
                                export_section.push((
                                    export.name.to_string(),
                                    export.index,
                                    kind.to_string(),
                                ));
                            }
                            Err(e) => {
                                validation_errors.push(ValidationError {
                                    message: format!("Export section error: {}", e),
                                    offset: None,
                                });
                            }
                        }
                    }
                }
                Ok(Payload::ImportSection(reader)) => {
                    for import in reader {
                        match import {
                            Ok(import) => {
                                let kind = match &import.ty {
                                    TypeRef::Func(_) => "function",
                                    TypeRef::Memory(_) => "memory",
                                    TypeRef::Table(_) => "table",
                                    TypeRef::Global(_) => "global",
                                    TypeRef::Tag(_) => "tag",
                                };

                                let signature = if let TypeRef::Func(type_idx) = import.ty {
                                    type_section.get(type_idx as usize).map(|func_type| {
                                        FunctionSignature {
                                            params: func_type.params().iter()
                                                .map(valtype_to_string)
                                                .collect(),
                                            results: func_type.results().iter()
                                                .map(valtype_to_string)
                                                .collect(),
                                            type_index: type_idx,
                                        }
                                    })
                                } else {
                                    None
                                };

                                import_section.push(ImportAnalysis {
                                    module: import.module.to_string(),
                                    field: import.name.to_string(),
                                    kind: kind.to_string(),
                                    signature,
                                });
                            }
                            Err(e) => {
                                validation_errors.push(ValidationError {
                                    message: format!("Import section error: {}", e),
                                    offset: None,
                                });
                            }
                        }
                    }
                }
                Ok(Payload::CustomSection(reader)) if reader.name() == "name" => {
                    // Parse function names from name section
                    let name_reader = wasmparser::NameSectionReader::new(
                        wasmparser::BinaryReader::new(reader.data(), 0)
                    );
                    for section in name_reader {
                        if let Ok(wasmparser::Name::Function(func_names)) = section {
                            for naming in func_names {
                                if let Ok(naming) = naming {
                                    name_map.insert(naming.index, naming.name.to_string());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    validation_errors.push(ValidationError {
                        message: format!("Parse error: {}", e),
                        offset: None,
                    });
                }
                _ => {}
            }
        }

        // Analyze each function
        let import_count = import_section.iter()
            .filter(|imp| imp.kind == "function")
            .count() as u32;

        let mut functions = Vec::new();
        let mut total_instructions = 0;
        let mut max_complexity = 0;

        for (func_idx, (type_idx, body)) in function_section.iter()
            .zip(code_section.iter())
            .enumerate()
        {
            let func_index = import_count + func_idx as u32;
            let func_type = type_section.get(*type_idx as usize);

            if func_type.is_none() {
                validation_errors.push(ValidationError {
                    message: format!("Function {} references invalid type index {}", func_index, type_idx),
                    offset: None,
                });
                continue;
            }

            let func_type = func_type.unwrap();

            let signature = FunctionSignature {
                params: func_type.params().iter()
                    .map(valtype_to_string)
                    .collect(),
                results: func_type.results().iter()
                    .map(valtype_to_string)
                    .collect(),
                type_index: *type_idx,
            };

            let name = name_map.get(&func_index).cloned();

            // Find export name if exported
            let (is_exported, export_name) = export_section.iter()
                .find(|(_, idx, kind)| *idx == func_index && kind == "function")
                .map(|(name, _, _)| (true, Some(name.clone())))
                .unwrap_or((false, None));

            // Analyze function body
            let (complexity, instruction_stats, stack_depth, control_flow_patterns) =
                if self.deep_analysis {
                    self.analyze_function_body(body)?
                } else {
                    self.analyze_function_body_shallow(body)?
                };

            total_instructions += instruction_stats.total;
            max_complexity = max_complexity.max(complexity.cyclomatic_complexity);

            functions.push(FunctionAnalysis {
                function_index: func_index,
                name,
                signature,
                complexity,
                instruction_stats,
                stack_depth,
                control_flow_patterns,
                is_exported,
                export_name,
            });
        }

        let avg_complexity = if !functions.is_empty() {
            functions.iter()
                .map(|f| f.complexity.cyclomatic_complexity as f64)
                .sum::<f64>() / functions.len() as f64
        } else {
            0.0
        };

        let exports: Vec<ExportAnalysis> = export_section.into_iter()
            .map(|(name, index, kind)| ExportAnalysis { name, kind, index })
            .collect();

        Ok(ModuleBytecodeAnalysis {
            functions,
            module_stats: ModuleStats {
                total_functions: function_section.len() as u32,
                total_instructions,
                avg_complexity,
                max_complexity,
                import_count: import_section.len() as u32,
                export_count: exports.len() as u32,
            },
            imports: import_section,
            exports,
            validation_errors,
        })
    }

    /// Analyze function body in detail
    fn analyze_function_body(
        &self,
        body: &FunctionBody,
    ) -> DeepWasmResult<(ComplexityMetrics, InstructionStats, StackDepthAnalysis, Vec<ControlFlowPattern>)> {
        let mut instruction_count = 0;
        let mut branch_count = 0;
        let mut loop_count = 0;
        let mut call_count = 0;
        let mut nesting_depth: u32 = 0;
        let mut max_nesting = 0;
        let mut basic_block_count = 1; // Start with entry block

        let mut instruction_types: HashMap<String, u32> = HashMap::new();
        let mut category_breakdown = InstructionCategoryBreakdown {
            control_flow: 0,
            memory_ops: 0,
            numeric_ops: 0,
            variable_ops: 0,
            table_ops: 0,
            reference_ops: 0,
            parametric_ops: 0,
        };

        let mut stack_depths: Vec<u32> = Vec::new();
        let mut current_stack_depth = 0u32;

        let mut control_flow_patterns: Vec<ControlFlowPattern> = Vec::new();
        let mut offset = 0u32;

        let reader = body.get_operators_reader()
            .map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;

        for op in reader {
            let op = op.map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;
            instruction_count += 1;

            let op_name = format!("{:?}", op).split('(').next().unwrap_or("Unknown").to_string();
            *instruction_types.entry(op_name.clone()).or_insert(0) += 1;

            // Categorize instruction
            categorize_instruction(&op, &mut category_breakdown);

            // Track control flow
            match &op {
                Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
                    nesting_depth += 1;
                    max_nesting = max_nesting.max(nesting_depth);
                    basic_block_count += 1;

                    if matches!(op, Operator::Loop { .. }) {
                        loop_count += 1;
                        control_flow_patterns.push(ControlFlowPattern {
                            pattern_type: "loop".to_string(),
                            description: "Loop construct".to_string(),
                            offset,
                            suspicious: false,
                            suspicion_reason: None,
                        });
                    }
                }
                Operator::End => {
                    nesting_depth = nesting_depth.saturating_sub(1);
                }
                Operator::Br { .. } | Operator::BrIf { .. } | Operator::BrTable { .. } => {
                    branch_count += 1;
                    basic_block_count += 1;
                }
                Operator::Call { .. } | Operator::CallIndirect { .. } => {
                    call_count += 1;
                }
                Operator::Unreachable => {
                    control_flow_patterns.push(ControlFlowPattern {
                        pattern_type: "unreachable".to_string(),
                        description: "Unreachable instruction (trap)".to_string(),
                        offset,
                        suspicious: true,
                        suspicion_reason: Some("Unreachable code may indicate dead code or error paths".to_string()),
                    });
                }
                _ => {}
            }

            // Track stack depth (simplified)
            update_stack_depth(&op, &mut current_stack_depth);
            stack_depths.push(current_stack_depth);

            offset += 1;
        }

        // Calculate cyclomatic complexity: M = E - N + 2P
        // where E = edges, N = nodes (basic blocks), P = connected components (1 for single function)
        // Simplified: complexity = branch_count + loop_count + 1
        let cyclomatic_complexity = branch_count + loop_count + 1;

        let avg_stack_depth = if !stack_depths.is_empty() {
            stack_depths.iter().sum::<u32>() as f64 / stack_depths.len() as f64
        } else {
            0.0
        };

        let max_stack_depth = stack_depths.iter().copied().max().unwrap_or(0);

        Ok((
            ComplexityMetrics {
                cyclomatic_complexity,
                instruction_count,
                basic_block_count,
                branch_count,
                loop_count,
                call_count,
                nesting_depth: max_nesting,
            },
            InstructionStats {
                total: instruction_count,
                by_type: instruction_types,
                by_category: category_breakdown,
            },
            StackDepthAnalysis {
                max_depth: max_stack_depth,
                avg_depth: avg_stack_depth,
                entry_depth: 0,
                exit_depth: current_stack_depth,
            },
            control_flow_patterns,
        ))
    }

    /// Shallow analysis (faster, less detailed)
    fn analyze_function_body_shallow(
        &self,
        body: &FunctionBody,
    ) -> DeepWasmResult<(ComplexityMetrics, InstructionStats, StackDepthAnalysis, Vec<ControlFlowPattern>)> {
        let mut instruction_count = 0;
        let mut branch_count = 0;
        let mut loop_count = 0;
        let mut call_count = 0;

        let reader = body.get_operators_reader()
            .map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;

        for op in reader {
            let op = op.map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;
            instruction_count += 1;

            match op {
                Operator::Br { .. } | Operator::BrIf { .. } | Operator::BrTable { .. } => {
                    branch_count += 1;
                }
                Operator::Loop { .. } => {
                    loop_count += 1;
                }
                Operator::Call { .. } | Operator::CallIndirect { .. } => {
                    call_count += 1;
                }
                _ => {}
            }
        }

        Ok((
            ComplexityMetrics {
                cyclomatic_complexity: branch_count + loop_count + 1,
                instruction_count,
                basic_block_count: 1,
                branch_count,
                loop_count,
                call_count,
                nesting_depth: 0,
            },
            InstructionStats {
                total: instruction_count,
                by_type: HashMap::new(),
                by_category: InstructionCategoryBreakdown {
                    control_flow: 0,
                    memory_ops: 0,
                    numeric_ops: 0,
                    variable_ops: 0,
                    table_ops: 0,
                    reference_ops: 0,
                    parametric_ops: 0,
                },
            },
            StackDepthAnalysis {
                max_depth: 0,
                avg_depth: 0.0,
                entry_depth: 0,
                exit_depth: 0,
            },
            Vec::new(),
        ))
    }
}

impl Default for BytecodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert ValType to string representation
fn valtype_to_string(ty: &ValType) -> String {
    match ty {
        ValType::I32 => "i32".to_string(),
        ValType::I64 => "i64".to_string(),
        ValType::F32 => "f32".to_string(),
        ValType::F64 => "f64".to_string(),
        ValType::V128 => "v128".to_string(),
        ValType::Ref(ref_type) => format!("ref({:?})", ref_type),
    }
}

/// Categorize instruction by type
fn categorize_instruction(op: &Operator, breakdown: &mut InstructionCategoryBreakdown) {
    match op {
        // Control flow
        Operator::Unreachable | Operator::Nop | Operator::Block { .. } | Operator::Loop { .. }
        | Operator::If { .. } | Operator::Else | Operator::End | Operator::Br { .. }
        | Operator::BrIf { .. } | Operator::BrTable { .. } | Operator::Return
        | Operator::Call { .. } | Operator::CallIndirect { .. } => {
            breakdown.control_flow += 1;
        }

        // Memory operations
        Operator::I32Load { .. } | Operator::I64Load { .. } | Operator::F32Load { .. }
        | Operator::F64Load { .. } | Operator::I32Store { .. } | Operator::I64Store { .. }
        | Operator::F32Store { .. } | Operator::F64Store { .. } | Operator::MemorySize { .. }
        | Operator::MemoryGrow { .. } => {
            breakdown.memory_ops += 1;
        }

        // Variable operations
        Operator::LocalGet { .. } | Operator::LocalSet { .. } | Operator::LocalTee { .. }
        | Operator::GlobalGet { .. } | Operator::GlobalSet { .. } => {
            breakdown.variable_ops += 1;
        }

        // Table operations
        Operator::TableGet { .. } | Operator::TableSet { .. } | Operator::TableGrow { .. }
        | Operator::TableSize { .. } | Operator::TableFill { .. } => {
            breakdown.table_ops += 1;
        }

        // Reference operations
        Operator::RefNull { .. } | Operator::RefIsNull | Operator::RefFunc { .. } => {
            breakdown.reference_ops += 1;
        }

        // Parametric operations
        Operator::Drop | Operator::Select => {
            breakdown.parametric_ops += 1;
        }

        // Numeric operations (default)
        _ => {
            breakdown.numeric_ops += 1;
        }
    }
}

/// Update stack depth based on instruction
fn update_stack_depth(op: &Operator, depth: &mut u32) {
    match op {
        // Instructions that push values
        Operator::I32Const { .. } | Operator::I64Const { .. }
        | Operator::F32Const { .. } | Operator::F64Const { .. }
        | Operator::LocalGet { .. } | Operator::GlobalGet { .. } => {
            *depth = depth.saturating_add(1);
        }

        // Instructions that pop and push
        Operator::I32Eqz | Operator::I64Eqz => {
            // Pop 1, push 1 (net: 0)
        }

        // Instructions that pop values
        Operator::Drop => {
            *depth = depth.saturating_sub(1);
        }
        Operator::I32Store { .. } | Operator::I64Store { .. }
        | Operator::F32Store { .. } | Operator::F64Store { .. } => {
            *depth = depth.saturating_sub(2);
        }

        // Binary operations (pop 2, push 1)
        Operator::I32Add | Operator::I32Sub | Operator::I32Mul | Operator::I32DivS
        | Operator::I32DivU | Operator::I32RemS | Operator::I32RemU | Operator::I32And
        | Operator::I32Or | Operator::I32Xor | Operator::I32Shl | Operator::I32ShrS
        | Operator::I32ShrU | Operator::I32Rotl | Operator::I32Rotr | Operator::I32Eq
        | Operator::I32Ne | Operator::I32LtS | Operator::I32LtU | Operator::I32GtS
        | Operator::I32GtU | Operator::I32LeS | Operator::I32LeU | Operator::I32GeS
        | Operator::I32GeU => {
            *depth = depth.saturating_sub(1);
        }

        _ => {
            // For other instructions, we'd need more detailed analysis
            // This is a simplified version
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = BytecodeAnalyzer::new();
        assert!(analyzer.deep_analysis);
    }

    #[test]
    fn test_analyzer_shallow() {
        let analyzer = BytecodeAnalyzer::with_deep_analysis(false);
        assert!(!analyzer.deep_analysis);
    }

    #[test]
    fn test_valtype_conversion() {
        assert_eq!(valtype_to_string(&ValType::I32), "i32");
        assert_eq!(valtype_to_string(&ValType::I64), "i64");
        assert_eq!(valtype_to_string(&ValType::F32), "f32");
        assert_eq!(valtype_to_string(&ValType::F64), "f64");
    }

    #[test]
    fn test_analyze_minimal_wasm() {
        // Minimal valid WASM module with one function
        let minimal_wasm = vec![
            0x00, 0x61, 0x73, 0x6D, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version
            // Type section: one function type () -> ()
            0x01, 0x05, 0x01, 0x60, 0x00, 0x00,
            // Function section: one function
            0x03, 0x02, 0x01, 0x00,
            // Code section: one function with just 'end'
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let analyzer = BytecodeAnalyzer::new();
        let result = analyzer.analyze(&minimal_wasm);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.functions.len(), 1);
        assert_eq!(analysis.module_stats.total_functions, 1);
    }
}
