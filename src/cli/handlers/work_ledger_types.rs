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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

/// Claim ID mapping table: (claim_id, keywords) -- first match wins.
/// Order matters: more specific patterns must precede general ones.
const CLAIM_PATTERNS: &[(&str, &[&str])] = &[
    ("manifest", &["manifest", "files deleted", "baseline files"]),
    (
        "meta-falsification",
        &["meta-falsification", "falsification system", "falsifier"],
    ),
    (
        "coverage-gaming",
        &["coverage gaming", "coverage exclusion"],
    ),
    (
        "differential-coverage",
        &["differential coverage", "changed lines"],
    ),
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
    (
        "per-file-coverage",
        &["per-file coverage", "all files have"],
    ),
    ("lint", &["lint"]),
    // v3.1 defect churn prevention
    ("variant-coverage", &["match arm", "variant"]),
    ("fix-chain", &["fix-after-fix", "fix chain"]),
    (
        "cross-crate",
        &["cross-crate", "sibling project", "integration tests pass"],
    ),
    ("regression-gate", &["regression", "performance"]),
];
