// WGPU/WGSL pattern detection: extract_ptx_dest_register, detect_wgpu_memory_patterns
include!("scoring_wgpu_detection.rs");

// Known pattern detection: flash attention, tensor core, Rust project quality patterns
include!("scoring_pattern_detection.rs");

// Score calculation: falsifiability, reproducibility, statistical rigor, kaizen, quality gate
include!("scoring_calculation.rs");
