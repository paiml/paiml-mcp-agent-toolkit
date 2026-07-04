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
    /// Agent summary {model, effort, harness} when the receipt carries
    /// provenance (MACS-002). Absent on pre-MACS lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<LedgerAgentSummary>,
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
            agent: receipt.agent.as_ref().map(|a| LedgerAgentSummary {
                model: a.model.clone(),
                effort: a.effort.clone(),
                harness: a.harness.clone(),
            }),
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
    /// Google Anti-Gravity CLI or subagent (MACS-018)
    GoogleAntiGravity,
    /// Any other runner, verbatim
    Other(String),
}

impl AgentHarness {
    /// Known harness tokens in canonical kebab-case (underscores normalize).
    const KNOWN_TOKENS: [(&'static str, AgentHarness); 11] = [
        ("claude-code", AgentHarness::ClaudeCode),
        ("claudecode", AgentHarness::ClaudeCode),
        ("claude-agent-sdk", AgentHarness::ClaudeAgentSdk),
        ("ultracode-workflow", AgentHarness::UltracodeWorkflow),
        ("ultracode", AgentHarness::UltracodeWorkflow),
        ("ci-pipeline", AgentHarness::CiPipeline),
        ("ci", AgentHarness::CiPipeline),
        ("human", AgentHarness::Human),
        ("google-anti-gravity", AgentHarness::GoogleAntiGravity),
        ("agy", AgentHarness::GoogleAntiGravity),
        ("antigravity", AgentHarness::GoogleAntiGravity),
    ];

    /// Parse a declared harness token (kebab-case CLI/env form; underscores
    /// normalize to hyphens). Unknown tokens are preserved verbatim as `Other`.
    pub fn parse_token(s: &str) -> Self {
        let canonical = s.trim().to_lowercase().replace('_', "-");
        Self::KNOWN_TOKENS
            .iter()
            .find(|(token, _)| *token == canonical)
            .map(|(_, harness)| harness.clone())
            .unwrap_or_else(|| Self::Other(s.trim().to_string()))
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
    /// Anti-Gravity task ID (MACS-018)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Anti-Gravity conversation ID (MACS-018)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
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
    /// Acknowledgement of a prior blocking event (MACS-003). Events are
    /// append-only, so an ack is itself a reason-carrying event; a Refusal
    /// with no matching Ack blocks `pmat work complete`.
    Ack {
        /// ISO 8601 timestamp
        at: String,
        /// Record id of the acknowledged event (WorkEventRecord::id)
        ack_of: String,
        /// Root cause + disposition (must be non-empty)
        reason: String,
    },
}

/// A single line in `.pmat-work/<TICKET>/events.jsonl` (MACS-003).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkEventRecord {
    /// Short record id used by `--ack-event` (e.g. "ev-0197f0...")
    pub id: String,
    /// ISO 8601 timestamp when the record was written
    pub recorded_at: String,
    /// The event payload
    pub event: AgentEvent,
}


// ============================================================================
// MACS F1 — provenance resolution (MACS-002): declared-first, detection advisory
// Spec: docs/specifications/components/modern-agentic-coding-support.md §4-F1
// ============================================================================

/// Declared agent fields as received from `--agent-*` flags. clap also fills
/// them from `PMAT_AGENT_*` env vars; flags and env both count as "declared"
/// provenance (MACS E9) — advisory detection is separate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclaredAgent {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub harness: Option<String>,
    pub workflow_id: Option<String>,
    pub parent: Option<String>,
}

impl DeclaredAgent {
    fn any(&self) -> bool {
        self.model.is_some()
            || self.effort.is_some()
            || self.harness.is_some()
            || self.workflow_id.is_some()
            || self.parent.is_some()
    }
}

/// Resolve provenance from declared flags plus advisory env detection.
/// Declared always wins; detection only fills gaps and downgrades `source`
/// to `mixed`/`detected`. Returns None when nothing is declared or detected —
/// receipts then carry `agent: null` rather than fabricated provenance.
pub fn resolve_agent_provenance(declared: &DeclaredAgent) -> Option<AgentProvenance> {
    resolve_agent_provenance_with_env(declared, &|key| std::env::var(key).ok())
}

/// Testable core of [`resolve_agent_provenance`]: env access is injected.
pub fn resolve_agent_provenance_with_env(
    declared: &DeclaredAgent,
    env: &dyn Fn(&str) -> Option<String>,
) -> Option<AgentProvenance> {
    let detected_effort = env("CLAUDE_CODE_EFFORT_LEVEL");
    let detected_claude_code = detected_effort.is_some()
        || env("CLAUDECODE").is_some()
        || env("CLAUDE_CODE_ENTRYPOINT").is_some();

    let declared_any = declared.any();
    let mut used_detection = false;

    let effort = match &declared.effort {
        Some(e) => e.clone(),
        None => match &detected_effort {
            Some(e) => {
                used_detection = true;
                e.clone()
            }
            None => "unspecified".to_string(),
        },
    };
    let harness = match &declared.harness {
        Some(h) => AgentHarness::parse_token(h),
        None if detected_claude_code => {
            used_detection = true;
            AgentHarness::ClaudeCode
        }
        None => AgentHarness::Other("unspecified".to_string()),
    };

    if !declared_any && !used_detection {
        return None;
    }

    let source = match (declared_any, used_detection) {
        (true, false) => ProvenanceSource::Declared,
        (true, true) => ProvenanceSource::Mixed,
        (false, _) => ProvenanceSource::Detected,
    };

    Some(AgentProvenance {
        model: declared
            .model
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        effort,
        harness,
        workflow_id: declared.workflow_id.clone(),
        parent: declared.parent.clone(),
        task_id: None,
        conversation_id: None,
        source,
    })
}

/// Compact agent summary carried on JSONL ledger entries (MACS-002).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerAgentSummary {
    pub model: String,
    pub effort: String,
    pub harness: AgentHarness,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod agent_harness_token_tests {
    use super::*;

    #[test]
    fn agent_harness_parse_token_covers_all_variants() {
        // VariantCoverage: every arm of AgentHarness::parse_token
        for (token, want) in [
            ("claude-code", AgentHarness::ClaudeCode),
            ("claude_code", AgentHarness::ClaudeCode),
            ("claudecode", AgentHarness::ClaudeCode),
            ("Claude-Code", AgentHarness::ClaudeCode),
            ("claude-agent-sdk", AgentHarness::ClaudeAgentSdk),
            ("claude_agent_sdk", AgentHarness::ClaudeAgentSdk),
            ("ultracode-workflow", AgentHarness::UltracodeWorkflow),
            ("ultracode_workflow", AgentHarness::UltracodeWorkflow),
            ("ultracode", AgentHarness::UltracodeWorkflow),
            ("ci-pipeline", AgentHarness::CiPipeline),
            ("ci_pipeline", AgentHarness::CiPipeline),
            ("ci", AgentHarness::CiPipeline),
            ("human", AgentHarness::Human),
            ("HUMAN", AgentHarness::Human),
        ] {
            assert_eq!(AgentHarness::parse_token(token), want, "token {token}");
        }
        assert_eq!(
            AgentHarness::parse_token(" my-runner "),
            AgentHarness::Other("my-runner".to_string()),
            "unknown tokens preserved verbatim (trimmed)"
        );
    }
}
