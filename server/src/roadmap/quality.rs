//! Quality gate enforcement for roadmap tasks

use super::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Quality check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityCheck {
    Complexity(u32),
    TestCoverage(u8),
    Documentation,
    NoSatd,
    LintCompliance,
    RoadmapUpdated,
}

/// Result of a quality check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check: QualityCheck,
    pub passed: bool,
    pub message: String,
    pub details: Option<String>,
}

/// Quality report for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub task_id: String,
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<CheckResult>,
    pub overall_passed: bool,
}

impl QualityReport {
    pub fn new(task_id: &str) -> Self {
        Self {
            task_id: task_id.to_string(),
            timestamp: Utc::now(),
            checks: Vec::new(),
            overall_passed: true,
        }
    }

    pub fn add_check_result(&mut self, _check: QualityCheck, result: CheckResult) {
        if !result.passed {
            self.overall_passed = false;
        }
        self.checks.push(result);
    }

    pub fn passed(&self) -> bool {
        self.overall_passed
    }
}

/// Task-level quality gate
pub struct TaskQualityGate {
    task_id: String,
    checks: Vec<QualityCheck>,
    config: QualityGateConfig,
}

impl TaskQualityGate {
    pub fn new(task_id: &str, config: QualityGateConfig) -> Self {
        let checks = vec![
            QualityCheck::Complexity(config.complexity_max),
            QualityCheck::NoSatd,
            QualityCheck::LintCompliance,
            QualityCheck::Documentation,
            QualityCheck::RoadmapUpdated,
        ];

        Self {
            task_id: task_id.to_string(),
            checks,
            config,
        }
    }

    pub async fn validate(&self) -> Result<QualityReport> {
        let mut report = QualityReport::new(&self.task_id);

        for check in &self.checks {
            let result = match check {
                QualityCheck::Complexity(max) => self.check_complexity(*max).await?,
                QualityCheck::TestCoverage(min) => self.check_coverage(*min).await?,
                QualityCheck::Documentation => self.check_documentation().await?,
                QualityCheck::NoSatd => self.check_satd().await?,
                QualityCheck::LintCompliance => self.check_lint().await?,
                QualityCheck::RoadmapUpdated => self.check_roadmap_status().await?,
            };

            report.add_check_result(check.clone(), result);
        }

        Ok(report)
    }

    async fn check_complexity(&self, max: u32) -> Result<CheckResult> {
        // Run complexity analysis
        let output = std::process::Command::new("pmat")
            .args([
                "analyze",
                "complexity",
                "--max-cyclomatic",
                &max.to_string(),
                "--format",
                "json",
            ])
            .output()?;

        let passed = output.status.success();
        let message = if passed {
            format!("Complexity within limit (≤ {})", max)
        } else {
            format!("Complexity exceeds limit (> {})", max)
        };

        Ok(CheckResult {
            check: QualityCheck::Complexity(max),
            passed,
            message,
            details: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        })
    }

    async fn check_coverage(&self, min: u8) -> Result<CheckResult> {
        // Run coverage check
        let output = std::process::Command::new("cargo")
            .args(["tarpaulin", "--print-summary"])
            .output()?;

        // Parse coverage percentage from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let coverage = parse_coverage_percentage(&stdout).unwrap_or(0.0);

        let passed = coverage >= min as f64;
        let message = if passed {
            format!("Test coverage sufficient ({:.1}% ≥ {}%)", coverage, min)
        } else {
            format!("Test coverage insufficient ({:.1}% < {}%)", coverage, min)
        };

        Ok(CheckResult {
            check: QualityCheck::TestCoverage(min),
            passed,
            message,
            details: Some(stdout.to_string()),
        })
    }

    async fn check_documentation(&self) -> Result<CheckResult> {
        // Check if documentation has been updated
        let output = std::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD~1", "HEAD"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let docs_updated = stdout
            .lines()
            .any(|line| line.contains("docs/") || line.ends_with(".md"));

        let passed = docs_updated || !self.config.documentation_required;
        let message = if passed {
            "Documentation updated".to_string()
        } else {
            "Documentation not updated".to_string()
        };

        Ok(CheckResult {
            check: QualityCheck::Documentation,
            passed,
            message,
            details: None,
        })
    }

    async fn check_satd(&self) -> Result<CheckResult> {
        // Run SATD check
        let output = std::process::Command::new("pmat")
            .args(["analyze", "satd", "--strict", "--format", "json"])
            .output()?;

        let passed = output.status.success() || self.config.satd_tolerance > 0;
        let message = if passed {
            "No SATD violations found".to_string()
        } else {
            "SATD violations detected".to_string()
        };

        Ok(CheckResult {
            check: QualityCheck::NoSatd,
            passed,
            message,
            details: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        })
    }

    async fn check_lint(&self) -> Result<CheckResult> {
        if !self.config.lint_compliance {
            return Ok(CheckResult {
                check: QualityCheck::LintCompliance,
                passed: true,
                message: "Lint check skipped".to_string(),
                details: None,
            });
        }

        // Run lint check
        let output = std::process::Command::new("make").args(["lint"]).output()?;

        let passed = output.status.success();
        let message = if passed {
            "No lint violations".to_string()
        } else {
            "Lint violations found".to_string()
        };

        Ok(CheckResult {
            check: QualityCheck::LintCompliance,
            passed,
            message,
            details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }

    async fn check_roadmap_status(&self) -> Result<CheckResult> {
        // Check if roadmap has been updated
        let output = std::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD~1", "HEAD"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let roadmap_updated = stdout.lines().any(|line| line.contains("roadmap.md"));

        let passed = roadmap_updated;
        let message = if passed {
            "Roadmap status updated".to_string()
        } else {
            "Roadmap status not updated".to_string()
        };

        Ok(CheckResult {
            check: QualityCheck::RoadmapUpdated,
            passed,
            message,
            details: None,
        })
    }
}

fn parse_coverage_percentage(output: &str) -> Option<f64> {
    // Look for pattern like "Coverage: 85.5%"
    for line in output.lines() {
        if line.contains("Coverage:") {
            if let Some(pos) = line.find(':') {
                let rest = &line[pos + 1..].trim();
                if let Some(percent_pos) = rest.find('%') {
                    let num_str = &rest[..percent_pos].trim();
                    return num_str.parse().ok();
                }
            }
        }
    }
    None
}
