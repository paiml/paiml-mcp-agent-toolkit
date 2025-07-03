use crate::models::refactor::{RefactorStateMachine, RefactorConfig};
use crate::mcp_server::snapshots::SnapshotManager;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub struct StateManager {
    state: Option<RefactorStateMachine>,
    snapshot_manager: SnapshotManager,
    session_id: String,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: None,
            snapshot_manager: SnapshotManager::new(),
            session_id: Self::generate_session_id(),
        }
    }
    
    pub fn with_temp_dir(temp_dir: &Path) -> Self {
        Self {
            state: None,
            snapshot_manager: SnapshotManager::with_path(temp_dir),
            session_id: Self::generate_session_id(),
        }
    }

    pub fn start_session(&mut self, targets: Vec<PathBuf>, config: RefactorConfig) -> Result<(), String> {
        if self.state.is_some() {
            return Err("Session already active. Stop current session before starting a new one.".to_string());
        }
        
        info!("Starting new refactor session with {} targets", targets.len());
        
        self.state = Some(RefactorStateMachine::new(targets, config));
        self.session_id = Self::generate_session_id();
        
        // Save initial state
        self.save_snapshot()?;
        
        Ok(())
    }

    pub fn advance(&mut self) -> Result<(), String> {
        let state = self.state.as_mut().ok_or("No active session")?;
        state.advance()?;
        
        // Save after each state transition
        self.save_snapshot()?;
        
        Ok(())
    }

    pub fn get_state(&self) -> Result<&RefactorStateMachine, String> {
        self.state.as_ref().ok_or("No active session".to_string())
    }
    
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn stop_session(&mut self) -> Result<(), String> {
        if self.state.is_none() {
            return Err("No active session to stop".to_string());
        }
        
        info!("Stopping refactor session");
        
        // Clear in-memory state
        self.state = None;
        
        // Remove snapshot file
        self.snapshot_manager.remove_snapshot()?;
        
        Ok(())
    }

    fn save_snapshot(&self) -> Result<(), String> {
        if let Some(state) = &self.state {
            self.snapshot_manager.save_snapshot(state)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn load_from_snapshot(&mut self) -> Result<(), String> {
        match self.snapshot_manager.load_snapshot() {
            Ok(state) => {
                self.state = Some(state);
                info!("Loaded existing refactor state from snapshot");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to load snapshot: {}", e);
                Err(e)
            }
        }
    }

    fn generate_session_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        format!("refactor-session-{}", timestamp)
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}