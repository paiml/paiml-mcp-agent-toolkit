// IndexManager implementation
// Split from agent_context_tools.rs for maintainability

impl IndexManager {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            index: RwLock::new(None),
            project_path,
        }
    }

    /// Get or build the index, using incremental updates when possible
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
                    Ok(updated) => {
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
            if let Some(parent) = index_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create .pmat directory: {}", e))?;
            }
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
    pub async fn rebuild_index(&self) -> Result<AgentContextIndex, String> {
        let index = AgentContextIndex::build(&self.project_path)?;

        let index_path = self.project_path.join(".pmat/context.idx");
        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .pmat directory: {}", e))?;
        }
        index.save(&index_path)?;

        // Update cache
        {
            let mut guard = self.index.write().await;
            *guard = Some(index.clone());
        }

        Ok(index)
    }
}
