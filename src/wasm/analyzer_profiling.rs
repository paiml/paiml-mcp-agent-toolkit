// Instruction profiling and operator categorization
// Included from analyzer.rs - shares parent module scope

/// Instruction profiling for performance analysis
#[derive(Debug, Clone)]
pub struct InstructionProfiler {
    instruction_counts: HashMap<String, usize>,
    total_instructions: usize,
}

impl Default for InstructionProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionProfiler {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            instruction_counts: HashMap::new(),
            total_instructions: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn observe(&mut self, payload: &Payload) {
        if let Payload::CodeSectionEntry(body) = payload {
            // Count instructions by category
            if let Ok(reader) = body.get_operators_reader() {
                for operator in reader.into_iter().flatten() {
                    self.total_instructions += 1;
                    let category = categorize_operator(&operator);
                    *self.instruction_counts.entry(category).or_insert(0) += 1;
                }
            }
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finalize(&self) -> InstructionMix {
        InstructionMix {
            total_instructions: self.total_instructions,
            control_flow: *self.instruction_counts.get("control").unwrap_or(&0),
            memory_ops: *self.instruction_counts.get("memory").unwrap_or(&0),
            arithmetic: *self.instruction_counts.get("arithmetic").unwrap_or(&0),
            calls: *self.instruction_counts.get("call").unwrap_or(&0),
        }
    }
}

/// Categorize WASM operators by type
fn categorize_operator(op: &wasmparser::Operator) -> String {
    debug_assert!(true, "contract: categorize_operator");
    use wasmparser::Operator::{
        Block, Br, BrIf, BrTable, Call, CallIndirect, Else, End, F32Add, F32Div, F32Load, F32Mul,
        F32Store, F32Sub, F64Add, F64Div, F64Load, F64Mul, F64Store, F64Sub, I32Add, I32DivS,
        I32DivU, I32Load, I32Mul, I32Store, I32Sub, I64Add, I64DivS, I64DivU, I64Load, I64Mul,
        I64Store, I64Sub, If, Loop, MemoryGrow, MemorySize, Return,
    };

    match op {
        // Control flow
        Block { .. }
        | Loop { .. }
        | If { .. }
        | Else
        | End
        | Br { .. }
        | BrIf { .. }
        | BrTable { .. }
        | Return => "control".to_string(),

        // Memory operations
        I32Load { .. }
        | I64Load { .. }
        | F32Load { .. }
        | F64Load { .. }
        | I32Store { .. }
        | I64Store { .. }
        | F32Store { .. }
        | F64Store { .. }
        | MemoryGrow { .. }
        | MemorySize { .. } => "memory".to_string(),

        // Function calls
        Call { .. } | CallIndirect { .. } => "call".to_string(),

        // Arithmetic and logic
        I32Add | I32Sub | I32Mul | I32DivS | I32DivU | I64Add | I64Sub | I64Mul | I64DivS
        | I64DivU | F32Add | F32Sub | F32Mul | F32Div | F64Add | F64Sub | F64Mul | F64Div => {
            "arithmetic".to_string()
        }

        // Default
        _ => "other".to_string(),
    }
}
