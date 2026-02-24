    // ==========================================================================
    // Default and Construction Tests
    // ==========================================================================

    #[test]
    fn test_quality_thresholds_default() {
        let thresholds = QualityThresholds::default();

        assert_eq!(thresholds.max_complexity, 20); // Toyota Way standard
        assert_eq!(thresholds.satd_tolerance, 0); // Zero tolerance
        assert!((thresholds.dead_code_max_percentage - 10.0).abs() < f64::EPSILON);
        assert!((thresholds.min_quality_score - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();

        assert!((metrics.avg_complexity - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.max_complexity, 0);
        assert_eq!(metrics.satd_count, 0);
        assert!((metrics.dead_code_percentage - 0.0).abs() < f64::EPSILON);
        assert!((metrics.quality_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.files_analyzed, 0);
        assert_eq!(metrics.total_violations, 0);
    }

    #[test]
    fn test_agent_statistics_default() {
        let stats = AgentStatistics::default();

        assert_eq!(stats.sessions_count, 0);
        assert_eq!(stats.analyses_performed, 0);
        assert_eq!(stats.violations_detected, 0);
        assert_eq!(stats.refactorings_suggested, 0);
        assert_eq!(stats.total_uptime_seconds, 0);
    }

    #[test]
    fn test_agent_state_default() {
        let state = AgentState::default();

        assert_eq!(state.version, "1.0.0");
        assert!(state.monitored_projects.is_empty());
        assert!(state.quality_history.is_empty());
        assert!(state.config_overrides.is_empty());
        assert_eq!(state.statistics.sessions_count, 0);
    }

    // ==========================================================================
    // Serialization Tests
    // ==========================================================================

    #[test]
    fn test_agent_state_to_json() {
        let state = AgentState::default();
        let json = state.to_json().unwrap();

        assert!(json.contains("\"version\": \"1.0.0\""));
        assert!(json.contains("\"monitored_projects\""));
        assert!(json.contains("\"quality_history\""));
        assert!(json.contains("\"statistics\""));
    }

    #[test]
    fn test_agent_state_roundtrip_serialization() {
        let mut state = AgentState::default();
        state.config_overrides.insert(
            "custom_key".to_string(),
            serde_json::json!({"nested": "value", "number": 42}),
        );

        let json = state.to_json().unwrap();
        let deserialized: AgentState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, state.version);
        assert_eq!(deserialized.config_overrides.len(), 1);
        assert!(deserialized.config_overrides.contains_key("custom_key"));
    }

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics {
            avg_complexity: 10.5,
            max_complexity: 25,
            satd_count: 3,
            dead_code_percentage: 5.5,
            quality_score: 85.0,
            files_analyzed: 50,
            total_violations: 10,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();

        assert!((deserialized.avg_complexity - 10.5).abs() < f64::EPSILON);
        assert_eq!(deserialized.max_complexity, 25);
        assert_eq!(deserialized.satd_count, 3);
        assert!((deserialized.dead_code_percentage - 5.5).abs() < f64::EPSILON);
        assert!((deserialized.quality_score - 85.0).abs() < f64::EPSILON);
        assert_eq!(deserialized.files_analyzed, 50);
        assert_eq!(deserialized.total_violations, 10);
    }

    #[test]
    fn test_project_state_serialization() {
        let project = ProjectState {
            id: "proj_123".to_string(),
            path: PathBuf::from("/path/to/project"),
            started_at: Utc::now(),
            last_analyzed: Some(Utc::now()),
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
            thresholds: QualityThresholds::default(),
        };

        let json = serde_json::to_string(&project).unwrap();
        let deserialized: ProjectState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "proj_123");
        assert_eq!(deserialized.path, PathBuf::from("/path/to/project"));
        assert_eq!(deserialized.watch_patterns.len(), 2);
        assert!(deserialized.last_analyzed.is_some());
    }

    #[test]
    fn test_quality_snapshot_serialization() {
        let snapshot = QualitySnapshot {
            timestamp: Utc::now(),
            project_id: "test_proj".to_string(),
            metrics: QualityMetrics::default(),
            violations: vec!["violation_1".to_string(), "violation_2".to_string()],
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: QualitySnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.project_id, "test_proj");
        assert_eq!(deserialized.violations.len(), 2);
    }

    // ==========================================================================
    // StatePersistence Core Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_state_persistence_new_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.version, "1.0.0");
        assert!(state.monitored_projects.is_empty());
    }

    #[tokio::test]
    async fn test_state_persistence_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir");

        // Create persistence with nested path that doesn't exist yet
        let persistence = StatePersistence::new(&nested_path).unwrap();
        persistence.save().await.unwrap();

        // Verify the directory and file were created
        assert!(nested_path.join("agent_state.json").exists());
    }

    #[tokio::test]
    async fn test_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add a project
        let project = ProjectState {
            id: "test_project".to_string(),
            path: PathBuf::from("/test/path"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec!["*.rs".to_string()],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Save and verify
        persistence.save().await.unwrap();

        // Load from file
        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;

        assert!(state.monitored_projects.contains_key("test_project"));
    }

    #[tokio::test]
    async fn test_add_project_updates_last_updated() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let state_before = persistence.get_state().await;
        let before_time = state_before.last_updated;

        // Small delay to ensure timestamp differs
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let project = ProjectState {
            id: "test".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        let state_after = persistence.get_state().await;
        assert!(state_after.last_updated >= before_time);
    }

    #[tokio::test]
    async fn test_add_project_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add project with one path
        let project1 = ProjectState {
            id: "same_id".to_string(),
            path: PathBuf::from("/path/one"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project1).await.unwrap();

        // Add project with same ID but different path
        let project2 = ProjectState {
            id: "same_id".to_string(),
            path: PathBuf::from("/path/two"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project2).await.unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.monitored_projects.len(), 1);
        assert_eq!(
            state.monitored_projects.get("same_id").unwrap().path,
            PathBuf::from("/path/two")
        );
    }

    // ==========================================================================
    // Remove Project Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_remove_project_existing() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "to_remove".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();
        assert!(persistence
            .get_state()
            .await
            .monitored_projects
            .contains_key("to_remove"));

        persistence.remove_project("to_remove").await.unwrap();

        let state = persistence.get_state().await;
        assert!(!state.monitored_projects.contains_key("to_remove"));
    }

    #[tokio::test]
    async fn test_remove_project_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Removing non-existent project should not error
        let result = persistence.remove_project("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_project_updates_last_updated() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "to_remove".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        let before_time = persistence.get_state().await.last_updated;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        persistence.remove_project("to_remove").await.unwrap();

        let after_time = persistence.get_state().await.last_updated;
        assert!(after_time >= before_time);
    }

    // ==========================================================================
    // Metrics Update Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_metrics_update() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add project
        let project = ProjectState {
            id: "test".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Update metrics
        let metrics = QualityMetrics {
            avg_complexity: 5.5,
            max_complexity: 15,
            satd_count: 0,
            dead_code_percentage: 2.5,
            quality_score: 92.0,
            files_analyzed: 100,
            total_violations: 0,
        };

        persistence.update_metrics("test", metrics).await.unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.quality_history.len(), 1);
        assert_eq!(state.quality_history[0].project_id, "test");
    }

    #[tokio::test]
    async fn test_update_metrics_updates_project_state() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "test".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        let metrics = QualityMetrics {
            avg_complexity: 15.0,
            max_complexity: 30,
            satd_count: 5,
            dead_code_percentage: 8.0,
            quality_score: 75.0,
            files_analyzed: 200,
            total_violations: 15,
        };

        persistence
            .update_metrics("test", metrics.clone())
            .await
            .unwrap();

        let state = persistence.get_state().await;
        let proj = state.monitored_projects.get("test").unwrap();

        assert!((proj.current_metrics.avg_complexity - 15.0).abs() < f64::EPSILON);
        assert_eq!(proj.current_metrics.max_complexity, 30);
        assert_eq!(proj.current_metrics.satd_count, 5);
        assert!(proj.last_analyzed.is_some());
    }

    #[tokio::test]
    async fn test_update_metrics_nonexistent_project() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let metrics = QualityMetrics::default();

        // Should not error, just silently do nothing
        let result = persistence.update_metrics("nonexistent", metrics).await;
        assert!(result.is_ok());

        // History should be empty since project doesn't exist
        let state = persistence.get_state().await;
        assert!(state.quality_history.is_empty());
    }

    #[tokio::test]
    async fn test_update_metrics_adds_to_history() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "test".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Add multiple metrics updates
        for i in 0..5 {
            let metrics = QualityMetrics {
                avg_complexity: i as f64,
                max_complexity: i,
                satd_count: i as usize,
                dead_code_percentage: 0.0,
                quality_score: 100.0 - (i as f64),
                files_analyzed: 1,
                total_violations: 0,
            };
            persistence.update_metrics("test", metrics).await.unwrap();
        }

        let state = persistence.get_state().await;
        assert_eq!(state.quality_history.len(), 5);

        // Verify ordering (oldest first)
        for (i, snapshot) in state.quality_history.iter().enumerate() {
            assert_eq!(snapshot.metrics.max_complexity, i as u32);
        }
    }

    #[tokio::test]
    async fn test_update_metrics_trims_history() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "test".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Add more than 1000 entries
        for i in 0..1005 {
            let metrics = QualityMetrics {
                avg_complexity: 0.0,
                max_complexity: i,
                satd_count: 0,
                dead_code_percentage: 0.0,
                quality_score: 0.0,
                files_analyzed: 0,
                total_violations: 0,
            };
            persistence.update_metrics("test", metrics).await.unwrap();
        }

        let state = persistence.get_state().await;
        assert_eq!(state.quality_history.len(), 1000);

        // Oldest entries should have been removed (0-4), so first should be 5
        assert_eq!(state.quality_history[0].metrics.max_complexity, 5);
        // Last should be 1004
        assert_eq!(state.quality_history[999].metrics.max_complexity, 1004);
    }

