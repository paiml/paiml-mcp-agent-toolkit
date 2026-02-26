    // ========== HELPER FUNCTION TESTS ==========

    #[test]
    fn test_show_task_status_json() {
        let task = Task {
            id: "PMAT-0001".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::InProgress,
            complexity: Complexity::High,
            priority: Priority::P0,
            assignee: Some("dev".to_string()),
            started_at: Some(Utc::now()),
            completed_at: None,
        };

        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![task],
            completed_sprints: Vec::new(),
        };

        let result = show_task_status(&roadmap, "PMAT-0001", OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_task_status_table() {
        let task = Task {
            id: "PMAT-0002".to_string(),
            description: "Another task".to_string(),
            status: TaskStatus::Completed,
            complexity: Complexity::Low,
            priority: Priority::P2,
            assignee: None,
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
        };

        let mut roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: vec![task],
            completed_sprints: Vec::new(),
        };

        let result = show_task_status(&roadmap, "PMAT-0002", OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_task_status_not_found() {
        let roadmap = Roadmap {
            current_sprint: None,
            sprints: HashMap::new(),
            backlog: Vec::new(),
            completed_sprints: Vec::new(),
        };

        let result = show_task_status(&roadmap, "PMAT-9999", OutputFormat::Json);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_task_details() {
        let task = Task {
            id: "PMAT-0003".to_string(),
            description: "Display test".to_string(),
            status: TaskStatus::Blocked,
            complexity: Complexity::Medium,
            priority: Priority::P1,
            assignee: Some("tester".to_string()),
            started_at: Some(Utc::now()),
            completed_at: None,
        };

        // This function prints to stdout, just verify it doesn't panic
        display_task_details(&task);
    }

    #[test]
    fn test_display_task_details_with_completed() {
        let task = Task {
            id: "PMAT-0004".to_string(),
            description: "Completed task".to_string(),
            status: TaskStatus::Completed,
            complexity: Complexity::High,
            priority: Priority::P0,
            assignee: None,
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
        };

        display_task_details(&task);
    }

    #[test]
    fn test_calculate_sprint_progress() {
        let tasks = vec![
            Task {
                id: "PMAT-0001".to_string(),
                description: "Completed".to_string(),
                status: TaskStatus::Completed,
                complexity: Complexity::Low,
                priority: Priority::P1,
                assignee: None,
                started_at: None,
                completed_at: None,
            },
            Task {
                id: "PMAT-0002".to_string(),
                description: "In Progress".to_string(),
                status: TaskStatus::InProgress,
                complexity: Complexity::Low,
                priority: Priority::P1,
                assignee: None,
                started_at: None,
                completed_at: None,
            },
            Task {
                id: "PMAT-0003".to_string(),
                description: "Planned".to_string(),
                status: TaskStatus::Planned,
                complexity: Complexity::Low,
                priority: Priority::P1,
                assignee: None,
                started_at: None,
                completed_at: None,
            },
        ];

        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Test".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks,
            definition_of_done: Vec::new(),
            quality_gates: Vec::new(),
        };

        let (completed, in_progress, total) = calculate_sprint_progress(&sprint);

        assert_eq!(completed, 1);
        assert_eq!(in_progress, 1);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_calculate_sprint_progress_empty() {
        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Empty Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P0,
            tasks: Vec::new(),
            definition_of_done: Vec::new(),
            quality_gates: Vec::new(),
        };

        let (completed, in_progress, total) = calculate_sprint_progress(&sprint);

        assert_eq!(completed, 0);
        assert_eq!(in_progress, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_display_sprint_details() {
        let sprint = Sprint {
            version: "v2.0.0".to_string(),
            title: "Feature Sprint".to_string(),
            start_date: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            end_date: Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap(),
            priority: Priority::P0,
            tasks: vec![Task {
                id: "PMAT-0001".to_string(),
                description: "Task 1".to_string(),
                status: TaskStatus::Planned,
                complexity: Complexity::Medium,
                priority: Priority::P1,
                assignee: None,
                started_at: None,
                completed_at: None,
            }],
            definition_of_done: vec!["Done".to_string()],
            quality_gates: vec!["Gate".to_string()],
        };

        // Just verify it doesn't panic
        display_sprint_details(&sprint);
    }

    #[test]
    fn test_display_sprint_tasks() {
        let sprint = Sprint {
            version: "v3.0.0".to_string(),
            title: "Sprint with tasks".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            priority: Priority::P1,
            tasks: vec![
                Task {
                    id: "PMAT-0001".to_string(),
                    description: "First task".to_string(),
                    status: TaskStatus::Completed,
                    complexity: Complexity::Low,
                    priority: Priority::P2,
                    assignee: None,
                    started_at: None,
                    completed_at: None,
                },
                Task {
                    id: "PMAT-0002".to_string(),
                    description: "Second task".to_string(),
                    status: TaskStatus::InProgress,
                    complexity: Complexity::High,
                    priority: Priority::P0,
                    assignee: None,
                    started_at: None,
                    completed_at: None,
                },
            ],
            definition_of_done: Vec::new(),
            quality_gates: Vec::new(),
        };

        display_sprint_tasks(&sprint);
    }
