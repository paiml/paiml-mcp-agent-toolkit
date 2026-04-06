impl TieredStore {
    /// Query TDG records by git commit SHA (Sprint 65 Phase 3)
    ///
    /// Searches both warm and cold storage for records matching the specified commit.
    /// Supports full SHA, short SHA (7 chars), or git tags.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_by_commit(&self, commit_ref: &str) -> Result<Vec<FullTdgRecord>> {
        debug_assert!(!commit_ref.is_empty(), "commit_ref must not be empty");
        let mut results = Vec::new();

        // Search warm storage
        for item in self.warm_backend.iter()? {
            let (_key, value) = item?;
            let decompressed = decompress_size_prepended(&value)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&decompressed) {
                if let Some(git_ctx) = &record.git_context {
                    if git_ctx.commit_sha == commit_ref
                        || git_ctx.commit_sha_short == commit_ref
                        || git_ctx.tags.contains(&commit_ref.to_string())
                    {
                        results.push(record);
                    }
                }
            }
        }

        // Search cold storage
        for item in self.cold_backend.iter()? {
            let (_key, value) = item?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&value) {
                if let Some(git_ctx) = &record.git_context {
                    if git_ctx.commit_sha == commit_ref
                        || git_ctx.commit_sha_short == commit_ref
                        || git_ctx.tags.contains(&commit_ref.to_string())
                    {
                        results.push(record);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Query TDG records in a git commit range (Sprint 65 Phase 3)
    ///
    /// Returns all records with git context, allowing the caller to filter by commit range.
    /// The actual git range resolution happens in the handler (using git2).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_all_with_git_context(&self) -> Result<Vec<FullTdgRecord>> {
        let mut results = Vec::new();

        // Search warm storage
        for item in self.warm_backend.iter()? {
            let (_key, value) = item?;
            let decompressed = decompress_size_prepended(&value)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&decompressed) {
                if record.git_context.is_some() {
                    results.push(record);
                }
            }
        }

        // Search cold storage
        for item in self.cold_backend.iter()? {
            let (_key, value) = item?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&value) {
                if record.git_context.is_some() {
                    results.push(record);
                }
            }
        }

        // Sort by commit timestamp (newest first)
        results.sort_by(|a, b| {
            let a_time = a
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            let b_time = b
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            b_time.cmp(&a_time) // Reverse order (newest first)
        });

        Ok(results)
    }

    /// Query TDG records by file path (Sprint 65 Phase 3)
    ///
    /// Filters records to only those matching the specified file path.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn get_by_path(&self, target_path: &Path) -> Result<Vec<FullTdgRecord>> {
        debug_assert!(target_path.exists(), "target_path must exist: {}", target_path.display());
        let mut results = Vec::new();

        // Search warm storage
        for item in self.warm_backend.iter()? {
            let (_key, value) = item?;
            let decompressed = decompress_size_prepended(&value)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&decompressed) {
                if record.identity.path == target_path {
                    results.push(record);
                }
            }
        }

        // Search cold storage
        for item in self.cold_backend.iter()? {
            let (_key, value) = item?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&value) {
                if record.identity.path == target_path {
                    results.push(record);
                }
            }
        }

        // Sort by commit timestamp (newest first)
        results.sort_by(|a, b| {
            let a_time = a
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            let b_time = b
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            b_time.cmp(&a_time) // Reverse order (newest first)
        });

        Ok(results)
    }
}

/// Factory for creating tiered storage instances
pub struct TieredStorageFactory;

impl TieredStorageFactory {
    /// Create storage instance with default configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn create_default() -> Result<TieredStore> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        TieredStore::new(home_dir)
    }

    /// Create storage instance at specific path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn create_at_path(path: impl AsRef<Path>) -> Result<TieredStore> {
        TieredStore::new(path)
    }

    /// Create in-memory storage for testing
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn create_in_memory() -> TieredStore {
        TieredStore::in_memory()
    }
}
