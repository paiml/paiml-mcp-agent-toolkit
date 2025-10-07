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

impl Disassembler {
    /// Create a new disassembler
    pub fn new() -> Self {
        Self {
            detect_patterns: true,
        }
    }

    /// Create disassembler with custom settings
    pub fn with_pattern_detection(detect_patterns: bool) -> Self {
        Self { detect_patterns }
    }

    /// Disassemble a function
    pub fn disassemble_function(
        &self,
        function_index: u32,
        name: Option<String>,
        body: &FunctionBody,
    ) -> DeepWasmResult<DisassembledFunction> {
        let mut instructions = Vec::new();
        let mut offset = 0u32;

        let reader = body
            .get_operators_reader()
            .map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;

        for op in reader {
            let op = op.map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;

            let (mnemonic, operands) = format_operator(&op);
            let stack_effect = calculate_stack_effect(&op);
            let category = categorize_operator(&op);
            let cost_estimate = estimate_cost(&op);

            instructions.push(DisassembledInstruction {
                offset,
                mnemonic,
                operands,
                stack_effect,
                category,
                cost_estimate,
            });

            offset += 1;
        }

        // Build basic blocks
        let basic_blocks = if self.detect_patterns {
            self.build_basic_blocks(&instructions)?
        } else {
            Vec::new()
        };

        Ok(DisassembledFunction {
            function_index,
            name,
            instructions,
            basic_blocks,
        })
    }

    /// Detect instruction patterns
    pub fn detect_patterns(
        &self,
        instructions: &[DisassembledInstruction],
    ) -> Vec<InstructionPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Dead code after unreachable
        for (i, instr) in instructions.iter().enumerate() {
            if instr.mnemonic == "unreachable" {
                // Check if there are instructions after unreachable before end/else/br
                let mut dead_count = 0;
                for next_instr in instructions.iter().skip(i + 1) {
                    if matches!(
                        next_instr.mnemonic.as_str(),
                        "end" | "else" | "br" | "br_if" | "br_table" | "return"
                    ) {
                        break;
                    }
                    dead_count += 1;
                }

                if dead_count > 0 {
                    patterns.push(InstructionPattern {
                        name: "dead_code_after_unreachable".to_string(),
                        description: format!("{} unreachable instructions after unreachable", dead_count),
                        start_offset: instr.offset,
                        end_offset: instr.offset + dead_count,
                        instruction_count: dead_count + 1,
                        suspicious: true,
                        suspicion_reason: Some(
                            "Dead code detected - instructions after unreachable will never execute"
                                .to_string(),
                        ),
                    });
                }
            }
        }

        // Pattern 2: Infinite loop without side effects
        let mut i = 0;
        while i < instructions.len() {
            if instructions[i].mnemonic == "loop" {
                // Check if loop has a br that goes back to itself with no side effects
                let loop_start = i;
                let mut has_side_effect = false;
                let mut has_backward_br = false;

                for (j, instr) in instructions.iter().enumerate().skip(i + 1) {
                    if instr.mnemonic == "end" {
                        break;
                    }

                    if matches!(
                        instr.category.as_str(),
                        "memory" | "call" | "reference" | "table"
                    ) {
                        has_side_effect = true;
                    }

                    if instr.mnemonic == "br" && j > loop_start {
                        has_backward_br = true;
                    }
                }

                if has_backward_br && !has_side_effect {
                    patterns.push(InstructionPattern {
                        name: "infinite_loop_no_side_effects".to_string(),
                        description: "Loop with backward branch but no side effects".to_string(),
                        start_offset: instructions[loop_start].offset,
                        end_offset: instructions[i].offset,
                        instruction_count: (i - loop_start) as u32 + 1,
                        suspicious: true,
                        suspicion_reason: Some(
                            "Potential infinite loop with no observable side effects".to_string(),
                        ),
                    });
                }
            }
            i += 1;
        }

        // Pattern 3: Excessive stack manipulation
        let mut consecutive_drops = 0;
        let mut drop_start = 0;
        for (i, instr) in instructions.iter().enumerate() {
            if instr.mnemonic == "drop" {
                if consecutive_drops == 0 {
                    drop_start = i;
                }
                consecutive_drops += 1;
            } else {
                if consecutive_drops > 5 {
                    patterns.push(InstructionPattern {
                        name: "excessive_stack_drops".to_string(),
                        description: format!("{} consecutive drop instructions", consecutive_drops),
                        start_offset: instructions[drop_start].offset,
                        end_offset: instructions[i - 1].offset,
                        instruction_count: consecutive_drops,
                        suspicious: true,
                        suspicion_reason: Some(
                            "Excessive stack manipulation may indicate compiler inefficiency"
                                .to_string(),
                        ),
                    });
                }
                consecutive_drops = 0;
            }
        }

        // Pattern 4: Complex nested control flow
        let mut nesting_level: u32 = 0;
        let mut max_nesting: u32 = 0;
        let mut max_nesting_offset: u32 = 0;
        for instr in instructions {
            if matches!(instr.mnemonic.as_str(), "block" | "loop" | "if") {
                nesting_level += 1;
                if nesting_level > max_nesting {
                    max_nesting = nesting_level;
                    max_nesting_offset = instr.offset;
                }
            } else if instr.mnemonic == "end" {
                nesting_level = nesting_level.saturating_sub(1);
            }
        }

        if max_nesting > 10 {
            patterns.push(InstructionPattern {
                name: "deep_control_flow_nesting".to_string(),
                description: format!("Maximum nesting depth of {}", max_nesting),
                start_offset: max_nesting_offset,
                end_offset: max_nesting_offset,
                instruction_count: 1,
                suspicious: true,
                suspicion_reason: Some(
                    "Deep nesting may indicate complex control flow or compiler issues".to_string(),
                ),
            });
        }

        patterns
    }

    /// Build basic blocks from instructions
    fn build_basic_blocks(
        &self,
        instructions: &[DisassembledInstruction],
    ) -> DeepWasmResult<Vec<BasicBlock>> {
        let mut blocks = Vec::new();
        let mut current_block_id = 0u32;
        let mut current_block_start = 0u32;
        let mut block_type = "entry".to_string();

        for (i, instr) in instructions.iter().enumerate() {
            let is_block_terminator = matches!(
                instr.mnemonic.as_str(),
                "br" | "br_if" | "br_table" | "return" | "end"
            );

            let is_block_start = matches!(instr.mnemonic.as_str(), "block" | "loop" | "if");

            if is_block_start {
                // Start new block
                if i > 0 {
                    blocks.push(BasicBlock {
                        id: current_block_id,
                        start_offset: current_block_start,
                        end_offset: instructions[i - 1].offset,
                        block_type: block_type.clone(),
                        successors: vec![current_block_id + 1],
                    });
                    current_block_id += 1;
                }
                current_block_start = instr.offset;
                block_type = instr.mnemonic.clone();
            } else if is_block_terminator && i < instructions.len() - 1 {
                // End current block
                blocks.push(BasicBlock {
                    id: current_block_id,
                    start_offset: current_block_start,
                    end_offset: instr.offset,
                    block_type: block_type.clone(),
                    successors: vec![], // Would need deeper analysis for accurate successors
                });
                current_block_id += 1;
                current_block_start = instructions[i + 1].offset;
                block_type = "basic".to_string();
            }
        }

        // Add final block
        if !instructions.is_empty() {
            blocks.push(BasicBlock {
                id: current_block_id,
                start_offset: current_block_start,
                end_offset: instructions.last().unwrap().offset,
                block_type,
                successors: vec![],
            });
        }

        Ok(blocks)
    }
}

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Format operator as mnemonic and operands
fn format_operator(op: &Operator) -> (String, Vec<String>) {
    match op {
        Operator::Call { function_index } => {
            ("call".to_string(), vec![function_index.to_string()])
        }
        Operator::LocalGet { local_index } => {
            ("local.get".to_string(), vec![local_index.to_string()])
        }
        Operator::LocalSet { local_index } => {
            ("local.set".to_string(), vec![local_index.to_string()])
        }
        Operator::LocalTee { local_index } => {
            ("local.tee".to_string(), vec![local_index.to_string()])
        }
        Operator::GlobalGet { global_index } => {
            ("global.get".to_string(), vec![global_index.to_string()])
        }
        Operator::GlobalSet { global_index } => {
            ("global.set".to_string(), vec![global_index.to_string()])
        }
        Operator::I32Const { value } => ("i32.const".to_string(), vec![value.to_string()]),
        Operator::I64Const { value } => ("i64.const".to_string(), vec![value.to_string()]),
        Operator::F32Const { value } => {
            ("f32.const".to_string(), vec![format!("{:?}", value.bits())])
        }
        Operator::F64Const { value } => {
            ("f64.const".to_string(), vec![format!("{:?}", value.bits())])
        }
        Operator::I32Add => ("i32.add".to_string(), vec![]),
        Operator::I32Sub => ("i32.sub".to_string(), vec![]),
        Operator::I32Mul => ("i32.mul".to_string(), vec![]),
        Operator::I64Add => ("i64.add".to_string(), vec![]),
        Operator::I64Sub => ("i64.sub".to_string(), vec![]),
        Operator::I64Mul => ("i64.mul".to_string(), vec![]),
        Operator::Block { .. } => ("block".to_string(), vec![]),
        Operator::Loop { .. } => ("loop".to_string(), vec![]),
        Operator::If { .. } => ("if".to_string(), vec![]),
        Operator::Else => ("else".to_string(), vec![]),
        Operator::End => ("end".to_string(), vec![]),
        Operator::Br { relative_depth } => ("br".to_string(), vec![relative_depth.to_string()]),
        Operator::BrIf { relative_depth } => {
            ("br_if".to_string(), vec![relative_depth.to_string()])
        }
        Operator::Return => ("return".to_string(), vec![]),
        Operator::Unreachable => ("unreachable".to_string(), vec![]),
        Operator::Drop => ("drop".to_string(), vec![]),
        Operator::Select => ("select".to_string(), vec![]),
        _ => {
            let debug_str = format!("{:?}", op);
            let mnemonic = debug_str
                .split('(')
                .next()
                .unwrap_or("unknown")
                .to_lowercase();
            (mnemonic, vec![])
        }
    }
}

/// Calculate stack effect for an operator
fn calculate_stack_effect(op: &Operator) -> StackEffect {
    match op {
        // Constants: push 1
        Operator::I32Const { .. }
        | Operator::I64Const { .. }
        | Operator::F32Const { .. }
        | Operator::F64Const { .. } => StackEffect { pops: 0, pushes: 1 },

        // Local/global get: push 1
        Operator::LocalGet { .. } | Operator::GlobalGet { .. } => {
            StackEffect { pops: 0, pushes: 1 }
        }

        // Local/global set: pop 1
        Operator::LocalSet { .. } | Operator::GlobalSet { .. } => {
            StackEffect { pops: 1, pushes: 0 }
        }

        // Local tee: pop 1, push 1
        Operator::LocalTee { .. } => StackEffect { pops: 1, pushes: 1 },

        // Binary operations: pop 2, push 1
        Operator::I32Add
        | Operator::I32Sub
        | Operator::I32Mul
        | Operator::I32DivS
        | Operator::I32DivU
        | Operator::I32RemS
        | Operator::I32RemU
        | Operator::I64Add
        | Operator::I64Sub
        | Operator::I64Mul
        | Operator::I64DivS
        | Operator::I64DivU
        | Operator::F32Add
        | Operator::F32Sub
        | Operator::F32Mul
        | Operator::F32Div
        | Operator::F64Add
        | Operator::F64Sub
        | Operator::F64Mul
        | Operator::F64Div => StackEffect { pops: 2, pushes: 1 },

        // Unary operations: pop 1, push 1
        Operator::I32Eqz | Operator::I64Eqz => StackEffect { pops: 1, pushes: 1 },

        // Drop: pop 1
        Operator::Drop => StackEffect { pops: 1, pushes: 0 },

        // Select: pop 3, push 1
        Operator::Select => StackEffect { pops: 3, pushes: 1 },

        // Load operations: pop 1 (address), push 1 (value)
        Operator::I32Load { .. }
        | Operator::I64Load { .. }
        | Operator::F32Load { .. }
        | Operator::F64Load { .. } => StackEffect { pops: 1, pushes: 1 },

        // Store operations: pop 2 (address + value)
        Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. } => StackEffect { pops: 2, pushes: 0 },

        // Control flow with conditions: pop 1
        Operator::BrIf { .. } | Operator::If { .. } => StackEffect { pops: 1, pushes: 0 },

        // Default: no stack effect
        _ => StackEffect { pops: 0, pushes: 0 },
    }
}

/// Categorize operator
fn categorize_operator(op: &Operator) -> String {
    match op {
        Operator::Unreachable
        | Operator::Nop
        | Operator::Block { .. }
        | Operator::Loop { .. }
        | Operator::If { .. }
        | Operator::Else
        | Operator::End
        | Operator::Br { .. }
        | Operator::BrIf { .. }
        | Operator::BrTable { .. }
        | Operator::Return
        | Operator::Call { .. }
        | Operator::CallIndirect { .. } => "control".to_string(),

        Operator::I32Load { .. }
        | Operator::I64Load { .. }
        | Operator::F32Load { .. }
        | Operator::F64Load { .. }
        | Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. }
        | Operator::MemorySize { .. }
        | Operator::MemoryGrow { .. } => "memory".to_string(),

        Operator::LocalGet { .. }
        | Operator::LocalSet { .. }
        | Operator::LocalTee { .. }
        | Operator::GlobalGet { .. }
        | Operator::GlobalSet { .. } => "variable".to_string(),

        Operator::I32Const { .. }
        | Operator::I64Const { .. }
        | Operator::F32Const { .. }
        | Operator::F64Const { .. }
        | Operator::I32Add
        | Operator::I32Sub
        | Operator::I32Mul
        | Operator::I64Add
        | Operator::I64Sub
        | Operator::I64Mul
        | Operator::F32Add
        | Operator::F32Sub
        | Operator::F32Mul
        | Operator::F64Add
        | Operator::F64Sub
        | Operator::F64Mul => "numeric".to_string(),

        Operator::TableGet { .. }
        | Operator::TableSet { .. }
        | Operator::TableGrow { .. }
        | Operator::TableSize { .. } => "table".to_string(),

        Operator::RefNull { .. } | Operator::RefIsNull | Operator::RefFunc { .. } => {
            "reference".to_string()
        }

        _ => "other".to_string(),
    }
}

/// Estimate execution cost
fn estimate_cost(op: &Operator) -> u32 {
    match op {
        // Cheap operations
        Operator::Nop | Operator::Drop | Operator::LocalGet { .. } | Operator::LocalSet { .. } => 1,

        // Medium cost operations
        Operator::I32Const { .. }
        | Operator::I64Const { .. }
        | Operator::I32Add
        | Operator::I32Sub
        | Operator::I64Add
        | Operator::I64Sub => 2,

        // Expensive operations
        Operator::I32Mul | Operator::I64Mul | Operator::F32Mul | Operator::F64Mul => 5,

        Operator::I32DivS
        | Operator::I32DivU
        | Operator::I64DivS
        | Operator::I64DivU
        | Operator::F32Div
        | Operator::F64Div => 10,

        // Memory operations
        Operator::I32Load { .. }
        | Operator::I64Load { .. }
        | Operator::F32Load { .. }
        | Operator::F64Load { .. } => 3,

        Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. } => 4,

        // Call operations (very expensive)
        Operator::Call { .. } => 20,
        Operator::CallIndirect { .. } => 30,

        // Control flow
        Operator::Br { .. } | Operator::BrIf { .. } | Operator::Return => 2,
        Operator::BrTable { .. } => 5,

        // Default
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassembler_creation() {
        let disasm = Disassembler::new();
        assert!(disasm.detect_patterns);
    }

    #[test]
    fn test_format_operator() {
        let op = Operator::I32Add;
        let (mnemonic, operands) = format_operator(&op);
        assert_eq!(mnemonic, "i32.add");
        assert!(operands.is_empty());
    }

    #[test]
    fn test_stack_effect_binary_op() {
        let op = Operator::I32Add;
        let effect = calculate_stack_effect(&op);
        assert_eq!(effect.pops, 2);
        assert_eq!(effect.pushes, 1);
    }

    #[test]
    fn test_categorize_control_flow() {
        let op = Operator::Br { relative_depth: 0 };
        assert_eq!(categorize_operator(&op), "control");
    }

    #[test]
    fn test_cost_estimation() {
        assert_eq!(estimate_cost(&Operator::Nop), 1);
        assert_eq!(estimate_cost(&Operator::I32Add), 2);
        assert_eq!(estimate_cost(&Operator::Call { function_index: 0 }), 20);
    }
}
