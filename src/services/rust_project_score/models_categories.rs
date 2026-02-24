// ============================================================================
// CategoryScores - 6 Scoring Categories
// ============================================================================

/// Six scoring categories (106 points total)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScores {
    /// Rust tooling compliance (25pts)
    pub rust_tooling: CategoryScore,

    /// Code quality (26pts)
    pub code_quality: CategoryScore,

    /// Testing excellence (20pts)
    pub testing: CategoryScore,

    /// Documentation (15pts)
    pub documentation: CategoryScore,

    /// Performance & benchmarking (10pts)
    pub performance: CategoryScore,

    /// Dependency health (12pts)
    pub dependencies: CategoryScore,
}

impl CategoryScores {
    /// Calculate total score across all categories
    pub fn total(&self) -> f64 {
        self.rust_tooling.earned
            + self.code_quality.earned
            + self.testing.earned
            + self.documentation.earned
            + self.performance.earned
            + self.dependencies.earned
    }
}

impl Default for CategoryScores {
    fn default() -> Self {
        Self {
            rust_tooling: CategoryScore::new(0.0, 25.0),
            code_quality: CategoryScore::new(0.0, 26.0),
            testing: CategoryScore::new(0.0, 20.0),
            documentation: CategoryScore::new(0.0, 15.0),
            performance: CategoryScore::new(0.0, 10.0),
            dependencies: CategoryScore::new(0.0, 12.0),
        }
    }
}

// ============================================================================
// CategoryScore - Individual Category Metrics
// ============================================================================

/// Score for an individual category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    /// Points earned
    pub earned: f64,

    /// Maximum possible points
    pub max: f64,

    /// Whether this category is applicable to the project type.
    /// Non-applicable categories (e.g., Rust Tooling for a pure Lean project)
    /// are excluded from normalized grade calculation.
    pub applicable: bool,
}

impl CategoryScore {
    /// Create a new category score (applicable by default)
    pub fn new(earned: f64, max: f64) -> Self {
        Self {
            earned,
            max,
            applicable: true,
        }
    }

    /// Create a non-applicable score (scorer errored / not relevant)
    pub fn not_applicable(max: f64) -> Self {
        Self {
            earned: 0.0,
            max,
            applicable: false,
        }
    }

    /// Calculate percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.max == 0.0 {
            0.0
        } else {
            (self.earned / self.max) * 100.0
        }
    }

    /// Check if category has perfect score
    pub fn is_perfect(&self) -> bool {
        (self.earned - self.max).abs() < 0.01
    }
}

