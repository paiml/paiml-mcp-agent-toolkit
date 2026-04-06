// PerformanceDefectAnalyzer and ArchitectureDefectAnalyzer: constructors, trait impls,
// and helpers for Big-O complexity detection and architecture issue detection.

impl PerformanceDefectAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            analyzer: crate::services::big_o_analyzer::BigOAnalyzer::new(),
        }
    }
}

impl Default for PerformanceDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchitectureDefectAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ArchitectureDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DefectAnalyzer for PerformanceDefectAnalyzer {
    type Config = PerformanceConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut defects = Vec::new();
        let analysis_config = crate::services::big_o_analyzer::BigOAnalysisConfig {
            project_path: project_path.to_path_buf(),
            include_patterns: vec![],
            exclude_patterns: vec![],
            confidence_threshold: 50,
            analyze_space_complexity: true,
        };
        let report = self.analyzer.analyze(analysis_config).await?;

        for (index, func) in report.high_complexity_functions.iter().enumerate() {
            if self.is_problematic_complexity(&func.time_complexity, &config) {
                defects.push(self.function_complexity_to_defect(func, index + 1));
            }
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Performance
    }

    fn supports_incremental(&self) -> bool {
        true
    }
}

impl PerformanceDefectAnalyzer {
    fn is_problematic_complexity(
        &self,
        complexity: &crate::models::complexity_bound::ComplexityBound,
        config: &PerformanceConfig,
    ) -> bool {
        use crate::models::complexity_bound::BigOClass;

        matches!(
            complexity.class,
            BigOClass::Quadratic | BigOClass::Cubic | BigOClass::Exponential | BigOClass::Factorial
        ) || (config.include_nlogn && matches!(complexity.class, BigOClass::Linearithmic))
    }

    fn function_complexity_to_defect(&self, func: &FunctionComplexity, index: usize) -> Defect {
        use crate::models::complexity_bound::BigOClass;

        let severity = match func.time_complexity.class {
            BigOClass::Exponential | BigOClass::Factorial => Severity::Critical,
            BigOClass::Cubic => Severity::High,
            BigOClass::Quadratic => Severity::Medium,
            _ => Severity::Low,
        };

        let mut metrics = HashMap::new();
        metrics.insert(
            "time_complexity_class".to_string(),
            f64::from(func.time_complexity.class as u8),
        );
        metrics.insert(
            "space_complexity_class".to_string(),
            f64::from(func.space_complexity.class as u8),
        );
        metrics.insert("confidence".to_string(), f64::from(func.confidence));

        Defect {
            id: format!("PERF-{index:04}"),
            severity,
            category: DefectCategory::Performance,
            file_path: func.file_path.clone(),
            line_start: func.line_number as u32,
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!(
                "Function '{}' has high time complexity: {}",
                func.function_name,
                func.time_complexity.notation()
            ),
            rule_id: "high-complexity".to_string(),
            fix_suggestion: Some(self.generate_performance_suggestion(&func.time_complexity)),
            metrics,
        }
    }

    fn generate_performance_suggestion(
        &self,
        complexity: &crate::models::complexity_bound::ComplexityBound,
    ) -> String {
        use crate::models::complexity_bound::BigOClass;

        match complexity.class {
            BigOClass::Quadratic => {
                "Consider using a more efficient algorithm or data structure to reduce quadratic complexity"
            }
            BigOClass::Cubic => {
                "Cubic complexity is rarely acceptable; consider algorithmic improvements"
            }
            BigOClass::Exponential => {
                "Exponential complexity should be avoided; consider dynamic programming or approximation"
            }
            BigOClass::Factorial => {
                "Factorial complexity is unacceptable for most use cases; fundamental algorithm redesign needed"
            }
            _ => {
                "Review algorithm efficiency and consider optimization"
            }
        }
        .to_string()
    }
}

#[async_trait]
impl DefectAnalyzer for ArchitectureDefectAnalyzer {
    type Config = ArchitectureConfig;

    async fn analyze(&self, project_path: &Path, _config: Self::Config) -> Result<Vec<Defect>> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let defects = Vec::new();

        // Build dependency graph
        let project = crate::services::project_analyzer::Project::new(project_path)?;
        let _dag = project.build_dependency_graph().await?;

        // Analyze architecture issues
        // This is a placeholder for actual architecture analysis

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Architecture
    }

    fn supports_incremental(&self) -> bool {
        false
    }
}
