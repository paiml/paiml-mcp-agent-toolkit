// CUDA/SIMD analyzer core - included into mod.rs via include!()

/// Mutable state for SIMD per-line pattern analysis
struct SimdAnalysisState {
    scalar_ops: u32,
    sse_ops: u32,
    avx_ops: u32,
    avx512_ops: u32,
    in_unsafe_block: bool,
    unsafe_start_line: usize,
    has_safety_comment: bool,
}

impl SimdAnalysisState {
    fn new() -> Self {
        Self {
            scalar_ops: 0,
            sse_ops: 0,
            avx_ops: 0,
            avx512_ops: 0,
            in_unsafe_block: false,
            unsafe_start_line: 0,
            has_safety_comment: false,
        }
    }
}

// SIMD state tracking: safety comments, unsafe blocks, instruction counting
include!("analyzer_core_simd_state.rs");
// SIMD/CUDA/WGPU content analysis and defect detection checks
include!("analyzer_core_simd_checks.rs");
// Directory traversal, path filtering, and file dispatch
include!("analyzer_core_traversal.rs");

impl CudaSimdAnalyzer {
    /// Create new analyzer with default configuration
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            taxonomy: DefectTaxonomy::with_tauranta_patterns(),
            config: CudaSimdConfig::new(),
        }
    }

    /// Create analyzer with custom configuration
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_config(config: CudaSimdConfig) -> Self {
        Self {
            taxonomy: DefectTaxonomy::with_tauranta_patterns(),
            config,
        }
    }

    /// Analyze a file or directory
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn analyze(&self, path: &Path) -> anyhow::Result<CudaSimdTdgResult> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let mut defects = Vec::new();
        let mut cuda_files = 0;
        let mut simd_files = 0;
        let mut wgpu_files = 0;
        let mut files_analyzed = 0;

        let mut barrier_safety = BarrierSafetyResult::default();
        let mut coalescing = CoalescingResult::default();
        let tile_dimensions = TileDimensionResult {
            valid: true,
            tile_k: None,
            tile_kv: None,
            head_dim: None,
            shared_memory_required: None,
            shared_memory_available: Some(self.config.shared_memory_limit),
            issues: Vec::new(),
        };

        if path.is_file() {
            let analysis = self.analyze_file(path)?;
            files_analyzed = 1;
            cuda_files = analysis.cuda_files;
            simd_files = analysis.simd_files;
            wgpu_files = analysis.wgpu_files;
            defects = analysis.defects;
            barrier_safety = analysis.barrier_safety;
            coalescing = analysis.coalescing;
        } else if path.is_dir() {
            self.analyze_directory(
                path,
                &mut defects,
                &mut cuda_files,
                &mut simd_files,
                &mut wgpu_files,
                &mut files_analyzed,
                &mut barrier_safety,
                &mut coalescing,
            )?;
        }

        let score = self.calculate_score(&defects, &barrier_safety, &coalescing, path);
        let kaizen = self.build_kaizen_metrics(&defects);

        Ok(CudaSimdTdgResult {
            path: path.to_path_buf(),
            score,
            defects,
            barrier_safety,
            coalescing,
            tile_dimensions,
            kaizen,
            timestamp: chrono::Utc::now().to_rfc3339(),
            files_analyzed,
            cuda_files,
            simd_files,
            wgpu_files,
        })
    }
}
