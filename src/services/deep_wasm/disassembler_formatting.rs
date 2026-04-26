// disassembler_formatting.rs — included into disassembler.rs
// Free functions: format_operator, calculate_stack_effect, categorize_operator, estimate_cost.

/// Format operator as mnemonic and operands
fn format_operator(op: &Operator) -> (String, Vec<String>) {
    match op {
        Operator::Call { function_index } => ("call".to_string(), vec![function_index.to_string()]),
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
        // Wave 39 release-prep: BUG #2 fix — F32/F64 arithmetic ops were
        // falling to the `_ =>` default which produces "f64add" (no dot)
        // via `format!("{:?}", op).to_lowercase()`. Now use the WASM-canonical
        // dotted form, matching the I32/I64 family above.
        Operator::F32Add => ("f32.add".to_string(), vec![]),
        Operator::F32Sub => ("f32.sub".to_string(), vec![]),
        Operator::F32Mul => ("f32.mul".to_string(), vec![]),
        Operator::F32Div => ("f32.div".to_string(), vec![]),
        Operator::F64Add => ("f64.add".to_string(), vec![]),
        Operator::F64Sub => ("f64.sub".to_string(), vec![]),
        Operator::F64Mul => ("f64.mul".to_string(), vec![]),
        Operator::F64Div => ("f64.div".to_string(), vec![]),
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
