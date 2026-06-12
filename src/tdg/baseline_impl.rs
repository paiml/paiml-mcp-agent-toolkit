impl TdgBaseline {
    /// Create empty baseline
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(git_context: Option<GitContext>) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            name: None,
            created_at: Utc::now(),
            git_context,
            files: BTreeMap::new(),
            summary: BaselineSummary {
                total_files: 0,
                avg_score: 0.0,
                grade_distribution: BTreeMap::new(),
                languages: BTreeMap::new(),
            },
        }
    }

    /// Add file entry to baseline
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn add_entry(&mut self, path: PathBuf, entry: BaselineEntry) {
        self.files.insert(path, entry);
        self.recompute_summary();
    }

    /// Recompute summary statistics
    fn recompute_summary(&mut self) {
        self.summary.total_files = self.files.len();

        if self.files.is_empty() {
            self.summary.avg_score = 0.0;
            return;
        }

        // Calculate average score
        let total: f32 = self.files.values().map(|e| e.score.total).sum();
        self.summary.avg_score = total / self.files.len() as f32;

        // Calculate grade distribution
        self.summary.grade_distribution.clear();
        for entry in self.files.values() {
            *self
                .summary
                .grade_distribution
                .entry(entry.score.grade)
                .or_insert(0) += 1;
        }

        // Calculate language distribution
        self.summary.languages.clear();
        for entry in self.files.values() {
            let lang = format!("{:?}", entry.score.language);
            *self.summary.languages.entry(lang).or_insert(0) += 1;
        }
    }

    /// Compare this baseline with another
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn compare(&self, other: &TdgBaseline) -> BaselineComparison {
        let mut improved = Vec::new();
        let mut regressed = Vec::new();
        let mut unchanged = Vec::new();
        let mut added = Vec::new();
        let mut removed = Vec::new();

        // Check files in current baseline
        for (path, new_entry) in &other.files {
            if let Some(old_entry) = self.files.get(path) {
                let delta = new_entry.score.total - old_entry.score.total;

                if delta.abs() < 0.01 {
                    // Unchanged (within floating point tolerance)
                    unchanged.push(path.clone());
                } else if delta > 0.0 {
                    // Improved
                    improved.push(FileComparison {
                        path: path.clone(),
                        old_score: old_entry.score.clone(),
                        new_score: new_entry.score.clone(),
                        delta,
                        grade_change: (old_entry.score.grade, new_entry.score.grade),
                    });
                } else {
                    // Regressed
                    regressed.push(FileComparison {
                        path: path.clone(),
                        old_score: old_entry.score.clone(),
                        new_score: new_entry.score.clone(),
                        delta,
                        grade_change: (old_entry.score.grade, new_entry.score.grade),
                    });
                }
            } else {
                // File added
                added.push(path.clone());
            }
        }

        // Check for removed files
        for path in self.files.keys() {
            if !other.files.contains_key(path) {
                removed.push(path.clone());
            }
        }

        // Sort by delta magnitude
        improved.sort_by(|a, b| b.delta.total_cmp(&a.delta));
        regressed.sort_by(|a, b| a.delta.total_cmp(&b.delta));

        BaselineComparison {
            improved,
            regressed,
            unchanged,
            added,
            removed,
        }
    }

    /// Save baseline to JSON file
    ///
    /// Writes to a process-unique scratch file in the same directory, then
    /// renames into place so concurrent readers never observe a partial file.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self)?;
        let tmp_path = crate::utils::scratch::scratch_path(path);
        crate::utils::scratch::sweep_stale_scratch(path, std::time::Duration::from_secs(3600));
        std::fs::write(&tmp_path, json)?;
        if let Err(e) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(e.into());
        }
        Ok(())
    }

    /// Load baseline from JSON file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let baseline = serde_json::from_str(&json)?;
        Ok(baseline)
    }
}
