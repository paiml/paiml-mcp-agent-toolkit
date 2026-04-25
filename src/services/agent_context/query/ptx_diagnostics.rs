#![cfg_attr(coverage_nightly, coverage(off))]
//! PTX Diagnostics
//!
//! Quality analysis for PTX code: register pressure, branch density,
//! shared memory usage, barrier counts. Reuses CB-060 detectors.

use crate::services::agent_context::AgentContextIndex;
use regex::Regex;

/// Severity level for PTX diagnostics
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PtxSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for PtxSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtxSeverity::Info => write!(f, "info"),
            PtxSeverity::Warning => write!(f, "warning"),
            PtxSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// A single diagnostic finding for a PTX function
#[derive(Debug, Clone)]
pub struct PtxDiagnostic {
    pub severity: PtxSeverity,
    pub category: String,
    pub message: String,
    pub value: f32,
}

/// Diagnostics for a single function
#[derive(Debug, Clone)]
pub struct PtxFunctionDiagnostics {
    pub function_name: String,
    pub file_path: String,
    pub project: String,
    pub register_count: u32,
    pub branch_density: f32,
    pub shared_memory_bytes: u32,
    pub barrier_count: u32,
    pub diagnostics: Vec<PtxDiagnostic>,
}

/// Result of PTX diagnostics analysis across the workspace
pub struct PtxDiagnosticResult {
    pub functions: Vec<PtxFunctionDiagnostics>,
    pub total_critical: usize,
    pub total_warning: usize,
    pub total_info: usize,
}

/// Count distinct register names in PTX source (e.g., %r1, %f2, %p0)
fn count_registers(source: &str) -> u32 {
    let re = Regex::new(r"%[rfpb]\d+").expect("static regex must compile");
    let mut seen = std::collections::HashSet::new();
    for cap in re.find_iter(source) {
        seen.insert(cap.as_str());
    }
    seen.len() as u32
}

/// Compute branch density: branches per instruction ratio
fn compute_branch_density(source: &str) -> f32 {
    let mut instructions = 0u32;
    let mut branches = 0u32;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('.') {
            continue;
        }
        instructions += 1;
        if trimmed.starts_with("bra ")
            || trimmed.starts_with("@")
            || trimmed.contains("bra.uni")
            || trimmed.contains("if ")
            || trimmed.contains("else")
            || trimmed.contains("match ")
        {
            branches += 1;
        }
    }
    if instructions == 0 {
        0.0
    } else {
        branches as f32 / instructions as f32
    }
}

/// Count total shared memory bytes declared
fn count_shared_memory(source: &str) -> u32 {
    let re = Regex::new(r"\.shared\s+\.\w+\s+\w+\[(\d+)\]").expect("static regex must compile");
    let mut total = 0u32;
    for cap in re.captures_iter(source) {
        if let Some(size) = cap.get(1) {
            total += size.as_str().parse::<u32>().unwrap_or(0);
        }
    }
    // Also check __shared__ in CUDA source
    if source.contains("__shared__") {
        total = total.max(1); // At least mark as using shared memory
    }
    total
}

/// Count barrier/sync points
fn count_barriers(source: &str) -> u32 {
    let mut count = 0u32;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains("bar.sync")
            || trimmed.contains("__syncthreads")
            || trimmed.contains("barrier::")
            || trimmed.contains("membar.")
        {
            count += 1;
        }
    }
    count
}

/// Check if a function contains PTX-relevant code
fn is_ptx_relevant(source: &str, file_path: &str) -> bool {
    file_path.ends_with(".ptx")
        || file_path.ends_with(".cu")
        || file_path.ends_with(".cuh")
        || source.contains(".version ")
        || source.contains(".target sm_")
        || source.contains("__global__")
        || source.contains("__device__")
        || source.contains("__shared__")
        || source.contains("asm!(")
        || source.contains("ptx")
        || source.contains("cuda")
        || source.contains("detect_ptx")
        || source.contains("barrier_divergence")
        || source.contains("shared_memory")
}

/// Collect metric-based diagnostics (register pressure, branch density, shared memory, barriers)
fn collect_metric_diagnostics(
    register_count: u32,
    branch_density: f32,
    shared_memory_bytes: u32,
    barrier_count: u32,
) -> Vec<PtxDiagnostic> {
    let mut diags = Vec::new();
    collect_register_diag(&mut diags, register_count);
    collect_branch_diag(&mut diags, branch_density);
    collect_shmem_diag(&mut diags, shared_memory_bytes);
    collect_barrier_diag(&mut diags, barrier_count);
    diags
}

fn collect_register_diag(diags: &mut Vec<PtxDiagnostic>, count: u32) {
    if count > 64 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Critical,
            category: "register_pressure".into(),
            message: format!("{} registers (>64 risks spilling to local memory)", count),
            value: count as f32,
        });
    } else if count > 32 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Warning,
            category: "register_pressure".into(),
            message: format!("{} registers (>32 may reduce occupancy)", count),
            value: count as f32,
        });
    }
}

fn collect_branch_diag(diags: &mut Vec<PtxDiagnostic>, density: f32) {
    if density > 0.3 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Critical,
            category: "branch_density".into(),
            message: format!(
                "{:.0}% branch density (high divergence risk)",
                density * 100.0
            ),
            value: density,
        });
    } else if density > 0.15 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Warning,
            category: "branch_density".into(),
            message: format!(
                "{:.0}% branch density (moderate divergence risk)",
                density * 100.0
            ),
            value: density,
        });
    }
}

fn collect_shmem_diag(diags: &mut Vec<PtxDiagnostic>, bytes: u32) {
    if bytes > 48_000 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Critical,
            category: "shared_memory".into(),
            message: format!("{}B shared memory (exceeds 48KB limit)", bytes),
            value: bytes as f32,
        });
    } else if bytes > 0 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Info,
            category: "shared_memory".into(),
            message: format!("{}B shared memory", bytes),
            value: bytes as f32,
        });
    }
}

fn collect_barrier_diag(diags: &mut Vec<PtxDiagnostic>, count: u32) {
    if count > 5 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Warning,
            category: "barriers".into(),
            message: format!("{} sync points (complex synchronization)", count),
            value: count as f32,
        });
    } else if count > 0 {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Info,
            category: "barriers".into(),
            message: format!("{} sync point(s)", count),
            value: count as f32,
        });
    }
}

/// Collect CB-060 compliance diagnostics from existing detectors
fn collect_cb060_diagnostics(diags: &mut Vec<PtxDiagnostic>, source: &str) {
    use crate::cli::handlers::comply_handlers::comply_cb_detect::{
        detect_ptx_barrier_divergence_in_str, detect_shared_memory_unbounded_in_str,
        detect_tiled_kernel_no_bounds_in_str,
    };
    for (line, _sev, msg) in &detect_ptx_barrier_divergence_in_str(source) {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Critical,
            category: "CB-060-A".into(),
            message: format!("line {}: {}", line, msg),
            value: 0.0,
        });
    }
    for (line, _sev, msg) in &detect_shared_memory_unbounded_in_str(source) {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Warning,
            category: "CB-060-B".into(),
            message: format!("line {}: {}", line, msg),
            value: 0.0,
        });
    }
    for (line, _sev, msg) in &detect_tiled_kernel_no_bounds_in_str(source) {
        diags.push(PtxDiagnostic {
            severity: PtxSeverity::Warning,
            category: "CB-060-C".into(),
            message: format!("line {}: {}", line, msg),
            value: 0.0,
        });
    }
}

/// Run PTX diagnostics on the merged index
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn run_ptx_diagnostics(index: &AgentContextIndex) -> PtxDiagnosticResult {
    let mut functions = Vec::new();
    let mut total_critical = 0;
    let mut total_warning = 0;
    let mut total_info = 0;

    for func in index.all_functions() {
        if !is_ptx_relevant(&func.source, &func.file_path) {
            continue;
        }

        let register_count = count_registers(&func.source);
        let branch_density = compute_branch_density(&func.source);
        let shared_memory_bytes = count_shared_memory(&func.source);
        let barrier_count = count_barriers(&func.source);

        let mut diagnostics = collect_metric_diagnostics(
            register_count,
            branch_density,
            shared_memory_bytes,
            barrier_count,
        );
        collect_cb060_diagnostics(&mut diagnostics, &func.source);

        if diagnostics.is_empty() {
            continue;
        }

        for d in &diagnostics {
            match d.severity {
                PtxSeverity::Critical => total_critical += 1,
                PtxSeverity::Warning => total_warning += 1,
                PtxSeverity::Info => total_info += 1,
            }
        }

        functions.push(PtxFunctionDiagnostics {
            function_name: func.function_name.clone(),
            file_path: func.file_path.clone(),
            project: func
                .file_path
                .split('/')
                .next()
                .unwrap_or("local")
                .to_string(),
            register_count,
            branch_density,
            shared_memory_bytes,
            barrier_count,
            diagnostics,
        });
    }

    functions.sort_by(|a, b| {
        let max_a = a.diagnostics.iter().map(|d| &d.severity).max();
        let max_b = b.diagnostics.iter().map(|d| &d.severity).max();
        max_b.cmp(&max_a)
    });

    PtxDiagnosticResult {
        functions,
        total_critical,
        total_warning,
        total_info,
    }
}

/// Format PTX diagnostics as human-readable text
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_ptx_diagnostics_text(result: &PtxDiagnosticResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\x1b[1;4mPTX Diagnostics\x1b[0m ({} functions, {} critical, {} warning, {} info)\n\n",
        result.functions.len(),
        result.total_critical,
        result.total_warning,
        result.total_info
    ));

    if result.functions.is_empty() {
        out.push_str("  No PTX-related functions with diagnostics found.\n");
        return out;
    }

    for func in &result.functions {
        out.push_str(&format!(
            "  \x1b[1;36m{}\x1b[0m \x1b[2m{}\x1b[0m [{}]\n",
            func.function_name, func.file_path, func.project
        ));
        out.push_str(&format!(
            "    regs:{} branch:{:.0}% shmem:{}B barriers:{}\n",
            func.register_count,
            func.branch_density * 100.0,
            func.shared_memory_bytes,
            func.barrier_count
        ));
        for d in &func.diagnostics {
            let color = match d.severity {
                PtxSeverity::Critical => "\x1b[1;31m",
                PtxSeverity::Warning => "\x1b[1;33m",
                PtxSeverity::Info => "\x1b[2m",
            };
            out.push_str(&format!(
                "    {color}{:>8}\x1b[0m [{}] {}\n",
                d.severity, d.category, d.message
            ));
        }
        out.push('\n');
    }

    out
}

/// Format PTX diagnostics as JSON
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_ptx_diagnostics_json(result: &PtxDiagnosticResult) -> String {
    let functions: Vec<serde_json::Value> = result
        .functions
        .iter()
        .map(|f| {
            let diags: Vec<serde_json::Value> = f
                .diagnostics
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "severity": d.severity.to_string(),
                        "category": d.category,
                        "message": d.message,
                        "value": d.value,
                    })
                })
                .collect();
            serde_json::json!({
                "function_name": f.function_name,
                "file_path": f.file_path,
                "project": f.project,
                "register_count": f.register_count,
                "branch_density": f.branch_density,
                "shared_memory_bytes": f.shared_memory_bytes,
                "barrier_count": f.barrier_count,
                "diagnostics": diags,
            })
        })
        .collect();
    serde_json::json!({
        "ptx_diagnostics": {
            "functions": functions,
            "total_critical": result.total_critical,
            "total_warning": result.total_warning,
            "total_info": result.total_info,
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fdiag(sev: PtxSeverity, cat: &str, msg: &str, val: f32) -> PtxDiagnostic {
        PtxDiagnostic {
            severity: sev,
            category: cat.into(),
            message: msg.into(),
            value: val,
        }
    }

    fn func_diag(name: &str, sev: PtxSeverity) -> PtxFunctionDiagnostics {
        PtxFunctionDiagnostics {
            function_name: name.into(),
            file_path: format!("kernels/{name}.cu"),
            project: "demo".into(),
            register_count: 16,
            branch_density: 0.1,
            shared_memory_bytes: 0,
            barrier_count: 0,
            diagnostics: vec![fdiag(sev, "test", "msg", 0.0)],
        }
    }

    // ── PtxSeverity Display + Ord ───────────────────────────────────────────

    #[test]
    fn test_ptx_severity_display_arms() {
        assert_eq!(PtxSeverity::Info.to_string(), "info");
        assert_eq!(PtxSeverity::Warning.to_string(), "warning");
        assert_eq!(PtxSeverity::Critical.to_string(), "critical");
    }

    #[test]
    fn test_ptx_severity_ord_critical_greatest() {
        assert!(PtxSeverity::Critical > PtxSeverity::Warning);
        assert!(PtxSeverity::Warning > PtxSeverity::Info);
    }

    // ── count_registers ─────────────────────────────────────────────────────

    #[test]
    fn test_count_registers_distinct_regs() {
        // %r1, %r2, %f1, %p0 — 4 distinct
        let src =
            "mov.u32 %r1, 0;\nadd.u32 %r2, %r1, 1;\nmov.f32 %f1, 0.0;\nsetp.eq.s32 %p0, %r1, 0;";
        assert_eq!(count_registers(src), 4);
    }

    #[test]
    fn test_count_registers_dedupes_same_name() {
        // %r1 appears 3 times → counts once
        let src = "%r1 %r1 %r1 %r2";
        assert_eq!(count_registers(src), 2);
    }

    #[test]
    fn test_count_registers_empty_source_zero() {
        assert_eq!(count_registers(""), 0);
    }

    #[test]
    fn test_count_registers_no_register_pattern_zero() {
        assert_eq!(count_registers("just some text"), 0);
    }

    #[test]
    fn test_count_registers_b_register_kind() {
        // The regex is %[rfpb]\d+ — `%b3` should match
        let src = "%b3";
        assert_eq!(count_registers(src), 1);
    }

    // ── compute_branch_density ──────────────────────────────────────────────

    #[test]
    fn test_compute_branch_density_no_instructions_zero() {
        // Only blank/comment/directive lines → 0 instructions → 0.0 density
        let src = "// comment\n.version 7.0\n\n.target sm_70\n";
        assert!((compute_branch_density(src) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_branch_density_no_branches_zero() {
        let src = "mov.u32 %r1, 0;\nadd.u32 %r2, %r1, 1;\nret;";
        assert!((compute_branch_density(src) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_branch_density_bra_counted() {
        // 2 instructions, 1 of which is `bra` → 0.5 density
        let src = "mov.u32 %r1, 0;\nbra LBB1_5;";
        assert!((compute_branch_density(src) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_branch_density_at_predicate_counted() {
        // @-prefixed lines count as branches
        let src = "mov.u32 %r1, 0;\n@%p0 bra LBB1_2;";
        assert!((compute_branch_density(src) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_branch_density_if_else_counted() {
        let src = "mov.u32 %r1, 0;\nif (cond) {\nelse {";
        // 3 instructions, 2 branches (the if line and the else line)
        let d = compute_branch_density(src);
        assert!(d > 0.6 && d < 0.7, "expected ~0.667, got {d}");
    }

    #[test]
    fn test_compute_branch_density_match_counted() {
        let src = "mov.u32 %r1, 0;\nmatch x {";
        let d = compute_branch_density(src);
        assert!((d - 0.5).abs() < 1e-6);
    }

    // ── count_shared_memory ─────────────────────────────────────────────────

    #[test]
    fn test_count_shared_memory_ptx_directive() {
        // `.shared .u32 buf[256]` → 256 bytes
        let src = ".shared .u32 buf[256]";
        assert_eq!(count_shared_memory(src), 256);
    }

    #[test]
    fn test_count_shared_memory_multiple_arrays_summed() {
        let src = ".shared .u32 a[64]\n.shared .f32 b[128]";
        assert_eq!(count_shared_memory(src), 64 + 128);
    }

    #[test]
    fn test_count_shared_memory_cuda_marker_alone() {
        // No `.shared` directive but CUDA `__shared__` → at least 1
        let src = "__shared__ int s[64];";
        assert_eq!(count_shared_memory(src), 1);
    }

    #[test]
    fn test_count_shared_memory_directive_keeps_max_with_marker() {
        // PTX directive sums to 64; presence of __shared__ uses .max(1) → still 64
        let src = ".shared .u32 a[64]\n__shared__ stuff";
        assert_eq!(count_shared_memory(src), 64);
    }

    #[test]
    fn test_count_shared_memory_no_shared_zero() {
        assert_eq!(count_shared_memory("nothing here"), 0);
    }

    // ── count_barriers ──────────────────────────────────────────────────────

    #[test]
    fn test_count_barriers_bar_sync() {
        assert_eq!(count_barriers("bar.sync 0;"), 1);
    }

    #[test]
    fn test_count_barriers_syncthreads() {
        assert_eq!(count_barriers("__syncthreads();"), 1);
    }

    #[test]
    fn test_count_barriers_barrier_double_colon() {
        assert_eq!(count_barriers("barrier::arrive_and_wait();"), 1);
    }

    #[test]
    fn test_count_barriers_membar() {
        assert_eq!(count_barriers("membar.gl;"), 1);
    }

    #[test]
    fn test_count_barriers_multiline_summed() {
        let src = "bar.sync 0;\n__syncthreads();\nmembar.gl;";
        assert_eq!(count_barriers(src), 3);
    }

    #[test]
    fn test_count_barriers_no_barriers_zero() {
        assert_eq!(count_barriers("just regular code"), 0);
    }

    // ── is_ptx_relevant ─────────────────────────────────────────────────────

    #[test]
    fn test_is_ptx_relevant_ptx_extension() {
        assert!(is_ptx_relevant("", "kernel.ptx"));
    }

    #[test]
    fn test_is_ptx_relevant_cu_extension() {
        assert!(is_ptx_relevant("", "kernel.cu"));
    }

    #[test]
    fn test_is_ptx_relevant_cuh_extension() {
        assert!(is_ptx_relevant("", "kernel.cuh"));
    }

    #[test]
    fn test_is_ptx_relevant_version_directive() {
        assert!(is_ptx_relevant(".version 7.0", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_target_sm() {
        assert!(is_ptx_relevant(".target sm_70", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_global_marker() {
        assert!(is_ptx_relevant("__global__ void foo()", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_device_marker() {
        assert!(is_ptx_relevant("__device__ int x;", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_shared_marker() {
        assert!(is_ptx_relevant("__shared__ int x;", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_asm_macro() {
        assert!(is_ptx_relevant("asm!(\"mov\")", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_substring_keywords() {
        assert!(is_ptx_relevant("ptx flow", "any.rs"));
        assert!(is_ptx_relevant("cuda kernel", "any.rs"));
        assert!(is_ptx_relevant("detect_ptx", "any.rs"));
        assert!(is_ptx_relevant("barrier_divergence", "any.rs"));
        assert!(is_ptx_relevant("shared_memory", "any.rs"));
    }

    #[test]
    fn test_is_ptx_relevant_unrelated_returns_false() {
        assert!(!is_ptx_relevant(
            "fn add(x: i32) -> i32 { x }",
            "src/lib.rs"
        ));
    }

    // ── collect_register_diag ───────────────────────────────────────────────

    #[test]
    fn test_collect_register_diag_above_64_critical() {
        let mut d = Vec::new();
        collect_register_diag(&mut d, 80);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Critical);
        assert!(d[0].message.contains("80 registers"));
    }

    #[test]
    fn test_collect_register_diag_at_64_no_critical() {
        // boundary: > 64 critical, == 64 falls through to else-if (> 32 → warning)
        let mut d = Vec::new();
        collect_register_diag(&mut d, 64);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Warning);
    }

    #[test]
    fn test_collect_register_diag_above_32_warning() {
        let mut d = Vec::new();
        collect_register_diag(&mut d, 50);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Warning);
    }

    #[test]
    fn test_collect_register_diag_at_or_below_32_no_diag() {
        let mut d = Vec::new();
        collect_register_diag(&mut d, 32);
        assert!(d.is_empty());
        let mut d2 = Vec::new();
        collect_register_diag(&mut d2, 0);
        assert!(d2.is_empty());
    }

    // ── collect_branch_diag ─────────────────────────────────────────────────

    #[test]
    fn test_collect_branch_diag_above_03_critical() {
        let mut d = Vec::new();
        collect_branch_diag(&mut d, 0.5);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Critical);
        assert!(d[0].message.contains("50%"));
    }

    #[test]
    fn test_collect_branch_diag_above_015_warning() {
        let mut d = Vec::new();
        collect_branch_diag(&mut d, 0.20);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Warning);
    }

    #[test]
    fn test_collect_branch_diag_at_or_below_015_no_diag() {
        let mut d = Vec::new();
        collect_branch_diag(&mut d, 0.10);
        assert!(d.is_empty());
        let mut d2 = Vec::new();
        collect_branch_diag(&mut d2, 0.0);
        assert!(d2.is_empty());
    }

    // ── collect_shmem_diag ──────────────────────────────────────────────────

    #[test]
    fn test_collect_shmem_diag_above_48k_critical() {
        let mut d = Vec::new();
        collect_shmem_diag(&mut d, 50_000);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Critical);
    }

    #[test]
    fn test_collect_shmem_diag_above_zero_info() {
        let mut d = Vec::new();
        collect_shmem_diag(&mut d, 1024);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Info);
    }

    #[test]
    fn test_collect_shmem_diag_zero_no_diag() {
        let mut d = Vec::new();
        collect_shmem_diag(&mut d, 0);
        assert!(d.is_empty());
    }

    // ── collect_barrier_diag ────────────────────────────────────────────────

    #[test]
    fn test_collect_barrier_diag_above_5_warning() {
        let mut d = Vec::new();
        collect_barrier_diag(&mut d, 10);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Warning);
    }

    #[test]
    fn test_collect_barrier_diag_above_zero_info() {
        let mut d = Vec::new();
        collect_barrier_diag(&mut d, 2);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, PtxSeverity::Info);
    }

    #[test]
    fn test_collect_barrier_diag_zero_no_diag() {
        let mut d = Vec::new();
        collect_barrier_diag(&mut d, 0);
        assert!(d.is_empty());
    }

    // ── collect_metric_diagnostics (orchestrator) ───────────────────────────

    #[test]
    fn test_collect_metric_diagnostics_aggregates_all_kinds() {
        // Critical regs + critical branch + critical shmem + warning barriers
        let d = collect_metric_diagnostics(80, 0.5, 60_000, 10);
        assert_eq!(d.len(), 4);
    }

    #[test]
    fn test_collect_metric_diagnostics_empty_when_all_clean() {
        let d = collect_metric_diagnostics(10, 0.05, 0, 0);
        assert!(d.is_empty());
    }

    // ── format_ptx_diagnostics_text ─────────────────────────────────────────

    #[test]
    fn test_format_text_empty_result_emits_no_functions_message() {
        let r = PtxDiagnosticResult {
            functions: vec![],
            total_critical: 0,
            total_warning: 0,
            total_info: 0,
        };
        let s = format_ptx_diagnostics_text(&r);
        assert!(s.contains("PTX Diagnostics"));
        assert!(s.contains("No PTX-related functions"));
    }

    #[test]
    fn test_format_text_with_functions_includes_metrics() {
        let r = PtxDiagnosticResult {
            functions: vec![func_diag("kernel_a", PtxSeverity::Critical)],
            total_critical: 1,
            total_warning: 0,
            total_info: 0,
        };
        let s = format_ptx_diagnostics_text(&r);
        assert!(s.contains("kernel_a"));
        assert!(s.contains("regs:16"));
        assert!(s.contains("critical"));
    }

    #[test]
    fn test_format_text_severity_color_arms() {
        // All three severity arms hit different ANSI color codes
        let r = PtxDiagnosticResult {
            functions: vec![
                func_diag("a", PtxSeverity::Critical),
                func_diag("b", PtxSeverity::Warning),
                func_diag("c", PtxSeverity::Info),
            ],
            total_critical: 1,
            total_warning: 1,
            total_info: 1,
        };
        let s = format_ptx_diagnostics_text(&r);
        assert!(s.contains("\x1b[1;31m")); // critical
        assert!(s.contains("\x1b[1;33m")); // warning
        assert!(s.contains("\x1b[2m")); // info
    }

    // ── format_ptx_diagnostics_json ─────────────────────────────────────────

    #[test]
    fn test_format_json_empty_result_valid_json() {
        let r = PtxDiagnosticResult {
            functions: vec![],
            total_critical: 0,
            total_warning: 0,
            total_info: 0,
        };
        let s = format_ptx_diagnostics_json(&r);
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(parsed.get("ptx_diagnostics").is_some());
        let inner = &parsed["ptx_diagnostics"];
        assert_eq!(inner["functions"].as_array().unwrap().len(), 0);
        assert_eq!(inner["total_critical"], 0);
    }

    #[test]
    fn test_format_json_includes_function_fields() {
        let r = PtxDiagnosticResult {
            functions: vec![func_diag("kernel_x", PtxSeverity::Warning)],
            total_critical: 0,
            total_warning: 1,
            total_info: 0,
        };
        let s = format_ptx_diagnostics_json(&r);
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
        let funcs = parsed["ptx_diagnostics"]["functions"].as_array().unwrap();
        assert_eq!(funcs.len(), 1);
        let f = &funcs[0];
        assert_eq!(f["function_name"], "kernel_x");
        assert_eq!(f["register_count"], 16);
        assert!(f["diagnostics"].is_array());
        assert_eq!(f["diagnostics"][0]["severity"], "warning");
    }

    #[test]
    fn test_format_json_totals_propagate() {
        let r = PtxDiagnosticResult {
            functions: vec![],
            total_critical: 5,
            total_warning: 3,
            total_info: 2,
        };
        let s = format_ptx_diagnostics_json(&r);
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
        let inner = &parsed["ptx_diagnostics"];
        assert_eq!(inner["total_critical"], 5);
        assert_eq!(inner["total_warning"], 3);
        assert_eq!(inner["total_info"], 2);
    }
}
