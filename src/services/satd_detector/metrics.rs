//! Metrics computation and scoring for SATD analysis.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::models::error::TemplateError;

use super::types::{CategoryMetrics, SATDDetector, SATDMetrics, Severity, TechnicalDebt};

impl SATDDetector {
    /// Generate project-wide SATD metrics
    #[must_use]
    pub fn generate_metrics(&self, debts: &[TechnicalDebt], total_loc: u64) -> SATDMetrics {
        let debt_density = if total_loc > 0 {
            (debts.len() as f64 / total_loc as f64) * 1000.0
        } else {
            0.0
        };

        // Group by category
        let mut by_category: BTreeMap<String, CategoryMetrics> = BTreeMap::new();

        for debt in debts {
            let category_key = debt.category.to_string();
            let entry = by_category.entry(category_key).or_insert(CategoryMetrics {
                count: 0,
                files: BTreeSet::new(),
                avg_severity: 0.0,
            });

            entry.count += 1;
            entry.files.insert(debt.file.to_string_lossy().to_string());
        }

        // Calculate average severity for each category
        for (category_name, metrics) in &mut by_category {
            let category_debts: Vec<_> = debts
                .iter()
                .filter(|d| d.category.to_string() == *category_name)
                .collect();

            if !category_debts.is_empty() {
                let severity_sum: u32 = category_debts
                    .iter()
                    .map(|d| match d.severity {
                        Severity::Critical => 4,
                        Severity::High => 3,
                        Severity::Medium => 2,
                        Severity::Low => 1,
                    })
                    .sum();

                metrics.avg_severity = f64::from(severity_sum) / category_debts.len() as f64;
            }
        }

        let critical_debts: Vec<TechnicalDebt> = debts
            .iter()
            .filter(|d| d.severity == Severity::Critical)
            .cloned()
            .collect();

        SATDMetrics {
            total_debts: debts.len(),
            debt_density_per_kloc: debt_density,
            by_category,
            critical_debts,
            debt_age_distribution: vec![], // Would need git history analysis
        }
    }

    /// Calculate average age of technical debt items using git blame
    pub(crate) async fn calculate_average_debt_age(
        &self,
        debts: &[TechnicalDebt],
        project_root: &Path,
    ) -> Result<f64, TemplateError> {
        use chrono::Utc;

        let mut total_age_days = 0.0;
        let mut valid_debt_count = 0;
        let now = Utc::now();

        for debt in debts {
            if let Some(age_days) = self.calculate_debt_age(debt, project_root, &now).await {
                total_age_days += age_days;
                valid_debt_count += 1;
            }
        }

        Ok(if valid_debt_count > 0 {
            total_age_days / f64::from(valid_debt_count)
        } else {
            0.0
        })
    }

    async fn calculate_debt_age(
        &self,
        debt: &TechnicalDebt,
        project_root: &Path,
        now: &chrono::DateTime<chrono::Utc>,
    ) -> Option<f64> {
        let relative_path = self.get_relative_path(&debt.file, project_root)?;
        let blame_output = self
            .run_git_blame(&relative_path, debt.line, project_root)
            .await?;
        let timestamp = self.parse_git_blame_timestamp(&blame_output)?;
        self.calculate_age_from_timestamp(timestamp, now)
    }

    fn get_relative_path(&self, file_path: &Path, project_root: &Path) -> Option<PathBuf> {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        debug_assert!(
            project_root.exists(),
            "project_root must exist: {}",
            project_root.display()
        );
        file_path
            .strip_prefix(project_root)
            .ok()
            .map(std::path::Path::to_path_buf)
    }

    async fn run_git_blame(
        &self,
        relative_path: &Path,
        line: u32,
        project_root: &Path,
    ) -> Option<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args([
                "blame",
                "-L",
                &format!("{line},{line}"),
                "--porcelain",
                relative_path.to_str()?,
            ])
            .current_dir(project_root)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }

    fn parse_git_blame_timestamp(&self, blame_output: &str) -> Option<i64> {
        for line in blame_output.lines() {
            if let Some(timestamp_str) = line.strip_prefix("author-time ") {
                return timestamp_str.parse::<i64>().ok();
            }
        }
        None
    }

    fn calculate_age_from_timestamp(
        &self,
        timestamp: i64,
        now: &chrono::DateTime<chrono::Utc>,
    ) -> Option<f64> {
        use chrono::DateTime;

        let debt_date = DateTime::from_timestamp(timestamp, 0)?;
        Some((*now - debt_date).num_days() as f64)
    }
}
