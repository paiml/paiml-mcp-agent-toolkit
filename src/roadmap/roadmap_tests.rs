// Unit tests and property tests for the roadmap module.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::tempdir;

    // Test TaskStatus functionality
    #[test]
    fn test_task_status_emoji_conversion() {
        assert_eq!(TaskStatus::Planned.to_emoji(), "\u{1f4cb}");
        assert_eq!(TaskStatus::InProgress.to_emoji(), "\u{1f6a7}");
        assert_eq!(TaskStatus::Completed.to_emoji(), "\u{2705}");
        assert_eq!(TaskStatus::Blocked.to_emoji(), "\u{1f6ab}");
        assert_eq!(TaskStatus::Deferred.to_emoji(), "\u{23f8}\u{fe0f}");
    }

    #[test]
    fn test_task_status_from_emoji() {
        assert_eq!(
            TaskStatus::from_emoji("\u{1f4cb}"),
            Some(TaskStatus::Planned)
        );
        assert_eq!(
            TaskStatus::from_emoji("\u{1f6a7}"),
            Some(TaskStatus::InProgress)
        );
        assert_eq!(
            TaskStatus::from_emoji("\u{2705}"),
            Some(TaskStatus::Completed)
        );
        assert_eq!(
            TaskStatus::from_emoji("\u{1f6ab}"),
            Some(TaskStatus::Blocked)
        );
        assert_eq!(
            TaskStatus::from_emoji("\u{23f8}\u{fe0f}"),
            Some(TaskStatus::Deferred)
        );
        assert_eq!(TaskStatus::from_emoji("\u{274c}"), None);
    }

    // Test Complexity parsing
    #[test]
    fn test_complexity_from_str() {
        use std::str::FromStr;

        assert_eq!(
            Complexity::from_str("low").expect("internal error"),
            Complexity::Low
        );
        assert_eq!(
            Complexity::from_str("LOW").expect("internal error"),
            Complexity::Low
        );
        assert_eq!(
            Complexity::from_str("medium").expect("internal error"),
            Complexity::Medium
        );
        assert_eq!(
            Complexity::from_str("high").expect("internal error"),
            Complexity::High
        );
        assert!(Complexity::from_str("invalid").is_err());
    }

    #[test]
    fn test_complexity_to_string() {
        assert_eq!(Complexity::Low.to_string(), "low");
        assert_eq!(Complexity::Medium.to_string(), "medium");
        assert_eq!(Complexity::High.to_string(), "high");
    }

    // Test Priority parsing
    #[test]
    fn test_priority_from_str() {
        use std::str::FromStr;

        assert_eq!(
            Priority::from_str("P0").expect("internal error"),
            Priority::P0
        );
        assert_eq!(
            Priority::from_str("p0").expect("internal error"),
            Priority::P0
        );
        assert_eq!(
            Priority::from_str("P1").expect("internal error"),
            Priority::P1
        );
        assert_eq!(
            Priority::from_str("P2").expect("internal error"),
            Priority::P2
        );
        assert!(Priority::from_str("P3").is_err());
    }

    // Test Task seed generation
    #[test]
    fn test_task_seed_generation() {
        let task = Task {
            id: "PMAT-1234".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: None,
            completed_at: None,
        };

        assert_eq!(task.seed(), 1234);

        let task_invalid = Task {
            id: "INVALID-ID".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: None,
            completed_at: None,
        };

        assert_eq!(task_invalid.seed(), 42); // Default seed
    }

    // Test Roadmap file operations
    #[test]
    fn test_roadmap_save_and_load() {
        let dir = tempdir().expect("internal error");
        let roadmap_path = dir.path().join("test_roadmap.md");

        let mut roadmap = Roadmap {
            current_sprint: Some("v2.46.0".to_string()),
            sprints: HashMap::new(),
            backlog: vec![],
            completed_sprints: vec![],
        };

        // Add a sprint
        let sprint = Sprint {
            version: "v2.46.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            end_date: Utc.with_ymd_and_hms(2025, 1, 7, 0, 0, 0).unwrap(),
            priority: Priority::P0,
            tasks: vec![],
            definition_of_done: vec!["All tests pass".to_string()],
            quality_gates: vec!["Coverage > 80%".to_string()],
        };

        roadmap.sprints.insert("v2.46.0".to_string(), sprint);

        // Save and load
        roadmap.to_file(&roadmap_path).expect("internal error");
        let loaded = Roadmap::from_file(&roadmap_path).expect("internal error");

        assert_eq!(loaded.current_sprint, Some("v2.46.0".to_string()));
        assert!(loaded.sprints.contains_key("v2.46.0"));
    }

    // Test Roadmap task operations
    #[test]
    fn test_roadmap_get_sprint() {
        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![],
            completed_sprints: vec![],
        };

        let sprint = Sprint {
            version: "v2.46.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: vec![],
            definition_of_done: vec![],
            quality_gates: vec![],
        };

        roadmap.sprints.insert("v2.46.0".to_string(), sprint);

        assert!(roadmap.get_sprint("v2.46.0").is_some());
        assert!(roadmap.get_sprint("v2.47.0").is_none());
    }

    #[test]
    fn test_roadmap_get_task() {
        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![],
            completed_sprints: vec![],
        };

        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: None,
            completed_at: None,
        };

        let sprint = Sprint {
            version: "v2.46.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: vec![task.clone()],
            definition_of_done: vec![],
            quality_gates: vec![],
        };

        roadmap.sprints.insert("v2.46.0".to_string(), sprint);

        // Test finding task in sprint
        assert!(roadmap.get_task("PMAT-0001").is_some());
        assert!(roadmap.get_task("PMAT-9999").is_none());

        // Test finding task in backlog
        let backlog_task = Task {
            id: "PMAT-0002".to_string(),
            description: "Backlog task".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Low,
            priority: Priority::P2,
            assignee: None,
            started_at: None,
            completed_at: None,
        };
        roadmap.backlog.push(backlog_task);

        assert!(roadmap.get_task("PMAT-0002").is_some());
    }

    #[test]
    fn test_roadmap_update_task_status() {
        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![],
            completed_sprints: vec![],
        };

        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: None,
            completed_at: None,
        };

        let sprint = Sprint {
            version: "v2.46.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: vec![task],
            definition_of_done: vec![],
            quality_gates: vec![],
        };

        roadmap.sprints.insert("v2.46.0".to_string(), sprint);

        // Update to InProgress
        roadmap
            .update_task_status("PMAT-0001", TaskStatus::InProgress)
            .expect("internal error");
        let task = roadmap.get_task("PMAT-0001").expect("internal error");
        assert_eq!(task.status, TaskStatus::InProgress);

        // Update to Completed
        roadmap
            .update_task_status("PMAT-0001", TaskStatus::Completed)
            .expect("internal error");
        let task = roadmap.get_task("PMAT-0001").expect("internal error");
        assert_eq!(task.status, TaskStatus::Completed);

        // Try updating non-existent task
        assert!(roadmap
            .update_task_status("PMAT-9999", TaskStatus::Completed)
            .is_err());
    }

    // Test configuration defaults
    #[test]
    fn test_roadmap_config_defaults() {
        let config = RoadmapConfig::default();

        assert!(config.enabled);
        assert_eq!(config.path, PathBuf::from("docs/execution/roadmap.md"));
        assert!(config.auto_generate_todos);
        assert!(config.enforce_quality_gates);
        assert!(config.require_task_ids);
        assert_eq!(config.task_id_pattern, "PMAT-[0-9]{4}");
    }

    #[test]
    fn test_quality_gate_config_defaults() {
        let config = QualityGateConfig::default();

        assert_eq!(config.complexity_max, 20);
        assert_eq!(config.coverage_min, 80);
        assert!(config.documentation_required);
        assert_eq!(config.satd_tolerance, 0);
        assert!(config.lint_compliance);
    }

    #[test]
    fn test_git_config_defaults() {
        let config = GitConfig::default();

        // DISABLED: per CLAUDE.md zero-branching policy
        assert!(!config.create_branches);
        assert_eq!(config.branch_pattern, "feature/{task_id}");
        assert_eq!(config.commit_pattern, "{task_id}: {message}");
        assert!(config.require_quality_check);
    }

    #[test]
    fn test_tracking_config_defaults() {
        let config = TrackingConfig::default();

        assert!(config.velocity_tracking);
        assert!(config.burndown_charts);
        assert!(config.quality_metrics);
        assert_eq!(config.export_format, "markdown");
    }

    // Test task status with timestamps
    #[test]
    fn test_update_task_status_timestamps() {
        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![],
            completed_sprints: vec![],
        };

        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Planned,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: None,
            started_at: None,
            completed_at: None,
        };

        let sprint = Sprint {
            version: "v2.46.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: vec![task],
            definition_of_done: vec![],
            quality_gates: vec![],
        };

        roadmap.sprints.insert("v2.46.0".to_string(), sprint);

        // Start task - should set started_at
        roadmap
            .update_task_status("PMAT-0001", TaskStatus::InProgress)
            .expect("internal error");
        if let Some(sprint) = roadmap.sprints.get("v2.46.0") {
            if let Some(task) = sprint.tasks.iter().find(|t| t.id == "PMAT-0001") {
                assert!(task.started_at.is_some());
                assert!(task.completed_at.is_none());
            }
        }

        // Complete task - should set completed_at
        roadmap
            .update_task_status("PMAT-0001", TaskStatus::Completed)
            .expect("internal error");
        if let Some(sprint) = roadmap.sprints.get("v2.46.0") {
            if let Some(task) = sprint.tasks.iter().find(|t| t.id == "PMAT-0001") {
                assert!(task.started_at.is_some());
                assert!(task.completed_at.is_some());
            }
        }
    }
}

