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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn lanes(&self) -> usize {
        match self {
            SimdWidth::Scalar => 1,
            SimdWidth::Neon128 | SimdWidth::Sse2 | SimdWidth::WasmSimd128 => 4,
            SimdWidth::Avx2 => 8,
            SimdWidth::Avx512 => 16,
        }
    }

    /// Typical speedup factor (from trueno-zram measurements)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn compute_speedup(&self) -> f64 {
        // Contract: compute_speedup returns a bounded score
        let result = match self {
            SimdWidth::Scalar => 1.0,
            SimdWidth::Neon128 | SimdWidth::Sse2 | SimdWidth::WasmSimd128 => 4.0,
            SimdWidth::Avx2 => 10.0,   // 8-12x measured
            SimdWidth::Avx512 => 12.0, // 8-13x measured
        };
        debug_assert!(result >= 1.0, "speedup must be >= 1.0: {}", result);
        result
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn default_hardware_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".pmat")
        .join("hardware.toml")
}

/// Load hardware capability from TOML file
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn scale_budgets_for_hardware(
    base_budgets: &[BrickBudget],
    hardware: &HardwareCapability,
) -> Vec<BrickBudget> {
    debug_assert!(!base_budgets.is_empty(), "base_budgets must not be empty");
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

