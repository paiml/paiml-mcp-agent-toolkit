// ============================================================================
// RustProjectScore - Main Score Container
// ============================================================================

/// Comprehensive Rust project quality score (v1.1)
///
/// Total score: 0-106 points across 6 categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustProjectScore {
    /// Total score (0-106 points)
    pub total_score: f64,

    /// Letter grade (A+ to F)
    pub grade: Grade,

    /// Breakdown by category
    pub categories: CategoryScores,

    /// Actionable recommendations
    pub recommendations: Vec<Recommendation>,

    /// Metadata (timestamp, project, version)
    pub metadata: ScoreMetadata,

    /// Score velocity (Kaizen tracking) - NEW in v1.1
    pub velocity: Option<ScoreVelocity>,
}

impl RustProjectScore {
    /// Create a new score with zero values
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            total_score: 0.0,
            grade: Grade::F,
            categories: CategoryScores::default(),
            recommendations: Vec::new(),
            metadata: ScoreMetadata::new("unknown".to_string(), "1.1.0".to_string()),
            velocity: None,
        }
    }
}

impl Default for RustProjectScore {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalizedScore for RustProjectScore {
    fn raw(&self) -> f64 {
        debug_assert!(true, "contract: raw");
        self.total_score
    }

    fn max_raw(&self) -> f64 {
        debug_assert!(true, "contract: max_raw");
        RUST_PROJECT_MAX_POINTS
    }
}

impl fmt::Display for RustProjectScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rust Project Score: {:.1}/100 ({}) [raw: {:.1}/{}]",
            self.normalized(),
            self.grade,
            self.total_score,
            RUST_PROJECT_MAX_POINTS as u32
        )
    }
}

// ============================================================================
// Grade - Letter Grade Enum
// ============================================================================

/// Letter grade based on NORMALIZED percentage (0-100 scale)
///
/// PMAT-454: All grading now uses normalized 0-100 percentages
///
/// Thresholds (normalized 0-100):
/// - A+ : 95-100%
/// - A  : 90-94%
/// - A- : 85-89%
/// - B+ : 80-84%
/// - B  : 70-79%
/// - C  : 60-69%
/// - D  : 50-59%
/// - F  : 0-49%
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    APlus,
    A,
    AMinus,
    BPlus,
    B,
    C,
    D,
    F,
}

impl Grade {
    /// Calculate grade from raw score and max possible points
    ///
    /// PMAT-454: Now properly normalizes to 0-100 before grading
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_score(score: f64, max: f64) -> Self {
        debug_assert!(score >= 0.0, "score must be non-negative");
        // Normalize to 0-100 percentage
        let normalized = if max > 0.0 {
            (score / max * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        if normalized >= 95.0 {
            Grade::APlus
        } else if normalized >= 90.0 {
            Grade::A
        } else if normalized >= 85.0 {
            Grade::AMinus
        } else if normalized >= 80.0 {
            Grade::BPlus
        } else if normalized >= 70.0 {
            Grade::B
        } else if normalized >= 60.0 {
            Grade::C
        } else if normalized >= 50.0 {
            Grade::D
        } else {
            Grade::F
        }
    }

    /// Calculate grade from already-normalized percentage (0-100)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_normalized(normalized: f64) -> Self {
        Self::from_score(normalized, 100.0)
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

