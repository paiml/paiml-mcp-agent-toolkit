// CLI handlers for `pmat work claim` (ULTRA-002).
//
// Every handler refuses loudly rather than returning a soft success: a claim
// that was not granted exits non-zero, a release of a path nobody holds is an
// error, and pointing `-p` at a directory that does not exist is an error
// instead of "0 claims, all clear".

use crate::cli::commands::QaOutputFormat;

/// One path an acquire could not take, with the evidence needed to unblock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClaimConflict {
    /// Path that is already spoken for
    pub path: String,
    /// The active claim that covers it (may be a parent directory)
    pub held_path: String,
    /// Agent holding it
    pub held_by: String,
    /// Journal record that granted it
    pub held_record: String,
    /// When the holder's claim lapses
    pub expires_at: String,
    /// Seconds until it lapses (negative once expired)
    pub seconds_remaining: i64,
    /// Holder's ticket, if declared
    pub work_item_id: Option<String>,
}

/// Machine-readable result of `pmat work claim acquire`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClaimAcquireOutcome {
    /// Whether the agent now owns every requested path
    pub granted: bool,
    /// Requesting agent
    pub agent: String,
    /// Journal record id, when granted
    pub record_id: Option<String>,
    /// Normalized paths requested
    pub paths: Vec<String>,
    /// Lifetime of the claim in seconds
    pub ttl_secs: u64,
    /// Paths that blocked the acquire
    pub conflicts: Vec<FileClaimConflict>,
    /// Agents whose lapsed claims this acquire took over
    pub superseded_expired: Vec<String>,
}

/// Resolve and validate the project path a claim journal lives under.
fn claim_project_path(path: Option<PathBuf>) -> Result<PathBuf> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    if !project_path.exists() {
        anyhow::bail!(
            "work claim: path does not exist: {}",
            project_path.display()
        );
    }
    Ok(project_path)
}

/// Normalize, de-duplicate and order the requested paths.
fn normalize_claim_paths(raw: &[String], project_root: &Path) -> Result<Vec<String>> {
    if raw.is_empty() {
        anyhow::bail!("work claim: no paths given; a claim over nothing is not a claim");
    }
    let mut out: Vec<String> = Vec::new();
    for r in raw {
        let normalized = normalize_claim_path(r, project_root)?;
        if !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out.sort();
    Ok(out)
}

/// Build a conflict row from the active claim that blocks `requested`.
fn conflict_from(
    requested: &str,
    held: &ActiveFileClaim,
    now: chrono::DateTime<chrono::Utc>,
) -> FileClaimConflict {
    let seconds_remaining = chrono::DateTime::parse_from_rfc3339(&held.expires_at)
        .map(|t| (t.with_timezone(&chrono::Utc) - now).num_seconds())
        .unwrap_or(0);
    FileClaimConflict {
        path: requested.to_string(),
        held_path: held.path.clone(),
        held_by: held.agent.clone(),
        held_record: held.record_id.clone(),
        expires_at: held.expires_at.clone(),
        seconds_remaining,
        work_item_id: held.work_item_id.clone(),
    }
}

/// Live foreign claims covering any requested path.
fn find_conflicts(
    active: &[ActiveFileClaim],
    paths: &[String],
    agent: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<FileClaimConflict> {
    let mut conflicts = Vec::new();
    for path in paths {
        for held in active {
            if held.agent != agent && !held.expired && claim_paths_overlap(&held.path, path) {
                conflicts.push(conflict_from(path, held, now));
            }
        }
    }
    conflicts
}

/// Agents whose lapsed claims an acquire would take over.
fn superseded_expired_agents(
    active: &[ActiveFileClaim],
    paths: &[String],
    agent: &str,
) -> Vec<String> {
    let mut agents: Vec<String> = active
        .iter()
        .filter(|c| {
            c.expired && c.agent != agent && paths.iter().any(|p| claim_paths_overlap(&c.path, p))
        })
        .map(|c| c.agent.clone())
        .collect();
    agents.sort();
    agents.dedup();
    agents
}

/// `pmat work claim acquire` — take every path or take none.
#[allow(clippy::too_many_arguments)]
pub async fn handle_work_claim_acquire(
    paths: Vec<String>,
    agent: String,
    ttl_secs: u64,
    work_item: Option<String>,
    note: Option<String>,
    force_reason: Option<String>,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = claim_project_path(path)?;
    if ttl_secs == 0 {
        anyhow::bail!(
            "work claim: --ttl 0 would create a claim that never lapses; give a positive TTL"
        );
    }
    let wanted = normalize_claim_paths(&paths, &project_path)?;
    let ledger = FileClaimLedger::new(&project_path);
    let now = chrono::Utc::now();
    let active = ledger.active_claims(now)?;

    let conflicts = find_conflicts(&active, &wanted, &agent, now);
    if !conflicts.is_empty() && force_reason.is_none() {
        let outcome = FileClaimAcquireOutcome {
            granted: false,
            agent,
            record_id: None,
            paths: wanted,
            ttl_secs,
            conflicts,
            superseded_expired: Vec::new(),
        };
        render_acquire_outcome(&outcome, format);
        anyhow::bail!(
            "work claim: {} path(s) are held by another agent; nothing was claimed",
            outcome.conflicts.len()
        );
    }

    let mut record = new_acquire_record(&agent, wanted.clone(), ttl_secs, now);
    record.work_item_id = work_item;
    record.note = note;
    record.forced_reason = force_reason;
    record.superseded_expired = superseded_expired_agents(&active, &wanted, &agent);
    ledger.append(&record)?;

    confirm_or_yield(&ledger, &record, &wanted, now)?;

    let outcome = FileClaimAcquireOutcome {
        granted: true,
        agent,
        record_id: Some(record.id.clone()),
        paths: wanted,
        ttl_secs,
        conflicts: Vec::new(),
        superseded_expired: record.superseded_expired.clone(),
    };
    render_acquire_outcome(&outcome, format);
    Ok(())
}

/// Re-read the journal after appending and confirm this record won every path.
/// Two agents that append in the same instant both land in the file; the fold
/// awards each path to whichever line came first, so the loser must hand the
/// paths back instead of believing its own write.
fn confirm_or_yield(
    ledger: &FileClaimLedger,
    record: &FileClaimRecord,
    wanted: &[String],
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let settled = ledger.active_claims(now)?;
    let lost: Vec<&String> = wanted
        .iter()
        .filter(|p| {
            !settled
                .iter()
                .any(|c| c.record_id == record.id && &&c.path == p)
        })
        .collect();
    if lost.is_empty() {
        return Ok(());
    }
    ledger.append(&new_release_record(
        &record.agent,
        record.paths.clone(),
        now,
    ))?;
    anyhow::bail!(
        "work claim: lost an append race for {} path(s) ({}); the claim was rolled back and \
         nothing is held. Retry after the holder releases.",
        lost.len(),
        lost.iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// `pmat work claim release` — give paths back.
pub async fn handle_work_claim_release(
    paths: Vec<String>,
    agent: String,
    all: bool,
    force_reason: Option<String>,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = claim_project_path(path)?;
    let ledger = FileClaimLedger::new(&project_path);
    let now = chrono::Utc::now();
    let active = ledger.active_claims(now)?;

    let wanted = release_targets(&paths, all, &agent, &active, &project_path)?;
    let foreign: Vec<&ActiveFileClaim> = active
        .iter()
        .filter(|c| c.agent != agent && wanted.contains(&c.path))
        .collect();
    if !foreign.is_empty() && force_reason.is_none() {
        anyhow::bail!(
            "work claim release: {} path(s) are held by another agent ({}); \
             pass --force --reason <why> to take them",
            foreign.len(),
            foreign
                .iter()
                .map(|c| format!("{} -> {}", c.path, c.agent))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let mut record = new_release_record(&agent, wanted.clone(), now);
    record.forced_reason = force_reason;
    ledger.append(&record)?;
    render_release(&record, format);
    Ok(())
}

/// Resolve which paths a release affects; refuses when it would release nothing.
fn release_targets(
    paths: &[String],
    all: bool,
    agent: &str,
    active: &[ActiveFileClaim],
    project_root: &Path,
) -> Result<Vec<String>> {
    if all {
        if !paths.is_empty() {
            anyhow::bail!("work claim release: --all takes no paths");
        }
        let mut held: Vec<String> = active
            .iter()
            .filter(|c| c.agent == agent)
            .map(|c| c.path.clone())
            .collect();
        held.sort();
        held.dedup();
        if held.is_empty() {
            anyhow::bail!("work claim release: agent '{agent}' holds nothing; nothing to release");
        }
        return Ok(held);
    }
    let wanted = normalize_claim_paths(paths, project_root)?;
    let unheld: Vec<&String> = wanted
        .iter()
        .filter(|p| !active.iter().any(|c| &&c.path == p))
        .collect();
    if !unheld.is_empty() {
        anyhow::bail!(
            "work claim release: no active claim on {}; a release that frees nothing is not a release",
            unheld.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
        );
    }
    Ok(wanted)
}

/// `pmat work claim list` — who owns what right now.
pub async fn handle_work_claim_list(
    agent: Option<String>,
    include_expired: bool,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = claim_project_path(path)?;
    let ledger = FileClaimLedger::new(&project_path);
    let now = chrono::Utc::now();
    let claims: Vec<ActiveFileClaim> = ledger
        .active_claims(now)?
        .into_iter()
        .filter(|c| include_expired || !c.expired)
        .filter(|c| agent.as_ref().is_none_or(|a| &c.agent == a))
        .collect();
    render_claim_list(&claims, include_expired, &ledger.journal_path(), format);
    Ok(())
}

/// `pmat work claim check` — ask before doing the work, exit non-zero if taken.
pub async fn handle_work_claim_check(
    paths: Vec<String>,
    agent: Option<String>,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = claim_project_path(path)?;
    let wanted = normalize_claim_paths(&paths, &project_path)?;
    let ledger = FileClaimLedger::new(&project_path);
    let now = chrono::Utc::now();
    let active = ledger.active_claims(now)?;
    let asking_as = agent.unwrap_or_else(|| "\u{0}none".to_string());
    let conflicts = find_conflicts(&active, &wanted, &asking_as, now);

    render_check(&wanted, &conflicts, format);
    if conflicts.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "work claim check: {} of {} path(s) are claimed by another agent",
        conflicts.len(),
        wanted.len()
    )
}
