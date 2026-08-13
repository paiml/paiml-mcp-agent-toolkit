// Triage coverage accounting (ULTRA-003) — a bounded pass must state its bound.
//
// Measured problem: one agent triaged 39 findings and filed 7, dropping 32
// without saying so. Nothing in its output was false; the omission was the
// defect, and it was only caught by counting by hand. A bounded pass that does
// not declare its bound reads as full coverage.
//
// So: an agent records what it examined, what it acted on, and — when those
// differ — names every item it did not act on and why. `record` REFUSES a
// record whose arithmetic does not close, and `verify` fails when a work item
// has no triage record at all, because "nobody stated their coverage" must not
// render as "coverage was complete".
//
// Storage: `.pmat-work/triage.jsonl`, append-only, one record per line.

/// One line in `.pmat-work/triage.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageRecord {
    /// Record id, e.g. "tr-0197f0..."
    pub id: String,
    /// ISO 8601 timestamp
    pub recorded_at: String,
    /// Agent identity
    pub agent: String,
    /// What was being triaged, in the agent's own words
    pub scope: String,
    /// How many candidates the pass looked at
    pub examined: u32,
    /// How many it actually acted on
    pub acted: u32,
    /// Identifiers of the ones it did not act on
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deferred: Vec<String>,
    /// Why the gap exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Ticket this pass belongs to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
}

impl TriageRecord {
    /// Everything wrong with this record's accounting. Empty means the pass
    /// accounts for every item it examined.
    pub fn audit(&self) -> Vec<String> {
        let mut defects = Vec::new();
        if self.scope.trim().is_empty() {
            defects.push("scope is empty: a pass with no stated subject cannot be audited".into());
        }
        if self.acted > self.examined {
            defects.push(format!(
                "acted ({}) exceeds examined ({}): the pass acted on items it never counted",
                self.acted, self.examined
            ));
            return defects;
        }
        let gap = (self.examined - self.acted) as usize;
        if gap == 0 {
            return defects;
        }
        if self.deferred.len() != gap {
            defects.push(format!(
                "{} item(s) went unacted but {} were named in --deferred: {} item(s) would \
                 disappear silently",
                gap,
                self.deferred.len(),
                gap.abs_diff(self.deferred.len())
            ));
        }
        if self.reason.as_ref().is_none_or(|r| r.trim().is_empty()) {
            defects.push(format!("{gap} item(s) went unacted with no --reason given"));
        }
        defects
    }

    /// Items examined but not acted on.
    pub fn gap(&self) -> u32 {
        self.examined.saturating_sub(self.acted)
    }
}

/// Append-only triage journal over `.pmat-work/triage.jsonl`.
pub struct TriageLedger {
    work_dir: PathBuf,
}

impl TriageLedger {
    /// Open the journal for a project (the file need not exist yet).
    pub fn new(project_path: &Path) -> Self {
        Self {
            work_dir: project_path.join(".pmat-work"),
        }
    }

    /// Path to `triage.jsonl`.
    pub fn journal_path(&self) -> PathBuf {
        self.work_dir.join("triage.jsonl")
    }

    /// Every record in file order. A malformed line is an error: a journal
    /// that skips lines under-reports exactly the omissions it exists to find.
    pub fn load_records(&self) -> Result<Vec<TriageRecord>> {
        let path = self.journal_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&path).context("Failed to read triage.jsonl")?;
        let mut records = Vec::new();
        for (idx, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            records.push(serde_json::from_str::<TriageRecord>(line).with_context(|| {
                format!("triage.jsonl line {} is not a triage record", idx + 1)
            })?);
        }
        Ok(records)
    }

    /// Append one record.
    pub fn append(&self, record: &TriageRecord) -> Result<()> {
        use std::io::Write;
        std::fs::create_dir_all(&self.work_dir).context("Failed to create .pmat-work directory")?;
        let mut line =
            serde_json::to_string(record).context("Failed to serialize triage record")?;
        line.push('\n');
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.journal_path())
            .context("Failed to open triage.jsonl")?;
        file.write_all(line.as_bytes())
            .context("Failed to append triage record")?;
        Ok(())
    }
}

/// Build a triage record (id and timestamp filled in here).
pub fn new_triage_record(
    agent: &str,
    scope: &str,
    examined: u32,
    acted: u32,
    now: chrono::DateTime<chrono::Utc>,
) -> TriageRecord {
    TriageRecord {
        id: format!("tr-{}", Uuid::now_v7().simple()),
        recorded_at: now.to_rfc3339(),
        agent: agent.to_string(),
        scope: scope.to_string(),
        examined,
        acted,
        deferred: Vec::new(),
        reason: None,
        work_item_id: None,
    }
}

/// A record that failed its audit, paired with the reasons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageDefect {
    /// Offending record id
    pub record_id: String,
    /// Agent that wrote it
    pub agent: String,
    /// Its stated scope
    pub scope: String,
    /// Why the accounting does not close
    pub defects: Vec<String>,
}

/// Result of `pmat work triage verify`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriageVerification {
    /// Records considered after filtering
    pub records: usize,
    /// Total items examined across them
    pub examined: u32,
    /// Total items acted on
    pub acted: u32,
    /// Total items explicitly deferred
    pub deferred: usize,
    /// Records whose accounting does not close
    pub unaccounted: Vec<TriageDefect>,
}

impl TriageVerification {
    /// True only when something was measured and all of it accounts.
    pub fn ok(&self) -> bool {
        self.records > 0 && self.unaccounted.is_empty()
    }
}

/// Fold records into a verification report.
pub fn verify_triage_records(records: &[TriageRecord]) -> TriageVerification {
    let mut report = TriageVerification::default();
    for record in records {
        report.records += 1;
        report.examined += record.examined;
        report.acted += record.acted;
        report.deferred += record.deferred.len();
        let defects = record.audit();
        if !defects.is_empty() {
            report.unaccounted.push(TriageDefect {
                record_id: record.id.clone(),
                agent: record.agent.clone(),
                scope: record.scope.clone(),
                defects,
            });
        }
    }
    report
}
