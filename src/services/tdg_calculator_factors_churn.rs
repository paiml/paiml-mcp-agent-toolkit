// Factor calculation: churn (git history + fallback)
// Split from tdg_calculator_factors.rs for complexity budget

fn churn_score_from_age(days_old: u64) -> f64 {
    if days_old < 7 {
        3.0
    } else if days_old < 30 {
        2.0
    } else if days_old < 90 {
        1.0
    } else {
        0.5
    }
}

impl TDGCalculator {
    /// Calculate churn factor based on git history
    async fn calculate_churn_factor(&self, path: &Path) -> Result<f64> {
        let analysis = self.get_or_compute_churn_analysis().await?;
        let relative_path = path.strip_prefix(&self.project_root).unwrap_or(path);

        if let Some(file_metrics) = analysis
            .files
            .iter()
            .find(|f| f.path == relative_path || f.relative_path == relative_path.to_string_lossy())
        {
            let monthly_rate = file_metrics.commit_count as f64 / 3.0;
            let normalized = (1.0 + monthly_rate).ln() / 2.0;
            Ok(normalized.min(5.0))
        } else {
            self.calculate_churn_fallback(path).await
        }
    }

    /// Get cached churn analysis or compute it once for the entire project
    pub async fn get_or_compute_churn_analysis(
        &self,
    ) -> Result<crate::models::churn::CodeChurnAnalysis> {
        let mut cache = self.cached_churn_analysis.lock().await;

        if let Some(ref analysis) = *cache {
            return Ok(analysis.clone());
        }

        tracing::info!("Computing churn analysis for project (this should only happen once)...");
        match GitAnalysisService::analyze_code_churn(&self.project_root, 90) {
            Ok(analysis) => {
                tracing::info!("Churn analysis computed successfully and cached");
                *cache = Some(analysis.clone());
                Ok(analysis)
            }
            Err(e) => {
                let empty_analysis = crate::models::churn::CodeChurnAnalysis {
                    generated_at: chrono::Utc::now(),
                    period_days: 90,
                    repository_root: self.project_root.clone(),
                    files: vec![],
                    summary: crate::models::churn::ChurnSummary {
                        total_commits: 0,
                        total_files_changed: 0,
                        hotspot_files: vec![],
                        stable_files: vec![],
                        author_contributions: HashMap::new(),
                        mean_churn_score: 0.0,
                        variance_churn_score: 0.0,
                        stddev_churn_score: 0.0,
                    },
                };
                *cache = Some(empty_analysis);
                Err(e.into())
            }
        }
    }

    /// Fallback churn calculation based on file modification time
    async fn calculate_churn_fallback(&self, path: &Path) -> Result<f64> {
        let Ok(metadata) = tokio::fs::metadata(path).await else {
            return Ok(1.0);
        };
        let Ok(modified) = metadata.modified() else {
            return Ok(1.0);
        };
        let Ok(elapsed) = modified.elapsed() else {
            return Ok(1.0);
        };
        Ok(churn_score_from_age(elapsed.as_secs() / 86400))
    }
}
