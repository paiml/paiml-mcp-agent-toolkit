// MACS-019 delegation — `pmat work delegate` (issue #985).
//
// What this replaced: the handler found the roadmap item, threw it away, and
// printed `✅ MACS-019: Task delegated and provenance boundaries preserved.`
// Nothing was forwarded, no provenance boundary was written, no journal line
// was appended, and the exit code was 0. `--agy` swapped one word of the
// banner ("Google Anti-Gravity" for "Agent") and changed nothing else.
//
// What it does now: writes the two artifacts the sentence claims.
//
//   1. A **handoff bundle** at `.pmat-work/<ID>/delegation/handoff-<rec>.json`
//      carrying the task context the receiving agent needs — title, status,
//      priority, spec path, GitHub issue and every acceptance criterion. That
//      is "task context forwarding" as a file a second agent can read.
//   2. A **provenance boundary** on the ticket's append-only journal
//      (`events.jsonl`, MACS-003) as `AgentEvent::Delegation`, naming the
//      delegating agent, the target, the bundle and its sha256.
//
// And it refuses when the boundary cannot be established: a boundary needs
// BOTH sides named, so a delegation whose delegator is unidentified exits
// non-zero and writes nothing, rather than journalling `agent: null` and
// calling that a preserved boundary.
//
// This is the target format question of #984 turned around: the bundle is
// PMAT's own artifact, consumed by PMAT's own ledger, so no external schema
// has to be guessed. `--agy` selects the recorded target and the harness token
// on the boundary record; it does not invent a Google Anti-Gravity file layout.

/// Schema tag stored in every handoff bundle, so a reader can tell a v1 bundle
/// from whatever a later MACS-019 revision emits.
pub const DELEGATION_SCHEMA_VERSION: &str = "macs-019/1";

/// Lowercase hex sha256 of a byte slice — the ledger's one spelling, shared by
/// the receipt content hash and the delegation handoff digest.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Who the work is being handed to. The token is what lands on the journal,
/// so it must stay stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationTarget {
    /// A peer PMAT agent (default)
    Agent,
    /// Google Anti-Gravity (`--agy`)
    GoogleAntiGravity,
}

impl DelegationTarget {
    /// `--agy` selects the Anti-Gravity target.
    pub fn from_agy_flag(agy: bool) -> Self {
        if agy {
            DelegationTarget::GoogleAntiGravity
        } else {
            DelegationTarget::Agent
        }
    }

    /// Stable token written to the bundle and the journal.
    pub fn token(self) -> &'static str {
        match self {
            DelegationTarget::Agent => "agent",
            DelegationTarget::GoogleAntiGravity => "google-anti-gravity",
        }
    }

    /// The harness the receiving side is expected to run under. This is the
    /// same vocabulary `AgentHarness::parse_token` already accepts, so a
    /// receipt written by the delegate can be matched against the boundary.
    pub fn receiving_harness(self) -> AgentHarness {
        match self {
            DelegationTarget::Agent => AgentHarness::Other("unspecified".to_string()),
            DelegationTarget::GoogleAntiGravity => AgentHarness::GoogleAntiGravity,
        }
    }
}

/// The forwarded task context. Every field is copied from the roadmap item —
/// nothing here is synthesised, so an empty acceptance-criteria list means the
/// ticket has none, not that forwarding failed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationHandoff {
    /// Schema tag ([`DELEGATION_SCHEMA_VERSION`])
    pub schema_version: String,
    /// Ticket being delegated
    pub work_item_id: String,
    /// Human-readable title (the field `contract.json` lacks — see #984)
    pub title: String,
    /// Ticket status at the moment of delegation
    pub status: String,
    /// Ticket priority at the moment of delegation
    pub priority: String,
    /// GitHub issue number, when the ticket is synced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_issue: Option<u64>,
    /// Specification path, when the ticket names one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    /// Every acceptance criterion, verbatim
    pub acceptance_criteria: Vec<String>,
    /// When the handoff was written (ISO 8601)
    pub delegated_at: String,
    /// Target token ([`DelegationTarget::token`])
    pub target: String,
    /// Harness the receiving agent is expected to declare
    pub receiving_harness: AgentHarness,
    /// Resolved provenance of the delegating agent — the near side of the
    /// boundary. Never `None`: an unidentified delegator is refused.
    pub delegated_by: AgentProvenance,
}

/// What a successful delegation wrote. Returned so the caller can print paths
/// it did not have to guess at, and so tests assert on artifacts rather than
/// on stdout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationOutcome {
    /// Journal record id of the boundary event
    pub event_id: String,
    /// Absolute path of the handoff bundle
    pub handoff_path: PathBuf,
    /// sha256 of the handoff bundle bytes
    pub digest: String,
    /// The bundle that was written
    pub handoff: DelegationHandoff,
}

/// Read `PMAT_AGENT_*` into a [`DeclaredAgent`].
///
/// `work delegate` carries no `--agent-*` flags, but MACS E9 counts env and
/// flags equally as *declared* provenance, so the env vars have to be honoured
/// here or delegation would be unusable from the harnesses that set them.
pub fn declared_agent_from_env(env: &dyn Fn(&str) -> Option<String>) -> DeclaredAgent {
    let get = |k: &str| {
        env(k)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    DeclaredAgent {
        model: get("PMAT_AGENT_MODEL"),
        effort: get("PMAT_AGENT_EFFORT"),
        harness: get("PMAT_AGENT_HARNESS"),
        workflow_id: get("PMAT_AGENT_WORKFLOW_ID"),
        parent: get("PMAT_AGENT_PARENT"),
    }
}

/// The refusal used when the delegating agent cannot be identified.
///
/// Kept as its own function because it is the whole point of the ticket: the
/// unmeasured case has to be representable and has to fail, and the message
/// has to name what would make it measurable.
fn unidentified_delegator(id: &str, target: DelegationTarget) -> String {
    format!(
        "work delegate: refusing to delegate {id} to {}: the delegating agent is unidentified, \
         and a provenance boundary with only one side named is not a boundary.\n\
         \x20 Declare it and re-run, e.g.:\n\
         \x20   PMAT_AGENT_MODEL=<model> PMAT_AGENT_HARNESS=<claude-code|ultracode-workflow|\
         google-anti-gravity|ci-pipeline|human> pmat work delegate {id}\n\
         \x20 Nothing was written: no handoff bundle, no journal entry (MACS-019/MACS-002).",
        target.token()
    )
}

/// Build the handoff bundle from a roadmap item. Pure, so the mapping from
/// ticket to forwarded context is testable without touching a filesystem.
pub fn build_handoff(
    item: &crate::models::roadmap::RoadmapItem,
    target: DelegationTarget,
    delegated_by: AgentProvenance,
    now: &str,
) -> DelegationHandoff {
    DelegationHandoff {
        schema_version: DELEGATION_SCHEMA_VERSION.to_string(),
        work_item_id: item.id.clone(),
        title: item.title.clone(),
        status: format!("{:?}", item.status),
        priority: format!("{:?}", item.priority),
        github_issue: item.github_issue,
        spec: item.spec.as_ref().map(|p| p.display().to_string()),
        acceptance_criteria: item.acceptance_criteria.clone(),
        delegated_at: now.to_string(),
        target: target.token().to_string(),
        receiving_harness: target.receiving_harness(),
        delegated_by,
    }
}

/// Perform the delegation: write the handoff bundle, then append the boundary
/// event. Bundle first — an event that points at a bundle which does not exist
/// is exactly the fabricated-receipt shape this release is about.
///
/// Returns `Err` (writing nothing) when the delegating agent is unidentified.
pub fn delegate_work_item(
    item: &crate::models::roadmap::RoadmapItem,
    target: DelegationTarget,
    project_path: &Path,
    declared: &DeclaredAgent,
) -> Result<DelegationOutcome> {
    delegate_work_item_with_env(item, target, project_path, declared, &|key| {
        std::env::var(key).ok()
    })
}

/// Testable core of [`delegate_work_item`]: advisory harness detection reads
/// through the injected env, so a test can assert the refusal without the
/// ambient `CLAUDECODE`/`CLAUDE_CODE_*` markers of the harness running it
/// silently supplying the missing side of the boundary.
pub fn delegate_work_item_with_env(
    item: &crate::models::roadmap::RoadmapItem,
    target: DelegationTarget,
    project_path: &Path,
    declared: &DeclaredAgent,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<DelegationOutcome> {
    let provenance = resolve_agent_provenance_with_env(declared, env)
        .ok_or_else(|| anyhow::anyhow!(unidentified_delegator(&item.id, target)))?;

    let now = chrono::Utc::now().to_rfc3339();
    let handoff = build_handoff(item, target, provenance, &now);

    let ledger = FalsificationLedger::new(project_path);
    let dir = project_path
        .join(".pmat-work")
        .join(&item.id)
        .join("delegation");
    std::fs::create_dir_all(&dir).context("Failed to create delegation directory")?;

    let record = format!("d-{}", Uuid::now_v7().simple());
    let handoff_path = dir.join(format!("handoff-{record}.json"));
    let json =
        serde_json::to_string_pretty(&handoff).context("Failed to serialize delegation handoff")?;
    std::fs::write(&handoff_path, &json).context("Failed to write delegation handoff")?;

    let digest = sha256_hex(json.as_bytes());

    let event_id = ledger.append_event(
        &item.id,
        AgentEvent::Delegation {
            at: now,
            to: target.token().to_string(),
            handoff: handoff_path.display().to_string(),
            digest: digest.clone(),
            delegated_by: handoff.delegated_by.clone(),
        },
    )?;

    Ok(DelegationOutcome {
        event_id,
        handoff_path,
        digest,
        handoff,
    })
}

/// Render a completed delegation. Names both artifacts and both sides of the
/// boundary, so the transcript proves the delegation instead of asserting it.
pub fn render_delegation(outcome: &DelegationOutcome) -> String {
    let h = &outcome.handoff;
    let mut out = format!(
        "🤝 Delegated {} to {} (MACS-019)\n",
        h.work_item_id, h.target
    );
    out.push_str(&format!("    title: {}\n", h.title));
    out.push_str(&format!(
        "    from:  {} / {} (effort {}, provenance {:?})\n",
        h.delegated_by.model,
        h.delegated_by.harness.token(),
        h.delegated_by.effort,
        h.delegated_by.source
    ));
    out.push_str(&format!(
        "    handoff: {} ({} acceptance criterion/criteria forwarded)\n",
        outcome.handoff_path.display(),
        h.acceptance_criteria.len()
    ));
    out.push_str(&format!("    sha256:  {}\n", outcome.digest));
    out.push_str(&format!(
        "    boundary: {} on .pmat-work/{}/events.jsonl\n",
        outcome.event_id, h.work_item_id
    ));
    out.push_str("✓ task context forwarded and the provenance boundary is on the journal");
    out
}

/// Entry point used by `pmat work delegate`. Prints the outcome; the error
/// path prints nothing and exits non-zero.
pub fn run_work_delegate(
    item: &crate::models::roadmap::RoadmapItem,
    agy: bool,
    project_path: &Path,
) -> Result<()> {
    let target = DelegationTarget::from_agy_flag(agy);
    let declared = declared_agent_from_env(&|k| std::env::var(k).ok());
    let outcome = delegate_work_item(item, target, project_path, &declared)?;
    println!("{}", render_delegation(&outcome));
    Ok(())
}
