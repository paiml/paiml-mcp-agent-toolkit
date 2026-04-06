// disassembler_core.rs — included into disassembler.rs
// Core Disassembler impl methods: new, with_pattern_detection, disassemble_function,
// detect_patterns, build_basic_blocks, and Default impl.

impl Disassembler {
    /// Create a new disassembler
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            detect_patterns: true,
        }
    }

    /// Create disassembler with custom settings
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_pattern_detection(detect_patterns: bool) -> Self {
        Self { detect_patterns }
    }

    /// Disassemble a function
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn disassemble_function(
        &self,
        function_index: u32,
        name: Option<String>,
        body: &FunctionBody,
    ) -> DeepWasmResult<DisassembledFunction> {
        let mut instructions = Vec::new();

        let reader = body
            .get_operators_reader()
            .map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;

        for (offset, op) in reader.into_iter().enumerate() {
            let offset = offset as u32;
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn detect_patterns(
        &self,
        instructions: &[DisassembledInstruction],
    ) -> Vec<InstructionPattern> {
        let mut patterns = Vec::new();
        detect_dead_code_after_unreachable(instructions, &mut patterns);
        detect_infinite_loops(instructions, &mut patterns);
        detect_excessive_drops(instructions, &mut patterns);
        detect_deep_nesting(instructions, &mut patterns);
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
                end_offset: instructions.last().expect("checked is_empty").offset,
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

fn detect_dead_code_after_unreachable(instructions: &[DisassembledInstruction], patterns: &mut Vec<InstructionPattern>) {
    for (i, instr) in instructions.iter().enumerate() {
        if instr.mnemonic != "unreachable" { continue; }
        let dead_count = instructions.iter().skip(i + 1)
            .take_while(|next| !matches!(next.mnemonic.as_str(), "end" | "else" | "br" | "br_if" | "br_table" | "return"))
            .count() as u32;
        if dead_count > 0 {
            patterns.push(InstructionPattern {
                name: "dead_code_after_unreachable".to_string(),
                description: format!("{} unreachable instructions after unreachable", dead_count),
                start_offset: instr.offset,
                end_offset: instr.offset + dead_count,
                instruction_count: dead_count + 1,
                suspicious: true,
                suspicion_reason: Some("Dead code detected - instructions after unreachable will never execute".to_string()),
            });
        }
    }
}

fn detect_infinite_loops(instructions: &[DisassembledInstruction], patterns: &mut Vec<InstructionPattern>) {
    for (i, instr) in instructions.iter().enumerate() {
        if instr.mnemonic != "loop" { continue; }
        let loop_start = i;
        let mut has_side_effect = false;
        let mut has_backward_br = false;
        for (j, inner) in instructions.iter().enumerate().skip(i + 1) {
            if inner.mnemonic == "end" { break; }
            if matches!(inner.category.as_str(), "memory" | "call" | "reference" | "table") {
                has_side_effect = true;
            }
            if inner.mnemonic == "br" && j > loop_start {
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
                suspicion_reason: Some("Potential infinite loop with no observable side effects".to_string()),
            });
        }
    }
}

fn detect_excessive_drops(instructions: &[DisassembledInstruction], patterns: &mut Vec<InstructionPattern>) {
    let mut consecutive_drops: u32 = 0;
    let mut drop_start = 0;
    for (i, instr) in instructions.iter().enumerate() {
        if instr.mnemonic == "drop" {
            if consecutive_drops == 0 { drop_start = i; }
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
                    suspicion_reason: Some("Excessive stack manipulation may indicate compiler inefficiency".to_string()),
                });
            }
            consecutive_drops = 0;
        }
    }
}

fn detect_deep_nesting(instructions: &[DisassembledInstruction], patterns: &mut Vec<InstructionPattern>) {
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
            suspicion_reason: Some("Deep nesting may indicate complex control flow or compiler issues".to_string()),
        });
    }
}
