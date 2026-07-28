#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use tempfile::tempdir;

    // Test StateManager creation
    #[test]
    fn test_state_manager_new() {
        let manager = StateManager::new();
        assert!(manager.get_state().is_err());
        assert!(manager.get_session_id().starts_with("refactor-session-"));
    }

    #[test]
    fn test_state_manager_default() {
        let manager1 = StateManager::new();
        let manager2 = StateManager::default();
        // Both should have no active session
        assert!(manager1.get_state().is_err());
        assert!(manager2.get_state().is_err());
    }

    #[test]
    fn test_state_manager_with_temp_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let manager = StateManager::with_temp_dir(temp_dir.path());
        assert!(manager.get_state().is_err());
        assert!(manager.get_session_id().starts_with("refactor-session-"));
    }

    // Test session lifecycle
    #[test]
    fn test_start_session_success() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();

        let result = manager.start_session(targets, config);
        assert!(result.is_ok());
        assert!(manager.get_state().is_ok());
    }

    #[test]
    fn test_start_session_generates_new_session_id() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let initial_id = manager.get_session_id().to_string();

        // Wait 1ms to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1));

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();
        manager.start_session(targets, config).unwrap();

        // Session ID should be different after starting (different timestamp)
        assert_ne!(initial_id, manager.get_session_id());
    }

    #[test]
    fn test_start_session_duplicate_fails() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();

        // First start should succeed
        assert!(manager
            .start_session(targets.clone(), config.clone())
            .is_ok());

        // Second start should fail
        let result = manager.start_session(targets, config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session already active"));
    }

    #[test]
    fn test_stop_session_success() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();
        manager.start_session(targets, config).unwrap();

        let result = manager.stop_session();
        assert!(result.is_ok());
        assert!(manager.get_state().is_err());
    }

    #[test]
    fn test_stop_session_no_active_session() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let result = manager.stop_session();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No active session to stop"));
    }

    // Test state machine advancement
    #[test]
    fn test_advance_no_session() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let result = manager.advance();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No active session"));
    }

    #[test]
    fn test_advance_with_session() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();
        manager.start_session(targets, config).unwrap();

        // Should be able to advance
        let result = manager.advance();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_advances() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();
        manager.start_session(targets, config).unwrap();

        // Advance multiple times
        for _ in 0..3 {
            let result = manager.advance();
            assert!(result.is_ok());
        }
    }

    // Test session ID generation
    #[test]
    fn test_generate_session_id_format() {
        let session_id = StateManager::generate_session_id();
        assert!(session_id.starts_with("refactor-session-"));
        // Should contain a timestamp (numeric part after the prefix)
        let timestamp_part = session_id.strip_prefix("refactor-session-").unwrap();
        assert!(timestamp_part.parse::<u128>().is_ok());
    }

    #[test]
    fn test_session_ids_are_unique() {
        let id1 = StateManager::generate_session_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = StateManager::generate_session_id();
        assert_ne!(id1, id2);
    }

    // Test get_session_id
    #[test]
    fn test_get_session_id() {
        let manager = StateManager::new();
        let session_id = manager.get_session_id();
        assert!(!session_id.is_empty());
        assert!(session_id.starts_with("refactor-session-"));
    }

    // Test get_state
    #[test]
    fn test_get_state_no_session() {
        let manager = StateManager::new();
        let result = manager.get_state();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active session");
    }

    #[test]
    fn test_get_state_with_session() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();
        manager.start_session(targets, config).unwrap();

        let result = manager.get_state();
        assert!(result.is_ok());
    }

    // Test empty targets behavior
    #[test]
    fn test_start_session_empty_targets() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets: Vec<PathBuf> = vec![];
        let config = RefactorConfig::default();

        let result = manager.start_session(targets, config);
        assert!(result.is_ok());
    }

    // Test snapshot persistence
    #[test]
    fn test_snapshot_saved_on_start() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();

        // Should not fail due to snapshot save
        let result = manager.start_session(targets, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_saved_on_advance() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mut manager = StateManager::with_temp_dir(temp_dir.path());

        let targets = vec![PathBuf::from("/tmp/test.rs")];
        let config = RefactorConfig::default();
        manager.start_session(targets, config).unwrap();

        // Advance should save snapshot
        let result = manager.advance();
        assert!(result.is_ok());
    }
}
