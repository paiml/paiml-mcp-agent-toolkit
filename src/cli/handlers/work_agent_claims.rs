// Agent file claims (MACS F4 / ULTRA-002) — an enforced ownership invariant
// for multi-agent workflows.
//
// Measured problem: every ultracode prompt had to hand-partition file ownership
// across 6-11 concurrent agents. When the partition was wrong, work was lost —
// in one round FIVE findings returned "blocked-by-file-claim" having done
// nothing, because two agents needed the same file and neither could tell.
//
// This turns that prompt convention into an invariant the ledger enforces:
// `acquire` refuses on conflict (non-zero exit), `release` gives the paths
// back, and a crashed agent's claim expires by TTL instead of blocking the
// pool forever. Expiry is REPRESENTABLE, never silent: an expired claim is
// reported as expired and a supersession is recorded on the journal.
//
// Storage: `.pmat-work/claims.jsonl`, append-only, one JSON record per line —
// the same shape as `ledger.jsonl` and `events.jsonl`. State is a fold over
// the journal in file order; file order is also the tie-break that resolves a
// race between two agents appending at the same instant.

/// Default claim lifetime. A crashed agent must not hold a path forever, and
/// "no expiry" would make that the default outcome.
pub const DEFAULT_CLAIM_TTL_SECS: u64 = 3600;

/// Upper bound on a single appended line. POSIX guarantees an `O_APPEND` write
/// below `PIPE_BUF` (4096) is atomic; above it two concurrent agents can
/// interleave and corrupt the journal, so an over-large claim is refused
/// rather than silently risked.
const MAX_CLAIM_LINE_BYTES: usize = 4000;

/// What a journal line does to the claim state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileClaimAction {
    /// Take ownership of the listed paths
    Acquire,
    /// Give the listed paths back
    Release,
}

/// One line in `.pmat-work/claims.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileClaimRecord {
    /// Record id, e.g. "cl-0197f0..."
    pub id: String,
    /// ISO 8601 timestamp the record was written
    pub recorded_at: String,
    /// Acquire or release
    pub action: FileClaimAction,
    /// Agent identity (free-form; the workflow's name for the subagent)
    pub agent: String,
    /// Normalized, repo-relative paths
    pub paths: Vec<String>,
    /// When an acquire stops holding (RFC 3339). Absent on releases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Ticket this claim belongs to, if any
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    /// Operator note
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Reason a claim was taken from / released out from under another agent
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forced_reason: Option<String>,
    /// Agents whose expired claims this acquire superseded. Recorded so an
    /// expiry is auditable rather than a silent handover.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub superseded_expired: Vec<String>,
}

/// A path currently owned by an agent, as folded from the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveFileClaim {
    /// Journal record that granted it
    pub record_id: String,
    /// Owning agent
    pub agent: String,
    /// Normalized repo-relative path
    pub path: String,
    /// When it was taken
    pub acquired_at: String,
    /// When it lapses (RFC 3339)
    pub expires_at: String,
    /// Ticket, if declared
    pub work_item_id: Option<String>,
    /// Line index in the journal — the total order that settles a race
    pub seq: usize,
    /// True once `now` is past `expires_at`
    pub expired: bool,
}

/// Normalize one claim path to a repo-relative, slash-joined form.
///
/// Refuses rather than guesses: `..` is ambiguous under a shared root, globs
/// are not what prefix-claims mean, and an absolute path outside the project
/// cannot be claimed in this project's journal.
pub fn normalize_claim_path(raw: &str, project_root: &Path) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("claim path is empty");
    }
    if trimmed.contains('*') || trimmed.contains('?') {
        anyhow::bail!(
            "claim path '{trimmed}' contains a glob; claim the directory instead \
             (a directory claim covers everything beneath it)"
        );
    }
    let relative = strip_project_root(trimmed, project_root)?;
    join_normalized_components(&relative, trimmed)
}

/// Make an absolute path repo-relative, or refuse if it is outside the project.
fn strip_project_root(trimmed: &str, project_root: &Path) -> Result<String> {
    let candidate = Path::new(trimmed);
    if !candidate.is_absolute() {
        return Ok(trimmed.to_string());
    }
    let root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    match candidate.strip_prefix(&root) {
        Ok(rel) => Ok(rel.to_string_lossy().to_string()),
        Err(_) => anyhow::bail!(
            "claim path '{trimmed}' is outside the project root {}; \
             claims are recorded repo-relative",
            root.display()
        ),
    }
}

/// Drop `.` components, refuse `..`, and reject a path that normalizes to
/// nothing (which would silently claim the whole repository).
fn join_normalized_components(relative: &str, original: &str) -> Result<String> {
    let mut parts: Vec<&str> = Vec::new();
    for component in relative.split('/') {
        match component {
            "" | "." => continue,
            ".." => anyhow::bail!(
                "claim path '{original}' contains '..'; pass a path relative to \
                 the project root"
            ),
            other => parts.push(other),
        }
    }
    if parts.is_empty() {
        anyhow::bail!("claim path '{original}' normalizes to the project root; refusing");
    }
    Ok(parts.join("/"))
}

/// True when two normalized claims cover any common file: equal, or one is a
/// directory prefix of the other. `src/cli` does not overlap `src/cli_x`.
pub fn claim_paths_overlap(a: &str, b: &str) -> bool {
    a == b || is_dir_prefix(a, b) || is_dir_prefix(b, a)
}

fn is_dir_prefix(dir: &str, child: &str) -> bool {
    child.len() > dir.len() && child.starts_with(dir) && child.as_bytes()[dir.len()] == b'/'
}

/// Append-only claim journal over `.pmat-work/claims.jsonl`.
pub struct FileClaimLedger {
    work_dir: PathBuf,
}

impl FileClaimLedger {
    /// Open the journal for a project (the file need not exist yet).
    pub fn new(project_path: &Path) -> Self {
        Self {
            work_dir: project_path.join(".pmat-work"),
        }
    }

    /// Path to `claims.jsonl`.
    pub fn journal_path(&self) -> PathBuf {
        self.work_dir.join("claims.jsonl")
    }

    /// Every record in file order. A malformed line is an error, not a skip:
    /// a claim journal that quietly drops lines under-reports conflicts, which
    /// is the failure this command exists to prevent.
    pub fn load_records(&self) -> Result<Vec<FileClaimRecord>> {
        let path = self.journal_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&path).context("Failed to read claims.jsonl")?;
        let mut records = Vec::new();
        for (idx, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            records.push(
                serde_json::from_str::<FileClaimRecord>(line).with_context(|| {
                    format!("claims.jsonl line {} is not a claim record", idx + 1)
                })?,
            );
        }
        Ok(records)
    }

    /// Append one record. Refuses a line long enough to lose `O_APPEND`
    /// atomicity, because a torn line corrupts every later verdict.
    pub fn append(&self, record: &FileClaimRecord) -> Result<()> {
        use std::io::Write;
        std::fs::create_dir_all(&self.work_dir).context("Failed to create .pmat-work directory")?;
        let mut line = serde_json::to_string(record).context("Failed to serialize claim record")?;
        if line.len() > MAX_CLAIM_LINE_BYTES {
            anyhow::bail!(
                "claim record is {} bytes, over the {MAX_CLAIM_LINE_BYTES}-byte atomic-append \
                 limit; split it into several `pmat work claim acquire` calls",
                line.len()
            );
        }
        line.push('\n');
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.journal_path())
            .context("Failed to open claims.jsonl")?;
        file.write_all(line.as_bytes())
            .context("Failed to append claim record")?;
        Ok(())
    }

    /// Fold the journal into the set of paths currently owned, evaluated at
    /// `now`. Expired claims are retained and flagged, not dropped.
    pub fn active_claims(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ActiveFileClaim>> {
        Ok(fold_claims(&self.load_records()?, now))
    }
}

/// Pure fold used by [`FileClaimLedger::active_claims`]; `now` is injected so
/// expiry is testable without sleeping.
pub fn fold_claims(
    records: &[FileClaimRecord],
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<ActiveFileClaim> {
    let mut active: Vec<ActiveFileClaim> = Vec::new();
    for (seq, record) in records.iter().enumerate() {
        match record.action {
            FileClaimAction::Acquire => apply_acquire(&mut active, record, seq, now),
            FileClaimAction::Release => apply_release(&mut active, record),
        }
    }
    for claim in &mut active {
        claim.expired = is_expired(&claim.expires_at, now);
    }
    active
}

/// An acquire takes a path only if nothing live and foreign already covers it.
/// The agent's own overlapping claim is replaced (a refresh), and a live
/// foreign claim makes this acquire a no-op — that is how the loser of an
/// append race is decided, deterministically, by file order.
fn apply_acquire(
    active: &mut Vec<ActiveFileClaim>,
    record: &FileClaimRecord,
    seq: usize,
    now: chrono::DateTime<chrono::Utc>,
) {
    let expires_at = record.expires_at.clone().unwrap_or_default();
    for path in &record.paths {
        let blocked = active.iter().any(|c| {
            c.agent != record.agent
                && claim_paths_overlap(&c.path, path)
                && !is_expired(&c.expires_at, now)
        });
        if blocked && record.forced_reason.is_none() {
            continue;
        }
        // Drop what this acquire replaces: the agent's own overlapping claim
        // (a refresh), a lapsed claim it supersedes, and — under --force —
        // whatever it took. Leaving a superseded lapsed row behind would list
        // one path as owned twice.
        active.retain(|c| {
            !(claim_paths_overlap(&c.path, path)
                && (c.agent == record.agent
                    || record.forced_reason.is_some()
                    || is_expired(&c.expires_at, now)))
        });
        active.push(ActiveFileClaim {
            record_id: record.id.clone(),
            agent: record.agent.clone(),
            path: path.clone(),
            acquired_at: record.recorded_at.clone(),
            expires_at: expires_at.clone(),
            work_item_id: record.work_item_id.clone(),
            seq,
            expired: false,
        });
    }
}

/// A release drops the agent's own exact paths; `--force` releases another
/// agent's, and carries the reason on the record.
fn apply_release(active: &mut Vec<ActiveFileClaim>, record: &FileClaimRecord) {
    let forced = record.forced_reason.is_some();
    active.retain(|c| {
        !(record.paths.iter().any(|p| p == &c.path) && (forced || c.agent == record.agent))
    });
}

/// True when `now` is at or past an RFC 3339 expiry. An unparseable or empty
/// expiry counts as expired: a claim whose lifetime cannot be read must not
/// block the pool forever.
pub fn is_expired(expires_at: &str, now: chrono::DateTime<chrono::Utc>) -> bool {
    match chrono::DateTime::parse_from_rfc3339(expires_at) {
        Ok(t) => now >= t.with_timezone(&chrono::Utc),
        Err(_) => true,
    }
}

/// Build an acquire record (id and timestamps filled in here).
pub fn new_acquire_record(
    agent: &str,
    paths: Vec<String>,
    ttl_secs: u64,
    now: chrono::DateTime<chrono::Utc>,
) -> FileClaimRecord {
    FileClaimRecord {
        id: format!("cl-{}", Uuid::now_v7().simple()),
        recorded_at: now.to_rfc3339(),
        action: FileClaimAction::Acquire,
        agent: agent.to_string(),
        paths,
        expires_at: Some((now + chrono::Duration::seconds(ttl_secs as i64)).to_rfc3339()),
        work_item_id: None,
        note: None,
        forced_reason: None,
        superseded_expired: Vec::new(),
    }
}

/// Build a release record.
pub fn new_release_record(
    agent: &str,
    paths: Vec<String>,
    now: chrono::DateTime<chrono::Utc>,
) -> FileClaimRecord {
    FileClaimRecord {
        id: format!("cl-{}", Uuid::now_v7().simple()),
        recorded_at: now.to_rfc3339(),
        action: FileClaimAction::Release,
        agent: agent.to_string(),
        paths,
        expires_at: None,
        work_item_id: None,
        note: None,
        forced_reason: None,
        superseded_expired: Vec::new(),
    }
}
