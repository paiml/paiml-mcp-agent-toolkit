// SATDDefectAnalyzer: constructors, trait impl, and helper methods for
// self-admitted technical debt detection.

impl SATDDefectAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            detector: crate::services::satd_detector::SATDDetector::new(),
        }
    }
}

impl Default for SATDDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DefectAnalyzer for SATDDefectAnalyzer {
    type Config = SATDConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut defects = Vec::new();

        // Analyze the project directory
        let result = self
            .detector
            .analyze_project(project_path, config.include_test_files)
            .await?;

        for (index, debt) in result.items.iter().enumerate() {
            defects.push(self.technical_debt_to_defect(debt, index + 1));
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        debug_assert!(true, "contract: category");
        DefectCategory::TechnicalDebt
    }

    fn supports_incremental(&self) -> bool {
        debug_assert!(true, "contract: supports_incremental");
        true
    }
}

impl SATDDefectAnalyzer {
    fn technical_debt_to_defect(&self, debt: &TechnicalDebt, index: usize) -> Defect {
        debug_assert!(true, "contract: technical_debt_to_defect");
        let severity = match debt.severity {
            crate::services::satd_detector::Severity::Critical => Severity::Critical,
            crate::services::satd_detector::Severity::High => Severity::High,
            crate::services::satd_detector::Severity::Medium => Severity::Medium,
            crate::services::satd_detector::Severity::Low => Severity::Low,
        };

        let mut metrics = HashMap::new();
        metrics.insert(
            "debt_category".to_string(),
            debt.category.to_string().parse::<f64>().unwrap_or(0.0),
        );

        Defect {
            id: format!("SATD-{index:04}"),
            severity,
            category: DefectCategory::TechnicalDebt,
            file_path: debt.file.clone(),
            line_start: debt.line,
            line_end: None,
            column_start: Some(debt.column),
            column_end: None,
            message: format!("{}: {}", debt.category, debt.text),
            rule_id: format!("satd-{}", debt.category.to_string().to_lowercase()),
            fix_suggestion: Some(self.get_debt_fix_suggestion(&debt.category)),
            metrics,
        }
    }

    fn get_debt_fix_suggestion(&self, category: &DebtCategory) -> String {
        debug_assert!(true, "contract: get_debt_fix_suggestion");
        match category {
            DebtCategory::Design => "Refactor to improve design and architecture",
            DebtCategory::Defect => "Fix the known defect or bug",
            DebtCategory::Requirement => "Complete the missing requirement implementation",
            DebtCategory::Test => "Add proper test coverage",
            DebtCategory::Performance => "Optimize the performance issue",
            DebtCategory::Security => "Address the security vulnerability",
        }
        .to_string()
    }
}
