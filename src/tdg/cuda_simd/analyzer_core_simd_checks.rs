// CudaSimdAnalyzer SIMD/CUDA/WGPU content analysis and defect detection checks

impl CudaSimdAnalyzer {
    fn analyze_cuda_content(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.detect_barrier_issues(content, path, analysis);
        self.detect_memory_patterns(content, path, analysis);
        self.detect_known_patterns(content, path, analysis);
    }

    fn analyze_wgpu_content(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let barrier_count =
            content.matches("workgroupBarrier").count() + content.matches("storageBarrier").count();
        analysis.barrier_safety.total_barriers += barrier_count;
        analysis.barrier_safety.safe_barriers += barrier_count;
        self.detect_wgpu_memory_patterns(content, path, analysis);
    }

    /// Comprehensive SIMD bug detection based on trueno research
    fn analyze_simd_content(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let lines: Vec<&str> = content.lines().collect();
        let mut state = SimdAnalysisState::new();

        let has_target_feature = content.contains("#[target_feature(enable");
        let has_avx512 = content.contains("avx512") || content.contains(concat!("_mm", "512_"));
        let has_avx = content.contains(concat!("_mm", "256_")) || content.contains("avx2");
        let _has_sse = content.contains("_mm_") || content.contains("sse");

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            state.track_safety_comment(trimmed);
            state.track_unsafe_blocks(line_num, trimmed, &lines, path, analysis);
            state.count_instructions(trimmed, content, analysis);
            self.check_simd_alignment(line_num, trimmed, &lines, path, analysis);
            self.check_simd_bounds(line_num, trimmed, content, path, analysis);
            if self.check_simd_vzeroupper(trimmed, content, line_num, path, analysis) {
                break; // Only report once per file
            }
        }

        // Post-loop file-level checks
        Self::check_missing_target_feature(has_avx512, has_avx, has_target_feature, content, path, analysis);
        Self::check_vectorization_ratio(&state, path, analysis);
        Self::check_suboptimal_width(&state, has_avx, path, analysis);

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }
    }

    /// P0: Alignment fault risk for aligned SIMD loads
    fn check_simd_alignment(
        &self, line_num: usize, trimmed: &str, lines: &[&str], path: &Path, analysis: &mut FileAnalysis,
    ) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Use concat! to avoid self-matching during CB-021 compliance scanning
        if trimmed.contains(concat!("_mm", "256_load_si256"))
            || trimmed.contains(concat!("_mm", "512_load_si512"))
            || trimmed.contains(concat!("_mm", "256_load_ps"))
            || trimmed.contains(concat!("_mm", "512_load_ps"))
        {
            let context_start = line_num.saturating_sub(10);
            let context = lines[context_start..=line_num].join("\n");
            let has_align = context.contains("align")
                || context.contains("ALIGN")
                || context.contains("repr(align")
                || context.contains("as_ptr()");
            if !has_align {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SIMD_ALIGN_FAULT".to_string(),
                        description: "Aligned SIMD load without visible alignment guarantee".to_string(),
                        severity: DefectSeverity::P0Critical,
                        detection_method: "SIMD pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Aligned loads require 32/64-byte aligned pointers".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some("Use _loadu_ variant or ensure pointer is aligned".to_string()),
                });
            }
        }
    }

    /// P0: Bounds overflow risk for SIMD loads without length checks
    fn check_simd_bounds(
        &self, line_num: usize, trimmed: &str, content: &str, path: &Path, analysis: &mut FileAnalysis,
    ) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Use concat! to avoid self-matching during CB-021 compliance scanning
        if (trimmed.contains(concat!("_mm", "256_loadu_")) || trimmed.contains(concat!("_mm", "512_loadu_")))
            && !content.contains("len()")
            && !content.contains(".len")
        {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SIMD_BOUNDS_OVERFLOW".to_string(),
                    description: "SIMD load without visible bounds check".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "SIMD pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("SIMD loads may read past buffer end".to_string()),
                },
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(trimmed.to_string()),
                suggestion: Some("Ensure i + SIMD_WIDTH <= len before SIMD operations".to_string()),
            });
        }
    }

    /// P1: SSE/AVX transition penalty. Returns true if defect was reported (caller should break).
    fn check_simd_vzeroupper(
        &self, trimmed: &str, content: &str, line_num: usize, path: &Path, analysis: &mut FileAnalysis,
    ) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Use concat! to avoid self-matching during CB-021 compliance scanning
        let is_sse_only = trimmed.contains("_mm_")
            && !trimmed.contains(concat!("_mm", "256_"))
            && !trimmed.contains(concat!("_mm", "512_"));
        let has_wider = content.contains(concat!("_mm", "256_")) || content.contains(concat!("_mm", "512_"));
        let no_zeroupper = !content.contains("vzeroupper") && !content.contains(concat!("_mm", "256_zeroupper"));

        if is_sse_only && has_wider && no_zeroupper {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SIMD_VZEROUPPER".to_string(),
                    description: "Mixed SSE/AVX without vzeroupper (transition penalty)".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "SIMD pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("SSE instructions after AVX cause ~70 cycle penalty".to_string()),
                },
                file_path: path.to_path_buf(),
                line: Some(line_num + 1),
                snippet: Some(trimmed.to_string()),
                suggestion: Some(
                    concat!("Add _mm", "256_zeroupper() before SSE code or use all AVX").to_string(),
                ),
            });
            return true;
        }
        false
    }

    /// P1: Missing target_feature attribute
    fn check_missing_target_feature(
        has_avx512: bool, has_avx: bool, has_target_feature: bool,
        content: &str, path: &Path, analysis: &mut FileAnalysis,
    ) {
        if (has_avx512 || has_avx) && !has_target_feature && !content.contains("is_x86_feature_detected") {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SIMD_MISSING_TARGET".to_string(),
                    description: "SIMD intrinsics without #[target_feature] or runtime detection".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "SIMD pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("May crash on CPUs without required features".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Add #[target_feature(enable = \"avx2\")] or runtime detection".to_string()),
            });
        }
    }

    /// P1: Low vectorization ratio check
    fn check_vectorization_ratio(state: &SimdAnalysisState, path: &Path, analysis: &mut FileAnalysis) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let total_ops = state.scalar_ops + state.sse_ops + state.avx_ops + state.avx512_ops;
        if total_ops > 0 {
            let vectorized = state.sse_ops + state.avx_ops + state.avx512_ops;
            let ratio = vectorized as f32 / total_ops as f32;
            if ratio < 0.5 && state.scalar_ops > 5 {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SIMD_LOW_VECTORIZATION".to_string(),
                        description: format!("Low vectorization ratio: {:.0}% (threshold: 50%)", ratio * 100.0),
                        severity: DefectSeverity::P1Performance,
                        detection_method: "SIMD pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Scalar fallback reducing SIMD benefits".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: None,
                    snippet: Some(format!("scalar: {}, vectorized: {}", state.scalar_ops, vectorized)),
                    suggestion: Some("Check for alignment issues or loop trip count problems".to_string()),
                });
            }
        }
    }

    /// P2: Using narrower SIMD than available
    fn check_suboptimal_width(state: &SimdAnalysisState, has_avx: bool, path: &Path, analysis: &mut FileAnalysis) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if state.sse_ops > state.avx_ops && has_avx && state.avx_ops == 0 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SIMD_SUBOPTIMAL_WIDTH".to_string(),
                    description: "Using SSE when AVX is available".to_string(),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "SIMD pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("2x wider AVX could double throughput".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("SSE ops: {}, AVX ops: {}", state.sse_ops, state.avx_ops)),
                suggestion: Some("Consider upgrading to AVX2 for 256-bit operations".to_string()),
            });
        }
    }
}
