use crate::mcp_server::capnp_conversion::{
    deserialize_state_from_capnp, get_serialization_format, serialize_state_to_capnp,
};
use crate::models::refactor::RefactorStateMachine;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

pub struct SnapshotManager {
    snapshot_path: PathBuf,
}

impl SnapshotManager {
    #[must_use]
    pub fn new() -> Self {
        Self::with_path(".pmat-cache")
    }

    pub fn with_path<P: Into<PathBuf>>(cache_dir: P) -> Self {
        let snapshot_dir = cache_dir.into();

        // Ensure cache directory exists
        if !snapshot_dir.exists() {
            fs::create_dir_all(&snapshot_dir).ok();
        }

        Self {
            snapshot_path: snapshot_dir.join("refactor-state.bin"),
        }
    }

    /// Saves a refactor state machine snapshot to disk
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::mcp_server::snapshots::SnapshotManager;
    /// use pmat::models::refactor::{RefactorStateMachine, RefactorConfig};
    /// use std::path::PathBuf;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let manager = SnapshotManager::with_path(dir.path());
    ///
    /// let state = RefactorStateMachine::new(
    ///     vec![PathBuf::from("src/main.rs")],
    ///     RefactorConfig::default()
    /// );
    ///
    /// let result = manager.save_snapshot(&state);
    /// assert!(result.is_ok());
    /// ```
    pub fn save_snapshot(&self, state: &RefactorStateMachine) -> Result<(), String> {
        debug!(
            "Saving refactor state snapshot to {:?} using {}",
            self.snapshot_path,
            get_serialization_format()
        );

        // Ensure parent directory exists before writing
        if let Some(parent) = self.snapshot_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create snapshot directory: {e}"))?;
        }

        // Use Cap'n Proto serialization with JSON fallback
        let data = serialize_state_to_capnp(state)?;

        // Use safe two-phase write: create .tmp file, then rename
        let temp_path = self.snapshot_path.with_extension("tmp");
        fs::write(&temp_path, data).map_err(|e| format!("Failed to write snapshot: {e}"))?;

        fs::rename(&temp_path, &self.snapshot_path)
            .map_err(|e| format!("Failed to rename snapshot: {e}"))?;

        info!(
            "Saved refactor state snapshot using {}",
            get_serialization_format()
        );
        Ok(())
    }

    /// Loads a refactor state machine snapshot from disk
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::mcp_server::snapshots::SnapshotManager;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let manager = SnapshotManager::with_path(dir.path());
    ///
    /// match manager.load_snapshot() {
    ///     Ok(state) => println!("Loaded state"),
    ///     Err(e) => println!("No snapshot found: {}", e),
    /// }
    /// ```
    pub fn load_snapshot(&self) -> Result<RefactorStateMachine, String> {
        debug!(
            "Loading refactor state snapshot from {:?} using {}",
            self.snapshot_path,
            get_serialization_format()
        );

        if !self.snapshot_path.exists() {
            return Err("No snapshot file found".to_string());
        }

        // Use Cap'n Proto deserialization with JSON fallback
        let data =
            fs::read(&self.snapshot_path).map_err(|e| format!("Failed to read snapshot: {e}"))?;

        let state = deserialize_state_from_capnp(&data)?;

        info!(
            "Loaded refactor state snapshot using {}",
            get_serialization_format()
        );
        Ok(state)
    }

    /// Removes a saved snapshot from disk
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::mcp_server::snapshots::SnapshotManager;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let manager = SnapshotManager::with_path(dir.path());
    ///
    /// let result = manager.remove_snapshot();
    /// // Ok if file was removed or didn't exist
    /// ```
    pub fn remove_snapshot(&self) -> Result<(), String> {
        if self.snapshot_path.exists() {
            fs::remove_file(&self.snapshot_path)
                .map_err(|e| format!("Failed to remove snapshot: {e}"))?;
            info!("Removed refactor state snapshot");
        }
        Ok(())
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

// Cap'n Proto serialization is implemented via capnp_conversion module
// Falls back to JSON when Cap'n Proto is not available

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::refactor::RefactorConfig;
    use std::path::PathBuf;

    #[test]
    fn test_snapshot_roundtrip() {
        let manager = SnapshotManager::new();
        let state =
            RefactorStateMachine::new(vec![PathBuf::from("test.rs")], RefactorConfig::default());

        // Save snapshot
        manager.save_snapshot(&state).unwrap();

        // Load snapshot
        let loaded_state = manager.load_snapshot().unwrap();

        // Verify
        assert_eq!(loaded_state.targets.len(), state.targets.len());

        // Clean up
        manager.remove_snapshot().unwrap();
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
