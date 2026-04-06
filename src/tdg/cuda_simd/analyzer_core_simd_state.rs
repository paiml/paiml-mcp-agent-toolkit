// SimdAnalysisState methods - tracking SAFETY comments, unsafe blocks, and instruction counts

impl SimdAnalysisState {
    /// Track SAFETY comments for unsafe blocks
    fn track_safety_comment(&mut self, trimmed: &str) {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        if trimmed.contains("// SAFETY:") || trimmed.contains("/// SAFETY:") {
            self.has_safety_comment = true;
        }
    }

    /// Track unsafe block entry/exit and report missing SAFETY comments
    fn track_unsafe_blocks(
        &mut self,
        line_num: usize,
        trimmed: &str,
        lines: &[&str],
        path: &Path,
        analysis: &mut FileAnalysis,
    ) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        debug_assert!(!lines.is_empty(), "lines must not be empty");
        if trimmed.contains("unsafe {") || trimmed.starts_with("unsafe ") {
            self.in_unsafe_block = true;
            self.unsafe_start_line = line_num + 1;
            self.has_safety_comment = false;
        }
        if self.in_unsafe_block && trimmed.contains('}') {
            let block_content = lines[self.unsafe_start_line - 1..=line_num].join("\n");
            if (block_content.contains("_mm") || block_content.contains("arch::"))
                && !self.has_safety_comment
            {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SIMD_UNSAFE_NO_SAFETY".to_string(),
                        description: "unsafe SIMD block without SAFETY comment".to_string(),
                        severity: DefectSeverity::P2Efficiency,
                        detection_method: "SIMD pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Undocumented safety invariants".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(self.unsafe_start_line),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some(
                        "Add // SAFETY: comment explaining alignment and bounds guarantees".to_string(),
                    ),
                });
            }
            self.in_unsafe_block = false;
        }
    }

    /// Count SIMD instruction types (AVX-512, AVX, SSE, scalar)
    fn count_instructions(&mut self, trimmed: &str, content: &str, analysis: &mut FileAnalysis) {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        debug_assert!(!content.is_empty(), "content must not be empty");
        // Use concat! to avoid self-matching during CB-021 compliance scanning
        if trimmed.contains(concat!("_mm", "512_")) {
            self.avx512_ops += 1;
            analysis.coalescing.total_operations += 1;
            analysis.coalescing.coalesced_operations += 1;
        } else if trimmed.contains(concat!("_mm", "256_")) {
            self.avx_ops += 1;
            analysis.coalescing.total_operations += 1;
            analysis.coalescing.coalesced_operations += 1;
        } else if trimmed.contains("_mm_")
            && !trimmed.contains(concat!("_mm", "256_"))
            && !trimmed.contains(concat!("_mm", "512_"))
        {
            self.sse_ops += 1;
            analysis.coalescing.total_operations += 1;
            analysis.coalescing.coalesced_operations += 1;
        }

        // Scalar operations in SIMD context
        if (trimmed.contains(".iter()") || trimmed.contains("for "))
            && (content.contains(concat!("_mm", "256_")) || content.contains(concat!("_mm", "512_")))
            && !trimmed.contains("chunks")
        {
            self.scalar_ops += 1;
        }

        // Detect unaligned loads for coalescing
        if trimmed.contains(concat!("_mm", "256_loadu_")) || trimmed.contains(concat!("_mm", "512_loadu_")) {
            analysis.coalescing.total_operations += 1;
            analysis.coalescing.coalesced_operations += 1;
        }
    }
}
