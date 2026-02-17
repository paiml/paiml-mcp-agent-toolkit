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
