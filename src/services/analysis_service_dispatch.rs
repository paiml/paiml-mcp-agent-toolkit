// Service trait implementation for AnalysisService: process dispatch, validation, metrics
// Included from analysis_service.rs - shares parent scope (no use imports allowed)

#[async_trait::async_trait]
impl Service for AnalysisService {
    type Input = AnalysisInput;
    type Output = AnalysisOutput;
    type Error = anyhow::Error;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let start = std::time::Instant::now();

        let results = match input.operation {
            AnalysisOperation::Complexity => {
                let complexity = self.analyze_complexity(&input.path, &input.options).await?;
                AnalysisResults::Complexity(complexity)
            }
            AnalysisOperation::Satd => {
                let satd = self.analyze_satd(&input.path, &input.options).await?;
                AnalysisResults::Satd(satd)
            }
            AnalysisOperation::DeadCode => {
                let dead_code = self.analyze_dead_code(&input.path, &input.options).await?;
                AnalysisResults::DeadCode(dead_code)
            }
            AnalysisOperation::All => {
                let complexity = self.analyze_complexity(&input.path, &input.options).await?;
                let satd = self.analyze_satd(&input.path, &input.options).await?;
                let dead_code = self.analyze_dead_code(&input.path, &input.options).await?;

                AnalysisResults::Combined(CombinedResults {
                    complexity,
                    satd,
                    dead_code,
                })
            }
        };

        let duration = start.elapsed();
        let mut metrics = self.metrics.write().await;
        metrics.record_request(duration, true);

        // Calculate summary
        let (files_analyzed, total_issues, critical_issues) = match &results {
            AnalysisResults::Complexity(c) => (
                c.total_files,
                c.violations.len(),
                c.violations.iter().filter(|v| v.complexity > 20).count(),
            ),
            AnalysisResults::Satd(s) => (s.total_files, s.violations.len(), s.violations.len()),
            AnalysisResults::DeadCode(d) => (d.total_files, d.unused_items.len(), 0),
            AnalysisResults::Combined(c) => (
                c.complexity.total_files,
                c.complexity.violations.len()
                    + c.satd.violations.len()
                    + c.dead_code.unused_items.len(),
                c.complexity
                    .violations
                    .iter()
                    .filter(|v| v.complexity > 20)
                    .count()
                    + c.satd.violations.len(),
            ),
        };

        Ok(AnalysisOutput {
            operation: input.operation,
            results,
            summary: AnalysisSummary {
                files_analyzed,
                total_issues,
                critical_issues,
                duration_ms: duration.as_millis() as u64,
            },
        })
    }

    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError> {
        if !input.path.exists() {
            return Err(ValidationError::InvalidValue {
                field: "path".to_string(),
                reason: "Path does not exist".to_string(),
            });
        }

        if let Some(max) = input.options.max_complexity {
            if max == 0 || max > 100 {
                return Err(ValidationError::InvalidValue {
                    field: "max_complexity".to_string(),
                    reason: "Must be between 1 and 100".to_string(),
                });
            }
        }

        Ok(())
    }

    fn metrics(&self) -> ServiceMetrics {
        // Return a clone of current metrics
        self.metrics.blocking_read().clone()
    }

    fn name(&self) -> &'static str {
        "AnalysisService"
    }
}
