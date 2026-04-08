/// Thresholds for file size classification
pub mod thresholds {
    pub const IDEAL_MAX: usize = 200;
    pub const ACCEPTABLE_MAX: usize = 500;
    pub const WARNING_MAX: usize = 1000;
    pub const PROBLEM_MAX: usize = 2000;
    // >2000 is CRITICAL
}

/// File size classification based on line count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSizeClass {
    /// 0-200 lines: Optimal cognitive chunk
    Ideal,
    /// 201-500 lines: Within SRP tolerance
    Acceptable,
    /// 501-1000 lines: Approaching limit
    Warning,
    /// 1001-2000 lines: Exceeds cognitive capacity
    Problem,
    /// >2000 lines: Untestable monolith
    Critical,
}

impl FileSizeClass {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// From lines.
    pub fn from_lines(lines: usize) -> Self {
        match lines {
            0..=200 => Self::Ideal,
            201..=500 => Self::Acceptable,
            501..=1000 => Self::Warning,
            1001..=2000 => Self::Problem,
            _ => Self::Critical,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// As str.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ideal => "ideal",
            Self::Acceptable => "acceptable",
            Self::Warning => "warning",
            Self::Problem => "problem",
            Self::Critical => "critical",
        }
    }
}

/// Health grade based on composite score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthGrade {
    A, // 90-100
    B, // 80-89
    C, // 70-79
    D, // 60-69
    E, // 50-59
    F, // 0-49
}

impl HealthGrade {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    /// From score.
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::A,
            80..=89 => Self::B,
            70..=79 => Self::C,
            60..=69 => Self::D,
            50..=59 => Self::E,
            _ => Self::F,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// As str.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is passing.
    pub fn is_passing(&self) -> bool {
        matches!(self, Self::A | Self::B | Self::C)
    }
}

/// Health metrics for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealthMetrics {
    pub path: PathBuf,
    pub lines: usize,
    pub test_lines: usize,
    pub tlr: f32,
    pub required_tlr: f32,
    pub avg_complexity: f32,
    pub churn_30d: usize,
    pub health_score: u8,
    pub grade: HealthGrade,
    pub size_class: FileSizeClass,
}

impl FileHealthMetrics {
    /// Calculate health score using the composite formula
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn calculate(
        path: PathBuf,
        lines: usize,
        test_lines: usize,
        avg_complexity: f32,
        churn_30d: usize,
    ) -> Self {
        let required_tlr = Self::required_tlr_for_size(lines);
        let tlr = if lines > 0 {
            test_lines as f32 / lines as f32
        } else {
            1.0
        };

        // Size Score (30 points max)
        let size_score: u8 = match lines {
            0..=200 => 30,
            201..=500 => 25,
            501..=1000 => 15,
            1001..=2000 => 5,
            _ => 0,
        };

        // TLR Score (40 points max)
        let tlr_ratio = (tlr / required_tlr).min(1.0);
        let tlr_score = (tlr_ratio * 40.0) as u8;

        // Complexity Score (20 points max)
        let complexity_score: u8 = if avg_complexity <= 5.0 {
            20
        } else if avg_complexity <= 10.0 {
            15
        } else if avg_complexity <= 15.0 {
            10
        } else if avg_complexity <= 20.0 {
            5
        } else {
            0
        };

        // Stability Score (10 points max)
        let stability_score: u8 = match churn_30d {
            0..=2 => 10,
            3..=5 => 7,
            6..=10 => 4,
            _ => 0,
        };

        let health_score = size_score + tlr_score + complexity_score + stability_score;
        let grade = HealthGrade::from_score(health_score);
        let size_class = FileSizeClass::from_lines(lines);

        Self {
            path,
            lines,
            test_lines,
            tlr,
            required_tlr,
            avg_complexity,
            churn_30d,
            health_score,
            grade,
            size_class,
        }
    }

    /// Get required TLR based on file size (scaling thresholds)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn required_tlr_for_size(lines: usize) -> f32 {
        match lines {
            0..=100 => 0.3,
            101..=300 => 0.5,
            301..=500 => 0.7,
            501..=1000 => 1.0,
            _ => 1.5,
        }
    }
}
