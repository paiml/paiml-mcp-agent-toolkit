#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{CrossCrateBaseline, CrossCrateReport};
use anyhow::Result;
use std::path::Path;

// --- Ratchet baseline ---

/// Compute the ratchet threshold for a given rule.
/// MinHash-based rules (CC-001, CC-003, CC-005) get 25% tolerance because
/// probabilistic signatures and lazy source loading cause +/-15-30% variance.
/// Deterministic rules (CC-002, CC-004) use exact comparison.
pub(super) fn ratchet_threshold(rule: &str, baseline_count: usize) -> usize {
    match rule {
        "CC-001" | "CC-003" | "CC-005" => baseline_count + baseline_count / 4,
        _ => baseline_count,
    }
}

pub(super) fn save_ratchet_baseline(
    report: &CrossCrateReport,
    workspace_path: &Path,
) -> Result<()> {
    let baseline = CrossCrateBaseline::from_report(report);
    baseline.save(workspace_path)?;
    eprintln!(
        "Baseline saved to .pmat/cross-crate-baseline.json ({} findings)",
        baseline.total_findings
    );
    Ok(())
}

impl CrossCrateBaseline {
    pub(super) fn from_report(report: &CrossCrateReport) -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Howard Hinnant's civil date algorithm
        let z = (secs / 86400) as i64 + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = (z - era * 146097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        let generated = format!("{y:04}-{m:02}-{d:02}");

        Self {
            version: "1.0".to_string(),
            generated,
            rule_counts: report.summary.rules_triggered.clone(),
            total_findings: report.summary.total_findings,
        }
    }

    pub(super) fn save(&self, workspace_path: &Path) -> Result<()> {
        let pmat_dir = workspace_path.join(".pmat");
        std::fs::create_dir_all(&pmat_dir)?;
        let baseline_path = pmat_dir.join("cross-crate-baseline.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(baseline_path, content)?;
        Ok(())
    }

    pub(super) fn load(workspace_path: &Path) -> Option<Self> {
        let baseline_path = workspace_path
            .join(".pmat")
            .join("cross-crate-baseline.json");
        let content = std::fs::read_to_string(baseline_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Check if any rule count increased vs the baseline.
    /// Returns list of (rule, old_count, new_count) violations.
    ///
    /// Uses a 25% tolerance margin for MinHash-based rules (CC-001, CC-003, CC-005)
    /// because source lazy-loading and probabilistic signatures cause +/-15-30% variance
    /// between runs. Deterministic rules (CC-002, CC-004) use exact comparison.
    pub(super) fn check_ratchet(&self, report: &CrossCrateReport) -> Vec<(String, usize, usize)> {
        let mut violations = Vec::new();

        for (rule, &new_count) in &report.summary.rules_triggered {
            let old_count = self.rule_counts.get(rule).copied().unwrap_or(0);
            let threshold = ratchet_threshold(rule, old_count);
            if new_count > threshold {
                violations.push((rule.clone(), old_count, new_count));
            }
        }

        // Also check total with 25% tolerance
        let total_threshold = self.total_findings + self.total_findings / 4;
        if report.summary.total_findings > total_threshold {
            let already_has_total = violations.iter().any(|(r, _, _)| r == "TOTAL");
            if !already_has_total {
                violations.push((
                    "TOTAL".to_string(),
                    self.total_findings,
                    report.summary.total_findings,
                ));
            }
        }

        violations
    }
}
