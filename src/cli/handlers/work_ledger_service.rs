/// Falsification ledger service
pub struct FalsificationLedger {
    /// Root of .pmat-work/ directory
    work_dir: PathBuf,
}

impl FalsificationLedger {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(project_path: &Path) -> Self {
        Self {
            work_dir: project_path.join(".pmat-work"),
        }
    }

    /// Persist receipt to per-item falsification directory
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

        let json = serde_json::to_string_pretty(receipt).context("Failed to serialize receipt")?;
        std::fs::write(&path, json).context("Failed to write receipt")?;

        Ok(path)
    }

    /// Append compact entry to global JSONL ledger
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn append_to_ledger(&self, receipt: &FalsificationReceipt) -> Result<()> {
        std::fs::create_dir_all(&self.work_dir).context("Failed to create .pmat-work directory")?;

        let ledger_path = self.work_dir.join("ledger.jsonl");
        let entry = LedgerEntry::from_receipt(receipt);
        let mut line = serde_json::to_string(&entry).context("Failed to serialize ledger entry")?;
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn latest_receipt(&self, work_item_id: &str) -> Result<Option<FalsificationReceipt>> {
        let falsification_dir = self.work_dir.join(work_item_id).join("falsification");

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

        // Sort by filename (which encodes timestamp) -- latest last
        receipt_files.sort();

        let Some(latest_path) = receipt_files.last() else {
            return Ok(None);
        };

        let content =
            std::fs::read_to_string(latest_path).context("Failed to read latest receipt")?;
        let receipt: FalsificationReceipt =
            serde_json::from_str(&content).context("Failed to parse receipt JSON")?;

        Ok(Some(receipt))
    }

    /// O(1) freshness check: load latest receipt and verify it matches HEAD
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_fresh_receipt(&self, work_item_id: &str, current_sha: &str) -> Result<bool> {
        let receipt = self.latest_receipt(work_item_id)?;
        match receipt {
            Some(r) => {
                Ok(r.is_fresh(current_sha, MAX_RECEIPT_AGE_SECS) && r.summary.allows_completion)
            }
            None => Ok(false),
        }
    }

    /// Verify integrity of all receipts for a work item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn verify_integrity(&self, work_item_id: &str) -> Result<IntegrityReport> {
        let falsification_dir = self.work_dir.join(work_item_id).join("falsification");

        if !falsification_dir.exists() {
            return Ok(IntegrityReport {
                total: 0,
                valid: 0,
                tampered: 0,
                missing: 0,
            });
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

        Ok(IntegrityReport {
            total: json_files.len(),
            valid,
            tampered,
            missing,
        })
    }

    /// Check a single receipt file for integrity (Ok(true) = valid, Ok(false) = tampered)
    fn check_receipt_file(path: &Path) -> Result<bool> {
        let content = std::fs::read_to_string(path)?;
        let receipt: FalsificationReceipt = serde_json::from_str(&content)?;
        Ok(receipt.verify_integrity())
    }

    // ========================================================================
    // MACS-003 — append-only interruption events (.pmat-work/<T>/events.jsonl)
    // ========================================================================

    /// Path to a ticket's append-only events journal (MACS-003).
    fn events_path(&self, work_item_id: &str) -> PathBuf {
        self.work_dir.join(work_item_id).join("events.jsonl")
    }

    /// Append an interruption event (MACS E5). Returns the record id used by
    /// `pmat work event --ack-event <id>`. Events are append-only: an
    /// acknowledgement is itself an event (`AgentEvent::Ack`), never a
    /// mutation of a prior line.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn append_event(&self, work_item_id: &str, event: AgentEvent) -> Result<String> {
        use std::io::Write;
        let dir = self.work_dir.join(work_item_id);
        std::fs::create_dir_all(&dir).context("Failed to create work item directory")?;
        let record = WorkEventRecord {
            id: format!("ev-{}", Uuid::now_v7().simple()),
            recorded_at: chrono::Utc::now().to_rfc3339(),
            event,
        };
        let line = serde_json::to_string(&record).context("Failed to serialize event record")?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.events_path(work_item_id))
            .context("Failed to open events.jsonl")?;
        file.write_all(format!("{line}\n").as_bytes())
            .context("Failed to append event")?;
        Ok(record.id)
    }

    /// Load all event records for a ticket (empty if none recorded).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load_events(&self, work_item_id: &str) -> Result<Vec<WorkEventRecord>> {
        let path = self.events_path(work_item_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&path).context("Failed to read events.jsonl")?;
        let mut records = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            records.push(
                serde_json::from_str::<WorkEventRecord>(line)
                    .context("Failed to parse event line")?,
            );
        }
        Ok(records)
    }

    /// Refusal records with no matching `Ack` (MACS E5) — these block
    /// `pmat work complete` until acknowledged with a reason.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn unacked_refusals(&self, work_item_id: &str) -> Result<Vec<WorkEventRecord>> {
        let records = self.load_events(work_item_id)?;
        let acked: std::collections::HashSet<String> = records
            .iter()
            .filter_map(|r| match &r.event {
                AgentEvent::Ack { ack_of, .. } => Some(ack_of.clone()),
                _ => None,
            })
            .collect();
        Ok(records
            .into_iter()
            .filter(|r| {
                matches!(r.event, AgentEvent::Refusal { .. }) && !acked.contains(&r.id)
            })
            .collect())
    }
}
