#![cfg_attr(coverage_nightly, coverage(off))]
//! Falsification Ledger: Append-only receipt tracking for work completion
//!
//! Every `pmat work complete` produces an immutable FalsificationReceipt that:
//! - Records what was tested, what passed/failed, and any overrides
//! - Gates work completion (stale or failing receipts block)
//! - Appends to a global JSONL ledger for audit trails
//!
//! Storage layout:
//! ```text
//! .pmat-work/
//! ├── {item-id}/
//! │   ├── contract.json
//! │   └── falsification/
//! │       └── receipt-2026-02-14T10-30-00Z.json
//! └── ledger.jsonl
//! ```

use super::work_falsification::{ClaimResult, FalsificationReport};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Maximum age (in seconds) for a receipt to be considered fresh
const MAX_RECEIPT_AGE_SECS: u64 = 86400; // 24 hours

/// What triggered the falsification run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FalsificationTrigger {
    /// Triggered by `pmat work complete`
    WorkComplete,
    /// Triggered by manual CLI invocation
    ManualCli,
    /// Triggered by CI pipeline
    CiPipeline,
    /// Triggered by MCP tool
    McpTool,
    /// Triggered by pre-commit hook
    PreCommit,
}

/// Per-claim verdict in the receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationVerdict {
    /// Hypothesis that was tested
    pub hypothesis: String,
    /// Method used (as string for readability in JSON)
    pub method: String,
    /// Whether the claim was falsified (true = problem found)
    pub falsified: bool,
    /// Whether this was a blocking check
    pub is_blocking: bool,
    /// Human-readable explanation
    pub explanation: String,
    /// Summary of evidence (if any)
    pub evidence_summary: Option<String>,
}

/// Override record for accountability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimOverride {
    /// Name of the overridden claim
    pub claim_id: String,
    /// Accountability ticket
    pub ticket: String,
    /// Reason for override
    pub reason: String,
}

/// Summary of receipt for quick checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptSummary {
    /// Total claims tested
    pub total: usize,
    /// Claims that passed
    pub passed: usize,
    /// Claims that failed (blocking)
    pub failed: usize,
    /// Claims with warnings (non-blocking)
    pub warnings: usize,
    /// Claims overridden
    pub overridden: usize,
    /// Whether this receipt allows work completion
    pub allows_completion: bool,
    /// Health score 0.0-1.0 (passed / total)
    pub health_score: f64,
}

/// Immutable falsification receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReceipt {
    /// UUID v7 (time-sortable)
    pub id: String,
    /// Git SHA at time of falsification
    pub git_sha: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// What triggered this falsification
    pub trigger: FalsificationTrigger,
    /// Work item ID
    pub work_item_id: String,
    /// Per-claim verdicts
    pub verdicts: Vec<FalsificationVerdict>,
    /// Override records
    pub overrides: Vec<ClaimOverride>,
    /// Aggregate summary
    pub summary: ReceiptSummary,
    /// SHA-256 content hash (covers all fields above)
    pub content_hash: String,
}

impl FalsificationReceipt {
    /// Build a receipt from a FalsificationReport + context
    pub fn from_report(
        report: &FalsificationReport,
        git_sha: String,
        work_item_id: String,
        trigger: FalsificationTrigger,
        override_claims: Option<&Vec<String>>,
        ticket: Option<&String>,
    ) -> Self {
        let id = Uuid::now_v7().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let verdicts: Vec<FalsificationVerdict> = report
            .claim_results
            .iter()
            .map(|cr| FalsificationVerdict {
                hypothesis: cr.hypothesis.clone(),
                method: format!("{:?}", cr.method),
                falsified: cr.result.falsified,
                is_blocking: cr.is_blocking,
                explanation: cr.result.explanation.clone(),
                evidence_summary: cr.result.evidence.as_ref().map(|e| format!("{:?}", e)),
            })
            .collect();

        // Build override records from --override-claims + --ticket
        let overrides = build_overrides(&report.claim_results, override_claims, ticket);
        let overridden_count = overrides.len();

        // Compute summary
        let total = report.total_claims;
        let passed = report.passed;
        let failed = report.failed;
        let warnings = report.warnings;

        // Completion allowed if no unoverridden blocking failures
        let unoverridden_blocking = report
            .claim_results
            .iter()
            .filter(|cr| cr.result.falsified && cr.is_blocking)
            .filter(|cr| {
                let claim_name = hypothesis_to_claim_id(&cr.hypothesis);
                !overrides.iter().any(|o| o.claim_id == claim_name)
            })
            .count();
        let allows_completion = unoverridden_blocking == 0;

        let health_score = if total > 0 {
            passed as f64 / total as f64
        } else {
            1.0
        };

        let summary = ReceiptSummary {
            total,
            passed,
            failed,
            warnings,
            overridden: overridden_count,
            allows_completion,
            health_score,
        };

        let mut receipt = Self {
            id,
            git_sha,
            timestamp,
            trigger,
            work_item_id,
            verdicts,
            overrides,
            summary,
            content_hash: String::new(), // Computed below
        };

        receipt.content_hash = receipt.compute_content_hash();
        receipt
    }

    /// Compute SHA-256 hash of receipt content (excluding content_hash itself)
    fn compute_content_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.git_sha.as_bytes());
        hasher.update(self.timestamp.as_bytes());
        hasher.update(format!("{:?}", self.trigger).as_bytes());
        hasher.update(self.work_item_id.as_bytes());
        for v in &self.verdicts {
            hasher.update(v.hypothesis.as_bytes());
            hasher.update(v.method.as_bytes());
            hasher.update(if v.falsified { b"T" } else { b"F" });
            hasher.update(if v.is_blocking { b"T" } else { b"F" });
            hasher.update(v.explanation.as_bytes());
        }
        for o in &self.overrides {
            hasher.update(o.claim_id.as_bytes());
            hasher.update(o.ticket.as_bytes());
        }
        hasher.update(format!("{}", self.summary.allows_completion).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify content hash integrity
    pub fn verify_integrity(&self) -> bool {
        self.content_hash == self.compute_content_hash()
    }

    /// Check if receipt is fresh (matches HEAD SHA and within max_age)
    pub fn is_fresh(&self, current_sha: &str, max_age_secs: u64) -> bool {
        if self.git_sha != current_sha {
            return false;
        }
        let Ok(receipt_time) = chrono::DateTime::parse_from_rfc3339(&self.timestamp) else {
            return false;
        };
        let age = chrono::Utc::now()
            .signed_duration_since(receipt_time)
            .num_seconds();
        age >= 0 && (age as u64) <= max_age_secs
    }
}

/// Build override records from CLI args
fn build_overrides(
    claim_results: &[ClaimResult],
    override_claims: Option<&Vec<String>>,
    ticket: Option<&String>,
) -> Vec<ClaimOverride> {
    let Some(overrides) = override_claims else {
        return Vec::new();
    };
    let Some(ticket_id) = ticket else {
        return Vec::new();
    };

    claim_results
        .iter()
        .filter(|cr| cr.result.falsified && cr.is_blocking)
        .filter(|cr| {
            let claim_id = hypothesis_to_claim_id(&cr.hypothesis);
            overrides
                .iter()
                .any(|o| o.to_lowercase() == claim_id.to_lowercase())
        })
        .map(|cr| ClaimOverride {
            claim_id: hypothesis_to_claim_id(&cr.hypothesis),
            ticket: ticket_id.clone(),
            reason: format!("Override approved via --override-claims (ticket: {})", ticket_id),
        })
        .collect()
}

/// Claim ID mapping table: (claim_id, keywords) — first match wins.
/// Order matters: more specific patterns must precede general ones.
const CLAIM_PATTERNS: &[(&str, &[&str])] = &[
    ("manifest", &["manifest", "files deleted", "baseline files"]),
    ("meta-falsification", &["meta-falsification", "falsification system", "falsifier"]),
    ("coverage-gaming", &["coverage gaming", "coverage exclusion"]),
    ("differential-coverage", &["differential coverage", "changed lines"]),
    ("coverage", &["total coverage", "coverage >= 95"]),
    ("tdg", &["tdg"]),
    ("complexity", &["complexity"]),
    ("supply-chain", &["supply chain", "vulnerable dependencies"]),
    ("file-size", &["file size", "500 lines"]),
    ("spec-quality", &["spec", "specification"]),
    ("github-sync", &["github", "changes pushed"]),
    ("book", &["book", "pmat-book"]),
    ("satd", &["satd", "todo/fixme"]),
    ("dead-code", &["dead code"]),
    ("per-file-coverage", &["per-file coverage", "all files have"]),
    ("lint", &["lint"]),
    // v3.1 defect churn prevention
    ("variant-coverage", &["match arm", "variant"]),
    ("fix-chain", &["fix-after-fix", "fix chain"]),
    ("cross-crate", &["cross-crate", "sibling project", "integration tests pass"]),
    ("regression-gate", &["regression", "performance"]),
];

/// Convert hypothesis text to a stable claim ID (mirrors claim_to_override_name in core_handlers)
fn hypothesis_to_claim_id(hypothesis: &str) -> String {
    let h = hypothesis.to_lowercase();
    // "examples" + "compile" requires conjunctive match (both keywords)
    if h.contains("examples") && h.contains("compile") {
        return "examples".to_string();
    }
    for &(claim_id, keywords) in CLAIM_PATTERNS {
        if keywords.iter().any(|kw| h.contains(kw)) {
            return claim_id.to_string();
        }
    }
    // Fallback: sanitize hypothesis into a slug
    h.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(30)
        .collect()
}

/// Compact JSONL entry for global ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Receipt ID
    pub receipt_id: String,
    /// Work item ID
    pub work_item_id: String,
    /// Timestamp
    pub timestamp: String,
    /// Git SHA
    pub git_sha: String,
    /// Trigger type
    pub trigger: FalsificationTrigger,
    /// Quick summary
    pub passed: usize,
    pub failed: usize,
    pub overridden: usize,
    pub allows_completion: bool,
    /// Content hash for cross-reference
    pub content_hash: String,
}

impl LedgerEntry {
    pub fn from_receipt(receipt: &FalsificationReceipt) -> Self {
        Self {
            receipt_id: receipt.id.clone(),
            work_item_id: receipt.work_item_id.clone(),
            timestamp: receipt.timestamp.clone(),
            git_sha: receipt.git_sha.clone(),
            trigger: receipt.trigger.clone(),
            passed: receipt.summary.passed,
            failed: receipt.summary.failed,
            overridden: receipt.summary.overridden,
            allows_completion: receipt.summary.allows_completion,
            content_hash: receipt.content_hash.clone(),
        }
    }
}

/// Integrity report from ledger verification
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub total: usize,
    pub valid: usize,
    pub tampered: usize,
    pub missing: usize,
}

/// Falsification ledger service
pub struct FalsificationLedger {
    /// Root of .pmat-work/ directory
    work_dir: PathBuf,
}

impl FalsificationLedger {
    pub fn new(project_path: &Path) -> Self {
        Self {
            work_dir: project_path.join(".pmat-work"),
        }
    }

    /// Persist receipt to per-item falsification directory
    pub fn persist_receipt(&self, receipt: &FalsificationReceipt) -> Result<PathBuf> {
        let falsification_dir = self
            .work_dir
            .join(&receipt.work_item_id)
            .join("falsification");
        std::fs::create_dir_all(&falsification_dir)
            .context("Failed to create falsification directory")?;

        // Sanitize timestamp for filename (replace : with -)
        let safe_ts = receipt.timestamp.replace(':', "-");
        let filename = format!("receipt-{}.json", safe_ts);
        let path = falsification_dir.join(filename);

        let json = serde_json::to_string_pretty(receipt)
            .context("Failed to serialize receipt")?;
        std::fs::write(&path, json).context("Failed to write receipt")?;

        Ok(path)
    }

    /// Append compact entry to global JSONL ledger
    pub fn append_to_ledger(&self, receipt: &FalsificationReceipt) -> Result<()> {
        std::fs::create_dir_all(&self.work_dir)
            .context("Failed to create .pmat-work directory")?;

        let ledger_path = self.work_dir.join("ledger.jsonl");
        let entry = LedgerEntry::from_receipt(receipt);
        let mut line = serde_json::to_string(&entry)
            .context("Failed to serialize ledger entry")?;
        line.push('\n');

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ledger_path)
            .context("Failed to open ledger.jsonl")?;
        file.write_all(line.as_bytes())
            .context("Failed to append to ledger")?;

        Ok(())
    }

    /// Load the latest receipt for a work item (by sorted directory listing)
    pub fn latest_receipt(&self, work_item_id: &str) -> Result<Option<FalsificationReceipt>> {
        let falsification_dir = self
            .work_dir
            .join(work_item_id)
            .join("falsification");

        if !falsification_dir.exists() {
            return Ok(None);
        }

        let mut receipt_files: Vec<PathBuf> = std::fs::read_dir(&falsification_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension().map(|e| e == "json").unwrap_or(false)
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("receipt-"))
                        .unwrap_or(false)
            })
            .collect();

        // Sort by filename (which encodes timestamp) — latest last
        receipt_files.sort();

        let Some(latest_path) = receipt_files.last() else {
            return Ok(None);
        };

        let content = std::fs::read_to_string(latest_path)
            .context("Failed to read latest receipt")?;
        let receipt: FalsificationReceipt = serde_json::from_str(&content)
            .context("Failed to parse receipt JSON")?;

        Ok(Some(receipt))
    }

    /// O(1) freshness check: load latest receipt and verify it matches HEAD
    pub fn has_fresh_receipt(&self, work_item_id: &str, current_sha: &str) -> Result<bool> {
        let receipt = self.latest_receipt(work_item_id)?;
        match receipt {
            Some(r) => Ok(r.is_fresh(current_sha, MAX_RECEIPT_AGE_SECS) && r.summary.allows_completion),
            None => Ok(false),
        }
    }

    /// Verify integrity of all receipts for a work item
    pub fn verify_integrity(&self, work_item_id: &str) -> Result<IntegrityReport> {
        let falsification_dir = self
            .work_dir
            .join(work_item_id)
            .join("falsification");

        if !falsification_dir.exists() {
            return Ok(IntegrityReport { total: 0, valid: 0, tampered: 0, missing: 0 });
        }

        let json_files: Vec<PathBuf> = std::fs::read_dir(&falsification_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
            .collect();

        let mut valid = 0;
        let mut tampered = 0;
        let mut missing = 0;

        for path in &json_files {
            match Self::check_receipt_file(path) {
                Ok(true) => valid += 1,
                Ok(false) => tampered += 1,
                Err(_) => missing += 1,
            }
        }

        Ok(IntegrityReport { total: json_files.len(), valid, tampered, missing })
    }

    /// Check a single receipt file for integrity (Ok(true) = valid, Ok(false) = tampered)
    fn check_receipt_file(path: &Path) -> Result<bool> {
        let content = std::fs::read_to_string(path)?;
        let receipt: FalsificationReceipt = serde_json::from_str(&content)?;
        Ok(receipt.verify_integrity())
    }
}

/// Get current git SHA from project path
pub fn get_current_git_sha(project_path: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::work_contract::{
        EvidenceType, FalsificationMethod, FalsificationResult,
    };

    fn make_report(all_passed: bool) -> FalsificationReport {
        let mut results = vec![ClaimResult {
            index: 1,
            hypothesis: "All baseline files still exist".to_string(),
            method: FalsificationMethod::ManifestIntegrity,
            result: FalsificationResult::passed("All 10 files present"),
            is_blocking: true,
        }];

        if !all_passed {
            results.push(ClaimResult {
                index: 2,
                hypothesis: "Total coverage >= 95%".to_string(),
                method: FalsificationMethod::AbsoluteCoverage,
                result: FalsificationResult::failed(
                    "80.0% < 95.0%",
                    EvidenceType::NumericComparison {
                        actual: 80.0,
                        threshold: 95.0,
                    },
                ),
                is_blocking: true,
            });
        }

        let passed = results.iter().filter(|r| !r.result.falsified).count();
        let failed = results
            .iter()
            .filter(|r| r.result.falsified && r.is_blocking)
            .count();

        FalsificationReport {
            total_claims: results.len(),
            passed,
            failed,
            warnings: 0,
            all_passed,
            claim_results: results,
        }
    }

    #[test]
    fn receipt_from_report_all_passed() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "PMAT-100".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(receipt.summary.allows_completion);
        assert_eq!(receipt.summary.passed, 1);
        assert_eq!(receipt.summary.failed, 0);
        assert_eq!(receipt.summary.overridden, 0);
        assert_eq!(receipt.git_sha, "abc123");
        assert_eq!(receipt.work_item_id, "PMAT-100");
        assert!(!receipt.content_hash.is_empty());
    }

    #[test]
    fn receipt_from_report_with_failures() {
        let report = make_report(false);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "def456".to_string(),
            "PMAT-101".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(!receipt.summary.allows_completion);
        assert_eq!(receipt.summary.passed, 1);
        assert_eq!(receipt.summary.failed, 1);
        assert_eq!(receipt.verdicts.len(), 2);
    }

    #[test]
    fn receipt_from_report_with_overrides() {
        let report = make_report(false);
        let overrides = vec!["coverage".to_string()];
        let ticket = "DEBT-001".to_string();
        let receipt = FalsificationReceipt::from_report(
            &report,
            "ghi789".to_string(),
            "PMAT-102".to_string(),
            FalsificationTrigger::WorkComplete,
            Some(&overrides),
            Some(&ticket),
        );

        assert!(receipt.summary.allows_completion);
        assert_eq!(receipt.summary.overridden, 1);
        assert_eq!(receipt.overrides[0].claim_id, "coverage");
        assert_eq!(receipt.overrides[0].ticket, "DEBT-001");
    }

    #[test]
    fn content_hash_stable() {
        let report = make_report(true);
        let r1 = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::ManualCli,
            None,
            None,
        );
        // Hash is based on id + timestamp which are unique, so verify it's non-empty and valid hex
        assert!(!r1.content_hash.is_empty());
        assert!(r1.content_hash.chars().all(|c| c.is_ascii_hexdigit()));
        // Re-computing on same receipt should match
        assert!(r1.verify_integrity());
    }

    #[test]
    fn content_hash_detects_tampering() {
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::CiPipeline,
            None,
            None,
        );

        assert!(receipt.verify_integrity());

        // Tamper with a field
        receipt.git_sha = "tampered".to_string();
        assert!(!receipt.verify_integrity());
    }

    #[test]
    fn freshness_matches_sha() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(receipt.is_fresh("abc123", MAX_RECEIPT_AGE_SECS));
    }

    #[test]
    fn freshness_stale_sha() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        assert!(!receipt.is_fresh("different_sha", MAX_RECEIPT_AGE_SECS));
    }

    #[test]
    fn freshness_expired() {
        let report = make_report(true);
        let mut receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        // Set timestamp to 48 hours ago
        let old_time = chrono::Utc::now() - chrono::Duration::hours(48);
        receipt.timestamp = old_time.to_rfc3339();

        assert!(!receipt.is_fresh("abc123", MAX_RECEIPT_AGE_SECS));
    }

    #[test]
    fn ledger_persist_and_load() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc123".to_string(),
            "PMAT-200".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        // Persist
        let path = ledger.persist_receipt(&receipt).unwrap();
        assert!(path.exists());

        // Load latest
        let loaded = ledger.latest_receipt("PMAT-200").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, receipt.id);
        assert_eq!(loaded.git_sha, "abc123");
        assert!(loaded.verify_integrity());
    }

    #[test]
    fn ledger_append_only() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let r1 = FalsificationReceipt::from_report(
            &report,
            "sha1".to_string(),
            "ITEM-1".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        let r2 = FalsificationReceipt::from_report(
            &report,
            "sha2".to_string(),
            "ITEM-2".to_string(),
            FalsificationTrigger::ManualCli,
            None,
            None,
        );

        ledger.append_to_ledger(&r1).unwrap();
        ledger.append_to_ledger(&r2).unwrap();

        let ledger_path = temp_dir.path().join(".pmat-work/ledger.jsonl");
        let content = std::fs::read_to_string(&ledger_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        // Verify each line is valid JSON
        let entry1: LedgerEntry = serde_json::from_str(lines[0]).unwrap();
        let entry2: LedgerEntry = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(entry1.work_item_id, "ITEM-1");
        assert_eq!(entry2.work_item_id, "ITEM-2");
    }

    #[test]
    fn latest_receipt_returns_most_recent() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let r1 = FalsificationReceipt::from_report(
            &report,
            "sha_old".to_string(),
            "PMAT-300".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );

        // Small delay to ensure different timestamps in filenames
        ledger.persist_receipt(&r1).unwrap();

        // Create second receipt with different SHA
        let r2 = FalsificationReceipt::from_report(
            &report,
            "sha_new".to_string(),
            "PMAT-300".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&r2).unwrap();

        let latest = ledger.latest_receipt("PMAT-300").unwrap().unwrap();
        assert_eq!(latest.git_sha, "sha_new");
    }

    #[test]
    fn has_fresh_receipt_true() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "current_head".to_string(),
            "PMAT-400".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&receipt).unwrap();

        assert!(ledger.has_fresh_receipt("PMAT-400", "current_head").unwrap());
    }

    #[test]
    fn has_fresh_receipt_false_stale_sha() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "old_sha".to_string(),
            "PMAT-401".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&receipt).unwrap();

        assert!(!ledger.has_fresh_receipt("PMAT-401", "new_sha").unwrap());
    }

    #[test]
    fn has_fresh_receipt_false_no_receipts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        assert!(!ledger.has_fresh_receipt("PMAT-999", "any_sha").unwrap());
    }

    #[test]
    fn verify_integrity_report() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "PMAT-500".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        ledger.persist_receipt(&receipt).unwrap();

        let integrity = ledger.verify_integrity("PMAT-500").unwrap();
        assert_eq!(integrity.total, 1);
        assert_eq!(integrity.valid, 1);
        assert_eq!(integrity.tampered, 0);
        assert_eq!(integrity.missing, 0);
    }

    #[test]
    fn verify_integrity_detects_tampering() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ledger = FalsificationLedger::new(temp_dir.path());

        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "PMAT-501".to_string(),
            FalsificationTrigger::WorkComplete,
            None,
            None,
        );
        let path = ledger.persist_receipt(&receipt).unwrap();

        // Tamper with the file
        let mut content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        content["git_sha"] = serde_json::Value::String("tampered".to_string());
        std::fs::write(&path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let integrity = ledger.verify_integrity("PMAT-501").unwrap();
        assert_eq!(integrity.total, 1);
        assert_eq!(integrity.tampered, 1);
        assert_eq!(integrity.valid, 0);
    }

    #[test]
    fn hypothesis_to_claim_id_mapping() {
        assert_eq!(
            hypothesis_to_claim_id("All baseline files still exist"),
            "manifest"
        );
        assert_eq!(
            hypothesis_to_claim_id("Total coverage >= 95%"),
            "coverage"
        );
        assert_eq!(
            hypothesis_to_claim_id("TDG score >= baseline"),
            "tdg"
        );
        assert_eq!(
            hypothesis_to_claim_id("No coverage exclusion gaming"),
            "coverage-gaming"
        );
        assert_eq!(
            hypothesis_to_claim_id("make lint passes"),
            "lint"
        );
        assert_eq!(
            hypothesis_to_claim_id("No dead code introduced"),
            "dead-code"
        );
        // v3.1 defect churn claims
        assert_eq!(
            hypothesis_to_claim_id("All match arm variants have test coverage"),
            "variant-coverage"
        );
        assert_eq!(
            hypothesis_to_claim_id("No fix-after-fix chains exceed limit"),
            "fix-chain"
        );
        assert_eq!(
            hypothesis_to_claim_id("Cross-crate integration tests pass"),
            "cross-crate"
        );
        assert_eq!(
            hypothesis_to_claim_id("No performance regressions detected"),
            "regression-gate"
        );
    }

    #[test]
    fn ledger_entry_from_receipt() {
        let report = make_report(true);
        let receipt = FalsificationReceipt::from_report(
            &report,
            "abc".to_string(),
            "X-1".to_string(),
            FalsificationTrigger::PreCommit,
            None,
            None,
        );
        let entry = LedgerEntry::from_receipt(&receipt);
        assert_eq!(entry.receipt_id, receipt.id);
        assert_eq!(entry.work_item_id, "X-1");
        assert_eq!(entry.trigger, FalsificationTrigger::PreCommit);
        assert!(entry.allows_completion);
    }

    #[test]
    fn receipt_trigger_variants() {
        // Verify all trigger variants serialize/deserialize
        let triggers = vec![
            FalsificationTrigger::WorkComplete,
            FalsificationTrigger::ManualCli,
            FalsificationTrigger::CiPipeline,
            FalsificationTrigger::McpTool,
            FalsificationTrigger::PreCommit,
        ];
        for trigger in triggers {
            let json = serde_json::to_string(&trigger).unwrap();
            let deserialized: FalsificationTrigger = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, trigger);
        }
    }
}
