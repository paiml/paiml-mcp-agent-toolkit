// Unit tests for health score calculation: velocity, aging, dependency,
// overall health scoring, and report formatting.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::maintenance::roadmap::{Sprint, SprintStatus, Ticket};
    use crate::maintenance::ticket::{Priority, TicketStatus};
    use std::path::PathBuf;

    #[test]
    fn test_calculate_velocity_empty() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![],
        };

        let velocity = calculate_velocity(&roadmap);

        assert_eq!(velocity.total_tickets, 0);
        assert_eq!(velocity.completed_tickets, 0);
        assert_eq!(velocity.completion_rate, 0.0);
    }

    #[test]
    fn test_calculate_velocity_partial() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![Sprint {
                number: 1,
                name: "Test".into(),
                focus: "Testing".into(),
                status: SprintStatus::InProgress,
                duration: "2 days".into(),
                tickets: vec![
                    Ticket {
                        id: "TICKET-1".into(),
                        description: "Test".into(),
                        completed: true,
                        commit: Some("abc".into()),
                    },
                    Ticket {
                        id: "TICKET-2".into(),
                        description: "Test".into(),
                        completed: false,
                        commit: None,
                    },
                ],
                quality_gates: vec![],
            }],
        };

        let velocity = calculate_velocity(&roadmap);

        assert_eq!(velocity.total_tickets, 2);
        assert_eq!(velocity.completed_tickets, 1);
        assert_eq!(velocity.completion_rate, 50.0);
    }

    #[test]
    fn test_calculate_velocity_all_complete() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![Sprint {
                number: 1,
                name: "Test".into(),
                focus: "Testing".into(),
                status: SprintStatus::Complete,
                duration: "2 days".into(),
                tickets: vec![Ticket {
                    id: "TICKET-1".into(),
                    description: "Test".into(),
                    completed: true,
                    commit: Some("abc".into()),
                }],
                quality_gates: vec![],
            }],
        };

        let velocity = calculate_velocity(&roadmap);

        assert_eq!(velocity.total_tickets, 1);
        assert_eq!(velocity.completed_tickets, 1);
        assert_eq!(velocity.completion_rate, 100.0);
    }

    #[test]
    fn test_velocity_to_score() {
        let velocity = VelocityMetrics {
            total_tickets: 10,
            completed_tickets: 7,
            completion_rate: 70.0,
        };

        let score = velocity_to_score(&velocity);
        assert_eq!(score, 70.0);
    }

    #[test]
    fn test_analyze_aging_no_red_tickets() {
        let tickets = vec![TicketFile {
            id: "TICKET-1".into(),
            title: "Test".into(),
            status: TicketStatus::Green,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1h".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Test".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        }];

        let aging = analyze_aging(&tickets);

        assert_eq!(aging.red_tickets.len(), 0);
        assert_eq!(aging.stale_tickets.len(), 0);
    }

    #[test]
    fn test_analyze_aging_with_red_tickets() {
        let tickets = vec![TicketFile {
            id: "TICKET-1".into(),
            title: "Test".into(),
            status: TicketStatus::Red,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1h".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Test".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        }];

        let aging = analyze_aging(&tickets);

        assert_eq!(aging.red_tickets.len(), 1);
        assert_eq!(aging.red_tickets[0], "TICKET-1");
        assert_eq!(aging.stale_tickets.len(), 1);
    }

    #[test]
    fn test_aging_to_score_no_stale() {
        let aging = AgingMetrics {
            red_tickets: vec![],
            stale_tickets: vec![],
        };

        let score = aging_to_score(&aging, 10);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_aging_to_score_with_stale() {
        let aging = AgingMetrics {
            red_tickets: vec!["TICKET-1".into()],
            stale_tickets: vec!["TICKET-1".into()],
        };

        let score = aging_to_score(&aging, 10);
        assert_eq!(score, 90.0); // 1 stale out of 10 = 90%
    }

    #[test]
    fn test_aging_to_score_empty_project() {
        let aging = AgingMetrics {
            red_tickets: vec![],
            stale_tickets: vec![],
        };

        let score = aging_to_score(&aging, 0);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_analyze_dependencies_no_broken() {
        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec!["TICKET-2".into()],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
            TicketFile {
                id: "TICKET-2".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let ticket_map: HashMap<_, _> = tickets.iter().map(|t| (t.id.clone(), t)).collect();

        let deps = analyze_dependencies(&tickets, &ticket_map);

        assert_eq!(deps.total_dependencies, 1);
        assert_eq!(deps.broken_dependencies, 0);
    }

    #[test]
    fn test_analyze_dependencies_with_broken() {
        let tickets = vec![TicketFile {
            id: "TICKET-1".into(),
            title: "Test".into(),
            status: TicketStatus::Green,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1h".into(),
            dependencies: vec!["TICKET-MISSING".into()],
            sprint: "Sprint 1".into(),
            objective: "Test".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        }];

        let ticket_map: HashMap<_, _> = tickets.iter().map(|t| (t.id.clone(), t)).collect();

        let deps = analyze_dependencies(&tickets, &ticket_map);

        assert_eq!(deps.total_dependencies, 1);
        assert_eq!(deps.broken_dependencies, 1);
    }

    #[test]
    fn test_dependency_to_score_no_deps() {
        let deps = DependencyMetrics {
            total_dependencies: 0,
            broken_dependencies: 0,
        };

        let score = dependency_to_score(&deps);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_dependency_to_score_all_good() {
        let deps = DependencyMetrics {
            total_dependencies: 10,
            broken_dependencies: 0,
        };

        let score = dependency_to_score(&deps);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_dependency_to_score_some_broken() {
        let deps = DependencyMetrics {
            total_dependencies: 10,
            broken_dependencies: 2,
        };

        let score = dependency_to_score(&deps);
        assert_eq!(score, 80.0); // 2 broken out of 10 = 80%
    }

    #[test]
    fn test_calculate_health_score() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![Sprint {
                number: 1,
                name: "Test".into(),
                focus: "Testing".into(),
                status: SprintStatus::InProgress,
                duration: "2 days".into(),
                tickets: vec![Ticket {
                    id: "TICKET-1".into(),
                    description: "Test".into(),
                    completed: true,
                    commit: Some("abc".into()),
                }],
                quality_gates: vec![],
            }],
        };

        let tickets = vec![TicketFile {
            id: "TICKET-1".into(),
            title: "Test".into(),
            status: TicketStatus::Green,
            priority: Priority::P0,
            complexity: 5,
            estimated_time: "1h".into(),
            dependencies: vec![],
            sprint: "Sprint 1".into(),
            objective: "Test".into(),
            success_criteria: vec![],
            file_path: PathBuf::new(),
        }];

        let health = calculate_health_score(&roadmap, &tickets).unwrap();

        assert!(health.overall_score >= 0.0);
        assert!(health.overall_score <= 100.0);
        assert_eq!(health.version, "v1.0.0");
        assert_eq!(health.metrics.velocity_score, 100.0);
        assert_eq!(health.metrics.aging_score, 100.0);
        assert_eq!(health.metrics.dependency_score, 100.0);
    }

    #[test]
    fn test_calculate_health_score_mixed() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![Sprint {
                number: 1,
                name: "Test".into(),
                focus: "Testing".into(),
                status: SprintStatus::InProgress,
                duration: "2 days".into(),
                tickets: vec![
                    Ticket {
                        id: "TICKET-1".into(),
                        description: "Test".into(),
                        completed: true,
                        commit: Some("abc".into()),
                    },
                    Ticket {
                        id: "TICKET-2".into(),
                        description: "Test".into(),
                        completed: false,
                        commit: None,
                    },
                ],
                quality_gates: vec![],
            }],
        };

        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
            TicketFile {
                id: "TICKET-2".into(),
                title: "Test".into(),
                status: TicketStatus::Red,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let health = calculate_health_score(&roadmap, &tickets).unwrap();

        assert!(health.overall_score >= 0.0);
        assert!(health.overall_score <= 100.0);
        assert_eq!(health.metrics.velocity_score, 50.0); // 1 of 2 complete
        assert_eq!(health.metrics.aging_score, 50.0); // 1 of 2 stale
    }

    #[test]
    fn test_format_health_report_high_score() {
        let health = HealthScore {
            overall_score: 85.5,
            metrics: HealthMetrics {
                velocity_score: 90.0,
                aging_score: 80.0,
                dependency_score: 85.0,
                quality_score: 90.0,
            },
            timestamp: "2025-10-05T10:00:00Z".into(),
            version: "v1.0.0".into(),
        };

        let report = format_health_report(&health);

        assert!(report.contains("Project Health Report: v1.0.0"));
        assert!(report.contains("85.5%"));
        assert!(report.contains("\u{2705}")); // High score emoji
        assert!(report.contains("**Velocity**: 90.0%"));
        assert!(report.contains("**Ticket Aging**: 80.0%"));
        assert!(report.contains("**Dependencies**: 85.0%"));
        assert!(report.contains("**Quality Gates**: 90.0%"));
    }

    #[test]
    fn test_format_health_report_medium_score() {
        let health = HealthScore {
            overall_score: 65.0,
            metrics: HealthMetrics {
                velocity_score: 70.0,
                aging_score: 60.0,
                dependency_score: 65.0,
                quality_score: 70.0,
            },
            timestamp: "2025-10-05T10:00:00Z".into(),
            version: "v1.0.0".into(),
        };

        let report = format_health_report(&health);

        assert!(report.contains("65.0%"));
        assert!(report.contains("\u{26a0}\u{fe0f}")); // Medium score emoji
    }

    #[test]
    fn test_format_health_report_low_score() {
        let health = HealthScore {
            overall_score: 45.0,
            metrics: HealthMetrics {
                velocity_score: 50.0,
                aging_score: 40.0,
                dependency_score: 45.0,
                quality_score: 50.0,
            },
            timestamp: "2025-10-05T10:00:00Z".into(),
            version: "v1.0.0".into(),
        };

        let report = format_health_report(&health);

        assert!(report.contains("45.0%"));
        assert!(report.contains("\u{274c}")); // Low score emoji
    }
}
