use crate::models::refactor::RefactorStateMachine;
use crate::mcp_server::capnp_conversion::{serialize_state_to_capnp, deserialize_state_from_capnp, get_serialization_format};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

pub struct SnapshotManager {
    snapshot_path: PathBuf,
}

impl SnapshotManager {
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

    pub fn save_snapshot(&self, state: &RefactorStateMachine) -> Result<(), String> {
        debug!("Saving refactor state snapshot to {:?} using {}", 
               self.snapshot_path, get_serialization_format());
        
        // Ensure parent directory exists before writing
        if let Some(parent) = self.snapshot_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create snapshot directory: {}", e))?;
        }
        
        // Use Cap'n Proto serialization with JSON fallback
        let data = serialize_state_to_capnp(state)?;
        
        // Atomic write: write to temp file then rename
        let temp_path = self.snapshot_path.with_extension("tmp");
        fs::write(&temp_path, data)
            .map_err(|e| format!("Failed to write snapshot: {}", e))?;
        
        fs::rename(&temp_path, &self.snapshot_path)
            .map_err(|e| format!("Failed to rename snapshot: {}", e))?;
        
        info!("Saved refactor state snapshot using {}", get_serialization_format());
        Ok(())
    }

    pub fn load_snapshot(&self) -> Result<RefactorStateMachine, String> {
        debug!("Loading refactor state snapshot from {:?} using {}", 
               self.snapshot_path, get_serialization_format());
        
        if !self.snapshot_path.exists() {
            return Err("No snapshot file found".to_string());
        }
        
        // Use Cap'n Proto deserialization with JSON fallback
        let data = fs::read(&self.snapshot_path)
            .map_err(|e| format!("Failed to read snapshot: {}", e))?;
        
        let state = deserialize_state_from_capnp(&data)?;
        
        info!("Loaded refactor state snapshot using {}", get_serialization_format());
        Ok(state)
    }

    pub fn remove_snapshot(&self) -> Result<(), String> {
        if self.snapshot_path.exists() {
            fs::remove_file(&self.snapshot_path)
                .map_err(|e| format!("Failed to remove snapshot: {}", e))?;
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
        let state = RefactorStateMachine::new(
            vec![PathBuf::from("test.rs")],
            RefactorConfig::default(),
        );

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