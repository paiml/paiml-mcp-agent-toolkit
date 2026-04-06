// PtxAnalysisState: mutable state for PTX multi-line pattern analysis
// Included into detection.rs via include!()

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
