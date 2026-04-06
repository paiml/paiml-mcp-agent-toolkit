// Impl blocks for defect report types
// Included from defect_report.rs - shares parent module scope

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

impl std::fmt::Display for DefectCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefectCategory::Complexity => write!(f, "Complexity"),
            DefectCategory::TechnicalDebt => write!(f, "Technical Debt"),
            DefectCategory::DeadCode => write!(f, "Dead Code"),
            DefectCategory::Duplication => write!(f, "Duplication"),
            DefectCategory::Performance => write!(f, "Performance"),
            DefectCategory::Architecture => write!(f, "Architecture"),
            DefectCategory::TestCoverage => write!(f, "Test Coverage"),
        }
    }
}

impl DefectCategory {
    /// Get all categories for iteration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::defect_report::DefectCategory;
    ///
    /// let categories = DefectCategory::all();
    /// assert_eq!(categories.len(), 7);
    /// assert!(categories.contains(&DefectCategory::Complexity));
    /// assert!(categories.contains(&DefectCategory::TestCoverage));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Complexity,
            Self::TechnicalDebt,
            Self::DeadCode,
            Self::Duplication,
            Self::Performance,
            Self::Architecture,
            Self::TestCoverage,
        ]
    }
}

impl Defect {
    /// Create a new defect ID with the given prefix and index
    /// Generates a unique defect ID with prefix and index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::defect_report::Defect;
    ///
    /// let id = Defect::generate_id("TEST", 0);
    /// assert_eq!(id, "TEST-001");
    ///
    /// let id2 = Defect::generate_id("BUG", 99);
    /// assert_eq!(id2, "BUG-100");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_id(prefix: &str, index: usize) -> DefectId {
        format!("{}-{:03}", prefix, index + 1)
    }

    /// Calculate severity weight for scoring
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::defect_report::{Defect, Severity, DefectCategory};
    /// use std::path::PathBuf;
    /// use std::collections::HashMap;
    ///
    /// let defect = Defect {
    ///     id: "TEST-001".to_string(),
    ///     severity: Severity::High,
    ///     category: DefectCategory::TechnicalDebt,
    ///     file_path: PathBuf::from("src/main.rs"),
    ///     line_start: 45,
    ///     line_end: None,
    ///     column_start: None,
    ///     column_end: None,
    ///     message: "Potential memory leak".to_string(),
    ///     rule_id: "MEM001".to_string(),
    ///     fix_suggestion: None,
    ///     metrics: HashMap::new(),
    /// };
    ///
    /// assert_eq!(defect.severity_weight(), 5.0);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn severity_weight(&self) -> f64 {
        match self.severity {
            Severity::Critical => 10.0,
            Severity::High => 5.0,
            Severity::Medium => 3.0,
            Severity::Low => 1.0,
        }
    }
}

impl Default for FileRankingConfig {
    fn default() -> Self {
        let mut category_weights = HashMap::new();
        category_weights.insert(DefectCategory::Complexity, 1.5);
        category_weights.insert(DefectCategory::Performance, 2.0);
        category_weights.insert(DefectCategory::Architecture, 1.8);
        category_weights.insert(DefectCategory::TechnicalDebt, 1.2);
        category_weights.insert(DefectCategory::DeadCode, 1.0);
        category_weights.insert(DefectCategory::Duplication, 1.3);
        category_weights.insert(DefectCategory::TestCoverage, 0.8);

        Self {
            use_severity: true,
            use_count: true,
            category_weights,
        }
    }
}
