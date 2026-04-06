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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn mean_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.total_ns as f64 / self.count as f64) / 1000.0
        }
    }

    /// Calculate throughput in elements/second
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn throughput(&self) -> f64 {
        if self.total_ns == 0 {
            0.0
        } else {
            (self.total_elements as f64 * 1_000_000_000.0) / self.total_ns as f64
        }
    }

    /// Calculate coefficient of variation (CV)
    /// Approximated from min/max range
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
