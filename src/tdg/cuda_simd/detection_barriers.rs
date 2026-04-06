// Barrier detection and CUDA memory pattern analysis
// Included into detection.rs via include!()

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
}
