//! CUDA-SIMD Score Types
//!
//! Extracted from cuda_simd.rs for file health compliance (CB-040).
//! Contains the 100-point Karl Popper falsification scoring structures.

use serde::{Deserialize, Serialize};

/// Category A: Falsifiability & Testability (25 points) - GATEWAY
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FalsifiabilityScore {
    /// A.1: All bar.sync reachable from all threads (5 pts)
    pub barrier_safety: f64,
    /// A.2: Shared memory indices within tile dimensions (5 pts)
    pub bounds_verification: f64,
    /// A.3: Branch coverage includes warp divergence cases (5 pts)
    pub divergence_testing: f64,
    /// A.4: ThreadSanitizer or equivalent analysis (5 pts)
    pub memory_race_detection: f64,
    /// A.5: Register/shared memory within SM limits (5 pts)
    pub occupancy_bounds: f64,
}

impl FalsifiabilityScore {
    /// Calculate total for Category A
    #[must_use]
    pub fn total(&self) -> f64 {
        self.barrier_safety
            + self.bounds_verification
            + self.divergence_testing
            + self.memory_race_detection
            + self.occupancy_bounds
    }

    /// Maximum possible score for Category A
    pub const MAX: f64 = 25.0;

    /// Gateway threshold - if below this, total score is 0
    pub const GATEWAY_THRESHOLD: f64 = 15.0;
}

/// Category B: Reproducibility Infrastructure (25 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReproducibilityScore {
    /// B.1: Bitwise reproducible results (8 pts)
    pub deterministic_output: f64,
    /// B.2: CUDA/Driver/SM version locked (5 pts)
    pub version_pinning: f64,
    /// B.3: GPU model, compute capability documented (5 pts)
    pub hardware_specification: f64,
    /// B.4: Criterion-style statistical benchmarking (4 pts)
    pub benchmark_harness: f64,
    /// B.5: Automated regression on GPU hardware (3 pts)
    pub ci_cd_integration: f64,
}

impl ReproducibilityScore {
    /// Calculate total for Category B
    #[must_use]
    pub fn total(&self) -> f64 {
        self.deterministic_output
            + self.version_pinning
            + self.hardware_specification
            + self.benchmark_harness
            + self.ci_cd_integration
    }

    /// Maximum possible score for Category B
    pub const MAX: f64 = 25.0;
}

/// Category C: Transparency & Openness (20 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransparencyScore {
    /// C.1: Generated PTX accessible and documented (6 pts)
    pub ptx_inspection: f64,
    /// C.2: --ptxas-options=-v output analyzed (5 pts)
    pub register_allocation: f64,
    /// C.3: SM occupancy explicitly computed (5 pts)
    pub occupancy_calculation: f64,
    /// C.4: Shared memory bank mapping documented (4 pts)
    pub memory_layout: f64,
}

impl TransparencyScore {
    /// Calculate total for Category C
    #[must_use]
    pub fn total(&self) -> f64 {
        self.ptx_inspection
            + self.register_allocation
            + self.occupancy_calculation
            + self.memory_layout
    }

    /// Maximum possible score for Category C
    pub const MAX: f64 = 20.0;
}

/// Category D: Statistical Rigor (15 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatisticalRigorScore {
    /// D.1: ≥3s warmup before measurement (4 pts)
    pub warmup_iterations: f64,
    /// D.2: ≥100 samples for statistical significance (4 pts)
    pub sample_count: f64,
    /// D.3: IQR-based outlier detection reported (4 pts)
    pub outlier_analysis: f64,
    /// D.4: 95% CI on throughput metrics (3 pts)
    pub confidence_intervals: f64,
}

impl StatisticalRigorScore {
    /// Calculate total for Category D
    #[must_use]
    pub fn total(&self) -> f64 {
        self.warmup_iterations
            + self.sample_count
            + self.outlier_analysis
            + self.confidence_intervals
    }

    /// Maximum possible score for Category D
    pub const MAX: f64 = 15.0;
}

/// Category E: Historical Integrity (10 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoricalIntegrityScore {
    /// E.1: PARITY/PAR ticket references (4 pts)
    pub fault_lineage: f64,
    /// E.2: Tests derived from historical bugs (3 pts)
    pub regression_tests: f64,
    /// E.3: 5-Why analysis for each P0 defect (3 pts)
    pub root_cause_documentation: f64,
}

impl HistoricalIntegrityScore {
    /// Calculate total for Category E
    #[must_use]
    pub fn total(&self) -> f64 {
        self.fault_lineage + self.regression_tests + self.root_cause_documentation
    }

    /// Maximum possible score for Category E
    pub const MAX: f64 = 10.0;
}

/// Category F: GPU/SIMD Specific (5 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuSimdSpecificScore {
    /// F.1: Active threads / warp size ratio (2 pts)
    pub warp_efficiency: f64,
    /// F.2: Achieved vs theoretical bandwidth (2 pts)
    pub memory_throughput: f64,
    /// F.3: FMA/memory instruction ratio (1 pt)
    pub instruction_mix: f64,
}

impl GpuSimdSpecificScore {
    /// Calculate total for Category F
    #[must_use]
    pub fn total(&self) -> f64 {
        self.warp_efficiency + self.memory_throughput + self.instruction_mix
    }

    /// Maximum possible score for Category F
    pub const MAX: f64 = 5.0;
}

/// Complete 100-point Popper Falsification Score
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopperScore {
    /// Category A: Falsifiability & Testability (25 pts) - GATEWAY
    pub falsifiability: FalsifiabilityScore,
    /// Category B: Reproducibility Infrastructure (25 pts)
    pub reproducibility: ReproducibilityScore,
    /// Category C: Transparency & Openness (20 pts)
    pub transparency: TransparencyScore,
    /// Category D: Statistical Rigor (15 pts)
    pub statistical_rigor: StatisticalRigorScore,
    /// Category E: Historical Integrity (10 pts)
    pub historical_integrity: HistoricalIntegrityScore,
    /// Category F: GPU/SIMD Specific (5 pts)
    pub gpu_simd_specific: GpuSimdSpecificScore,
    /// Total score (0-100, or 0 if gateway fails)
    pub total: f64,
    /// Whether the gateway (Category A ≥ 15) passed
    pub gateway_passed: bool,
    /// Grade interpretation
    pub grade: CudaTdgGrade,
}

impl PopperScore {
    /// Calculate total score with gateway rule
    #[must_use]
    pub fn calculate(
        falsifiability: FalsifiabilityScore,
        reproducibility: ReproducibilityScore,
        transparency: TransparencyScore,
        statistical_rigor: StatisticalRigorScore,
        historical_integrity: HistoricalIntegrityScore,
        gpu_simd_specific: GpuSimdSpecificScore,
    ) -> Self {
        let category_a = falsifiability.total();
        let gateway_passed = category_a >= FalsifiabilityScore::GATEWAY_THRESHOLD;

        let raw_total = category_a
            + reproducibility.total()
            + transparency.total()
            + statistical_rigor.total()
            + historical_integrity.total()
            + gpu_simd_specific.total();

        // Gateway rule: if Category A < 15, total = 0
        let total = if gateway_passed { raw_total } else { 0.0 };
        let grade = CudaTdgGrade::from_score(total, gateway_passed);

        Self {
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd_specific,
            total,
            gateway_passed,
            grade,
        }
    }
}

/// Grade interpretation for CUDA-SIMD TDG scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CudaTdgGrade {
    /// 90-100: Production-ready, minimal debt
    APLus,
    /// 80-89: Production-ready with monitoring
    A,
    /// 70-79: Acceptable, prioritize improvements
    B,
    /// 60-69: Technical debt accumulating
    C,
    /// 50-59: Significant remediation needed
    D,
    /// 0-49: Not production-ready
    #[default]
    F,
    /// Gateway failure: Falsifiability requirements not met
    GatewayFail,
}

impl CudaTdgGrade {
    /// Convert score to grade
    #[must_use]
    pub fn from_score(score: f64, gateway_passed: bool) -> Self {
        if !gateway_passed {
            return Self::GatewayFail;
        }
        match score {
            s if s >= 90.0 => Self::APLus,
            s if s >= 80.0 => Self::A,
            s if s >= 70.0 => Self::B,
            s if s >= 60.0 => Self::C,
            s if s >= 50.0 => Self::D,
            _ => Self::F,
        }
    }
}

impl std::fmt::Display for CudaTdgGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::APLus => write!(f, "A+"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::F => write!(f, "F"),
            Self::GatewayFail => write!(f, "FAIL (Gateway)"),
        }
    }
}
