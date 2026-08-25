// IndexManager implementation
// Split from agent_context_tools.rs for maintainability

impl IndexManager {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            index: RwLock::new(None),
            project_path,
        }
    }

    /// Get or build the index, using incremental updates when possible
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_index(&self) -> Result<AgentContextIndex, String> {
        // First check if we have a cached index
        {
            let guard = self.index.read().await;
            if let Some(idx) = guard.as_ref() {
                return Ok(idx.clone());
            }
        }

        // Build or load index
        let index_path = self.project_path.join(".pmat/context.idx");
        let index = if index_path.exists() {
            let existing = AgentContextIndex::load(&index_path)?;
            // Try incremental update if checksums are available
            if !existing.manifest().file_checksums.is_empty() {
                match AgentContextIndex::build_incremental(&self.project_path, &existing) {
                    Ok(mut updated) => {
                        // Backfill deferred (empty) source for reused entries
                        // before save() rewrites the DB, or every reused row
                        // would persist with source='' (source-wipe bug).
                        updated.load_all_source();
                        let _ = updated.save(&index_path);
                        updated
                    }
                    Err(_) => existing,
                }
            } else {
                existing
            }
        } else {
            let idx = AgentContextIndex::build(&self.project_path)?;
            // Create directory and save
            // #1070: `.pmat/` gets its ignore rule as it is created, so building
            // an index does not leave `?? .pmat/` in the indexed repo.
            crate::utils::pmat_cache_dir::ensure_parent_dir(&index_path)
                .map_err(|e| format!("Failed to create .pmat directory: {}", e))?;
            idx.save(&index_path)?;
            idx
        };

        // Auto-discover and merge sibling project indexes
        let siblings = AgentContextIndex::discover_sibling_indexes(&self.project_path);
        let mut index = index;
        if !siblings.is_empty() {
            index.merge_siblings(&siblings);
        }

        // Cache it
        {
            let mut guard = self.index.write().await;
            *guard = Some(index.clone());
        }

        Ok(index)
    }

    /// Force rebuild the index
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn rebuild_index(&self) -> Result<AgentContextIndex, String> {
        let index = AgentContextIndex::build(&self.project_path)?;

        let index_path = self.project_path.join(".pmat/context.idx");
        // #1070: as above — the directory carries its own ignore rule.
        crate::utils::pmat_cache_dir::ensure_parent_dir(&index_path)
            .map_err(|e| format!("Failed to create .pmat directory: {}", e))?;
        index.save(&index_path)?;

        // Update cache
        {
            let mut guard = self.index.write().await;
            *guard = Some(index.clone());
        }

        Ok(index)
    }
}
