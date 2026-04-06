// Comprehensive PTX bug detection and post-analysis
// Included into detection.rs via include!()

impl CudaSimdAnalyzer {
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
