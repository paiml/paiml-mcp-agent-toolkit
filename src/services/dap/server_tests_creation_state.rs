    // Server Creation Tests

    #[test]
    fn test_server_new_creates_uninitialized_server() {
        let server = DapServer::new();
        assert!(!server.is_initialized());
        assert!(!server.is_running());
        assert!(!server.has_program_loaded());
        assert!(server.current_program().is_none());
    }

    #[test]
    fn test_server_with_recording_none() {
        let server = DapServer::with_recording(None);
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_server_with_recording_some() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_server_default_creates_new_server() {
        let server1 = DapServer::new();
        let server2 = DapServer::default();
        assert_eq!(server1.is_initialized(), server2.is_initialized());
    }

    // Server State Tests

    #[test]
    fn test_is_initialized_false_initially() {
        let server = DapServer::new();
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_is_running_false_initially() {
        let server = DapServer::new();
        assert!(!server.is_running());
    }

    #[test]
    fn test_has_program_loaded_false_initially() {
        let server = DapServer::new();
        assert!(!server.has_program_loaded());
    }

    #[test]
    fn test_current_program_none_initially() {
        let server = DapServer::new();
        assert!(server.current_program().is_none());
    }

    #[test]
    fn test_current_language_none_initially() {
        let server = DapServer::new();
        assert!(server.current_language().is_none());
    }

    #[test]
    fn test_has_ast_for_false_initially() {
        let server = DapServer::new();
        assert!(!server.has_ast_for("/some/path.rs"));
    }

    #[test]
    fn test_current_stopped_file_none_initially() {
        let server = DapServer::new();
        assert!(server.current_stopped_file().is_none());
    }

    #[test]
    fn test_current_stopped_line_none_initially() {
        let server = DapServer::new();
        assert!(server.current_stopped_line().is_none());
    }

