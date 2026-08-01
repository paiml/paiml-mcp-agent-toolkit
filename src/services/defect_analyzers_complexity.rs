// ComplexityDefectAnalyzer: trait impl and helper methods for TDG-based complexity detection.

/// Repository root enclosing `start`, or `start` itself when there is none.
///
/// Git history must be read relative to the ANALYSED path, never to the
/// process CWD: `TDGCalculator::new()` defaults `project_root` to `"."`, so
/// `pmat report -p X` read churn from whatever repository the caller happened
/// to stand in, and aborted with `No git repository found at "."` when the
/// caller stood outside one at all.
pub(crate) fn enclosing_repo_root(start: &Path) -> PathBuf {
    let absolute = start
        .canonicalize()
        .unwrap_or_else(|_| start.to_path_buf());
    let mut cursor: Option<&Path> = Some(absolute.as_path());
    while let Some(dir) = cursor {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
        cursor = dir.parent();
    }
    absolute
}

#[async_trait]
impl DefectAnalyzer for ComplexityDefectAnalyzer {
    type Config = ComplexityConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();
        let tdg_calculator = crate::services::tdg_calculator::TDGCalculator::new()
            .with_project_root(enclosing_repo_root(project_path));

        // Analyze all source files
        let files = discover_source_files(project_path).await?;
        let scores = tdg_calculator.calculate_batch(files.clone()).await?;

        let mut index = 0;
        for (file_path, score) in files.into_iter().zip(scores) {
            if score.value > config.max_tdg_score {
                index += 1;
                defects.push(self.tdg_score_to_defect(file_path, score, index, &config));
            }
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Complexity
    }

    fn supports_incremental(&self) -> bool {
        true
    }
}

impl ComplexityDefectAnalyzer {
    fn tdg_score_to_defect(
        &self,
        file_path: PathBuf,
        score: TDGScore,
        index: usize,
        config: &ComplexityConfig,
    ) -> Defect {
        let severity = match score.severity {
            TDGSeverity::Critical => Severity::Critical,
            TDGSeverity::Warning => Severity::High,
            TDGSeverity::Normal => {
                if score.value > config.high_threshold {
                    Severity::Medium
                } else {
                    Severity::Low
                }
            }
        };

        let mut metrics = HashMap::new();
        metrics.insert("tdg_score".to_string(), score.value);
        metrics.insert("complexity_factor".to_string(), score.components.complexity);
        metrics.insert("churn_factor".to_string(), score.components.churn);
        metrics.insert("coupling_factor".to_string(), score.components.coupling);
        metrics.insert("domain_risk".to_string(), score.components.domain_risk);
        metrics.insert(
            "duplication_factor".to_string(),
            score.components.duplication,
        );
        metrics.insert("confidence".to_string(), score.confidence);

        Defect {
            id: format!("CPLX-{index:04}"),
            severity,
            category: DefectCategory::Complexity,
            file_path,
            line_start: 1, // TDG is file-level
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!(
                "File has high complexity with TDG score of {:.2} (threshold: {:.1})",
                score.value, config.max_tdg_score
            ),
            rule_id: "tdg-complexity".to_string(),
            fix_suggestion: Some(self.generate_fix_suggestion(&score)),
            metrics,
        }
    }

    fn generate_fix_suggestion(&self, score: &TDGScore) -> String {
        let mut suggestions = Vec::new();

        if score.components.complexity > 0.7 {
            suggestions.push("reduce cyclomatic complexity by extracting methods");
        }
        if score.components.coupling > 0.7 {
            suggestions.push("reduce coupling by improving module boundaries");
        }
        if score.components.duplication > 0.5 {
            suggestions.push("eliminate code duplication");
        }

        if suggestions.is_empty() {
            "Consider refactoring to reduce overall complexity".to_string()
        } else {
            format!("Consider: {}", suggestions.join(", "))
        }
    }
}
