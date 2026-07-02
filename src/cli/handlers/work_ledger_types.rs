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
    /// From receipt.
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

// ============================================================================
// MACS F1 — Agent provenance types (Component 32, MACS-001)
// Spec: docs/specifications/components/modern-agentic-coding-support.md §4-F1
// Contract: contracts/macs-provenance-v1.yaml
// ============================================================================

/// Which kind of runner produced the work (MACS F1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentHarness {
    /// Claude Code, interactive or `-p` (headless)
    ClaudeCode,
    /// Claude Agent SDK
    ClaudeAgentSdk,
    /// Dynamic-workflow subagent spawned by ultracode
    UltracodeWorkflow,
    /// CI pipeline
    CiPipeline,
    /// A human ran the command directly
    Human,
    /// Any other runner, verbatim
    Other(String),
}

impl AgentHarness {
    /// Parse a declared harness token (kebab-case CLI/env form).
    /// Unknown tokens are preserved verbatim as `Other`.
    pub fn parse_token(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "claude-code" | "claude_code" | "claudecode" => Self::ClaudeCode,
            "claude-agent-sdk" | "claude_agent_sdk" => Self::ClaudeAgentSdk,
            "ultracode-workflow" | "ultracode_workflow" | "ultracode" => Self::UltracodeWorkflow,
            "ci-pipeline" | "ci_pipeline" | "ci" => Self::CiPipeline,
            "human" => Self::Human,
            _ => Self::Other(s.trim().to_string()),
        }
    }
}

/// How provenance was captured: declared flags are canonical, env detection
/// is advisory (MACS E9).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceSource {
    /// From explicit `--agent-*` flags or `PMAT_AGENT_*` env
    Declared,
    /// Inferred from harness markers (e.g. CLAUDE_CODE_EFFORT_LEVEL)
    Detected,
    /// Some fields declared, some detected
    Mixed,
}

/// Declared-first provenance for a falsification receipt (MACS F1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentProvenance {
    /// e.g. "claude-fable-5" — API model id, verbatim
    pub model: String,
    /// "low" | "medium" | "high" | "xhigh" | "max" — as sent to the model
    pub effort: String,
    /// Which kind of runner produced the work
    pub harness: AgentHarness,
    /// Ultracode workflow id, if any (MACS E2)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    /// Parent agent/session id for nested subagents (MACS E2)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Declared (flags) | detected (env) | mixed
    pub source: ProvenanceSource,
}

/// Interruptions that must never be silent (MACS E5).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentEvent {
    /// A turn ended in a refusal (flagged request in non-interactive mode)
    Refusal {
        /// ISO 8601 timestamp
        at: String,
        /// Optional operator note
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    /// The harness switched models mid-loop
    ModelSwitch {
        /// ISO 8601 timestamp
        at: String,
        /// Model id before the switch, verbatim
        from: String,
        /// Model id after the switch, verbatim
        to: String,
    },
    /// The session was restarted (workflow runs are session-bound, MACS E7)
    SessionRestart {
        /// ISO 8601 timestamp
        at: String,
    },
    /// A dynamic workflow fanned out subagents
    WorkflowSpawn {
        /// ISO 8601 timestamp
        at: String,
        /// Workflow id
        workflow_id: String,
        /// Number of subagents spawned
        subagents: u32,
    },
}
