// CudaSimdAnalyzer directory traversal, path filtering, and file dispatch

impl CudaSimdAnalyzer {
    /// Extensions this analyzer reads. Everything else the walk returns is
    /// dropped without opening the file.
    const ANALYZED_EXTENSIONS: &'static [&'static str] =
        &["cu", "cuh", "ptx", "rs", "wgsl", "c", "cpp", "h", "hpp"];

    fn analyze_directory(
        &self,
        path: &Path,
        defects: &mut Vec<DetectedDefect>,
        cuda_files: &mut usize,
        simd_files: &mut usize,
        wgpu_files: &mut usize,
        files_analyzed: &mut usize,
        barrier_safety: &mut BarrierSafetyResult,
        coalescing: &mut CoalescingResult,
    ) -> anyhow::Result<()> {
        // THE shared project walk, not a private one. This used to be a raw
        // `walkdir` with a hand-written directory blacklist and no gitignore
        // support at all, so on this repository it descended into the
        // gitignored `.claude/worktrees/` — 48 checkouts of pmat inside pmat.
        // Measured on this tree: 205,607 files analysed and 1,154 defects
        // before, 4,334 files and 25 defects after — a corpus that was ~48
        // copies of one project. `rust_project_score::gpu_simd_scorer` awards
        // its points from `result.defects`, so it was scoring the duplicates
        // too. (The Popper *score* itself did not move here — 66.5/C either
        // way — because its components come from project-level pattern checks
        // that saturate at one copy; the counts, and everything derived from
        // them, did.)
        for file_path in crate::services::file_discovery::project_files(path)? {
            let Some(ext) = file_path.extension() else {
                continue;
            };
            let ext = ext.to_string_lossy().to_lowercase();
            if !Self::ANALYZED_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }
            if let Ok(analysis) = self.analyze_file(&file_path) {
                *files_analyzed += 1;
                *cuda_files += analysis.cuda_files;
                *simd_files += analysis.simd_files;
                *wgpu_files += analysis.wgpu_files;
                defects.extend(analysis.defects);
                barrier_safety.total_barriers += analysis.barrier_safety.total_barriers;
                barrier_safety.safe_barriers += analysis.barrier_safety.safe_barriers;
                barrier_safety
                    .unsafe_barriers
                    .extend(analysis.barrier_safety.unsafe_barriers);
                coalescing.total_operations += analysis.coalescing.total_operations;
                coalescing.coalesced_operations += analysis.coalescing.coalesced_operations;
                coalescing
                    .problematic_accesses
                    .extend(analysis.coalescing.problematic_accesses);
            }
        }

        if barrier_safety.total_barriers > 0 {
            barrier_safety.safety_score =
                barrier_safety.safe_barriers as f64 / barrier_safety.total_barriers as f64;
        }
        if coalescing.total_operations > 0 {
            coalescing.efficiency =
                coalescing.coalesced_operations as f64 / coalescing.total_operations as f64;
        }

        Ok(())
    }

    fn analyze_file(&self, path: &Path) -> anyhow::Result<FileAnalysis> {
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let mut analysis = FileAnalysis::default();

        match ext.as_str() {
            "cu" | "cuh" | "ptx" => {
                analysis.cuda_files = 1;
                self.analyze_cuda_content(&content, path, &mut analysis);
            }
            "wgsl" => {
                analysis.wgpu_files = 1;
                self.analyze_wgpu_content(&content, path, &mut analysis);
            }
            "rs" => {
                if content.contains("std::arch::") || content.contains("core::arch::") {
                    analysis.simd_files = 1;
                    self.analyze_simd_content(&content, path, &mut analysis);
                }
                if content.contains("wgpu::")
                    || content.contains("@compute")
                    || content.contains("@workgroup_size")
                {
                    analysis.wgpu_files = 1;
                    self.analyze_wgpu_content(&content, path, &mut analysis);
                }
            }
            "c" | "cpp" | "h" | "hpp" => {
                // Use concat! to avoid self-matching during CB-021 compliance scanning
                if content.contains("immintrin.h")
                    || content.contains(concat!("_mm", "256_"))
                    || content.contains(concat!("_mm", "512_"))
                    || content.contains("arm_neon.h")
                {
                    analysis.simd_files = 1;
                    self.analyze_simd_content(&content, path, &mut analysis);
                }
            }
            _ => {}
        }

        Ok(analysis)
    }
}
