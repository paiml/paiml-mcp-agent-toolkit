// Instruction utility functions: type conversion, categorization, stack depth tracking

/// Convert ValType to string representation
fn valtype_to_string(ty: &ValType) -> String {
    debug_assert!(true, "contract: valtype_to_string");
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
    debug_assert!(true, "contract: categorize_instruction");
    match op {
        // Control flow
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
        | Operator::CallIndirect { .. } => {
            breakdown.control_flow += 1;
        }

        // Memory operations
        Operator::I32Load { .. }
        | Operator::I64Load { .. }
        | Operator::F32Load { .. }
        | Operator::F64Load { .. }
        | Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. }
        | Operator::MemorySize { .. }
        | Operator::MemoryGrow { .. } => {
            breakdown.memory_ops += 1;
        }

        // Variable operations
        Operator::LocalGet { .. }
        | Operator::LocalSet { .. }
        | Operator::LocalTee { .. }
        | Operator::GlobalGet { .. }
        | Operator::GlobalSet { .. } => {
            breakdown.variable_ops += 1;
        }

        // Table operations
        Operator::TableGet { .. }
        | Operator::TableSet { .. }
        | Operator::TableGrow { .. }
        | Operator::TableSize { .. }
        | Operator::TableFill { .. } => {
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
    debug_assert!(true, "contract: update_stack_depth");
    match op {
        // Instructions that push values
        Operator::I32Const { .. }
        | Operator::I64Const { .. }
        | Operator::F32Const { .. }
        | Operator::F64Const { .. }
        | Operator::LocalGet { .. }
        | Operator::GlobalGet { .. } => {
            *depth = depth.saturating_add(1);
        }

        // Instructions that pop and push (net: 0)
        Operator::I32Eqz | Operator::I64Eqz => {}

        // Instructions that pop values
        Operator::Drop => {
            *depth = depth.saturating_sub(1);
        }
        Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. } => {
            *depth = depth.saturating_sub(2);
        }

        // Binary operations (pop 2, push 1)
        Operator::I32Add
        | Operator::I32Sub
        | Operator::I32Mul
        | Operator::I32DivS
        | Operator::I32DivU
        | Operator::I32RemS
        | Operator::I32RemU
        | Operator::I32And
        | Operator::I32Or
        | Operator::I32Xor
        | Operator::I32Shl
        | Operator::I32ShrS
        | Operator::I32ShrU
        | Operator::I32Rotl
        | Operator::I32Rotr
        | Operator::I32Eq
        | Operator::I32Ne
        | Operator::I32LtS
        | Operator::I32LtU
        | Operator::I32GtS
        | Operator::I32GtU
        | Operator::I32LeS
        | Operator::I32LeU
        | Operator::I32GeS
        | Operator::I32GeU => {
            *depth = depth.saturating_sub(1);
        }

        _ => {
            // For other instructions, we'd need more detailed analysis
            // This is a simplified version
        }
    }
}
