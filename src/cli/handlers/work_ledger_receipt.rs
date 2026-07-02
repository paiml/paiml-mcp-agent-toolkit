/// Current receipt schema version written by this build (MACS F1).
pub const RECEIPT_SCHEMA_VERSION: u32 = 2;

fn default_receipt_schema_version() -> u32 {
    1
}

/// Immutable falsification receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReceipt {
    /// 1 = pre-MACS (agent fields absent from hash), 2 = MACS F1.
    /// Hash rules are keyed on this so v1 receipts verify under v1 rules.
    #[serde(default = "default_receipt_schema_version")]
    pub schema_version: u32,
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
    /// Which agent configuration produced this receipt (MACS F1).
    /// None on v1 receipts and when no provenance was declared or detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentProvenance>,
    /// Interruption events recorded on the ticket (refusals, model switches).
    /// Never silent: a Refusal blocks completion until acknowledged (MACS E5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_events: Vec<AgentEvent>,
    /// Ladder level the contract claimed at falsification time (MACS-006).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_level: Option<String>,
    /// Evidence-computed ladder level at falsification time (MACS-006).
    /// Recorded as history — authority stays with the live computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub achieved_level: Option<String>,
    /// SHA-256 content hash (covers all fields above; rules keyed on
    /// schema_version — v1: legacy byte-concat, v2: canonical JSON)
    pub content_hash: String,
}

impl FalsificationReceipt {
    /// Build a receipt from a FalsificationReport + context
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            schema_version: RECEIPT_SCHEMA_VERSION,
            id,
            git_sha,
            timestamp,
            trigger,
            work_item_id,
            verdicts,
            overrides,
            summary,
            agent: None,
            agent_events: Vec::new(),
            claimed_level: None,
            achieved_level: None,
            content_hash: String::new(), // Computed below
        };

        receipt.content_hash = receipt.compute_content_hash();
        receipt
    }

    /// Attach agent provenance + interruption events and re-seal the content
    /// hash (MACS-002). Consumes and returns the receipt so the sealed hash
    /// can never be forgotten.
    #[must_use]
    pub fn with_agent(
        mut self,
        agent: Option<AgentProvenance>,
        agent_events: Vec<AgentEvent>,
    ) -> Self {
        self.agent = agent;
        self.agent_events = agent_events;
        self.content_hash = self.compute_content_hash();
        self
    }

    /// Record the claimed and evidence-computed ladder levels and re-seal
    /// the content hash (MACS-006).
    #[must_use]
    pub fn with_ladder(mut self, claimed: String, achieved: String) -> Self {
        self.claimed_level = Some(claimed);
        self.achieved_level = Some(achieved);
        self.content_hash = self.compute_content_hash();
        self
    }

    /// Compute SHA-256 hash of receipt content (excluding content_hash itself).
    /// Rules are keyed on schema_version (MACS F1): v1 receipts keep the
    /// legacy byte-concat rules so pre-MACS receipts still verify; v2 hashes
    /// canonical JSON of every field, covering agent provenance and events.
    fn compute_content_hash(&self) -> String {
        if self.schema_version >= 2 {
            self.compute_content_hash_v2()
        } else {
            self.compute_content_hash_v1()
        }
    }

    /// Legacy (schema_version = 1) hash: raw concatenated bytes, agent fields
    /// and summary counts excluded. Frozen — do not extend.
    fn compute_content_hash_v1(&self) -> String {
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
        hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>()
    }

    /// MACS (schema_version >= 2) hash: SHA-256 over canonical JSON of the
    /// whole receipt with content_hash blanked. Canonical = object keys
    /// sorted, None/empty optional fields omitted (skip_serializing_if), so
    /// the hash is stable under both field reordering and future additive
    /// schema evolution (contracts/macs-provenance-v1.yaml#hash_stability).
    fn compute_content_hash_v2(&self) -> String {
        let mut hashable = self.clone();
        hashable.content_hash = String::new();
        let value = serde_json::to_value(&hashable)
            .expect("receipt serialization is infallible: string keys, finite floats");
        let canonical = canonical_json(&value);
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>()
    }

    /// Verify content hash integrity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn verify_integrity(&self) -> bool {
        self.content_hash == self.compute_content_hash()
    }

    /// Check if receipt is fresh (matches HEAD SHA and within max_age)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            reason: format!(
                "Override approved via --override-claims (ticket: {})",
                ticket_id
            ),
        })
        .collect()
}

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

/// Render a JSON value canonically: object keys sorted, arrays in order,
/// scalars via serde_json (itoa/ryu — deterministic across platforms).
/// Sorting is explicit because serde_json in this workspace is feature-unified
/// with preserve_order (insertion-ordered maps).
fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            let entries: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    let key = serde_json::Value::String(k.clone());
                    format!("{}:{}", canonical_json(&key), canonical_json(&map[k]))
                })
                .collect();
            format!("{{{}}}", entries.join(","))
        }
        serde_json::Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(canonical_json).collect();
            format!("[{}]", inner.join(","))
        }
        scalar => serde_json::to_string(scalar)
            .expect("scalar JSON serialization is infallible"),
    }
}

/// Get current git SHA from project path
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn get_current_git_sha(project_path: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
