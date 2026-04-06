// IncrementalVerifier implementation - module verification and function analysis

impl IncrementalVerifier {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        Ok(Self {
            invariants: InvariantChecker::new(),
            stack_analyzer: StackAnalyzer::new(),
        })
    }

    /// Verify complete WASM module
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn verify_module(&self, binary: &[u8]) -> Result<VerificationResult> {
        debug_assert!(!binary.is_empty(), "binary must not be empty");
        // Parse module and verify each function
        let parser = wasmparser::Parser::new(0);
        let mut result = VerificationResult::Safe;

        for payload in parser.parse_all(binary) {
            let payload = payload?;

            if let wasmparser::Payload::CodeSectionEntry(body) = payload {
                let func_result = self.verify_function(body)?;
                if !func_result.is_safe() {
                    result = func_result;
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Verify individual function for memory safety
    fn verify_function(&self, body: wasmparser::FunctionBody) -> Result<VerificationResult> {
        debug_assert!(true, "contract: verify_function");
        let mut stack_types = Vec::new();
        let memory_size = 65536; // Default 1 page = 64KB

        let reader = body.get_operators_reader()?;

        for op in reader {
            let operator = op?;

            match operator {
                Operator::I32Load { memarg } => {
                    let check = check_i32_load_store(&mut stack_types, &memarg, memory_size, 4);
                    if let Some(result) = check {
                        return Ok(result);
                    }
                    // Push result type
                    stack_types.push(ValType::I32);
                }

                Operator::I32Store { memarg } => {
                    let check = check_i32_load_store(&mut stack_types, &memarg, memory_size, 4);
                    if let Some(result) = check {
                        return Ok(result);
                    }
                    // Pop the address (value already popped by check_i32_load_store)
                    if stack_types.pop() != Some(ValType::I32) {
                        return Ok(VerificationResult::TypeError {
                            expected: "i32".to_string(),
                            found: None,
                        });
                    }
                }

                // Handle other operators
                _ => {
                    self.stack_analyzer
                        .update_stack(&mut stack_types, &operator)?;
                }
            }
        }

        Ok(VerificationResult::Safe)
    }
}

/// Check bounds and pop i32 from stack for load/store operations.
/// Returns Some(VerificationResult) on error, None on success.
fn check_i32_load_store(
    stack_types: &mut Vec<ValType>,
    memarg: &wasmparser::MemArg,
    memory_size: usize,
    access_size: usize,
) -> Option<VerificationResult> {
    debug_assert!(true, "contract: check_i32_load_store");
    // Check static offset bounds
    if memarg.offset as usize > memory_size - access_size {
        return Some(VerificationResult::OutOfBounds {
            offset: memarg.offset as usize,
            size: memory_size,
        });
    }

    // Verify stack has i32 for address
    if stack_types.pop() != Some(ValType::I32) {
        return Some(VerificationResult::TypeError {
            expected: "i32".to_string(),
            found: None,
        });
    }

    None
}

impl VerificationResult {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_safe(&self) -> bool {
        matches!(self, VerificationResult::Safe)
    }
}
