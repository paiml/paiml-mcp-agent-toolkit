/// Falsification ledger service
pub struct FalsificationLedger {
    /// Root of .pmat-work/ directory
    work_dir: PathBuf,
}

impl FalsificationLedger {
    pub fn new(project_path: &Path) -> Self {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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

        let json = serde_json::to_string_pretty(receipt).context("Failed to serialize receipt")?;
        std::fs::write(&path, json).context("Failed to write receipt")?;

        Ok(path)
    }

    /// Append compact entry to global JSONL ledger
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
    pub fn latest_receipt(&self, work_item_id: &str) -> Result<Option<FalsificationReceipt>> {
        debug_assert!(!work_item_id.is_empty(), "work_item_id must not be empty");
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
    pub fn has_fresh_receipt(&self, work_item_id: &str, current_sha: &str) -> Result<bool> {
        debug_assert!(!work_item_id.is_empty(), "work_item_id must not be empty");
        let receipt = self.latest_receipt(work_item_id)?;
        match receipt {
            Some(r) => {
                Ok(r.is_fresh(current_sha, MAX_RECEIPT_AGE_SECS) && r.summary.allows_completion)
            }
            None => Ok(false),
        }
    }

    /// Verify integrity of all receipts for a work item
    pub fn verify_integrity(&self, work_item_id: &str) -> Result<IntegrityReport> {
        debug_assert!(!work_item_id.is_empty(), "work_item_id must not be empty");
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
}
