// PTX and CUDA memory pattern detection
// Included into mod.rs via include!()
//
// Split into submodules for file health (CB-040):
// - detection_state.rs: PtxAnalysisState struct impl (per-line pattern checks)
// - detection_barriers.rs: CudaSimdAnalyzer barrier detection and memory patterns
// - detection_ptx.rs: Comprehensive PTX bug detection and post-analysis

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

include!("detection_state.rs");
include!("detection_barriers.rs");
include!("detection_ptx.rs");
