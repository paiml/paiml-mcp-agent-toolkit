/// Shadow stack for profiling
#[derive(Debug, Clone)]
pub struct ShadowStack {
    pub frames: Vec<StackFrame>,
    pub timestamp: std::time::Instant,
}

impl ShadowStack {
    /// Create from raw bytes (from shared memory)
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        debug_assert!(!bytes.is_empty(), "bytes must not be empty");
        let mut frames = Vec::new();

        // Parse stack frames from bytes (simplified)
        for chunk in bytes.chunks(4) {
            if chunk.len() == 4 {
                let func_idx = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                if func_idx > 0 {
                    frames.push(StackFrame {
                        function_index: func_idx,
                        instruction_offset: 0,
                    });
                }
            }
        }

        Self {
            frames,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Sample current shadow stack (simulation)
    #[must_use]
    pub fn sample() -> Self {
        // This would read from actual shadow memory in production
        Self {
            frames: vec![
                StackFrame {
                    function_index: 1,
                    instruction_offset: 10,
                },
                StackFrame {
                    function_index: 5,
                    instruction_offset: 42,
                },
            ],
            timestamp: std::time::Instant::now(),
        }
    }

    /// Get call stack depth
    #[must_use]
    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Check if function is in stack
    #[must_use]
    pub fn contains_function(&self, func_idx: u32) -> bool {
        self.frames.iter().any(|f| f.function_index == func_idx)
    }
}

/// Individual stack frame
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_index: u32,
    pub instruction_offset: u32,
}

/// Instruction categories for profiling
enum InstructionCategory {
    ControlFlow,
    Memory,
    Arithmetic,
    Call,
    Other,
}

/// Categorize instruction for profiling
fn categorize_for_profiling(op: &Operator) -> InstructionCategory {
    use Operator::{
        Block, Br, BrIf, BrTable, Call, CallIndirect, Else, End, F32Add, F32Div, F32Load, F32Max,
        F32Min, F32Mul, F32Store, F32Sub, F64Add, F64Div, F64Load, F64Max, F64Min, F64Mul,
        F64Store, F64Sub, I32Add, I32And, I32DivS, I32DivU, I32Load, I32Load16S, I32Load16U,
        I32Load8S, I32Load8U, I32Mul, I32Or, I32RemS, I32RemU, I32Rotl, I32Rotr, I32Shl, I32ShrS,
        I32ShrU, I32Store, I32Store16, I32Store8, I32Sub, I32Xor, I64Add, I64And, I64DivS, I64DivU,
        I64Load, I64Load16S, I64Load16U, I64Load32S, I64Load32U, I64Load8S, I64Load8U, I64Mul,
        I64Or, I64RemS, I64RemU, I64Rotl, I64Rotr, I64Shl, I64ShrS, I64ShrU, I64Store, I64Store16,
        I64Store32, I64Store8, I64Sub, I64Xor, If, Loop, MemoryGrow, MemorySize, Return,
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
        | Return => InstructionCategory::ControlFlow,

        // Memory operations
        I32Load { .. }
        | I64Load { .. }
        | F32Load { .. }
        | F64Load { .. }
        | I32Store { .. }
        | I64Store { .. }
        | F32Store { .. }
        | F64Store { .. }
        | I32Load8S { .. }
        | I32Load8U { .. }
        | I32Load16S { .. }
        | I32Load16U { .. }
        | I64Load8S { .. }
        | I64Load8U { .. }
        | I64Load16S { .. }
        | I64Load16U { .. }
        | I64Load32S { .. }
        | I64Load32U { .. }
        | I32Store8 { .. }
        | I32Store16 { .. }
        | I64Store8 { .. }
        | I64Store16 { .. }
        | I64Store32 { .. }
        | MemoryGrow { .. }
        | MemorySize { .. } => InstructionCategory::Memory,

        // Function calls
        Call { .. } | CallIndirect { .. } => InstructionCategory::Call,

        // Arithmetic and logic
        I32Add | I32Sub | I32Mul | I32DivS | I32DivU | I32RemS | I32RemU | I32And | I32Or
        | I32Xor | I32Shl | I32ShrS | I32ShrU | I32Rotl | I32Rotr | I64Add | I64Sub | I64Mul
        | I64DivS | I64DivU | I64RemS | I64RemU | I64And | I64Or | I64Xor | I64Shl | I64ShrS
        | I64ShrU | I64Rotl | I64Rotr | F32Add | F32Sub | F32Mul | F32Div | F32Min | F32Max
        | F64Add | F64Sub | F64Mul | F64Div | F64Min | F64Max => InstructionCategory::Arithmetic,

        // Everything else
        _ => InstructionCategory::Other,
    }
}
