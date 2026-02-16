// PTX and CUDA memory pattern detection
// Included into mod.rs via include!()

/// Mutable state for PTX multi-line pattern analysis
struct PtxAnalysisState {
    shared_load_regs: Vec<String>,
    loop_labels: std::collections::HashSet<String>,
    loop_end_labels: std::collections::HashSet<String>,
    in_loop: bool,
    loop_start_line: usize,
    barrier_seen_in_loop: bool,
    last_st_shared_line: Option<usize>,
    last_mov: Option<(usize, String, String)>,
    after_unconditional: bool,
    unconditional_line: usize,
}

impl PtxAnalysisState {
    fn new() -> Self {
        Self {
            shared_load_regs: Vec::new(),
            loop_labels: std::collections::HashSet::new(),
            loop_end_labels: std::collections::HashSet::new(),
            in_loop: false,
            loop_start_line: 0,
            barrier_seen_in_loop: false,
            last_st_shared_line: None,
            last_mov: None,
            after_unconditional: false,
            unconditional_line: 0,
        }
    }

    /// First pass: identify loop labels (labels with back-edge branches)
    fn identify_loop_labels(&mut self, lines: &[&str], content: &str) {
        for line in lines {
            let trimmed = line.trim();
            if trimmed.ends_with(':') && !trimmed.starts_with('.') {
                let label = trimmed.trim_end_matches(':').to_string();
                let bra_pattern = format!("bra {};", label);
                let bra_pattern2 = format!("bra {}", label);
                if content.contains(&bra_pattern) || content.contains(&bra_pattern2) {
                    self.loop_end_labels.insert(format!("{}_end", label));
                    self.loop_end_labels.insert(format!("{}_done", label));
                    self.loop_labels.insert(label);
                }
            }
        }
    }

    /// Check comment line for placeholder patterns
    fn check_placeholder(&self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lower = trimmed.to_lowercase();
        let placeholders = [
            "omitted", "simplified", "placeholder", "todo",
            "fixme", "not implemented", "for now", "for brevity",
        ];
        for pattern in &placeholders {
            if lower.contains(pattern) {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "PLACEHOLDER".to_string(),
                        description: format!("Placeholder/incomplete code: '{}'", pattern),
                        severity: DefectSeverity::P1Performance,
                        detection_method: "Comment analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Code is incomplete and may not work correctly".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some("Implement complete kernel logic".to_string()),
                });
                break;
            }
        }
    }

    /// Update loop/label tracking state
    fn track_labels(&mut self, line_num: usize, trimmed: &str) {
        if trimmed.ends_with(':') && !trimmed.starts_with('.') {
            let label = trimmed.trim_end_matches(':');
            if self.loop_labels.contains(label) {
                self.in_loop = true;
                self.loop_start_line = line_num + 1;
                self.barrier_seen_in_loop = false;
            }
            if self.loop_end_labels.contains(label)
                || label.contains("_end")
                || label.contains("_done")
            {
                self.in_loop = false;
            }
            self.after_unconditional = false;
        }
    }

    /// Track barrier instructions
    fn track_barriers(&mut self, trimmed: &str) {
        if trimmed.contains("bar.sync") || trimmed.contains("bar.arrive") {
            if self.in_loop {
                self.barrier_seen_in_loop = true;
            }
            self.last_st_shared_line = None;
        }
    }

    /// P0: SHARED_U64 - 64-bit register for shared memory
    fn check_shared_u64(&self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        if (trimmed.contains("st.shared") || trimmed.contains("ld.shared"))
            && trimmed.contains("[%rd")
        {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SHARED_U64".to_string(),
                    description: "Shared memory accessed with 64-bit register".to_string(),
                    severity: DefectSeverity::P0Critical,
                    detection_method: "PTX pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Shared memory requires 32-bit addressing".to_string()),
                },
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(trimmed.to_string()),
                suggestion: Some("Replace %rd* with %r* for shared memory addressing".to_string()),
            });
        }
    }

    /// P0: cvta.shared creates generic address corruption
    fn check_cvta_shared(&self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        if trimmed.contains("cvta.shared") {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "CVTA_SHARED".to_string(),
                    description: "cvta.shared creates 64-bit generic address that SASS may clobber".to_string(),
                    severity: DefectSeverity::P0Critical,
                    detection_method: "PTX pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Generic address from cvta.shared causes address corruption".to_string()),
                },
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(trimmed.to_string()),
                suggestion: Some("Use direct ld.shared/st.shared with 32-bit offset instead".to_string()),
            });
        }
    }

    /// P0: Missing barrier between st.shared and ld.shared + F082 data-dependent addressing
    fn check_shared_memory_ops(&mut self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        if trimmed.contains("st.shared") {
            self.last_st_shared_line = Some(line_num);
        }

        if trimmed.contains("ld.shared") {
            if let Some(reg) = CudaSimdAnalyzer::extract_ptx_dest_register(trimmed) {
                self.shared_load_regs.push(reg);
            }
            if let Some(st_line) = self.last_st_shared_line {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "MISSING_BARRIER".to_string(),
                        description: "ld.shared follows st.shared without barrier synchronization".to_string(),
                        severity: DefectSeverity::P0Critical,
                        detection_method: "PTX dataflow analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Race condition: threads may read stale data".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(format!("st.shared at line {}, ld.shared at line {}", st_line + 1, line_num + 1)),
                    suggestion: Some(format!("Add bar.sync 0; between lines {} and {}", st_line + 1, line_num + 1)),
                });
            }
        }

        // F082: Address computed from shared memory value
        if (trimmed.contains("add.u64") || trimmed.contains("add.s64") || trimmed.contains("cvt.u64"))
            && !self.shared_load_regs.is_empty()
        {
            for reg in &self.shared_load_regs {
                if trimmed.contains(reg.as_str()) {
                    analysis.defects.push(DetectedDefect {
                        defect_class: DefectClass {
                            ticket_id: "F082".to_string(),
                            description: "Address computed from shared memory load (data-dependent addressing)".to_string(),
                            severity: DefectSeverity::P0Critical,
                            detection_method: "PTX dataflow analysis".to_string(),
                            resolved: false,
                            root_cause: Some("Address register depends on value loaded from shared memory, causing non-uniform memory access".to_string()),
                        },
                        file_path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        snippet: Some(trimmed.to_string()),
                        suggestion: Some("Compute address from thread ID or constant offsets only".to_string()),
                    });
                }
            }
        }
    }

    /// P0: PARITY-114 early exit before barrier
    fn check_early_exit(&self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        let is_exit_branch = trimmed.contains("bra exit")
            || (trimmed.contains("bra ") && trimmed.contains("done"));
        if is_exit_branch && self.in_loop && !self.barrier_seen_in_loop {
            let kind = if trimmed.starts_with('@') { "Conditional" } else { "Unconditional" };
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "PARITY-114".to_string(),
                    description: format!("{} early exit before barrier in loop", kind),
                    severity: DefectSeverity::P0Critical,
                    detection_method: "PTX CFG analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Some threads exit before bar.sync, causing remaining threads to hang".to_string()),
                },
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(trimmed.to_string()),
                suggestion: Some(format!("Move bounds check AFTER loop body (loop starts at line {})", self.loop_start_line)),
            });
        }
    }

    /// P1: Loop branches to END instead of START
    fn check_loop_branch_end(&self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        if trimmed.starts_with("bra ") && !trimmed.starts_with('@') {
            if let Some(target) = trimmed.strip_prefix("bra ").map(|s| s.trim_end_matches(';').trim()) {
                if target.contains("_end") || target.ends_with("_done") {
                    analysis.defects.push(DetectedDefect {
                        defect_class: DefectClass {
                            ticket_id: "LOOP_BRANCH_END".to_string(),
                            description: "Unconditional branch to loop end label".to_string(),
                            severity: DefectSeverity::P1Performance,
                            detection_method: "PTX CFG analysis".to_string(),
                            resolved: false,
                            root_cause: Some("Loop may be incomplete or have early exit".to_string()),
                        },
                        file_path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        snippet: Some(trimmed.to_string()),
                        suggestion: Some("Verify this branch target is intentional".to_string()),
                    });
                }
            }
        }
    }

    /// P2: Dead code after unconditional jump + track unconditional jumps
    fn check_dead_code(&mut self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        if self.after_unconditional && !trimmed.ends_with(':') && trimmed != "}" {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "DEAD_CODE".to_string(),
                    description: "Unreachable code after unconditional jump".to_string(),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "PTX CFG analysis".to_string(),
                    resolved: false,
                    root_cause: Some(format!("Code unreachable after line {}", self.unconditional_line + 1)),
                },
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(trimmed.to_string()),
                suggestion: Some("Remove unreachable code or add label".to_string()),
            });
            self.after_unconditional = false;
        }

        if trimmed == "ret;" || (trimmed.starts_with("bra ") && !trimmed.starts_with('@')) {
            self.after_unconditional = true;
            self.unconditional_line = line_num;
        }
    }

    /// P2: Redundant mov chains
    fn check_redundant_mov(&mut self, line_num: usize, trimmed: &str, path: &Path, analysis: &mut FileAnalysis) {
        let mov_pattern = regex::Regex::new(r"^\s*mov\.\w+\s+(%\w+),\s*(%\w+)").ok();
        if let Some(ref re) = mov_pattern {
            if let Some(caps) = re.captures(trimmed) {
                let dest = caps.get(1).map(|m| m.as_str().to_string());
                let src = caps.get(2).map(|m| m.as_str().to_string());
                if let (Some(d), Some(s)) = (dest, src) {
                    if let Some((prev_line, prev_dest, _)) = &self.last_mov {
                        if &s == prev_dest {
                            analysis.defects.push(DetectedDefect {
                                defect_class: DefectClass {
                                    ticket_id: "REDUNDANT_MOV".to_string(),
                                    description: "Redundant register move chain".to_string(),
                                    severity: DefectSeverity::P2Efficiency,
                                    detection_method: "PTX dataflow analysis".to_string(),
                                    resolved: false,
                                    root_cause: Some(format!("mov chain at lines {} and {}", prev_line + 1, line_num + 1)),
                                },
                                file_path: path.to_path_buf(),
                                line: Some(line_num + 1),
                                snippet: Some(trimmed.to_string()),
                                suggestion: Some("Combine mov chain into single mov".to_string()),
                            });
                        }
                    }
                    self.last_mov = Some((line_num, d, s));
                }
            }
        } else {
            self.last_mov = None;
        }
    }

    /// Track memory operations for coalescing analysis
    fn track_memory_ops(&self, trimmed: &str, analysis: &mut FileAnalysis) {
        if trimmed.contains("ld.global") || trimmed.contains("st.global") {
            analysis.coalescing.total_operations += 1;
            if trimmed.contains("%tid") || trimmed.contains("param") {
                analysis.coalescing.coalesced_operations += 1;
            }
        }
        if trimmed.contains("ld.shared") || trimmed.contains("st.shared") {
            analysis.coalescing.total_operations += 1;
            analysis.coalescing.coalesced_operations += 1;
        }
    }
}

impl CudaSimdAnalyzer {

    fn detect_barrier_issues(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.contains("__syncthreads")
                || line.contains("__syncwarp")
                || line.contains("bar.sync")
            {
                analysis.barrier_safety.total_barriers += 1;
                if self.check_barrier_has_early_return(&lines, line_num) {
                    self.report_barrier_issue(line, line_num, path, analysis);
                } else {
                    analysis.barrier_safety.safe_barriers += 1;
                }
            }
        }

        if analysis.barrier_safety.total_barriers > 0 {
            analysis.barrier_safety.safety_score = analysis.barrier_safety.safe_barriers as f64
                / analysis.barrier_safety.total_barriers as f64;
        }
    }

    /// Check if there's an early return in the same function scope before a barrier
    fn check_barrier_has_early_return(&self, lines: &[&str], barrier_line: usize) -> bool {
        let before_barrier = lines[..barrier_line].join("\n");
        if !before_barrier.contains("return;")
            && !before_barrier.contains("return ")
            && !before_barrier.contains("exit")
        {
            return false;
        }

        let mut brace_depth = 0i32;
        for prev_line in lines[..barrier_line].iter().rev() {
            if prev_line.contains('}') { brace_depth += 1; }
            if prev_line.contains('{') {
                brace_depth -= 1;
                if brace_depth < 0 { break; }
            }
            if brace_depth == 0
                && (prev_line.contains("return;") || prev_line.contains("return "))
            {
                return true;
            }
        }
        false
    }

    /// Report an unsafe barrier issue with defect
    fn report_barrier_issue(&self, line: &str, line_num: usize, path: &Path, analysis: &mut FileAnalysis) {
        let barrier_type = if line.contains("__syncthreads") {
            "__syncthreads"
        } else if line.contains("__syncwarp") {
            "__syncwarp"
        } else {
            "bar.sync"
        };
        analysis.barrier_safety.unsafe_barriers.push(BarrierIssue {
            line: line_num + 1,
            barrier_type: barrier_type.to_string(),
            issue: "PARITY-114: Possible thread exit before barrier".to_string(),
            exit_paths: vec!["Early return detected before barrier".to_string()],
        });
        if let Some(defect_class) = self.taxonomy.get("PARITY-114") {
            analysis.defects.push(DetectedDefect {
                defect_class: defect_class.clone(),
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(line.trim().to_string()),
                suggestion: Some("Ensure all threads reach barrier or use cooperative groups".to_string()),
            });
        }
    }

    fn detect_memory_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let is_ptx = path.extension().is_some_and(|e| e == "ptx");

        if is_ptx {
            self.detect_ptx_memory_patterns(content, path, analysis);
            return;
        }

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("[threadIdx.x") || line.contains("[tid") || line.contains("global_mem[") {
                analysis.coalescing.total_operations += 1;
                if line.contains("* stride") || line.contains("* STRIDE") {
                    analysis.coalescing.problematic_accesses.push(MemoryAccessIssue {
                        line: line_num + 1,
                        pattern: AccessPattern::Strided { stride: 0 },
                        impact: "Strided access may reduce memory throughput".to_string(),
                    });
                } else {
                    analysis.coalescing.coalesced_operations += 1;
                }
            }
            if line.contains("__shared__") && line.contains("[threadIdx") {
                if line.contains("% 32") || line.contains("& 31") {
                    analysis.coalescing.coalesced_operations += 1;
                }
            }
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }
    }

    /// Comprehensive PTX bug detection based on trueno research and Tauranta fault history
    ///
    /// Detects (from trueno-explain/src/ptx/bugs.rs and trueno-ptx-debug):
    ///
    /// ## P0 Critical
    /// - F082: Address computed from shared memory load (data-dependent addressing)
    /// - SHARED_U64: Shared memory accessed with 64-bit register (should be 32-bit)
    /// - LOOP_BRANCH_END: Loop branches to END label instead of START
    /// - MISSING_BARRIER: Missing bar.sync between st.shared and ld.shared
    /// - EARLY_EXIT_BARRIER: Early thread exit before barrier (PARITY-114)
    /// - GENERIC_ADDR_CORRUPTION: cvta.shared creates 64-bit generic address
    ///
    /// ## P1 High (Performance)
    /// - REG_SPILLS: Register spills to local memory
    /// - HIGH_REG_PRESSURE: >64 registers reduces occupancy
    /// - PRED_OVERFLOW: >8 predicate registers causes spills
    /// - PLACEHOLDER_CODE: Incomplete code detected ("omitted", "simplified")
    /// - EMPTY_LOOP: Loop body contains no computation
    /// - NO_BOUNDS_CHECK: Missing thread bounds check before memory access
    ///
    /// ## P2 Medium (Efficiency)
    /// - REDUNDANT_MOV: Redundant register move chains
    /// - UNOPT_MEM: Multiple single loads could be vectorized
    /// - DEAD_CODE: Unreachable code after ret or unconditional branch
    fn detect_ptx_memory_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();
        let mut state = PtxAnalysisState::new();

        state.identify_loop_labels(&lines, content);
        let (total_registers, predicate_count) = Self::count_ptx_registers(content);

        // Main analysis pass: delegate per-line checks to state methods
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let is_comment = trimmed.starts_with("//");

            if is_comment {
                state.check_placeholder(line_num, trimmed, path, analysis);
            }
            if trimmed.is_empty() || is_comment {
                continue;
            }

            state.track_labels(line_num, trimmed);
            state.track_barriers(trimmed);
            state.check_shared_u64(line_num, trimmed, path, analysis);
            state.check_cvta_shared(line_num, trimmed, path, analysis);
            state.check_shared_memory_ops(line_num, trimmed, path, analysis);
            state.check_early_exit(line_num, trimmed, path, analysis);
            state.check_loop_branch_end(line_num, trimmed, path, analysis);
            state.check_dead_code(line_num, trimmed, path, analysis);
            state.check_redundant_mov(line_num, trimmed, path, analysis);
            state.track_memory_ops(trimmed, analysis);
        }

        Self::ptx_post_analysis(content, path, analysis, total_registers, predicate_count);
    }

    /// Count PTX register declarations for pressure analysis
    fn count_ptx_registers(content: &str) -> (usize, usize) {
        let mut total_registers: usize = 0;
        let mut predicate_count: usize = 0;

        if let Some(re) = regex::Regex::new(r"\.reg\s+\.\w+\s+%\w+<(\d+)>").ok() {
            for caps in re.captures_iter(content) {
                if let Some(count) = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) {
                    total_registers += count;
                }
            }
        }
        if let Some(re) = regex::Regex::new(r"\.reg\s+\.pred\s+%p<(\d+)>").ok() {
            if let Some(caps) = re.captures(content) {
                if let Some(count) = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) {
                    predicate_count = count;
                }
            }
        }
        (total_registers, predicate_count)
    }

    /// Post-analysis checks for register pressure, memory patterns, bounds, and entry points
    fn ptx_post_analysis(
        content: &str,
        path: &Path,
        analysis: &mut FileAnalysis,
        total_registers: usize,
        predicate_count: usize,
    ) {
        if content.contains(".local") {
            let local_count = content.matches(".local").count();
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "REG_SPILLS".to_string(),
                    description: format!("{} potential register spills to local memory", local_count),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX memory analysis".to_string(),
                    resolved: false,
                    root_cause: Some("High register pressure causing spills to slow local memory".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("{} .local declarations", local_count)),
                suggestion: Some("Reduce live variables or split kernel".to_string()),
            });
        }

        if total_registers > 64 {
            let occupancy = 65536 / (total_registers.max(1) * 32);
            let occupancy_pct = (occupancy as f64 / 32.0 * 100.0).min(100.0);
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "HIGH_REG_PRESSURE".to_string(),
                    description: format!("High register pressure: {} registers limits occupancy to {:.0}%", total_registers, occupancy_pct),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX register analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Too many registers reduce SM occupancy".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("{} registers declared", total_registers)),
                suggestion: Some("Reduce live variables or split into multiple kernels".to_string()),
            });
        }

        if predicate_count > 8 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "PRED_OVERFLOW".to_string(),
                    description: format!("Predicate overflow: {} predicates declared (max 8 hardware)", predicate_count),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX register analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Excess predicates cause spills".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("{} predicates", predicate_count)),
                suggestion: Some("Combine conditions or use branches".to_string()),
            });
        }

        let single_loads = content.matches("ld.global.f32").count();
        let vector_loads = content.matches("ld.global.v2.f32").count()
            + content.matches("ld.global.v4.f32").count();
        if single_loads >= 4 && vector_loads == 0 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "UNOPT_MEM".to_string(),
                    description: format!("{} single f32 loads, 0 vector loads", single_loads),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "PTX memory analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Multiple single loads could be vectorized".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Consider ld.global.v2.f32 or ld.global.v4.f32 for consecutive addresses".to_string()),
            });
        }

        let has_tid = content.contains("%tid.") || content.contains("%ntid.");
        let has_global_mem = content.contains("ld.global") || content.contains("st.global");
        let has_bounds_check = content.contains("setp.lt") || content.contains("setp.ge");
        let has_predicated_branch = content.contains("@%p") && content.contains("bra");
        if has_tid && has_global_mem && !has_bounds_check && !has_predicated_branch {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "NO_BOUNDS_CHECK".to_string(),
                    description: "Kernel accesses global memory but lacks bounds checking".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX CFG analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Thread may access out-of-bounds memory".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Add: setp.lt.u32 %p0, %tid, %size; @%p0 bra do_work;".to_string()),
            });
        }

        if !content.trim().is_empty() && !content.contains(".entry") {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "NO_ENTRY".to_string(),
                    description: "No kernel entry point (.entry) found".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX structure analysis".to_string(),
                    resolved: false,
                    root_cause: Some("PTX file lacks kernel entry point".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Add .entry <kernel_name>(...) declaration".to_string()),
            });
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }
    }
}
