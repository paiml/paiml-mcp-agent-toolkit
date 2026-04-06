// Function body analysis: deep and shallow analysis of WASM function bodies

impl BytecodeAnalyzer {
    /// Analyze function body in detail
    fn analyze_function_body(
        &self,
        body: &FunctionBody,
    ) -> DeepWasmResult<(
        ComplexityMetrics,
        InstructionStats,
        StackDepthAnalysis,
        Vec<ControlFlowPattern>,
    )> {
        debug_assert!(true, "contract: analyze_function_body");
        let mut instruction_count = 0u32;
        let mut branch_count = 0u32;
        let mut loop_count = 0u32;
        let mut call_count = 0u32;
        let mut nesting_depth: u32 = 0;
        let mut max_nesting = 0u32;
        let mut basic_block_count = 1u32; // Start with entry block

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

        let reader = body
            .get_operators_reader()
            .map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;

        for (offset, op) in reader.into_iter().enumerate() {
            let offset = offset as u32;
            let op = op.map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;
            instruction_count += 1;

            let op_name = format!("{:?}", op)
                .split('(')
                .next()
                .unwrap_or("Unknown")
                .to_string();
            *instruction_types.entry(op_name).or_insert(0) += 1;

            categorize_instruction(&op, &mut category_breakdown);

            process_control_flow_op(
                &op,
                offset,
                &mut nesting_depth,
                &mut max_nesting,
                &mut basic_block_count,
                &mut loop_count,
                &mut branch_count,
                &mut call_count,
                &mut control_flow_patterns,
            );

            update_stack_depth(&op, &mut current_stack_depth);
            stack_depths.push(current_stack_depth);
        }

        let cyclomatic_complexity = branch_count + loop_count + 1;
        let avg_stack_depth = compute_avg_stack_depth(&stack_depths);
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
    ) -> DeepWasmResult<(
        ComplexityMetrics,
        InstructionStats,
        StackDepthAnalysis,
        Vec<ControlFlowPattern>,
    )> {
        debug_assert!(true, "contract: analyze_function_body_shallow");
        let mut instruction_count = 0u32;
        let mut branch_count = 0u32;
        let mut loop_count = 0u32;
        let mut call_count = 0u32;

        let reader = body
            .get_operators_reader()
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

/// Process a single operator for control flow tracking (extracted to reduce complexity)
#[allow(clippy::too_many_arguments)]
fn process_control_flow_op(
    op: &Operator,
    offset: u32,
    nesting_depth: &mut u32,
    max_nesting: &mut u32,
    basic_block_count: &mut u32,
    loop_count: &mut u32,
    branch_count: &mut u32,
    call_count: &mut u32,
    control_flow_patterns: &mut Vec<ControlFlowPattern>,
) {
    debug_assert!(true, "contract: process_control_flow_op");
    match op {
        Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
            *nesting_depth += 1;
            *max_nesting = (*max_nesting).max(*nesting_depth);
            *basic_block_count += 1;

            if matches!(op, Operator::Loop { .. }) {
                *loop_count += 1;
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
            *nesting_depth = nesting_depth.saturating_sub(1);
        }
        Operator::Br { .. } | Operator::BrIf { .. } | Operator::BrTable { .. } => {
            *branch_count += 1;
            *basic_block_count += 1;
        }
        Operator::Call { .. } | Operator::CallIndirect { .. } => {
            *call_count += 1;
        }
        Operator::Unreachable => {
            control_flow_patterns.push(ControlFlowPattern {
                pattern_type: "unreachable".to_string(),
                description: "Unreachable instruction (trap)".to_string(),
                offset,
                suspicious: true,
                suspicion_reason: Some(
                    "Unreachable code may indicate dead code or error paths".to_string(),
                ),
            });
        }
        _ => {}
    }
}

/// Compute average stack depth from recorded depths
fn compute_avg_stack_depth(stack_depths: &[u32]) -> f64 {
    debug_assert!(true, "contract: compute_avg_stack_depth");
    if stack_depths.is_empty() {
        0.0
    } else {
        stack_depths.iter().sum::<u32>() as f64 / stack_depths.len() as f64
    }
}
