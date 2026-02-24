    // ==========================================================================
    // Update Statistics Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_update_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        persistence
            .update_statistics(|stats| {
                stats.sessions_count += 1;
                stats.analyses_performed += 10;
                stats.violations_detected += 5;
            })
            .await
            .unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.statistics.sessions_count, 1);
        assert_eq!(state.statistics.analyses_performed, 10);
        assert_eq!(state.statistics.violations_detected, 5);
    }

    #[tokio::test]
    async fn test_update_statistics_multiple_times() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        for _ in 0..10 {
            persistence
                .update_statistics(|stats| {
                    stats.sessions_count += 1;
                    stats.total_uptime_seconds += 60;
                })
                .await
                .unwrap();
        }

        let state = persistence.get_state().await;
        assert_eq!(state.statistics.sessions_count, 10);
        assert_eq!(state.statistics.total_uptime_seconds, 600);
    }

    #[tokio::test]
    async fn test_update_statistics_updates_last_updated() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let before_time = persistence.get_state().await.last_updated;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        persistence
            .update_statistics(|stats| {
                stats.refactorings_suggested += 1;
            })
            .await
            .unwrap();

        let after_time = persistence.get_state().await.last_updated;
        assert!(after_time >= before_time);
    }

    // ==========================================================================
    // Persistence and Load Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_save_and_load_full_state() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Add projects
        for i in 0..3 {
            let project = ProjectState {
                id: format!("project_{}", i),
                path: PathBuf::from(format!("/path/{}", i)),
                started_at: Utc::now(),
                last_analyzed: Some(Utc::now()),
                current_metrics: QualityMetrics {
                    avg_complexity: i as f64 * 5.0,
                    max_complexity: i * 10,
                    satd_count: i as usize,
                    dead_code_percentage: 0.0,
                    quality_score: 100.0 - (i as f64 * 10.0),
                    files_analyzed: 100,
                    total_violations: i as usize,
                },
                watch_patterns: vec![format!("*.{}", i)],
                thresholds: QualityThresholds::default(),
            };
            persistence.add_project(project).await.unwrap();
        }

        // Update statistics
        persistence
            .update_statistics(|stats| {
                stats.sessions_count = 5;
                stats.analyses_performed = 50;
            })
            .await
            .unwrap();

        // Save
        persistence.save().await.unwrap();

        // Load fresh instance
        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;

        assert_eq!(state.monitored_projects.len(), 3);
        assert!(state.monitored_projects.contains_key("project_0"));
        assert!(state.monitored_projects.contains_key("project_1"));
        assert!(state.monitored_projects.contains_key("project_2"));
        assert_eq!(state.statistics.sessions_count, 5);
        assert_eq!(state.statistics.analyses_performed, 50);
    }

    #[tokio::test]
    async fn test_save_atomic_write() {
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
        persistence.save().await.unwrap();

        // Verify temp file doesn't exist after save (it should be renamed)
        let temp_file = temp_dir.path().join("agent_state.tmp");
        assert!(!temp_file.exists());

        // Verify main file exists
        let main_file = temp_dir.path().join("agent_state.json");
        assert!(main_file.exists());
    }

    #[test]
    fn test_load_from_file_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("agent_state.json");

        // Write invalid JSON
        std::fs::write(&state_file, "{ invalid json }").unwrap();

        let result = StatePersistence::load_from_file(&state_file);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to deserialize"));
    }

    #[test]
    fn test_load_from_file_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.json");

        let result = StatePersistence::load_from_file(&nonexistent);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    // ==========================================================================
    // Clone and Debug Tests
    // ==========================================================================

    #[test]
    fn test_quality_metrics_clone() {
        let metrics = QualityMetrics {
            avg_complexity: 10.5,
            max_complexity: 20,
            satd_count: 3,
            dead_code_percentage: 5.0,
            quality_score: 85.0,
            files_analyzed: 100,
            total_violations: 10,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.max_complexity, metrics.max_complexity);
        assert_eq!(cloned.satd_count, metrics.satd_count);
    }

    #[test]
    fn test_quality_thresholds_clone() {
        let thresholds = QualityThresholds {
            max_complexity: 25,
            satd_tolerance: 5,
            dead_code_max_percentage: 15.0,
            min_quality_score: 70.0,
        };

        let cloned = thresholds.clone();
        assert_eq!(cloned.max_complexity, 25);
        assert_eq!(cloned.satd_tolerance, 5);
    }

    #[test]
    fn test_agent_state_debug() {
        let state = AgentState::default();
        let debug_str = format!("{:?}", state);

        assert!(debug_str.contains("AgentState"));
        assert!(debug_str.contains("version"));
    }

    #[test]
    fn test_project_state_debug() {
        let project = ProjectState {
            id: "debug_test".to_string(),
            path: PathBuf::from("/debug/path"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        let debug_str = format!("{:?}", project);
        assert!(debug_str.contains("ProjectState"));
        assert!(debug_str.contains("debug_test"));
    }

    // ==========================================================================
    // Edge Case Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_empty_watch_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "empty_patterns".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();
        persistence.save().await.unwrap();

        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;
        let proj = state.monitored_projects.get("empty_patterns").unwrap();

        assert!(proj.watch_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_watch_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "multi_patterns".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![
                "*.rs".to_string(),
                "*.toml".to_string(),
                "*.md".to_string(),
                "src/**/*.rs".to_string(),
            ],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();
        persistence.save().await.unwrap();

        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;
        let proj = state.monitored_projects.get("multi_patterns").unwrap();

        assert_eq!(proj.watch_patterns.len(), 4);
        assert!(proj.watch_patterns.contains(&"*.rs".to_string()));
        assert!(proj.watch_patterns.contains(&"src/**/*.rs".to_string()));
    }

    #[tokio::test]
    async fn test_config_overrides_complex_json() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        {
            let mut state = persistence.state.write().await;
            state.config_overrides.insert(
                "complex_config".to_string(),
                serde_json::json!({
                    "nested": {
                        "deeply": {
                            "value": [1, 2, 3]
                        }
                    },
                    "array": ["a", "b", "c"],
                    "number": 42.5,
                    "boolean": true,
                    "null_value": null
                }),
            );
        }

        persistence.save().await.unwrap();

        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;

        let config = state.config_overrides.get("complex_config").unwrap();
        assert!(config["nested"]["deeply"]["value"].is_array());
        assert_eq!(config["number"], 42.5);
        assert_eq!(config["boolean"], true);
    }

    #[tokio::test]
    async fn test_quality_snapshot_with_violations() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "with_violations".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();

        // Update metrics which adds to history
        let metrics = QualityMetrics {
            avg_complexity: 25.0,
            max_complexity: 50,
            satd_count: 10,
            dead_code_percentage: 20.0,
            quality_score: 50.0,
            files_analyzed: 50,
            total_violations: 25,
        };

        persistence
            .update_metrics("with_violations", metrics)
            .await
            .unwrap();

        let state = persistence.get_state().await;
        assert_eq!(state.quality_history.len(), 1);

        // Note: violations in snapshot are currently empty (from implementation)
        // This test documents current behavior
        assert!(state.quality_history[0].violations.is_empty());
    }

    #[tokio::test]
    async fn test_special_characters_in_project_id() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let special_id = "project/with:special@chars#123";
        let project = ProjectState {
            id: special_id.to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();
        persistence.save().await.unwrap();

        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;

        assert!(state.monitored_projects.contains_key(special_id));
    }

    #[tokio::test]
    async fn test_unicode_in_paths() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "unicode_project".to_string(),
            path: PathBuf::from("/path/to/项目/プロジェクト/проект"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics::default(),
            watch_patterns: vec!["*.日本語".to_string()],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();
        persistence.save().await.unwrap();

        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;
        let proj = state.monitored_projects.get("unicode_project").unwrap();

        assert!(proj.path.to_string_lossy().contains("项目"));
        assert!(proj.watch_patterns[0].contains("日本語"));
    }

    #[tokio::test]
    async fn test_extreme_metric_values() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let project = ProjectState {
            id: "extreme".to_string(),
            path: PathBuf::from("/test"),
            started_at: Utc::now(),
            last_analyzed: None,
            current_metrics: QualityMetrics {
                avg_complexity: f64::MAX,
                max_complexity: u32::MAX,
                satd_count: usize::MAX,
                dead_code_percentage: 100.0,
                quality_score: 0.0,
                files_analyzed: usize::MAX,
                total_violations: usize::MAX,
            },
            watch_patterns: vec![],
            thresholds: QualityThresholds::default(),
        };

        persistence.add_project(project).await.unwrap();
        persistence.save().await.unwrap();

        let loaded = StatePersistence::new(temp_dir.path()).unwrap();
        let state = loaded.get_state().await;
        let proj = state.monitored_projects.get("extreme").unwrap();

        assert_eq!(proj.current_metrics.max_complexity, u32::MAX);
    }

    #[tokio::test]
    async fn test_auto_save_interval_field() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        // Default auto-save interval should be 60 seconds
        assert_eq!(persistence.auto_save_interval, 60);
    }
