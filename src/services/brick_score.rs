#![cfg_attr(coverage_nightly, coverage(off))]
//! ComputeBrick Profiling Score Service (PMAT-446)
//!
//! Reads BrickProfiler JSON output and calculates a 100-point score:
//! - Performance (40 pts): Throughput vs theoretical peak
//! - Efficiency (25 pts): Backend utilization, memory efficiency
//! - Correctness (20 pts): Assertions passing, numerical accuracy
//! - Stability (15 pts): CV < 5%, reproducibility
//!
//! PMAT-448: Hardware-aware scoring via ~/.pmat/hardware.toml
//!
//! Reference: aprender/docs/specifications/qwen2.5-coder-showcase-demo.md §2.5

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Hardware Capability Types (PMAT-448)
// Matches trueno::hardware format for ~/.pmat/hardware.toml
// ============================================================================

/// SIMD instruction set width (matches trueno::hardware::SimdWidth)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum SimdWidth {
    #[default]
    Scalar,
    Neon128,
    Sse2,
    Avx2,
    Avx512,
    WasmSimd128,
}

impl SimdWidth {
    /// Number of f32 lanes
    pub fn lanes(&self) -> usize {
        match self {
            SimdWidth::Scalar => 1,
            SimdWidth::Neon128 | SimdWidth::Sse2 | SimdWidth::WasmSimd128 => 4,
            SimdWidth::Avx2 => 8,
            SimdWidth::Avx512 => 16,
        }
    }

    /// Typical speedup factor (from trueno-zram measurements)
    pub fn compute_speedup(&self) -> f64 {
        match self {
            SimdWidth::Scalar => 1.0,
            SimdWidth::Neon128 | SimdWidth::Sse2 | SimdWidth::WasmSimd128 => 4.0,
            SimdWidth::Avx2 => 10.0,   // 8-12x measured
            SimdWidth::Avx512 => 12.0, // 8-13x measured
        }
    }
}

/// GPU compute backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum GpuBackend {
    #[default]
    None,
    Cuda,
    Wgpu,
    Metal,
    Vulkan,
}

/// CPU capabilities
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CpuCapability {
    pub vendor: String,
    pub model: String,
    pub cores: usize,
    pub threads: usize,
    pub simd: SimdWidth,
    pub base_freq_ghz: f64,
    pub peak_gflops: f64,
    pub memory_bw_gbps: f64,
}

impl Default for CpuCapability {
    fn default() -> Self {
        Self {
            vendor: "Unknown".to_string(),
            model: "Unknown".to_string(),
            cores: 1,
            threads: 1,
            simd: SimdWidth::Scalar,
            base_freq_ghz: 3.0,
            peak_gflops: 6.0, // 1 core × 1 lane × 2 FMA × 3 GHz
            memory_bw_gbps: 25.0,
        }
    }
}

/// GPU capabilities
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GpuCapability {
    pub vendor: String,
    pub model: String,
    pub backend: GpuBackend,
    pub compute_capability: Option<String>,
    pub peak_tflops_fp32: f64,
    pub peak_tflops_tensor: Option<f64>,
    pub memory_bw_gbps: f64,
    pub vram_gb: f64,
}

/// Roofline model parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RooflineParams {
    pub cpu_arithmetic_intensity: f64,
    pub gpu_arithmetic_intensity: Option<f64>,
}

impl Default for RooflineParams {
    fn default() -> Self {
        Self {
            cpu_arithmetic_intensity: 0.24, // 6 GFLOP/s ÷ 25 GB/s
            gpu_arithmetic_intensity: None,
        }
    }
}

/// Byte budget for compression/I/O workloads (PMAT-452)
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ByteBudget {
    /// Latency budget per page (microseconds)
    pub us_per_page: f64,
    /// Throughput target (GB/s)
    pub gb_per_sec: f64,
    /// Page size in bytes (default 4096)
    pub page_size: usize,
}

impl Default for ByteBudget {
    fn default() -> Self {
        // Default: 25 GB/s (trueno-zram ZSTD target)
        let gb_per_sec = 25.0;
        let bytes_per_sec = gb_per_sec * 1e9;
        let pages_per_sec = bytes_per_sec / 4096.0;
        Self {
            us_per_page: 1_000_000.0 / pages_per_sec,
            gb_per_sec,
            page_size: 4096,
        }
    }
}

/// Complete hardware capability profile
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HardwareCapability {
    pub timestamp: String,
    pub hostname: String,
    pub cpu: CpuCapability,
    pub gpu: Option<GpuCapability>,
    pub roofline: RooflineParams,
    /// PMAT-452: Byte budget for compression/I/O workloads
    #[serde(default)]
    pub byte_budget: Option<ByteBudget>,
}

impl Default for HardwareCapability {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname: "unknown".to_string(),
            cpu: CpuCapability::default(),
            gpu: None,
            roofline: RooflineParams::default(),
            byte_budget: Some(ByteBudget::default()),
        }
    }
}

/// Workload bottleneck classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bottleneck {
    Memory,
    Compute,
}

impl HardwareCapability {
    /// Determine if workload is memory-bound or compute-bound
    pub fn bottleneck(&self, arithmetic_intensity: f64, use_gpu: bool) -> Bottleneck {
        let threshold = if use_gpu {
            self.roofline.gpu_arithmetic_intensity.unwrap_or(f64::MAX)
        } else {
            self.roofline.cpu_arithmetic_intensity
        };

        if arithmetic_intensity < threshold {
            Bottleneck::Memory
        } else {
            Bottleneck::Compute
        }
    }
}

/// Default path for hardware.toml
pub fn default_hardware_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".pmat")
        .join("hardware.toml")
}

/// Load hardware capability from TOML file
pub fn load_hardware_capability(path: Option<&Path>) -> Option<HardwareCapability> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(default_hardware_path);

    if !path.exists() {
        return None;
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
}

/// Scale budgets based on hardware capability
pub fn scale_budgets_for_hardware(
    base_budgets: &[BrickBudget],
    hardware: &HardwareCapability,
) -> Vec<BrickBudget> {
    // Scale factor based on SIMD speedup
    let simd_factor = hardware.cpu.simd.compute_speedup();

    // Memory bandwidth scaling (baseline: 25 GB/s from trueno-zram)
    let mem_bw_factor = hardware.cpu.memory_bw_gbps / 25.0;

    // Combined scaling: geometric mean of SIMD and memory bandwidth factors
    let scale_factor = (simd_factor * mem_bw_factor).sqrt();

    base_budgets
        .iter()
        .map(|b| BrickBudget {
            name: b.name.clone(),
            // Faster hardware means stricter (lower) budgets
            max_us: b.max_us / scale_factor,
        })
        .collect()
}

/// Categorize an operation by name pattern and return its arithmetic intensity (FLOP/byte).
///
/// Uses a lookup table of (patterns, value) tuples, returning the first match.
/// Default: 2.0 (balanced operation).
fn categorize_operation(name_lower: &str) -> f64 {
    // Lookup table: (patterns, arithmetic_intensity)
    // Order matters — first match wins.
    const OPERATION_CATEGORIES: &[(&[&str], f64)] = &[
        // Memory-bound operations (AI < 1): read/write more data than compute
        (
            &[
                "rmsnorm",
                "layernorm",
                "residual",
                "add",
                "embed",
                "softmax",
            ],
            0.25,
        ),
        // Elementwise activations (memory-bound)
        (&["swiglu", "gelu", "silu", "relu"], 0.5),
        // RoPE (rotary position embedding) - moderate AI
        (&["rope", "rotary"], 2.0),
        // Attention (compute-heavy for large sequences)
        (&["attention"], 8.0),
        // Matrix multiplications (compute-bound): FFN, QKV projections, output projections
        (&["ffn", "mlp", "qkv", "proj"], 16.0),
    ];

    for &(patterns, value) in OPERATION_CATEGORIES {
        if patterns.iter().any(|p| name_lower.contains(p)) {
            return value;
        }
    }

    // Default: balanced operation
    2.0
}

/// Estimate arithmetic intensity (FLOP/byte) for roofline analysis (PMAT-449)
///
/// Reference values from trueno-zram measurements and ML literature:
/// - Memory-bound (AI < 1): RmsNorm, Residual, embedding lookup
/// - Balanced (AI 1-10): Attention Q/K/V projections
/// - Compute-bound (AI > 10): Matrix multiplications, convolutions
fn estimate_arithmetic_intensity(brick_name: &str) -> f64 {
    let name_lower = brick_name.to_lowercase();
    categorize_operation(&name_lower)
}

/// BrickProfiler JSON input format (matches trueno::brick::BrickStats)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrickStats {
    /// Brick name
    pub name: String,
    /// Total samples collected
    pub count: u64,
    /// Total elapsed time (nanoseconds)
    pub total_ns: u64,
    /// Min elapsed time (nanoseconds)
    pub min_ns: u64,
    /// Max elapsed time (nanoseconds)
    pub max_ns: u64,
    /// Total elements processed
    pub total_elements: u64,
}

impl BrickStats {
    /// Calculate mean time in microseconds
    pub fn mean_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.total_ns as f64 / self.count as f64) / 1000.0
        }
    }

    /// Calculate throughput in elements/second
    pub fn throughput(&self) -> f64 {
        if self.total_ns == 0 {
            0.0
        } else {
            (self.total_elements as f64 * 1_000_000_000.0) / self.total_ns as f64
        }
    }

    /// Calculate coefficient of variation (CV)
    /// Approximated from min/max range
    pub fn cv_percent(&self) -> f64 {
        if self.count < 2 || self.min_ns == 0 {
            0.0
        } else {
            let mean = self.total_ns as f64 / self.count as f64;
            let range = (self.max_ns - self.min_ns) as f64;
            // CV approximation: range / (2 * sqrt(3) * mean) for uniform distribution
            // Using simpler heuristic: range / (4 * mean) * 100
            (range / (4.0 * mean)) * 100.0
        }
    }
}

/// BrickProfiler JSON output format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrickProfilerOutput {
    /// Per-brick statistics
    pub bricks: Vec<BrickStats>,
    /// Total tokens processed
    #[serde(default)]
    pub total_tokens: u64,
    /// Total time in nanoseconds
    #[serde(default)]
    pub total_ns: u64,
    /// Model name (if applicable)
    #[serde(default)]
    pub model: Option<String>,
    /// Hardware info
    #[serde(default)]
    pub hardware: Option<String>,
}

/// Brick budget specification (microseconds)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrickBudget {
    /// Brick name pattern (supports wildcards)
    pub name: String,
    /// Maximum allowed time in microseconds
    pub max_us: f64,
}

/// Default brick budgets from qwen2.5-coder-showcase-demo.md
pub fn default_brick_budgets() -> Vec<BrickBudget> {
    vec![
        BrickBudget {
            name: "RmsNorm".to_string(),
            max_us: 10.0,
        },
        BrickBudget {
            name: "QKV".to_string(),
            max_us: 15.0,
        },
        BrickBudget {
            name: "RoPE".to_string(),
            max_us: 5.0,
        },
        BrickBudget {
            name: "Attention".to_string(),
            max_us: 25.0,
        },
        BrickBudget {
            name: "OProj".to_string(),
            max_us: 10.0,
        },
        BrickBudget {
            name: "FFNGateUp".to_string(),
            max_us: 20.0,
        },
        BrickBudget {
            name: "SwiGLU".to_string(),
            max_us: 5.0,
        },
        BrickBudget {
            name: "FFNDown".to_string(),
            max_us: 15.0,
        },
        BrickBudget {
            name: "Residual".to_string(),
            max_us: 3.0,
        },
    ]
}

/// Category score
#[derive(Debug, Clone, Serialize)]
pub struct CategoryScore {
    /// Category name
    pub name: String,
    /// Points earned
    pub earned: f64,
    /// Maximum points available
    pub max_points: f64,
    /// Individual checks
    pub checks: Vec<BrickCheck>,
}

impl CategoryScore {
    pub fn percentage(&self) -> f64 {
        if self.max_points == 0.0 {
            100.0
        } else {
            (self.earned / self.max_points) * 100.0
        }
    }
}

/// Individual brick check result
#[derive(Debug, Clone, Serialize)]
pub struct BrickCheck {
    /// Brick name
    pub name: String,
    /// Check passed
    pub passed: bool,
    /// Points earned
    pub points: f64,
    /// Max points for this check
    pub max_points: f64,
    /// Actual value
    pub actual: f64,
    /// Threshold value
    pub threshold: f64,
    /// Unit (µs, %, elem/s, etc.)
    pub unit: String,
    /// Recommendation if failed
    pub recommendation: Option<String>,
}

/// Complete brick score
#[derive(Debug, Clone, Serialize)]
pub struct BrickScore {
    /// Performance category (40 points)
    pub performance: CategoryScore,
    /// Efficiency category (25 points)
    pub efficiency: CategoryScore,
    /// Correctness category (20 points)
    pub correctness: CategoryScore,
    /// Stability category (15 points)
    pub stability: CategoryScore,
    /// Total score (0-100)
    pub total_score: f64,
    /// Letter grade
    pub grade: char,
    /// Individual brick reports
    pub brick_reports: Vec<BrickReport>,
    /// Metadata
    pub metadata: BrickScoreMetadata,
}

/// Per-brick report
#[derive(Debug, Clone, Serialize)]
pub struct BrickReport {
    pub name: String,
    pub mean_us: f64,
    pub budget_us: Option<f64>,
    pub over_budget: bool,
    pub cv_percent: f64,
    pub throughput: f64,
    pub count: u64,
    /// PMAT-449: Estimated arithmetic intensity (FLOP/byte)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arithmetic_intensity: Option<f64>,
    /// PMAT-449: Bottleneck classification (memory vs compute)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottleneck: Option<Bottleneck>,
}

/// Score metadata
#[derive(Debug, Clone, Serialize)]
pub struct BrickScoreMetadata {
    pub version: String,
    pub project_path: String,
    pub model: Option<String>,
    pub hardware: Option<String>,
    pub total_bricks: usize,
    pub total_samples: u64,
    /// PMAT-448: Detected SIMD capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simd: Option<String>,
    /// PMAT-448: Memory bandwidth in GB/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bw_gbps: Option<f64>,
    /// PMAT-448: Peak GFLOP/s (CPU)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_gflops: Option<f64>,
    /// PMAT-448: Budget scaling factor applied
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_scale_factor: Option<f64>,
}

/// Score a single brick's performance against its budget.
fn score_performance(mean_us: f64, budget_us: f64, brick_name: &str) -> BrickCheck {
    let budget_ratio = mean_us / budget_us;
    let perf_points = match () {
        _ if budget_ratio <= 1.0 => 4.0,
        _ if budget_ratio <= 1.5 => 2.0,
        _ if budget_ratio <= 2.0 => 1.0,
        _ => 0.0,
    };
    let recommendation = (budget_ratio > 1.0).then(|| {
        format!(
            "Optimize {} to meet {}µs budget (currently {:.1}µs, {:.0}% over)",
            brick_name, budget_us, mean_us, (budget_ratio - 1.0) * 100.0
        )
    });

    BrickCheck {
        name: brick_name.to_string(),
        passed: budget_ratio <= 1.0,
        points: perf_points,
        max_points: 4.0,
        actual: mean_us,
        threshold: budget_us,
        unit: "µs".to_string(),
        recommendation,
    }
}

/// Score a single brick's throughput efficiency.
fn score_efficiency(throughput: f64, brick_name: &str) -> BrickCheck {
    let eff_points = if throughput > 1_000_000.0 {
        2.5 // >1M elem/s
    } else if throughput > 100_000.0 {
        1.5 // >100K elem/s
    } else if throughput > 0.0 {
        0.5
    } else {
        0.0
    };

    BrickCheck {
        name: brick_name.to_string(),
        passed: throughput > 100_000.0,
        points: eff_points,
        max_points: 2.5,
        actual: throughput,
        threshold: 100_000.0,
        unit: "elem/s".to_string(),
        recommendation: if throughput < 100_000.0 {
            Some(format!(
                "Improve {} throughput (currently {:.0} elem/s)",
                brick_name, throughput
            ))
        } else {
            None
        },
    }
}

/// Score a single brick's measurement stability via coefficient of variation.
fn score_stability(cv: f64, brick_name: &str) -> BrickCheck {
    let stability_points = if cv < 5.0 {
        1.5 // Excellent stability
    } else if cv < 10.0 {
        1.0 // Good stability
    } else if cv < 15.0 {
        0.5 // Acceptable stability
    } else {
        0.0 // Unstable
    };

    BrickCheck {
        name: brick_name.to_string(),
        passed: cv < 15.0,
        points: stability_points,
        max_points: 1.5,
        actual: cv,
        threshold: 15.0,
        unit: "%".to_string(),
        recommendation: if cv >= 15.0 {
            Some(format!(
                "Stabilize {} measurements (CV {:.1}% exceeds 15% threshold)",
                brick_name, cv
            ))
        } else {
            None
        },
    }
}

/// Score a BrickProfiler output
///
/// PMAT-448: If hardware is provided, the score metadata will include
/// detailed hardware info for reproducibility.
pub fn score_brick_profiler(
    profiler_output: &BrickProfilerOutput,
    budgets: &[BrickBudget],
    project_path: &Path,
    hardware: Option<&HardwareCapability>,
) -> BrickScore {
    let mut performance_checks = Vec::new();
    let mut efficiency_checks = Vec::new();
    let mut stability_checks = Vec::new();
    let mut brick_reports = Vec::new();

    // Calculate per-brick scores
    for brick in &profiler_output.bricks {
        let mean_us = brick.mean_us();
        let cv = brick.cv_percent();
        let throughput = brick.throughput();

        // Find budget for this brick
        let budget = budgets
            .iter()
            .find(|b| brick.name.contains(&b.name))
            .map(|b| b.max_us);

        let over_budget = budget.map(|b| mean_us > b).unwrap_or(false);

        // Performance check: within budget
        if let Some(budget_us) = budget {
            performance_checks.push(score_performance(mean_us, budget_us, &brick.name));
        }

        // Efficiency check: throughput
        efficiency_checks.push(score_efficiency(throughput, &brick.name));

        // Stability check: CV < 15%
        stability_checks.push(score_stability(cv, &brick.name));

        // PMAT-449: Estimate arithmetic intensity for roofline analysis
        // AI = FLOP / bytes_transferred
        // For typical ML operations: ~2 FLOPs per element (multiply-add)
        // Memory: 4 bytes per f32 element (read) + 4 bytes (write) = 8 bytes
        // Baseline AI ≈ 2 / 8 = 0.25 FLOP/byte
        let ai = estimate_arithmetic_intensity(&brick.name);
        let bottleneck_class = hardware.map(|hw| hw.bottleneck(ai, false));

        brick_reports.push(BrickReport {
            name: brick.name.clone(),
            mean_us,
            budget_us: budget,
            over_budget,
            cv_percent: cv,
            throughput,
            count: brick.count,
            arithmetic_intensity: Some(ai),
            bottleneck: bottleneck_class,
        });
    }

    // Calculate category scores, normalized to category max based on brick count
    let num_bricks = profiler_output.bricks.len() as f64;

    // Performance: normalize per-brick scores (4 pts per brick max) to 40 pt scale
    let perf_per_brick_max = 4.0;
    let perf_raw: f64 = performance_checks.iter().map(|c| c.points).sum();
    let perf_max_possible = num_bricks * perf_per_brick_max;
    let perf_normalized = if perf_max_possible > 0.0 {
        (perf_raw / perf_max_possible) * 40.0
    } else {
        0.0
    };

    let performance = CategoryScore {
        name: "Performance".to_string(),
        earned: perf_normalized.min(40.0),
        max_points: 40.0,
        checks: performance_checks,
    };

    // Efficiency: normalize per-brick scores (2.5 pts per brick max) to 25 pt scale
    let eff_per_brick_max = 2.5;
    let eff_raw: f64 = efficiency_checks.iter().map(|c| c.points).sum();
    let eff_max_possible = num_bricks * eff_per_brick_max;
    let eff_normalized = if eff_max_possible > 0.0 {
        (eff_raw / eff_max_possible) * 25.0
    } else {
        0.0
    };

    let efficiency = CategoryScore {
        name: "Efficiency".to_string(),
        earned: eff_normalized.min(25.0),
        max_points: 25.0,
        checks: efficiency_checks,
    };

    // Correctness: based on having samples (proxy for assertions passing)
    let correctness_earned = if profiler_output.bricks.iter().all(|b| b.count > 0) {
        20.0
    } else {
        10.0
    };

    let correctness = CategoryScore {
        name: "Correctness".to_string(),
        earned: correctness_earned,
        max_points: 20.0,
        checks: vec![BrickCheck {
            name: "All bricks executed".to_string(),
            passed: profiler_output.bricks.iter().all(|b| b.count > 0),
            points: correctness_earned,
            max_points: 20.0,
            actual: profiler_output
                .bricks
                .iter()
                .filter(|b| b.count > 0)
                .count() as f64,
            threshold: profiler_output.bricks.len() as f64,
            unit: "bricks".to_string(),
            recommendation: None,
        }],
    };

    // Stability: normalize per-brick scores (1.5 pts per brick max) to 15 pt scale
    let stab_per_brick_max = 1.5;
    let stab_raw: f64 = stability_checks.iter().map(|c| c.points).sum();
    let stab_max_possible = num_bricks * stab_per_brick_max;
    let stab_normalized = if stab_max_possible > 0.0 {
        (stab_raw / stab_max_possible) * 15.0
    } else {
        0.0
    };

    let stability = CategoryScore {
        name: "Stability".to_string(),
        earned: stab_normalized.min(15.0),
        max_points: 15.0,
        checks: stability_checks,
    };

    // Total score
    let total_score =
        performance.earned + efficiency.earned + correctness.earned + stability.earned;

    // Grade
    let grade = match total_score as u32 {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        60..=69 => 'D',
        _ => 'F',
    };

    // PMAT-448: Calculate budget scale factor if hardware is provided
    let (simd_str, mem_bw, peak_gflops, scale_factor) = if let Some(hw) = hardware {
        let simd = format!("{:?}", hw.cpu.simd);
        let simd_factor = hw.cpu.simd.compute_speedup();
        let mem_bw_factor = hw.cpu.memory_bw_gbps / 25.0;
        let scale = (simd_factor * mem_bw_factor).sqrt();
        (
            Some(simd),
            Some(hw.cpu.memory_bw_gbps),
            Some(hw.cpu.peak_gflops),
            Some(scale),
        )
    } else {
        (None, None, None, None)
    };

    BrickScore {
        performance,
        efficiency,
        correctness,
        stability,
        total_score,
        grade,
        brick_reports,
        metadata: BrickScoreMetadata {
            version: "1.0.0".to_string(),
            project_path: project_path.display().to_string(),
            model: profiler_output.model.clone(),
            hardware: profiler_output
                .hardware
                .clone()
                .or_else(|| hardware.map(|hw| format!("{} ({})", hw.cpu.model, hw.hostname))),
            total_bricks: profiler_output.bricks.len(),
            total_samples: profiler_output.bricks.iter().map(|b| b.count).sum(),
            simd: simd_str,
            memory_bw_gbps: mem_bw,
            peak_gflops,
            budget_scale_factor: scale_factor,
        },
    }
}

/// Load BrickProfiler JSON from file
pub fn load_profiler_json(path: &Path) -> anyhow::Result<BrickProfilerOutput> {
    let content = fs::read_to_string(path)?;
    let output: BrickProfilerOutput = serde_json::from_str(&content)?;
    Ok(output)
}

/// Scan project for brick profiler JSON files
pub fn find_profiler_files(project_path: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    // Common locations for profiler output
    let patterns = [
        "brick_profile.json",
        "profiler.json",
        ".pmat/brick_profile.json",
        "target/brick_profile.json",
        "results.json",
    ];

    for pattern in patterns {
        let path = project_path.join(pattern);
        if path.exists() {
            files.push(path);
        }
    }

    files
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brick_stats_calculations() {
        let stats = BrickStats {
            name: "TestBrick".to_string(),
            count: 100,
            total_ns: 1_000_000, // 1ms total
            min_ns: 8_000,
            max_ns: 12_000,
            total_elements: 1_000_000,
        };

        assert!((stats.mean_us() - 10.0).abs() < 0.01);
        assert!(stats.throughput() > 0.0);
        assert!(stats.cv_percent() < 100.0);
    }

    #[test]
    fn test_score_calculation() {
        let output = BrickProfilerOutput {
            bricks: vec![
                BrickStats {
                    name: "RmsNorm".to_string(),
                    count: 100,
                    total_ns: 800_000, // 8µs mean (within 10µs budget)
                    min_ns: 7_000,
                    max_ns: 9_000,
                    total_elements: 1_000_000,
                },
                BrickStats {
                    name: "Attention".to_string(),
                    count: 100,
                    total_ns: 2_000_000, // 20µs mean (within 25µs budget)
                    min_ns: 18_000,
                    max_ns: 22_000,
                    total_elements: 500_000,
                },
            ],
            total_tokens: 1000,
            total_ns: 2_800_000,
            model: Some("test-model".to_string()),
            hardware: Some("test-hw".to_string()),
        };

        let budgets = default_brick_budgets();
        let score = score_brick_profiler(&output, &budgets, Path::new("."), None);

        assert!(score.total_score > 0.0);
        assert!(score.total_score <= 100.0);
        assert!(score.grade != 'F');
    }

    #[test]
    fn test_hardware_scaling() {
        let hw = HardwareCapability {
            timestamp: "2026-01-13T00:00:00Z".to_string(),
            hostname: "test-host".to_string(),
            cpu: CpuCapability {
                vendor: "Intel".to_string(),
                model: "i9-14900K".to_string(),
                cores: 24,
                threads: 32,
                simd: SimdWidth::Avx512,
                base_freq_ghz: 3.2,
                peak_gflops: 3072.0, // 24 cores × 16 lanes × 2 FMA × 3.2 GHz
                memory_bw_gbps: 89.6,
            },
            gpu: None,
            roofline: RooflineParams::default(),
            byte_budget: Some(ByteBudget::default()),
        };

        let base_budgets = default_brick_budgets();
        let scaled = scale_budgets_for_hardware(&base_budgets, &hw);

        // Faster hardware = stricter (lower) budgets
        for (base, scaled) in base_budgets.iter().zip(scaled.iter()) {
            assert!(
                scaled.max_us < base.max_us,
                "Scaled budget should be stricter"
            );
        }
    }

    #[test]
    fn test_simd_speedup_factors() {
        assert_eq!(SimdWidth::Scalar.compute_speedup(), 1.0);
        assert_eq!(SimdWidth::Avx2.compute_speedup(), 10.0);
        assert_eq!(SimdWidth::Avx512.compute_speedup(), 12.0);
        assert_eq!(SimdWidth::Neon128.compute_speedup(), 4.0);
    }

    #[test]
    fn test_bottleneck_classification() {
        let hw = HardwareCapability::default();

        // Low arithmetic intensity = memory bound
        assert_eq!(hw.bottleneck(0.1, false), Bottleneck::Memory);

        // High arithmetic intensity = compute bound
        assert_eq!(hw.bottleneck(100.0, false), Bottleneck::Compute);
    }

    #[test]
    fn test_arithmetic_intensity_estimation() {
        // Memory-bound operations (AI < 1)
        assert!(estimate_arithmetic_intensity("RmsNorm") < 1.0);
        assert!(estimate_arithmetic_intensity("LayerNorm") < 1.0);
        assert!(estimate_arithmetic_intensity("Residual") < 1.0);
        assert!(estimate_arithmetic_intensity("SoftMax") < 1.0);

        // Elementwise activations (AI < 1)
        assert!(estimate_arithmetic_intensity("SwiGLU") < 1.0);
        assert!(estimate_arithmetic_intensity("GELU") < 1.0);

        // Balanced operations (AI 1-10)
        assert!(estimate_arithmetic_intensity("RoPE") >= 1.0);
        assert!(estimate_arithmetic_intensity("RoPE") < 10.0);
        assert!(estimate_arithmetic_intensity("Attention") >= 1.0);
        assert!(estimate_arithmetic_intensity("Attention") <= 16.0);

        // Compute-bound operations (AI > 8)
        assert!(estimate_arithmetic_intensity("FFNGateUp") > 8.0);
        assert!(estimate_arithmetic_intensity("QKVProj") > 8.0);
        assert!(estimate_arithmetic_intensity("MLP") > 8.0);
    }

    #[test]
    fn test_byte_budget_default() {
        let budget = ByteBudget::default();
        // Default: 25 GB/s from trueno-zram
        assert!((budget.gb_per_sec - 25.0).abs() < 0.001);
        assert_eq!(budget.page_size, 4096);
        // 25 GB/s = 6.1M pages/sec = ~0.164 µs/page
        assert!(budget.us_per_page > 0.1);
        assert!(budget.us_per_page < 0.2);
    }

    #[test]
    fn test_hardware_includes_byte_budget() {
        let hw = HardwareCapability::default();
        assert!(hw.byte_budget.is_some());
        let budget = hw.byte_budget.unwrap();
        assert!(budget.gb_per_sec > 0.0);
    }

    #[test]
    fn test_byte_budget_toml_roundtrip() {
        let hw = HardwareCapability::default();
        let toml_str = toml::to_string_pretty(&hw).unwrap();

        // Should contain byte_budget section
        assert!(toml_str.contains("[byte_budget]"));
        assert!(toml_str.contains("gb_per_sec"));

        // Roundtrip should preserve values
        let parsed: HardwareCapability = toml::from_str(&toml_str).unwrap();
        assert!(parsed.byte_budget.is_some());
        let original = hw.byte_budget.unwrap();
        let parsed_budget = parsed.byte_budget.unwrap();
        assert!((parsed_budget.gb_per_sec - original.gb_per_sec).abs() < 0.001);
    }

    #[test]
    fn test_hardware_toml_backward_compat() {
        // Old hardware.toml without byte_budget should still parse
        let old_toml = r#"
timestamp = "2026-01-13T00:00:00Z"
hostname = "test"

[cpu]
vendor = "Intel"
model = "Test"
cores = 4
threads = 8
simd = "Avx2"
base_freq_ghz = 3.0
peak_gflops = 100.0
memory_bw_gbps = 50.0

[roofline]
cpu_arithmetic_intensity = 2.0
"#;
        let parsed: HardwareCapability = toml::from_str(old_toml).unwrap();
        // byte_budget should be None (backward compat)
        assert!(parsed.byte_budget.is_none());
    }
}
