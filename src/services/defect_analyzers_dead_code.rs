// DeadCodeDefectAnalyzer: constructors, trait impl, and helper methods for
// dead/unreachable code detection.

impl DeadCodeDefectAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DeadCodeDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DefectAnalyzer for DeadCodeDefectAnalyzer {
    type Config = DeadCodeConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();

        // Build dependency graph
        let project = crate::services::project_analyzer::Project::new(project_path)?;
        let dag = project.build_dependency_graph().await?;
        let mut analyzer = crate::services::dead_code_analyzer::DeadCodeAnalyzer::new(1000);
        let report = analyzer.analyze_dependency_graph(&dag);

        // Convert dead functions
        for (index, item) in report.dead_functions.iter().enumerate() {
            if f64::from(item.confidence) >= config.min_confidence {
                defects.push(self.dead_code_item_to_defect(item, "DEAD-FN", index + 1));
            }
        }

        // Convert dead classes
        for (index, item) in report.dead_classes.iter().enumerate() {
            if f64::from(item.confidence) >= config.min_confidence {
                defects.push(self.dead_code_item_to_defect(item, "DEAD-CLS", index + 1));
            }
        }

        // Convert unreachable code
        for (index, block) in report.unreachable_code.iter().enumerate() {
            defects.push(self.unreachable_block_to_defect(block, index + 1));
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::DeadCode
    }

    fn supports_incremental(&self) -> bool {
        false // Requires full graph analysis
    }
}

impl DeadCodeDefectAnalyzer {
    fn dead_code_item_to_defect(&self, item: &DeadCodeItem, prefix: &str, index: usize) -> Defect {
        let severity = if item.confidence > 0.9 {
            Severity::High
        } else if item.confidence > 0.7 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let mut metrics = HashMap::new();
        metrics.insert("confidence".to_string(), f64::from(item.confidence));

        Defect {
            id: format!("{prefix}-{index:04}"),
            severity,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from(&item.file_path),
            line_start: item.line_number,
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!(
                "Dead {}: '{}' is never used (confidence: {:.0}%)",
                format!("{:?}", item.dead_type).to_lowercase(),
                item.name,
                item.confidence * 100.0
            ),
            rule_id: format!("dead-{}", format!("{:?}", item.dead_type).to_lowercase()),
            fix_suggestion: Some(format!(
                "Remove unused {} '{}'",
                format!("{:?}", item.dead_type).to_lowercase(),
                item.name
            )),
            metrics,
        }
    }

    fn unreachable_block_to_defect(&self, block: &UnreachableBlock, index: usize) -> Defect {
        let mut metrics = HashMap::new();
        metrics.insert(
            "lines".to_string(),
            f64::from(block.end_line - block.start_line + 1),
        );

        Defect {
            id: format!("UNREACH-{index:04}"),
            severity: Severity::High,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from(&block.file_path),
            line_start: block.start_line,
            line_end: Some(block.end_line),
            column_start: None,
            column_end: None,
            message: format!(
                "Unreachable code block ({} lines) - {}",
                block.end_line - block.start_line + 1,
                block.reason
            ),
            rule_id: "unreachable-code".to_string(),
            fix_suggestion: Some("Remove unreachable code".to_string()),
            metrics,
        }
    }
}
